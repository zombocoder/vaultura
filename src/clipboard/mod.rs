use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use arboard::Clipboard;

use crate::error::{Result, VaulturaError};

pub struct ClipboardManager {
    clear_seconds: u64,
    /// Tracks the generation count so stale clear-threads don't wipe newer clipboard content.
    generation: Arc<Mutex<u64>>,
}

impl ClipboardManager {
    pub fn new(clear_seconds: u64) -> Self {
        Self {
            clear_seconds,
            generation: Arc::new(Mutex::new(0)),
        }
    }

    /// Copy text to clipboard and schedule an auto-clear after `clear_seconds`.
    pub fn copy_and_clear(&self, text: &str) -> Result<()> {
        let mut clipboard =
            Clipboard::new().map_err(|e| VaulturaError::Clipboard(e.to_string()))?;
        clipboard
            .set_text(text)
            .map_err(|e| VaulturaError::Clipboard(e.to_string()))?;

        let gen = {
            let mut g = self.generation.lock().unwrap();
            *g += 1;
            *g
        };

        let clear_seconds = self.clear_seconds;
        let generation = Arc::clone(&self.generation);

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(clear_seconds));
            let current_gen = *generation.lock().unwrap();
            if current_gen == gen {
                if let Ok(mut cb) = Clipboard::new() {
                    let _ = cb.set_text("");
                }
            }
        });

        Ok(())
    }
}
