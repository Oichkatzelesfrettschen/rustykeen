#![doc = "KenKen puzzle UI built with GTK4 and Cairo"]
#![doc = ""]
#![doc = "This crate provides a modern, accessible desktop interface for KenKen puzzle solving"]
#![doc = "with CVD-aware color theming and keyboard-only navigation support."]

pub mod accessibility;
pub mod animator;
pub mod theme;
pub mod visualization;
pub mod widgets;

pub use animator::Animator;
pub use theme::ThemeEngine;
pub use visualization::GridRenderer;

/// Application state and top-level window manager
pub struct Application {
    theme_engine: ThemeEngine,
}

impl Application {
    /// Create a new KenKen application
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let theme_engine = ThemeEngine::new()?;
        Ok(Self { theme_engine })
    }

    /// Get the theme engine
    pub fn theme_engine(&self) -> &ThemeEngine {
        &self.theme_engine
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new().expect("Failed to initialize application")
    }
}
