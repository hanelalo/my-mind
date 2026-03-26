use anyhow::Result;
use arboard::Clipboard;
use tracing::{debug, info};

/// Manages clipboard operations with save/restore capability
pub struct ClipboardManager {
    saved_content: Option<String>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            saved_content: None,
        }
    }

    /// Save the current clipboard content
    pub fn save(&mut self) -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        self.saved_content = clipboard.get_text().ok();
        debug!("Saved clipboard content");
        Ok(())
    }

    /// Set text to clipboard
    pub fn set_text(&self, text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        info!("Set clipboard text ({} chars)", text.len());
        Ok(())
    }

    /// Restore the previously saved clipboard content
    pub fn restore(&mut self) -> Result<()> {
        if let Some(content) = self.saved_content.take() {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(&content)?;
            debug!("Restored clipboard content");
        }
        Ok(())
    }
}
