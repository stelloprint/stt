use anyhow::Result;
use directories::ProjectDirs;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrefsError {
    #[error("Failed to get app data directory")]
    NoAppDir,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActivationMode {
    Hold,
    Toggle,
}

impl Default for ActivationMode {
    fn default() -> Self {
        Self::Hold
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SilenceRms {
    Low,
    Medium,
    High,
}

impl Default for SilenceRms {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ModelProfile {
    #[serde(rename = "small.en")]
    EnglishSmall,
    #[serde(rename = "multilingual-small")]
    MultilingualSmall,
    #[serde(rename = "multilingual-medium")]
    MultilingualMedium,
}

impl Default for ModelProfile {
    fn default() -> Self {
        Self::MultilingualSmall
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hotkeys {
    pub left_chord: bool,
    pub right_chord: bool,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            left_chord: true,
            right_chord: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypingPrefs {
    pub newline_at_end: bool,
    pub throttle_ms: u32,
}

impl Default for TypingPrefs {
    fn default() -> Self {
        Self {
            newline_at_end: false,
            throttle_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceCommandMap {
    pub newline: String,
    pub new_paragraph: String,
    pub tab: String,
    pub period: String,
    pub comma: String,
    pub colon: String,
    pub semicolon: String,
    pub open_quote: String,
    pub close_quote: String,
    pub backtick: String,
    pub code_block: String,
}

impl Default for VoiceCommandMap {
    fn default() -> Self {
        Self {
            newline: "newline".to_string(),
            new_paragraph: "new paragraph".to_string(),
            tab: "tab".to_string(),
            period: "period".to_string(),
            comma: "comma".to_string(),
            colon: "colon".to_string(),
            semicolon: "semicolon".to_string(),
            open_quote: "open quote".to_string(),
            close_quote: "close quote".to_string(),
            backtick: "backtick".to_string(),
            code_block: "code block".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceCommands {
    pub enabled: bool,
    pub map: VoiceCommandMap,
}

impl Default for VoiceCommands {
    fn default() -> Self {
        Self {
            enabled: true,
            map: VoiceCommandMap::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordPrefs {
    pub chunk_seconds: u32,
    pub max_hours: u32,
    pub max_file_gb: u32,
}

impl Default for RecordPrefs {
    fn default() -> Self {
        Self {
            chunk_seconds: 60,
            max_hours: 8,
            max_file_gb: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    pub hotkeys: Hotkeys,
    pub mode: ActivationMode,
    pub silence_seconds: f32,
    pub silence_rms: SilenceRms,
    pub model_profile: ModelProfile,
    pub translate_to_english: bool,
    pub typing: TypingPrefs,
    pub voice_commands: VoiceCommands,
    pub record: RecordPrefs,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            hotkeys: Hotkeys::default(),
            mode: ActivationMode::Hold,
            silence_seconds: 3.0,
            silence_rms: SilenceRms::Medium,
            model_profile: ModelProfile::default(),
            translate_to_english: true,
            typing: TypingPrefs::default(),
            voice_commands: VoiceCommands::default(),
            record: RecordPrefs::default(),
        }
    }
}

pub struct Prefs {
    inner: RwLock<Preferences>,
    config_path: PathBuf,
}

impl Prefs {
    pub fn new() -> Result<Self, PrefsError> {
        let config_path = Self::get_config_path()?;
        let prefs = Self::load_from_file(&config_path).unwrap_or_default();

        Ok(Self {
            inner: RwLock::new(prefs),
            config_path,
        })
    }

    fn get_config_path() -> Result<PathBuf, PrefsError> {
        let proj_dirs = ProjectDirs::from("com", "stt", "sst").ok_or(PrefsError::NoAppDir)?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        Ok(config_dir.join("config.json"))
    }

    fn load_from_file(path: &PathBuf) -> Result<Preferences, PrefsError> {
        let content = fs::read_to_string(path)?;
        let prefs: Preferences = serde_json::from_str(&content)?;
        Ok(prefs)
    }

    pub fn save(&self) -> Result<(), PrefsError> {
        let prefs = self.inner.read();
        let content = serde_json::to_string_pretty(&*prefs)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn get(&self) -> Preferences {
        self.inner.read().clone()
    }

    pub fn update(&self, prefs: Preferences) -> Result<(), PrefsError> {
        *self.inner.write() = prefs;
        self.save()?;
        Ok(())
    }

    pub fn get_config_dir() -> Result<PathBuf, PrefsError> {
        let proj_dirs = ProjectDirs::from("com", "stt", "sst").ok_or(PrefsError::NoAppDir)?;

        let config_dir = proj_dirs.config_dir().to_path_buf();
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    pub fn get_data_dir() -> Result<PathBuf, PrefsError> {
        let proj_dirs = ProjectDirs::from("com", "stt", "sst").ok_or(PrefsError::NoAppDir)?;

        let data_dir = proj_dirs.data_dir().to_path_buf();
        fs::create_dir_all(&data_dir)?;
        Ok(data_dir)
    }

    pub fn get_models_dir() -> Result<PathBuf, PrefsError> {
        let data_dir = Self::get_data_dir()?;
        let models_dir = data_dir.join("models");
        fs::create_dir_all(&models_dir)?;
        Ok(models_dir)
    }
}
