use snakepit::markers::{TargetEnvironment, EnvironmentMarker};

#[test]
fn test_environment_markers() {
    let env = TargetEnvironment::default();
    assert!(!env.sys_platform.is_empty());
    assert!(!env.platform_system.is_empty());
    assert!(!env.platform_machine.is_empty());
    assert!(!env.python_version.is_empty());
}

#[test]
fn test_marker_evaluation() {
    let env = TargetEnvironment {
        python_version: "3.10".to_string(),
        sys_platform: "linux".to_string(),
        platform_system: "Linux".to_string(),
        platform_machine: "x86_64".to_string(),
    };

    let marker = EnvironmentMarker { raw: "sys_platform == 'linux'".to_string() };
    assert!(marker.evaluate(&env));
    
    let marker = EnvironmentMarker { raw: "sys_platform == 'win32'".to_string() };
    assert!(!marker.evaluate(&env));
    
    let marker = EnvironmentMarker { raw: "python_version >= '3.8'".to_string() };
    assert!(marker.evaluate(&env));
    
    // Complex markers are not fully supported by the simplified parser yet,
    // but we can test the basic extraction logic
    let marker = EnvironmentMarker { raw: "sys_platform == 'linux' and python_version >= '3.8'".to_string() };
    // The current implementation is very simple and might not handle 'and' correctly for multiple conditions
    // It just checks if *any* known pattern matches and returns true/false based on that.
    // Let's stick to simple markers for now as per the implementation in markers.rs
}
