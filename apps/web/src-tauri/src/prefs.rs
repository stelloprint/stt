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
    #[error("Validation error: {0}")]
    Validation(String),
}

pub fn validate_preferences(prefs: &Preferences) -> Result<(), PrefsError> {
    if !prefs.hotkeys.left_chord && !prefs.hotkeys.right_chord {
        return Err(PrefsError::Validation(
            "At least one hotkey must be enabled".to_string(),
        ));
    }
    if prefs.silence_seconds < 0.5 || prefs.silence_seconds > 30.0 {
        return Err(PrefsError::Validation(
            "silence_seconds must be between 0.5 and 30".to_string(),
        ));
    }
    if prefs.typing.throttle_ms > 1000 {
        return Err(PrefsError::Validation(
            "throttle_ms must not exceed 1000".to_string(),
        ));
    }
    if prefs.record.chunk_seconds < 10 || prefs.record.chunk_seconds > 300 {
        return Err(PrefsError::Validation(
            "chunk_seconds must be between 10 and 300".to_string(),
        ));
    }
    if prefs.record.max_hours < 1 || prefs.record.max_hours > 24 {
        return Err(PrefsError::Validation(
            "max_hours must be between 1 and 24".to_string(),
        ));
    }
    if prefs.record.max_file_gb < 1 || prefs.record.max_file_gb > 16 {
        return Err(PrefsError::Validation(
            "max_file_gb must be between 1 and 16".to_string(),
        ));
    }
    if prefs.voice_commands.enabled {
        let map = &prefs.voice_commands.map;
        if map.newline.is_empty()
            || map.new_paragraph.is_empty()
            || map.tab.is_empty()
            || map.period.is_empty()
            || map.comma.is_empty()
            || map.colon.is_empty()
            || map.semicolon.is_empty()
            || map.open_quote.is_empty()
            || map.close_quote.is_empty()
            || map.backtick.is_empty()
            || map.code_block.is_empty()
        {
            return Err(PrefsError::Validation(
                "All voice command mappings must be non-empty when enabled".to_string(),
            ));
        }
    }
    Ok(())
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
#[serde(default)]
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
    #[serde(default)]
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

        if let Err(e) = validate_preferences(&prefs) {
            eprintln!(
                "WARN: persisted preferences failed validation ({}) - using defaults",
                e
            );
            return Ok(Self {
                inner: RwLock::new(Preferences::default()),
                config_path,
            });
        }

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
        validate_preferences(&prefs)?;
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
        let proj_dirs = ProjectDirs::from("com", "stt", "sst").ok_or(PrefsError::NoAppDir)?;

        let data_dir = proj_dirs.data_dir();
        let models_dir = data_dir.join("models");
        fs::create_dir_all(&models_dir)?;
        Ok(models_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preferences_valid() {
        let prefs = Preferences::default();
        assert!(validate_preferences(&prefs).is_ok());
    }

    #[test]
    fn test_preferences_json_serialization_roundtrip() {
        let prefs = Preferences::default();
        let json = serde_json::to_string(&prefs).unwrap();
        let loaded: Preferences = serde_json::from_str(&json).unwrap();
        assert_eq!(prefs, loaded);
    }

    #[test]
    fn test_preferences_custom_json_parsing() {
        let json = r#"{
            "hotkeys": { "left_chord": true, "right_chord": false },
            "mode": "toggle",
            "silence_seconds": 5.0,
            "silence_rms": "high",
            "model_profile": "small.en",
            "translate_to_english": false,
            "typing": { "newline_at_end": true, "throttle_ms": 50 },
            "voice_commands": { "enabled": false, "map": {} },
            "record": { "chunk_seconds": 120, "max_hours": 4, "max_file_gb": 2 }
        }"#;
        let prefs: Preferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.hotkeys.left_chord, true);
        assert_eq!(prefs.hotkeys.right_chord, false);
        assert_eq!(prefs.mode, ActivationMode::Toggle);
        assert_eq!(prefs.silence_seconds, 5.0);
        assert_eq!(prefs.silence_rms, SilenceRms::High);
        assert_eq!(prefs.model_profile, ModelProfile::EnglishSmall);
        assert_eq!(prefs.translate_to_english, false);
        assert_eq!(prefs.typing.newline_at_end, true);
        assert_eq!(prefs.typing.throttle_ms, 50);
        assert_eq!(prefs.voice_commands.enabled, false);
        assert_eq!(prefs.record.chunk_seconds, 120);
        assert_eq!(prefs.record.max_hours, 4);
        assert_eq!(prefs.record.max_file_gb, 2);
    }

    #[test]
    fn test_validation_hotkeys_both_disabled() {
        let mut prefs = Preferences::default();
        prefs.hotkeys.left_chord = false;
        prefs.hotkeys.right_chord = false;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one hotkey"));
    }

    #[test]
    fn test_validation_hotkeys_one_enabled() {
        let mut prefs = Preferences::default();
        prefs.hotkeys.left_chord = true;
        prefs.hotkeys.right_chord = false;
        assert!(validate_preferences(&prefs).is_ok());
    }

    #[test]
    fn test_validation_silence_seconds_too_low() {
        let mut prefs = Preferences::default();
        prefs.silence_seconds = 0.1;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("silence_seconds"));
    }

    #[test]
    fn test_validation_silence_seconds_too_high() {
        let mut prefs = Preferences::default();
        prefs.silence_seconds = 50.0;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("silence_seconds"));
    }

    #[test]
    fn test_validation_silence_seconds_valid() {
        let mut prefs = Preferences::default();
        prefs.silence_seconds = 10.0;
        assert!(validate_preferences(&prefs).is_ok());
    }

    #[test]
    fn test_validation_throttle_too_high() {
        let mut prefs = Preferences::default();
        prefs.typing.throttle_ms = 2000;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("throttle_ms"));
    }

    #[test]
    fn test_validation_record_chunk_seconds_too_low() {
        let mut prefs = Preferences::default();
        prefs.record.chunk_seconds = 5;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("chunk_seconds"));
    }

    #[test]
    fn test_validation_record_max_hours_invalid() {
        let mut prefs = Preferences::default();
        prefs.record.max_hours = 48;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_hours"));
    }

    #[test]
    fn test_validation_record_max_file_gb_invalid() {
        let mut prefs = Preferences::default();
        prefs.record.max_file_gb = 32;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_file_gb"));
    }

    #[test]
    fn test_validation_voice_commands_empty_mappings() {
        let mut prefs = Preferences::default();
        prefs.voice_commands.enabled = true;
        prefs.voice_commands.map.newline = String::new();
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("voice command"));
    }

    #[test]
    fn test_validation_voice_commands_all_fields_checked() {
        let fields = [
            "newline",
            "new_paragraph",
            "tab",
            "period",
            "comma",
            "colon",
            "semicolon",
            "open_quote",
            "close_quote",
            "backtick",
            "code_block",
        ];
        for field in fields {
            let mut prefs = Preferences::default();
            prefs.voice_commands.enabled = true;
            match field {
                "newline" => prefs.voice_commands.map.newline = String::new(),
                "new_paragraph" => prefs.voice_commands.map.new_paragraph = String::new(),
                "tab" => prefs.voice_commands.map.tab = String::new(),
                "period" => prefs.voice_commands.map.period = String::new(),
                "comma" => prefs.voice_commands.map.comma = String::new(),
                "colon" => prefs.voice_commands.map.colon = String::new(),
                "semicolon" => prefs.voice_commands.map.semicolon = String::new(),
                "open_quote" => prefs.voice_commands.map.open_quote = String::new(),
                "close_quote" => prefs.voice_commands.map.close_quote = String::new(),
                "backtick" => prefs.voice_commands.map.backtick = String::new(),
                "code_block" => prefs.voice_commands.map.code_block = String::new(),
                _ => unreachable!(),
            }
            let result = validate_preferences(&prefs);
            assert!(
                result.is_err(),
                "Validation should fail when {} is empty",
                field
            );
        }
    }

    #[test]
    fn test_validation_voice_commands_disabled_ignores_mapping() {
        let mut prefs = Preferences::default();
        prefs.voice_commands.enabled = false;
        prefs.voice_commands.map.newline = String::new();
        assert!(validate_preferences(&prefs).is_ok());
    }

    #[test]
    fn test_model_profile_serialization() {
        let small_en = r#""small.en""#;
        let profile: ModelProfile = serde_json::from_str(small_en).unwrap();
        assert_eq!(profile, ModelProfile::EnglishSmall);

        let multi_small = r#""multilingual-small""#;
        let profile: ModelProfile = serde_json::from_str(multi_small).unwrap();
        assert_eq!(profile, ModelProfile::MultilingualSmall);

        let json = serde_json::to_string(&ModelProfile::MultilingualMedium).unwrap();
        assert_eq!(json, "\"multilingual-medium\"");
    }

    #[test]
    fn test_activation_mode_serialization() {
        let hold = r#""hold""#;
        let mode: ActivationMode = serde_json::from_str(hold).unwrap();
        assert_eq!(mode, ActivationMode::Hold);

        let toggle = r#""toggle""#;
        let mode: ActivationMode = serde_json::from_str(toggle).unwrap();
        assert_eq!(mode, ActivationMode::Toggle);

        let json = serde_json::to_string(&ActivationMode::Toggle).unwrap();
        assert_eq!(json, "\"toggle\"");
    }

    #[test]
    fn test_silence_rms_serialization() {
        let low = r#""low""#;
        let rms: SilenceRms = serde_json::from_str(low).unwrap();
        assert_eq!(rms, SilenceRms::Low);

        let json = serde_json::to_string(&SilenceRms::High).unwrap();
        assert_eq!(json, "\"high\"");
    }
    #[test]
    fn test_validation_both_hotkeys_disabled() {
        let mut prefs = Preferences::default();
        prefs.hotkeys.left_chord = false;
        prefs.hotkeys.right_chord = false;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_max_file_gb_exceeds_limit() {
        let mut prefs = Preferences::default();
        prefs.record.max_file_gb = 32;
        let result = validate_preferences(&prefs);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_silence_seconds_out_of_range() {
        let mut prefs = Preferences::default();
        prefs.silence_seconds = 100.0;
        assert!(validate_preferences(&prefs).is_err());

        prefs.silence_seconds = 0.1;
        assert!(validate_preferences(&prefs).is_err());
    }
}
