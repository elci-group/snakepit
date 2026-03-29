use snakepit::pep440::{Version, VersionSpecifier};
use std::str::FromStr;

#[test]
fn test_version_parsing() {
    let v = Version::from_str("1.2.3").unwrap();
    assert_eq!(v.to_string(), "1.2.3");
    assert!(!v.is_prerelease());

    let v = Version::from_str("1.2.3a1").unwrap();
    assert_eq!(v.to_string(), "1.2.3a1");
    assert!(v.is_prerelease());
    
    let v = Version::from_str("1.2.3.post1").unwrap();
    assert_eq!(v.to_string(), "1.2.3.post1");
    
    let v = Version::from_str("1.2.3.dev1").unwrap();
    assert_eq!(v.to_string(), "1.2.3.dev1");
}

#[test]
fn test_version_comparison() {
    let v1 = Version::from_str("1.2.3").unwrap();
    let v2 = Version::from_str("1.2.4").unwrap();
    assert!(v1 < v2);
    
    let v1 = Version::from_str("1.2.3a1").unwrap();
    let v2 = Version::from_str("1.2.3").unwrap();
    assert!(v1 < v2); // Pre-release is older than release
    
    let v1 = Version::from_str("1.2.3").unwrap();
    let v2 = Version::from_str("1.2.3.post1").unwrap();
    assert!(v1 < v2); // Post-release is newer
}

#[test]
fn test_version_specifiers() {
    let v = Version::from_str("1.2.3").unwrap();
    
    let spec = VersionSpecifier::from_str("==1.2.3").unwrap();
    assert!(spec.matches(&v));
    
    let spec = VersionSpecifier::from_str(">=1.2.0").unwrap();
    assert!(spec.matches(&v));
    
    let spec = VersionSpecifier::from_str("<2.0.0").unwrap();
    assert!(spec.matches(&v));
    
    let spec = VersionSpecifier::from_str("!=1.2.4").unwrap();
    assert!(spec.matches(&v));
    
    let spec = VersionSpecifier::from_str("~=1.2").unwrap();
    assert!(spec.matches(&v));
}

#[test]
fn test_complex_specifiers() {
    let v = Version::from_str("1.2.3").unwrap();
    
    // Comma separated
    let spec = VersionSpecifier::from_str(">=1.0.0, <2.0.0").unwrap();
    assert!(spec.matches(&v));
    
    let spec = VersionSpecifier::from_str(">=1.5.0, <2.0.0").unwrap();
    assert!(!spec.matches(&v));
}
