#![doc = "Theme engine with CVD-safe color variants"]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Color definition with OKLCH values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDefinition {
    pub oklch: (f32, f32, f32), // (lightness, chroma, hue)
}

/// Color token with variants for each CVD type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorToken {
    pub description: String,
    pub standard: ColorDefinition,
    pub protanopia: ColorDefinition,
    pub deuteranopia: ColorDefinition,
    pub tritanopia: ColorDefinition,
    pub monochrome: ColorDefinition,
}

impl ColorToken {
    /// Get color for current variant
    pub fn for_variant(&self, variant: ThemeVariant) -> &ColorDefinition {
        match variant {
            ThemeVariant::Standard => &self.standard,
            ThemeVariant::Protanopia => &self.protanopia,
            ThemeVariant::Deuteranopia => &self.deuteranopia,
            ThemeVariant::Tritanopia => &self.tritanopia,
            ThemeVariant::Monochrome => &self.monochrome,
        }
    }
}

/// Metadata for token file
#[derive(Debug, Serialize, Deserialize)]
struct TokenMetadata {
    name: String,
    version: String,
    #[serde(rename = "colorSpace")]
    color_space: String,
    description: String,
}

/// Root structure for tokens JSON file
#[derive(Debug, Serialize, Deserialize)]
struct TokensFile {
    metadata: TokenMetadata,
    colors: HashMap<String, ColorToken>,
}

/// CVD theme variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeVariant {
    /// Standard (color-sighted) palette
    Standard,
    /// Protanopia (red-blind) safe palette
    Protanopia,
    /// Deuteranopia (green-blind) safe palette
    Deuteranopia,
    /// Tritanopia (blue-yellow) safe palette
    Tritanopia,
    /// Monochrome (complete color blindness) safe palette
    Monochrome,
}

impl ThemeVariant {
    /// All available theme variants
    pub fn all() -> &'static [Self] {
        &[
            Self::Standard,
            Self::Protanopia,
            Self::Deuteranopia,
            Self::Tritanopia,
            Self::Monochrome,
        ]
    }
}

/// OKLCH color with perceptual uniformity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OklchColor {
    pub oklch: (f32, f32, f32), // (lightness, chroma, hue)
}

/// Theme engine managing OKLCH-based color palettes
pub struct ThemeEngine {
    current_variant: ThemeVariant,
    tokens: Option<HashMap<String, ColorToken>>,
}

impl ThemeEngine {
    /// Create a new theme engine
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            current_variant: ThemeVariant::Standard,
            tokens: None,
        })
    }

    /// Get current theme variant
    pub fn current_variant(&self) -> ThemeVariant {
        self.current_variant
    }

    /// Switch to a different theme variant
    pub fn set_variant(&mut self, variant: ThemeVariant) {
        self.current_variant = variant;
    }

    /// Load design tokens from JSON file
    pub fn load_tokens(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let tokens_json = std::fs::read_to_string(path)?;
        let tokens_file: TokensFile = serde_json::from_str(&tokens_json)?;
        self.tokens = Some(tokens_file.colors);
        Ok(())
    }

    /// Generate GTK4 CSS for current variant
    pub fn generate_css(&self) -> String {
        let mut css = String::from("@define-color theme_variant \"");
        css.push_str(match self.current_variant {
            ThemeVariant::Standard => "standard",
            ThemeVariant::Protanopia => "protanopia",
            ThemeVariant::Deuteranopia => "deuteranopia",
            ThemeVariant::Tritanopia => "tritanopia",
            ThemeVariant::Monochrome => "monochrome",
        });
        css.push_str("\";\n\n");

        if let Some(ref tokens) = self.tokens {
            css.push_str(&self.generate_color_css(tokens));
        }

        css.push_str(&self.generate_widget_css());
        css
    }

    fn generate_color_css(&self, tokens: &HashMap<String, ColorToken>) -> String {
        let mut css = String::from("/* Semantic color variables */\n");

        for (name, token) in tokens {
            let color = token.for_variant(self.current_variant);
            let (l, c, h) = color.oklch;
            let hex = self.oklch_to_hex(l, c, h);

            css.push_str(&format!("@define-color {} \"{}\";\n", name, hex));
        }

        css.push('\n');
        css
    }

    fn generate_widget_css(&self) -> String {
        String::from(
            r#"/* Widget styling */
window {
    background-color: @define-color background;
    color: @define-color text;
}

button {
    background-color: @define-color primary;
    color: white;
    border-radius: 4px;
    padding: 8px 16px;
}

button:hover {
    opacity: 0.8;
}

entry {
    background-color: @define-color surface;
    color: @define-color text;
    border: 1px solid @define-color border;
    padding: 6px;
}

/* KenKen-specific styling */
.kenken-grid {
    background-color: @define-color background;
}

.kenken-cell {
    border: 1px solid @define-color cage;
    background-color: @define-color surface;
}

.kenken-cell:selected {
    background-color: @define-color selectedCell;
}

.kenken-cage-border {
    color: @define-color cage;
    border-width: 2px;
}
"#,
        )
    }

    fn oklch_to_hex(&self, l: f32, c: f32, h: f32) -> String {
        let (r, g, b) = self.oklch_to_rgb(l, c, h);
        let ri = (r * 255.0) as u8;
        let gi = (g * 255.0) as u8;
        let bi = (b * 255.0) as u8;
        format!("#{:02x}{:02x}{:02x}", ri, gi, bi)
    }

    fn oklch_to_rgb(&self, l: f32, c: f32, h: f32) -> (f32, f32, f32) {
        let h_rad = h.to_radians();
        let a = c * h_rad.cos();
        let b = c * h_rad.sin();

        let lab_l = l;
        let lab_a = a;
        let lab_b = b;

        let var_x = self.lab_to_xyz_x(lab_l, lab_a);
        let var_y = self.lab_to_xyz_y(lab_l);
        let var_z = self.lab_to_xyz_z(lab_l, lab_b);

        let x = var_x / 0.95047;
        let y = var_y / 1.0;
        let z = var_z / 1.08883;

        let r = self.xyz_to_rgb_channel(x * 3.2406 + y * (-1.5372) + z * (-0.4986));
        let g = self.xyz_to_rgb_channel(x * (-0.9689) + y * 1.8758 + z * 0.0415);
        let b = self.xyz_to_rgb_channel(x * 0.0557 + y * (-0.204) + z * 1.057);

        (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }

    fn lab_to_xyz_x(&self, l: f32, a: f32) -> f32 {
        (a / 500.0 + self.lab_to_xyz_f_inv(l / 116.0)).powi(3)
    }

    fn lab_to_xyz_y(&self, l: f32) -> f32 {
        self.lab_to_xyz_f_inv(l / 116.0 + 0.02).powi(3)
    }

    fn lab_to_xyz_z(&self, l: f32, b: f32) -> f32 {
        (self.lab_to_xyz_f_inv(l / 116.0) - (b / 200.0)).powi(3)
    }

    fn lab_to_xyz_f_inv(&self, t: f32) -> f32 {
        if t > 0.206897 {
            t.powi(3)
        } else {
            (t - 16.0 / 116.0) / 7.787
        }
    }

    fn xyz_to_rgb_channel(&self, c: f32) -> f32 {
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }
}

impl Default for ThemeEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create theme engine")
    }
}
