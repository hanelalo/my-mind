use anyhow::Result;
use tracing::{info, warn};

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

    /// Request accessibility permission with system prompt dialog (macOS only).
    /// Returns true if already granted, false if user needs to grant.
    #[cfg(target_os = "macos")]
    pub fn request_accessibility_permission() -> bool {
        use core_foundation::base::TCFType;
        use core_foundation::boolean::CFBoolean;
        use core_foundation::dictionary::CFDictionary;
        use core_foundation::string::CFString;

        extern "C" {
            fn AXIsProcessTrustedWithOptions(
                options: core_foundation::dictionary::CFDictionaryRef,
            ) -> bool;
        }

        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let value = CFBoolean::true_value();
        let options = CFDictionary::from_CFType_pairs(&[(key, value)]);

        unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
    }

    /// Simulate paste shortcut (Cmd+V on macOS, Ctrl+V on others)
    pub fn paste() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            if !Self::has_accessibility_permission() {
                warn!("[paste] Accessibility permission NOT granted — CGEvent will be silently ignored");
                anyhow::bail!("Accessibility permission not granted. Please enable it in System Settings > Privacy & Security > Accessibility for My Mind.");
            }
            Self::paste_cg()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self::paste_enigo()
        }
    }

    /// Activate the target app and paste.
    pub fn activate_and_paste(bundle_id: &str) -> Result<()> {
        use super::FocusManager;

        info!("[paste] Activating {} then pasting...", bundle_id);

        FocusManager::activate_app(bundle_id)?;
        std::thread::sleep(std::time::Duration::from_millis(300));
        Self::paste()?;

        info!("[paste] Activate + paste completed successfully");
        Ok(())
    }

    /// macOS: Simulate Cmd+V using Core Graphics CGEvent API.
    /// Unlike enigo, CGEvent does not call TIS input source APIs
    /// and is safe to use from any thread (no crash in tokio context).
    #[cfg(target_os = "macos")]
    fn paste_cg() -> Result<()> {
        use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        // macOS virtual keycode for 'V'
        const KV_V: u16 = 0x09;

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| anyhow::anyhow!("Failed to create CGEventSource"))?;

        // Key down: V with Command modifier
        let key_down = CGEvent::new_keyboard_event(source.clone(), KV_V, true)
            .map_err(|_| anyhow::anyhow!("Failed to create key down event"))?;
        key_down.set_flags(CGEventFlags::CGEventFlagCommand);
        key_down.post(CGEventTapLocation::HID);

        // Key up: V with Command modifier
        let key_up = CGEvent::new_keyboard_event(source, KV_V, false)
            .map_err(|_| anyhow::anyhow!("Failed to create key up event"))?;
        key_up.set_flags(CGEventFlags::CGEventFlagCommand);
        key_up.post(CGEventTapLocation::HID);

        info!("Simulated Cmd+V paste via CGEvent");
        Ok(())
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
