#![doc = "Settings dialog for theme and accessibility options"]

use gtk4::prelude::*;
use gtk4::{
    Adjustment, Box as GtkBox, Button, CheckButton, ComboBoxText, Label, Orientation, Scale, Window,
};
use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;

use crate::theme::ThemeVariant;

/// Settings state
#[derive(Debug, Clone)]
pub struct Settings {
    /// Current theme variant
    pub theme_variant: ThemeVariant,
    /// High contrast mode enabled
    pub high_contrast: bool,
    /// Text size multiplier (1.0 = normal, 1.5 = large, 2.0 = extra large)
    pub text_size: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_variant: ThemeVariant::Standard,
            high_contrast: false,
            text_size: 1.0,
        }
    }
}

/// Callback type for settings changes
pub type SettingsCallback = Box<dyn Fn(&Settings)>;

/// Settings dialog widget
pub struct SettingsDialog {
    window: Window,
    settings: Rc<RefCell<Settings>>,
    on_change: Rc<RefCell<Option<SettingsCallback>>>,
}

impl SettingsDialog {
    /// Create a new settings dialog
    pub fn new(parent: &Window) -> Self {
        let settings: Rc<RefCell<Settings>> = Rc::new(RefCell::new(Settings::default()));
        let on_change: Rc<RefCell<Option<SettingsCallback>>> = Rc::new(RefCell::new(None));

        let window = Window::builder()
            .transient_for(parent)
            .title("Settings")
            .default_width(400)
            .default_height(300)
            .modal(true)
            .build();

        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Theme variant selector
        let theme_label = Label::new(Some("Theme Variant:"));
        theme_label.set_halign(gtk4::Align::Start);
        content.append(&theme_label);

        let theme_combo = ComboBoxText::new();
        theme_combo.append(Some("standard"), "Standard (Color-sighted)");
        theme_combo.append(Some("protanopia"), "Protanopia (Red-blind)");
        theme_combo.append(Some("deuteranopia"), "Deuteranopia (Green-blind)");
        theme_combo.append(Some("tritanopia"), "Tritanopia (Blue-yellow)");
        theme_combo.append(Some("monochrome"), "Monochrome");
        theme_combo.set_active(Some(0));

        let settings_clone = Rc::clone(&settings);
        let on_change_clone = Rc::clone(&on_change);
        theme_combo.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                let variant = match id.as_str() {
                    "protanopia" => ThemeVariant::Protanopia,
                    "deuteranopia" => ThemeVariant::Deuteranopia,
                    "tritanopia" => ThemeVariant::Tritanopia,
                    "monochrome" => ThemeVariant::Monochrome,
                    _ => ThemeVariant::Standard,
                };

                let mut s = settings_clone.borrow_mut();
                s.theme_variant = variant;
                drop(s);

                if let Some(ref callback) = *on_change_clone.borrow() {
                    callback(&settings_clone.borrow());
                }
            }
        });
        content.append(&theme_combo);

        // High contrast toggle
        let contrast_box = GtkBox::new(Orientation::Vertical, 6);
        let contrast_label = Label::new(Some("Accessibility:"));
        contrast_label.set_halign(gtk4::Align::Start);
        contrast_box.append(&contrast_label);

        let high_contrast_check = CheckButton::with_label("High Contrast Mode");

        let settings_clone = Rc::clone(&settings);
        let on_change_clone = Rc::clone(&on_change);
        high_contrast_check.connect_toggled(move |check| {
            let mut s = settings_clone.borrow_mut();
            s.high_contrast = check.is_active();
            drop(s);

            if let Some(ref callback) = *on_change_clone.borrow() {
                callback(&settings_clone.borrow());
            }
        });
        contrast_box.append(&high_contrast_check);

        content.append(&contrast_box);

        // Text size slider
        let text_size_box = GtkBox::new(Orientation::Vertical, 6);
        let text_size_label = Label::new(Some("Text Size:"));
        text_size_label.set_halign(gtk4::Align::Start);
        text_size_box.append(&text_size_label);

        let size_adjustment = Adjustment::new(1.0, 0.8, 2.0, 0.1, 0.5, 0.0);
        let size_scale = Scale::new(Orientation::Horizontal, Some(&size_adjustment));
        size_scale.set_draw_value(true);
        size_scale.set_value_pos(gtk4::PositionType::Right);

        let settings_clone = Rc::clone(&settings);
        let on_change_clone = Rc::clone(&on_change);
        size_scale.connect_value_changed(move |scale| {
            let value = scale.value() as f32;
            let mut s = settings_clone.borrow_mut();
            s.text_size = value;
            drop(s);

            if let Some(ref callback) = *on_change_clone.borrow() {
                callback(&settings_clone.borrow());
            }
        });
        text_size_box.append(&size_scale);

        content.append(&text_size_box);

        // Close button
        let close_button = Button::with_label("Close");
        let window_weak = window.downgrade();
        close_button.connect_clicked(move |_| {
            if let Some(window) = window_weak.upgrade() {
                window.close();
            }
        });
        content.append(&close_button);

        window.set_child(Some(&content));

        Self {
            window,
            settings,
            on_change,
        }
    }

    /// Get the GTK4 window
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get current settings
    pub fn settings(&self) -> Settings {
        self.settings.borrow().clone()
    }

    /// Set a callback for settings changes
    pub fn on_settings_changed<F>(&self, callback: F)
    where
        F: Fn(&Settings) + 'static,
    {
        *self.on_change.borrow_mut() = Some(Box::new(callback));
    }

    /// Show the dialog
    pub fn show(&self) {
        self.window.present();
    }
}
