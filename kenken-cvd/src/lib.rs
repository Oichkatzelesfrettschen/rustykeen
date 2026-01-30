#![doc = "CVD color vision deficiency simulation for KenKen accessibility"]

use std::fmt;

// FFI bindings to libDaltonLens
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod ffi {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub enum DLDeficiency {
        Protan = 0,
        Deutan = 1,
        Tritan = 2,
    }

    unsafe extern "C" {
        pub fn dl_simulate_cvd(
            deficiency: DLDeficiency,
            severity: f32,
            srgba_image: *mut u8,
            width: usize,
            height: usize,
            bytes_per_row: usize,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CvdType {
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl CvdType {
    fn as_ffi(&self) -> ffi::DLDeficiency {
        match self {
            Self::Protanopia => ffi::DLDeficiency::Protan,
            Self::Deuteranopia => ffi::DLDeficiency::Deutan,
            Self::Tritanopia => ffi::DLDeficiency::Tritan,
        }
    }
}

impl fmt::Display for CvdType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Protanopia => "Protanopia",
            Self::Deuteranopia => "Deuteranopia",
            Self::Tritanopia => "Tritanopia",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidDimensions { width: u32, height: u32 },
    SimulationFailed,
    InvalidSeverity(f32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidDimensions { width, height } => {
                write!(f, "Bad dimensions: {}x{}", width, height)
            }
            Self::SimulationFailed => f.write_str("Sim completed with error"),
            Self::InvalidSeverity(v) => write!(f, "Bad severity: {}", v),
        }
    }
}

impl std::error::Error for Error {}

pub fn simulate_cvd(
    input_image: &mut [u8],
    width: u32,
    height: u32,
    cvd_type: CvdType,
    severity: f32,
) -> Result<(), Error> {
    if width == 0 || height == 0 {
        return Err(Error::InvalidDimensions { width, height });
    }

    if !(0.0..=1.0).contains(&severity) {
        return Err(Error::InvalidSeverity(severity));
    }

    let expected_len = (width as usize) * (height as usize) * 4;
    if input_image.len() != expected_len {
        return Err(Error::InvalidDimensions { width, height });
    }

    unsafe {
        ffi::dl_simulate_cvd(
            cvd_type.as_ffi(),
            severity,
            input_image.as_mut_ptr(),
            width as usize,
            height as usize,
            (width as usize) * 4,
        );
    }

    Ok(())
}

pub fn all_cvd_types() -> &'static [CvdType] {
    &[
        CvdType::Protanopia,
        CvdType::Deuteranopia,
        CvdType::Tritanopia,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_cvd_small_image() {
        let mut image = vec![
            0xFF, 0x00, 0x00, 0xFF, // Red
            0x00, 0xFF, 0x00, 0xFF, // Green
            0x00, 0x00, 0xFF, 0xFF, // Blue
            0xFF, 0xFF, 0xFF, 0xFF, // White
        ];

        let original_red = image[0];
        let result = simulate_cvd(&mut image, 2, 2, CvdType::Protanopia, 1.0);
        assert!(result.is_ok());

        assert_ne!(image[0], original_red);
    }

    #[test]
    fn test_invalid_dimensions_zero() {
        let mut image = vec![0xFF; 16];
        let result = simulate_cvd(&mut image, 0, 2, CvdType::Protanopia, 1.0);
        assert!(matches!(result, Err(Error::InvalidDimensions { .. })));
    }

    #[test]
    fn test_invalid_size_mismatch() {
        let mut image = vec![0xFF; 8];
        let result = simulate_cvd(&mut image, 2, 2, CvdType::Protanopia, 1.0);
        assert!(matches!(result, Err(Error::InvalidDimensions { .. })));
    }

    #[test]
    fn test_invalid_severity() {
        let mut image = vec![0xFF; 16];
        let result = simulate_cvd(&mut image, 2, 2, CvdType::Deuteranopia, 1.5);
        assert!(matches!(result, Err(Error::InvalidSeverity(1.5))));
    }

    #[test]
    fn test_severity_range() {
        let mut image = vec![0xFF; 16];

        let result = simulate_cvd(&mut image, 2, 2, CvdType::Tritanopia, 0.0);
        assert!(result.is_ok());

        let mut image = vec![0xFF; 16];
        let result = simulate_cvd(&mut image, 2, 2, CvdType::Tritanopia, 1.0);
        assert!(result.is_ok());

        let mut image = vec![0xFF; 16];
        let result = simulate_cvd(&mut image, 2, 2, CvdType::Tritanopia, 0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_cvd_types() {
        for &cvd_type in all_cvd_types() {
            let mut image = vec![0xAA; 16];
            let result = simulate_cvd(&mut image, 2, 2, cvd_type, 1.0);
            assert!(result.is_ok());
        }
    }
}
