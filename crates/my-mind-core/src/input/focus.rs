use anyhow::Result;
use tracing::{info, warn};

pub struct FocusManager;

impl FocusManager {
    /// Get the bundle identifier of the currently frontmost application.
    pub fn get_frontmost_app() -> Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            Self::get_frontmost_app_macos()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(None)
        }
    }

    /// Activate (bring to front) the application with the given bundle identifier.
    pub fn activate_app(bundle_id: &str) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            Self::activate_app_macos(bundle_id)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = bundle_id;
            Ok(())
        }
    }

    #[cfg(target_os = "macos")]
    fn get_frontmost_app_macos() -> Result<Option<String>> {
        use std::process::Command;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(
                r#"tell application "System Events" to get bundle identifier of first application process whose frontmost is true"#,
            )
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("[focus] Failed to get frontmost app: {}", stderr.trim());
            return Ok(None);
        }

        let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if bundle_id.is_empty() {
            return Ok(None);
        }

        info!("[focus] Captured frontmost app: {}", bundle_id);
        Ok(Some(bundle_id))
    }

    #[cfg(target_os = "macos")]
    fn activate_app_macos(bundle_id: &str) -> Result<()> {
        use std::process::Command;

        // Validate bundle id format to prevent injection
        if !bundle_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            anyhow::bail!(
                "Invalid bundle identifier format: {}",
                bundle_id
            );
        }

        let script = format!(
            r#"tell application id "{}" to activate"#,
            bundle_id
        );

        info!("[focus] Activating app: {}", bundle_id);

        let status = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .status()?;

        if status.success() {
            info!("[focus] App activated successfully");
            Ok(())
        } else {
            anyhow::bail!("Failed to activate app: {}", bundle_id)
        }
    }
}
