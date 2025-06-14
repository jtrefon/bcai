//! BCAI - Blockchain AI Infrastructure
//!
//! A minimal working version for CI/CD

/// Main BCAI library
pub struct BCAI;

impl BCAI {
    pub fn new() -> Self {
        Self
    }

    pub fn version() -> &'static str {
        "0.1.0"
    }
}

impl Default for BCAI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcai_creation() {
        let _bcai = BCAI::new();
        assert_eq!(BCAI::version(), "0.1.0");
    }
}
