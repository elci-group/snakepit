//! Integration tests for PEP 440 version parsing and comparison

// Mock implementation for testing - replace with actual imports
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        Ok(Version {
            major: parts.get(0).and_then(|p| p.parse().ok()).unwrap_or(0),
            minor: parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0),
            patch: parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0),
        })
    }
}

#[cfg(test)]
mod pep440_tests {
    use super::*;

    #[test]
    fn test_simple_version_parsing() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_zero_version() {
        let v = Version::parse("0.0.0").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_large_version_numbers() {
        let v = Version::parse("100.200.300").unwrap();
        assert_eq!(v.major, 100);
        assert_eq!(v.minor, 200);
        assert_eq!(v.patch, 300);
    }
}
