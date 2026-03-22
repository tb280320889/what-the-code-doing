use wtcd_adapters::register_all_adapters;
use wtcd_core::types::ConfidenceBand;

fn fixture_path(relative: &str) -> String {
    format!(
        "{}/../../tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        relative
    )
}

#[test]
fn test_go_parse_core_features() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("sample.go").unwrap();

    let source = std::fs::read_to_string(fixture_path("go/sample.go")).unwrap();
    let result = adapter.parse(&source, "sample.go");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(result.exports.iter().any(|e| e.name == "Add"));
    assert!(result.exports.iter().any(|e| e.name == "Greeter"));
    assert!(result.exports.iter().any(|e| e.name == "Person"));
    assert!(result.exports.iter().any(|e| e.name == "Version"));
    assert!(result.imports.iter().any(|i| i.source == "fmt"));
    assert!(result.imports.iter().any(|i| i.source == "io"));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("go-meta:receiver:(p *Person)")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("go-meta:struct_field:Name string")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("go-meta:interface_method:Read")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("go-meta:embedded_struct:Base")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("visibility:Add:public")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("visibility:internalCounter:private")));
}

#[test]
fn test_go_side_effect_patterns() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("effects.go").unwrap();

    let source = std::fs::read_to_string(fixture_path("go/effects.go")).unwrap();
    let result = adapter.parse(&source, "effects.go");

    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("go-meta:goroutine")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("go-meta:channel")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("go-meta:directive:embed")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("go-meta:directive:generate")));
}

#[test]
fn test_go_syntax_error_graceful() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("broken.go").unwrap();

    let source = std::fs::read_to_string(fixture_path("go/syntax_error.go")).unwrap();
    let result = adapter.parse(&source, "broken.go");

    assert!(result.confidence == ConfidenceBand::Low || result.confidence == ConfidenceBand::None);
    assert!(result.error_message.is_some());
}
