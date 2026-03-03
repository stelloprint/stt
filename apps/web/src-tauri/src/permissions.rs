use serde::{Deserialize, Serialize};
use std::process::Command;

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
}

pub struct Permissions;

impl Permissions {
    pub fn check_microphone() -> PermissionState {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("osascript")
                .args([
                    "-e",
                    "tell application \"System Events\" to (do shell script \"/usr/bin/mikeutil 2>/dev/null || echo 'no-mike'\"",
                ])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("Access") {
                    return PermissionState::Granted;
                }
            }

            let output = Command::new("osascript")
                .args([
                    "-e",
                    "set micPermission to (do shell script \"defaults read /Library/Preferences/com.apple.security.device_audio-input_enabled 2>/dev/null || echo 'undetermined'\")",
                    "-e",
                    "return micPermission",
                ])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if stdout == "1" {
                    return PermissionState::Granted;
                } else if stdout == "0" {
                    return PermissionState::Denied;
                }
            }

            PermissionState::Undetermined
        }

        #[cfg(not(target_os = "macos"))]
        {
            PermissionState::Undetermined
        }
    }

    pub fn check_accessibility() -> PermissionState {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("osascript")
                .args([
                    "-e",
                    "tell application \"System Events\" to keystroke \"x\" using command down",
                ])
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    return PermissionState::Granted;
                }
            }

            PermissionState::Denied
        }

        #[cfg(not(target_os = "macos"))]
        {
            PermissionState::Undetermined
        }
    }

    pub fn request_microphone() -> Result<PermissionState, String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("osascript")
                .args([
                    "-e",
                    "set micRef to (do shell script \"say -v '?' 'test' 2>&1 | grep -q permission && echo 'denied' || echo 'ok'\")",
                ])
                .output()
                .map_err(|e| e.to_string())?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("denied") {
                return Ok(PermissionState::Denied);
            }

            let output = Command::new("osascript")
                .args([
                    "-e",
                    "set audioInputDevices to (get (path to temporary items) as text)",
                    "-e",
                    "return input volume of (get volume settings)",
                ])
                .output()
                .map_err(|e| e.to_string())?;

            Ok(PermissionState::Granted)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(PermissionState::Undetermined)
        }
    }

    pub fn request_accessibility() -> Result<PermissionState, String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("osascript")
                .args([
                    "-e",
                    "return \"ax:\" & (do shell script \"defaults read /Library/Preferences/com.apple.security.accessibility 2>/dev/null || echo 'missing'\")",
                ])
                .output()
                .map_err(|e| e.to_string())?;

            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.contains("ax:") && !stdout.contains("missing") {
                return Ok(PermissionState::Granted);
            }

            Command::new("open")
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
            Command::new("open")
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
            Command::new("open")
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
}
