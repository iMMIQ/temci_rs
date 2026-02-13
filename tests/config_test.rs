use temci::utils::config::RunConfig;

#[test]
fn test_parse_run_config() {
    let yaml = r#"
- attributes: {description: "ls"}
  data:
    etime: [0.02, 0.03, 0.025]
"#;
    let result = RunConfig::from_yaml_str(yaml).unwrap();
    assert_eq!(result.suites.len(), 1);
    assert_eq!(
        result.suites[0].attributes.get("description"),
        Some(&"ls".to_string())
    );
}
