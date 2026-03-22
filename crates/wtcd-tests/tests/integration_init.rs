#[test]
fn test_config_template_valid_yaml() {
    // Verify the default config template is valid YAML
    let config = wtcd_core::config::Config::from_yaml(
        r#"version: 1
repo_root: .
scope:
  source_roots:
    - src
  exclude_patterns: []
mirror: {}
output:
  format: json
"#,
    )
    .unwrap();

    assert_eq!(config.version, 1);
    assert_eq!(config.scope.source_roots, vec!["src"]);
}
