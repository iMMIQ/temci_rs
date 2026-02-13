use temci::utils::TemciError;

#[test]
fn test_error_display() {
    let err = TemciError::Config("test config error".to_string());
    assert_eq!(format!("{}", err), "Configuration error: test config error");
}

#[test]
fn test_run_error() {
    let err = TemciError::Run("command failed".to_string());
    assert!(matches!(err, TemciError::Run(_)));
}
