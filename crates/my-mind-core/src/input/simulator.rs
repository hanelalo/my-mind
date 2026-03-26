use anyhow::Result;
use tracing::info;

/// Simulates keyboard input to paste text into the focused application
pub struct InputSimulator;

impl InputSimulator {
    /// Check if accessibility permission is granted (macOS only)
    pub fn has_accessibility_permission() -> bool {
        #[cfg(target_os = "macos")]
        {
            extern "C" {
                fn AXIsProcessTrusted() -> bool;
            }
            unsafe { AXIsProcessTrusted() }
        }
        #[cfg(not(target_os = "macos"))]
        {
            true
        }
    }

    /// Simulate paste shortcut
    pub fn paste() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            Self::paste_macos()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self::paste_enigo()
        }
    }

    /// Activate the target app and paste in a single osascript call (macOS).
    /// This is more reliable than separate activate + paste calls.
    pub fn activate_and_paste(bundle_id: &str) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            Self::activate_and_paste_macos(bundle_id)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = bundle_id;
            Self::paste_enigo()
        }
    }

    #[cfg(target_os = "macos")]
    fn paste_macos() -> Result<()> {
        use std::process::Command;

        let status = Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
            .status()?;

        if status.success() {
            info!("Simulated Cmd+V paste via osascript");
            Ok(())
        } else {
            anyhow::bail!("osascript paste failed with status: {}", status)
        }
    }

    #[cfg(target_os = "macos")]
    fn activate_and_paste_macos(bundle_id: &str) -> Result<()> {
        use std::process::Command;

        // Validate bundle id format to prevent injection
        if !bundle_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            anyhow::bail!("Invalid bundle identifier format: {}", bundle_id);
        }

        let script = format!(
            r#"
tell application id "{bundle_id}" to activate
delay 0.5
tell application "System Events"
    set frontmost of process (name of first application process whose bundle identifier is "{bundle_id}") to true
    delay 0.2
    keystroke "v" using command down
end tell
"#,
            bundle_id = bundle_id
        );

        info!("[paste] Activating {} and pasting via single osascript...", bundle_id);

        let status = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .status()?;

        if status.success() {
            info!("[paste] Activate + paste completed successfully");
            Ok(())
        } else {
            anyhow::bail!("Activate + paste failed for: {}", bundle_id)
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn paste_enigo() -> Result<()> {
        use enigo::{Enigo, Keyboard, Settings, Key, Direction};

        let mut enigo = Enigo::new(&Settings::default())?;
        std::thread::sleep(std::time::Duration::from_millis(100));

        enigo.key(Key::Control, Direction::Press)?;
        enigo.key(Key::Unicode('v'), Direction::Click)?;
        enigo.key(Key::Control, Direction::Release)?;

        info!("Simulated Ctrl+V paste via enigo");
        Ok(())
    }
}
