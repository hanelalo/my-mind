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

    /// macOS: Get the frontmost app using `lsappinfo`.
    /// Unlike osascript + System Events, this does not require Automation permissions.
    #[cfg(target_os = "macos")]
    fn get_frontmost_app_macos() -> Result<Option<String>> {
        use std::process::Command;

        // Step 1: Get the ASN (Application Serial Number) of the frontmost app
        let front_output = Command::new("lsappinfo").arg("front").output()?;
        let asn = String::from_utf8_lossy(&front_output.stdout).trim().to_string();
        if asn.is_empty() || asn == "0x0-0x0" {
            warn!("[focus] lsappinfo front returned empty or null ASN");
            return Ok(None);
        }

        // Step 2: Get the bundle ID for that ASN
        let info_output = Command::new("lsappinfo")
            .args(["info", "-only", "bundleid", &asn])
            .output()?;
        let info = String::from_utf8_lossy(&info_output.stdout).trim().to_string();

        // Parse output format: "bundleid"="com.tencent.xinWeChat"
        if let Some(start) = info.find("=\"") {
            let rest = &info[start + 2..];
            if let Some(end) = rest.find('"') {
                let bundle_id = &rest[..end];
                if !bundle_id.is_empty() {
                    info!("[focus] Captured frontmost app: {}", bundle_id);
                    return Ok(Some(bundle_id.to_string()));
                }
            }
        }

        warn!("[focus] Could not parse bundle ID from lsappinfo output: {}", info);
        Ok(None)
    }

    /// macOS: Activate app using `open -b`.
    /// Unlike osascript, this does not require Automation permissions.
    #[cfg(target_os = "macos")]
    fn activate_app_macos(bundle_id: &str) -> Result<()> {
        use std::process::Command;

        // Validate bundle id format to prevent injection
        if !bundle_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            anyhow::bail!("Invalid bundle identifier format: {}", bundle_id);
        }

        info!("[focus] Activating app: {}", bundle_id);

        let status = Command::new("open")
            .args(["-b", bundle_id])
            .status()?;

        if status.success() {
            info!("[focus] App activated successfully");
            Ok(())
        } else {
            anyhow::bail!("Failed to activate app: {}", bundle_id)
        }
    }
}
