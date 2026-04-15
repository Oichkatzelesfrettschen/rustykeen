#![doc = "WCAG AAA contrast validation"]

use crate::theme::ColorDefinition;

/// WCAG AAA contrast ratio threshold for normal text
pub const WCAG_AAA_NORMAL_THRESHOLD: f32 = 7.0;

/// WCAG AAA contrast ratio threshold for large text
pub const WCAG_AAA_LARGE_THRESHOLD: f32 = 4.5;

/// WCAG AA contrast ratio threshold for normal text
pub const WCAG_AA_NORMAL_THRESHOLD: f32 = 4.5;

/// WCAG AA contrast ratio threshold for large text
pub const WCAG_AA_LARGE_THRESHOLD: f32 = 3.0;

/// Result of contrast validation
#[derive(Debug, Clone)]
pub struct ContrastResult {
    /// Calculated contrast ratio
    pub ratio: f32,
    /// Whether WCAG AAA normal text threshold is met
    pub wcag_aaa_normal: bool,
    /// Whether WCAG AAA large text threshold is met
    pub wcag_aaa_large: bool,
    /// Whether WCAG AA normal text threshold is met
    pub wcag_aa_normal: bool,
    /// Whether WCAG AA large text threshold is met
    pub wcag_aa_large: bool,
}

/// WCAG AAA contrast validator
pub struct ContrastValidator;

impl ContrastValidator {
    /// Calculate luminance of an OKLCH color using sRGB formula
    fn oklch_to_luminance(color: &ColorDefinition) -> f32 {
        let (l, c, h) = color.oklch;
        let (r, g, b) = Self::oklch_to_rgb(l, c, h);

        // sRGB luminance formula from WCAG
        let r_lin = Self::linearize_channel(r);
        let g_lin = Self::linearize_channel(g);
        let b_lin = Self::linearize_channel(b);

        0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
    }

    /// Convert sRGB component to linear RGB
    fn linearize_channel(c: f32) -> f32 {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Convert OKLCH to linear RGB (OkLab color space)
    fn oklch_to_rgb(l: f32, c: f32, h: f32) -> (f32, f32, f32) {
        // OKLCH to OkLab
        let h_rad = h.to_radians();
        let a = c * h_rad.cos();
        let b = c * h_rad.sin();

        // OkLab to linear RGB
        let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
        let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
        let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

        let l_cubed = l_ * l_ * l_;
        let m_cubed = m_ * m_ * m_;
        let s_cubed = s_ * s_ * s_;

        let r = 4.078_859_3 * l_cubed - 3.101_897_2 * m_cubed + 0.228_702_01 * s_cubed;
        let g = -1.062_985_2 * l_cubed + 2.412_037_1 * m_cubed - 0.238_478_76 * s_cubed;
        let b_out = -0.205_036_31 * l_cubed - 0.731_893_06 * m_cubed + 1.586_163_2 * s_cubed;

        (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b_out.clamp(0.0, 1.0))
    }

    /// Calculate contrast ratio between two colors (WCAG formula)
    /// Returns (L1 + 0.05) / (L2 + 0.05) where L1 >= L2 (lighter / darker)
    pub fn contrast_ratio(fg: &ColorDefinition, bg: &ColorDefinition) -> f32 {
        let l1 = Self::oklch_to_luminance(fg);
        let l2 = Self::oklch_to_luminance(bg);

        let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Validate contrast between two colors against all WCAG levels
    pub fn validate(fg: &ColorDefinition, bg: &ColorDefinition) -> ContrastResult {
        let ratio = Self::contrast_ratio(fg, bg);

        ContrastResult {
            ratio,
            wcag_aaa_normal: ratio >= WCAG_AAA_NORMAL_THRESHOLD,
            wcag_aaa_large: ratio >= WCAG_AAA_LARGE_THRESHOLD,
            wcag_aa_normal: ratio >= WCAG_AA_NORMAL_THRESHOLD,
            wcag_aa_large: ratio >= WCAG_AA_LARGE_THRESHOLD,
        }
    }

    /// Check if a contrast ratio meets WCAG AAA for normal text
    pub fn meets_wcag_aaa_normal(ratio: f32) -> bool {
        ratio >= WCAG_AAA_NORMAL_THRESHOLD
    }

    /// Check if a contrast ratio meets WCAG AAA for large text
    pub fn meets_wcag_aaa_large(ratio: f32) -> bool {
        ratio >= WCAG_AAA_LARGE_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_black_white_contrast() {
        let black = ColorDefinition {
            oklch: (0.0, 0.0, 0.0),
        };
        let white = ColorDefinition {
            oklch: (1.0, 0.0, 0.0),
        };

        let ratio = ContrastValidator::contrast_ratio(&white, &black);
        // Max contrast should be close to 21:1
        assert!(
            ratio > 20.0,
            "Black/white ratio should be >20, got {}",
            ratio
        );
    }

    #[test]
    fn test_wcag_aaa_validation() {
        let light = ColorDefinition {
            oklch: (0.95, 0.01, 260.0),
        };
        let dark = ColorDefinition {
            oklch: (0.10, 0.01, 260.0),
        };

        let result = ContrastValidator::validate(&dark, &light);
        assert!(
            result.wcag_aaa_normal,
            "Dark on light should meet WCAG AAA normal text (ratio: {})",
            result.ratio
        );
    }

    #[test]
    fn test_contrast_symmetry() {
        let color1 = ColorDefinition {
            oklch: (0.50, 0.20, 260.0),
        };
        let color2 = ColorDefinition {
            oklch: (0.80, 0.01, 260.0),
        };

        let ratio1 = ContrastValidator::contrast_ratio(&color1, &color2);
        let ratio2 = ContrastValidator::contrast_ratio(&color2, &color1);

        // Ratios should be identical regardless of order
        assert_eq!(ratio1, ratio2, "Contrast ratio should be symmetric");
    }

    #[test]
    fn test_luminance_calculation() {
        // Test known values
        let black = ColorDefinition {
            oklch: (0.0, 0.0, 0.0),
        };
        let lum = ContrastValidator::oklch_to_luminance(&black);
        assert!(lum < 0.01, "Black luminance should be near 0");
    }

    #[test]
    fn test_wcag_threshold_checks() {
        // 7.0 should pass AAA normal but not a hypothetical 8.0 threshold
        assert!(ContrastValidator::meets_wcag_aaa_normal(7.0));
        assert!(!ContrastValidator::meets_wcag_aaa_normal(6.9));
        assert!(ContrastValidator::meets_wcag_aaa_large(4.5));
        assert!(!ContrastValidator::meets_wcag_aaa_large(4.4));
    }
}
