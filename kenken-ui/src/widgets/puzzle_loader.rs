#![doc = "Puzzle file loading and management"]

use gtk4::prelude::*;
use gtk4::{FileChooserDialog, FileChooserAction, ResponseType};
use std::cell::RefCell;
use std::rc::Rc;
use std::fs;

/// Callback type for file loaded
pub type FileLoadedCallback = Box<dyn Fn(&str)>;

/// Puzzle loader for file operations
pub struct PuzzleLoader {
    on_loaded: Rc<RefCell<Option<FileLoadedCallback>>>,
}

impl PuzzleLoader {
    /// Create a new puzzle loader
    pub fn new() -> Self {
        Self {
            on_loaded: Rc::new(RefCell::new(None)),
        }
    }

    /// Open file chooser dialog to load puzzle file
    pub fn open_file_dialog(&self, parent: &gtk4::Window) {
        let dialog = FileChooserDialog::new(
            Some("Open Puzzle File"),
            Some(parent),
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );

        let on_loaded = Rc::clone(&self.on_loaded);
        dialog.connect_response(move |d, resp| {
            if resp == ResponseType::Accept {
                if let Some(file) = d.file() {
                    if let Some(path) = file.path() {
                        // Attempt to load puzzle from file
                        match fs::read_to_string(&path) {
                            Ok(contents) => {
                                if let Some(ref callback) = *on_loaded.borrow() {
                                    callback(&contents);
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to read puzzle file: {}", e);
                            }
                        }
                    }
                }
            }
            d.close();
        });

        dialog.show();
    }

    /// Set callback for when puzzle file is loaded (receives file contents)
    pub fn on_file_loaded<F>(&self, callback: F)
    where
        F: Fn(&str) + 'static,
    {
        *self.on_loaded.borrow_mut() = Some(Box::new(callback));
    }

    /// Load puzzle from file path (non-interactive)
    pub fn load_from_path(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))
    }
}

impl Default for PuzzleLoader {
    fn default() -> Self {
        Self::new()
    }
}
