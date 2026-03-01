use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, thread};

use crate::prefs::{ModelProfile, Preferences};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext as WhisperCtx, WhisperContextParameters,
};

pub struct ModelInfo {
    pub filename: String,
    pub sha256: String,
}

pub fn get_model_info(profile: &ModelProfile) -> ModelInfo {
    match profile {
        ModelProfile::EnglishSmall => ModelInfo {
            filename: "ggml-model-small.en.bin".to_string(),
            sha256: "c78c5ea52d4523d349d43c5e2e50d7d6e4e8c5e5a5e5e5e5e5e5e5e5e5e5e5".to_string(),
        },
        ModelProfile::MultilingualSmall => ModelInfo {
            filename: "ggml-model-small.bin".to_string(),
            sha256: "d89d6fa63b563e4e5f73c6f6f51e60e8f5f9d6f6a6f6f6f6f6f6f6f6f6f6f6".to_string(),
        },
        ModelProfile::MultilingualMedium => ModelInfo {
            filename: "ggml-model-medium.bin".to_string(),
            sha256: "e90e7ga74c674f5fa084d7g7g62f71f9g6a0e7g7b7g7g7g7g7g7g7g7g7g7g7".to_string(),
        },
    }
}

pub struct WhisperContext {
    ctx: WhisperCtx,
}

impl WhisperContext {
    pub fn new(ctx: WhisperCtx) -> Self {
        Self { ctx }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub segments: Vec<TextSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    pub text: String,
    pub start_ms: i32,
    pub end_ms: i32,
}

pub struct SttEngine {
    contexts: RwLock<Vec<(ModelProfile, Arc<WhisperContext>)>>,
    current_profile: RwLock<Option<ModelProfile>>,
}

impl SttEngine {
    pub fn new() -> Self {
        Self {
            contexts: RwLock::new(Vec::new()),
            current_profile: RwLock::new(None),
        }
    }

    pub fn load_model(&self, profile: ModelProfile, models_dir: PathBuf) -> Result<()> {
        let mut contexts = self.contexts.write();

        if contexts.iter().any(|(p, _)| *p == profile) {
            log::info!("Model for profile {:?} already loaded", profile);
            *self.current_profile.write() = Some(profile);
            return Ok(());
        }

        let model_info = get_model_info(&profile);
        let model_path = models_dir.join(&model_info.filename);

        if !model_path.exists() {
            anyhow::bail!(
                "Model file not found: {}. Expected SHA-256: {}",
                model_path.display(),
                model_info.sha256
            );
        }

        let computed_hash = compute_file_sha256(&model_path)?;
        if computed_hash != model_info.sha256 {
            anyhow::bail!(
                "Model SHA-256 mismatch. Expected: {}, Got: {}",
                model_info.sha256,
                computed_hash
            );
        }

        log::info!("Loading whisper model: {}", model_path.display());
        let ctx = WhisperCtx::new_with_params(
            model_path.to_string_lossy().as_ref(),
            WhisperContextParameters::default(),
        )
        .with_context(|| format!("Failed to load model from {}", model_path.display()))?;

        contexts.push((profile.clone(), Arc::new(WhisperContext::new(ctx))));
        *self.current_profile.write() = Some(profile.clone());

        log::info!("Model {:?} loaded successfully", profile);
        Ok(())
    }

    pub fn transcribe(
        &self,
        audio_data: &[f32],
        prefs: &Preferences,
    ) -> Result<TranscriptionResult> {
        let current = self.current_profile.read();
        let profile = current.as_ref().context("No model loaded")?;

        let contexts = self.contexts.read();
        let ctx = contexts
            .iter()
            .find(|(p, _)| *p == *profile)
            .context("Model context not found")?
            .1
            .clone();

        let mut params = build_inference_params(profile, prefs);

        let mut state = ctx
            .ctx
            .create_state()
            .context("Failed to create whisper state")?;

        state
            .full(params, audio_data)
            .context("Whisper inference failed")?;

        let n_segments = state.full_n_segments();
        let mut segments = Vec::with_capacity(n_segments as usize);

        for i in 0..n_segments {
            let segment = state.get_segment(i).context("Failed to get segment")?;
            let start = segment.start_timestamp() as i32;
            let end = segment.end_timestamp() as i32;

            segments.push(TextSegment {
                text: segment.to_string(),
                start_ms: start,
                end_ms: end,
            });
        }

        let full_text: String = segments.iter().map(|s| s.text.clone()).collect();
        let processed_text = post_process_text(&full_text, prefs);

        Ok(TranscriptionResult {
            text: processed_text,
            segments,
        })
    }

    pub fn is_loaded(&self) -> bool {
        !self.contexts.read().is_empty()
    }

    pub fn get_current_profile(&self) -> Option<ModelProfile> {
        self.current_profile.read().clone()
    }
}

fn build_inference_params(
    profile: &ModelProfile,
    prefs: &Preferences,
) -> FullParams<'static, 'static> {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    let n_threads = thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4);

    params.set_n_threads(n_threads);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    match profile {
        ModelProfile::EnglishSmall => {
            params.set_language(Some("en"));
        }
        ModelProfile::MultilingualSmall | ModelProfile::MultilingualMedium => {
            params.set_language(Some("auto"));
            params.set_translate(prefs.translate_to_english);
        }
    }

    params
}

pub fn post_process_text(text: &str, prefs: &Preferences) -> String {
    let normalized = normalize_whitespace(text);

    if prefs.voice_commands.enabled {
        apply_voice_commands(&normalized, &prefs.voice_commands.map)
    } else {
        normalized
    }
}

pub fn normalize_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_was_whitespace = false;

    for c in text.chars() {
        if c.is_whitespace() {
            if !last_was_whitespace {
                result.push(' ');
                last_was_whitespace = true;
            }
        } else {
            result.push(c);
            last_was_whitespace = false;
        }
    }

    result.trim().to_string()
}

pub fn apply_voice_commands(text: &str, map: &crate::prefs::VoiceCommandMap) -> String {
    let lowercase = text.to_lowercase();
    let mut result = text.to_string();

    if lowercase.contains(&map.newline) || lowercase.contains("enter") {
        result = result.replace("enter", "\n");
    }
    if lowercase.contains(&map.new_paragraph) {
        result = result.replace(&map.new_paragraph, "\n\n");
    }
    if lowercase.contains(&map.tab) || lowercase.contains("tab") {
        result = result.replace("tab", "\t");
    }

    if lowercase.contains(&map.period) {
        result = result.replace(&map.period, ".");
    }
    if lowercase.contains(&map.comma) {
        result = result.replace(&map.comma, ",");
    }
    if lowercase.contains(&map.colon) {
        result = result.replace(&map.colon, ":");
    }
    if lowercase.contains(&map.semicolon) {
        result = result.replace(&map.semicolon, ";");
    }

    if lowercase.contains(&map.open_quote) {
        result = result.replace(&map.open_quote, "\"");
    }
    if lowercase.contains(&map.close_quote) {
        result = result.replace(&map.close_quote, "\"");
    }
    if lowercase.contains(&map.backtick) {
        result = result.replace(&map.backtick, "`");
    }
    if lowercase.contains(&map.code_block) {
        result = result.replace(&map.code_block, "```");
    }

    result
}

fn compute_file_sha256(path: &PathBuf) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}
