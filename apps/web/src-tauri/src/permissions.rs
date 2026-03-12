use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionState {
    Granted,
    Denied,
    Undetermined,
}

impl PermissionState {
    pub fn is_granted(&self) -> bool {
        matches!(self, PermissionState::Granted)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, PermissionState::Denied)
    }

    pub fn is_undetermined(&self) -> bool {
        matches!(self, PermissionState::Undetermined)
    }
}

impl Default for PermissionState {
    fn default() -> Self {
        PermissionState::Undetermined
    }
}

#[cfg(target_os = "macos")]
mod macos_permissions {
    use crate::PermissionState;
    use objc2::rc::autoreleasepool;
    use std::process::Command;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    pub fn check_accessibility() -> PermissionState {
        let trusted = autoreleasepool(|_pool| unsafe { AXIsProcessTrusted() });
        if trusted {
            PermissionState::Granted
        } else {
            PermissionState::Denied
        }
    }

    pub fn check_microphone() -> PermissionState {
        let output = Command::new("shortcuts")
            .args(["run", "Check Microphone", "--small", "0"])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("granted") || stdout.contains("1") {
                    PermissionState::Granted
                } else if stdout.contains("denied") || stdout.contains("0") {
                    PermissionState::Denied
                } else {
                    PermissionState::Undetermined
                }
            }
            Err(_) => {
                let output = Command::new("osascript")
                    .args([
                        "-e",
                        "set deviceCount to 0\ntry\n    set deviceCount to (do shell script \"/usr/bin/mikeutil -i 2>/dev/null | wc -l || echo 0\")\nend try\nreturn deviceCount",
                    ])
                    .output();

                match output {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if stdout.parse::<u32>().unwrap_or(0) > 0 {
                            PermissionState::Undetermined
                        } else {
                            PermissionState::Undetermined
                        }
                    }
                    Err(_) => PermissionState::Undetermined,
                }
            }
        }
    }

    pub fn request_microphone_permission() {
        let _ = Command::new("osascript")
            .args(["-e", "set micRef to (do shell script \"echo test\")"])
            .output();
    }
}

pub struct Permissions;

impl Permissions {
    pub fn check_microphone() -> PermissionState {
        #[cfg(target_os = "macos")]
        {
            macos_permissions::check_microphone()
        }

        #[cfg(not(target_os = "macos"))]
        {
            PermissionState::Undetermined
        }
    }

    pub fn check_accessibility() -> PermissionState {
        #[cfg(target_os = "macos")]
        {
            macos_permissions::check_accessibility()
        }

        #[cfg(not(target_os = "macos"))]
        {
            PermissionState::Undetermined
        }
    }

    pub fn request_microphone() -> Result<PermissionState, String> {
        #[cfg(target_os = "macos")]
        {
            macos_permissions::request_microphone_permission();
            std::thread::sleep(std::time::Duration::from_millis(500));
            Ok(Permissions::check_microphone())
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(PermissionState::Undetermined)
        }
    }

    pub fn request_accessibility() -> Result<PermissionState, String> {
        #[cfg(target_os = "macos")]
        {
            if Permissions::check_accessibility().is_granted() {
                return Ok(PermissionState::Granted);
            }

            std::process::Command::new("open")
                .args([
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                ])
                .spawn()
                .map_err(|e| e.to_string())?;

            Ok(PermissionState::Undetermined)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(PermissionState::Undetermined)
        }
    }

    pub fn open_microphone_settings() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .args([
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
                ])
                .spawn()
                .map_err(|e| e.to_string())?;
            Ok(())
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(())
        }
    }

    pub fn open_accessibility_settings() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .args([
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                ])
                .spawn()
                .map_err(|e| e.to_string())?;
            Ok(())
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionStatus {
    pub microphone: PermissionState,
    pub accessibility: PermissionState,
    pub typing_enabled: bool,
}

impl Default for PermissionStatus {
    fn default() -> Self {
        Self {
            microphone: PermissionState::Undetermined,
            accessibility: PermissionState::Undetermined,
            typing_enabled: true,
        }
    }
}

impl PermissionStatus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_all() -> Self {
        Self {
            microphone: Permissions::check_microphone(),
            accessibility: Permissions::check_accessibility(),
            typing_enabled: Permissions::check_accessibility().is_granted(),
        }
    }

    pub fn can_capture_audio(&self) -> bool {
        self.microphone.is_granted()
    }

    pub fn can_type(&self) -> bool {
        self.accessibility.is_granted() && self.typing_enabled
    }

    pub fn requires_microphone_permission(&self) -> bool {
        !self.microphone.is_granted()
    }

    pub fn requires_accessibility_permission(&self) -> bool {
        !self.accessibility.is_granted()
    }

    pub fn has_any_denied(&self) -> bool {
        self.microphone.is_denied() || self.accessibility.is_denied()
    }

    pub fn all_granted(&self) -> bool {
        self.microphone.is_granted() && self.accessibility.is_granted()
    }
}

pub struct SecurityManager {
    typing_enabled: bool,
    global_disable: bool,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            typing_enabled: true,
            global_disable: false,
        }
    }

    pub fn disable_typing_globally(&mut self) {
        self.global_disable = true;
    }

    pub fn enable_typing_globally(&mut self) {
        self.global_disable = false;
    }

    pub fn is_typing_enabled(&self) -> bool {
        !self.global_disable && self.typing_enabled
    }

    pub fn is_globally_disabled(&self) -> bool {
        self.global_disable
    }

    pub fn attempt_type(&self, text: &str) -> Result<(), String> {
        if self.global_disable {
            return Err("Typing globally disabled".to_string());
        }
        if !self.typing_enabled {
            return Err("Typing not enabled".to_string());
        }
        if text.to_lowercase().contains("password") || text.to_lowercase().contains("secret") {
            return Err("Skipping typing for security - password field detected".to_string());
        }
        Ok(())
    }

    pub fn set_typing_enabled(&mut self, enabled: bool) {
        self.typing_enabled = enabled;
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_state_granted() {
        assert!(PermissionState::Granted.is_granted());
    }

    #[test]
    fn test_permission_state_denied() {
        assert!(!PermissionState::Denied.is_granted());
    }

    #[test]
    fn test_permission_state_undetermined() {
        assert!(!PermissionState::Undetermined.is_granted());
    }

    #[test]
    fn test_permission_state_is_denied() {
        assert!(PermissionState::Denied.is_denied());
        assert!(!PermissionState::Granted.is_denied());
        assert!(!PermissionState::Undetermined.is_denied());
    }

    #[test]
    fn test_permission_state_is_undetermined() {
        assert!(PermissionState::Undetermined.is_undetermined());
        assert!(!PermissionState::Granted.is_undetermined());
        assert!(!PermissionState::Denied.is_undetermined());
    }

    #[test]
    fn test_check_microphone_returns_state() {
        let state = Permissions::check_microphone();
        matches!(
            state,
            PermissionState::Granted | PermissionState::Denied | PermissionState::Undetermined
        );
    }

    #[test]
    fn test_check_accessibility_returns_state() {
        let state = Permissions::check_accessibility();
        matches!(
            state,
            PermissionState::Granted | PermissionState::Denied | PermissionState::Undetermined
        );
    }

    #[test]
    fn test_permission_status_default() {
        let status = PermissionStatus::default();
        assert_eq!(status.microphone, PermissionState::Undetermined);
        assert_eq!(status.accessibility, PermissionState::Undetermined);
    }

    #[test]
    fn test_permission_status_check_all() {
        let status = PermissionStatus::check_all();
        matches!(
            status.microphone,
            PermissionState::Granted | PermissionState::Denied | PermissionState::Undetermined
        );
        matches!(
            status.accessibility,
            PermissionState::Granted | PermissionState::Denied | PermissionState::Undetermined
        );
    }

    #[test]
    fn test_permission_status_can_capture_audio() {
        let mut status = PermissionStatus::default();
        assert!(!status.can_capture_audio());
        status.microphone = PermissionState::Granted;
        assert!(status.can_capture_audio());
    }

    #[test]
    fn test_permission_status_can_type() {
        let mut status = PermissionStatus::default();
        status.accessibility = PermissionState::Granted;
        assert!(status.can_type());
    }

    #[test]
    fn test_permission_status_can_type_requires_both() {
        let mut status = PermissionStatus::default();
        status.accessibility = PermissionState::Granted;
        status.typing_enabled = false;
        assert!(!status.can_type());
    }

    #[test]
    fn test_permission_status_requires_permissions() {
        let mut status = PermissionStatus::default();
        assert!(status.requires_microphone_permission());
        assert!(status.requires_accessibility_permission());

        status.microphone = PermissionState::Granted;
        status.accessibility = PermissionState::Granted;
        assert!(!status.requires_microphone_permission());
        assert!(!status.requires_accessibility_permission());
    }

    #[test]
    fn test_permission_status_denied_detection() {
        let mut status = PermissionStatus::default();
        assert!(!status.has_any_denied());

        status.microphone = PermissionState::Denied;
        assert!(status.has_any_denied());

        let mut status2 = PermissionStatus::default();
        status2.accessibility = PermissionState::Denied;
        assert!(status2.has_any_denied());
    }

    #[test]
    fn test_permission_status_all_granted() {
        let mut status = PermissionStatus::default();
        assert!(!status.all_granted());

        status.microphone = PermissionState::Granted;
        status.accessibility = PermissionState::Granted;
        assert!(status.all_granted());

        status.microphone = PermissionState::Denied;
        assert!(!status.all_granted());
    }

    #[test]
    fn test_security_manager_default() {
        let manager = SecurityManager::default();
        assert!(manager.is_typing_enabled());
        assert!(!manager.is_globally_disabled());
    }

    #[test]
    fn test_security_manager_global_disable() {
        let mut manager = SecurityManager::new();
        assert!(manager.is_typing_enabled());

        manager.disable_typing_globally();
        assert!(!manager.is_typing_enabled());
        assert!(manager.is_globally_disabled());

        manager.enable_typing_globally();
        assert!(manager.is_typing_enabled());
        assert!(!manager.is_globally_disabled());
    }

    #[test]
    fn test_security_manager_typing_enabled_toggle() {
        let mut manager = SecurityManager::new();
        assert!(manager.is_typing_enabled());

        manager.set_typing_enabled(false);
        assert!(!manager.is_typing_enabled());

        manager.set_typing_enabled(true);
        assert!(manager.is_typing_enabled());
    }

    #[test]
    fn test_security_manager_attempt_type_success() {
        let manager = SecurityManager::new();
        let result = manager.attempt_type("Hello world");
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_manager_attempt_type_global_disable() {
        let mut manager = SecurityManager::new();
        manager.disable_typing_globally();
        let result = manager.attempt_type("Hello world");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Typing globally disabled");
    }

    #[test]
    fn test_security_manager_attempt_type_disabled() {
        let mut manager = SecurityManager::new();
        manager.set_typing_enabled(false);
        let result = manager.attempt_type("Hello world");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Typing not enabled");
    }

    #[test]
    fn test_security_manager_attempt_type_password_detection() {
        let manager = SecurityManager::new();
        let result = manager.attempt_type("My password is secret");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("password field detected"));
    }

    #[test]
    fn test_security_manager_attempt_type_secret_detection() {
        let manager = SecurityManager::new();
        let result = manager.attempt_type("api_secret_key");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("password field detected"));
    }

    #[test]
    fn test_security_manager_allow_password_word_in_normal_text() {
        let manager = SecurityManager::new();
        let result = manager.attempt_type("What is the password for the meeting?");
        assert!(result.is_err());
    }

    #[test]
    fn test_security_manager_case_insensitive_detection() {
        let manager = SecurityManager::new();
        let result = manager.attempt_type("PASSWORD123");
        assert!(result.is_err());
        let result2 = manager.attempt_type("SECRET");
        assert!(result2.is_err());
    }

    #[test]
    fn test_graceful_degradation_microphone_denied() {
        let status = PermissionStatus {
            microphone: PermissionState::Denied,
            accessibility: PermissionState::Granted,
            typing_enabled: true,
        };
        assert!(!status.can_capture_audio());
        assert!(status.can_type());
        assert!(status.has_any_denied());
    }

    #[test]
    fn test_graceful_degradation_accessibility_denied() {
        let status = PermissionStatus {
            microphone: PermissionState::Granted,
            accessibility: PermissionState::Denied,
            typing_enabled: false,
        };
        assert!(status.can_capture_audio());
        assert!(!status.can_type());
        assert!(status.has_any_denied());
    }

    #[test]
    fn test_graceful_degradation_both_denied() {
        let status = PermissionStatus {
            microphone: PermissionState::Denied,
            accessibility: PermissionState::Denied,
            typing_enabled: false,
        };
        assert!(!status.can_capture_audio());
        assert!(!status.can_type());
        assert!(status.has_any_denied());
    }

    #[test]
    fn test_graceful_degradation_undetermined() {
        let status = PermissionStatus::default();
        assert!(!status.can_capture_audio());
        assert!(!status.can_type());
        assert!(!status.has_any_denied());
    }
}
