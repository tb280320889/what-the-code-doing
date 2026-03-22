use wtcd_adapters::register_all_adapters;
use wtcd_core::types::ConfidenceBand;

/// Build absolute path to fixture file relative to workspace root.
/// CARGO_MANIFEST_DIR is crates/wtcd-tests/, fixtures are at ../../tests/fixtures/
fn fixture_path(relative: &str) -> String {
    format!(
        "{}/../../tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        relative
    )
}

#[test]
fn test_basic_exports_extraction() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.ts").unwrap();

    let source = std::fs::read_to_string(fixture_path("ts/basic_exports.ts")).unwrap();
    let result = adapter.parse(&source, "test.ts");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(
        result.exports.len() >= 3,
        "Expected at least 3 exports, got {}",
        result.exports.len()
    );

    let names: Vec<&str> = result.exports.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"greet"), "Missing export: greet");
    assert!(
        names.contains(&"UserService"),
        "Missing export: UserService"
    );
    assert!(
        names.contains(&"MAX_RETRIES"),
        "Missing export: MAX_RETRIES"
    );
}

#[test]
fn test_type_exports_extraction() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.ts").unwrap();

    let source = std::fs::read_to_string(fixture_path("ts/type_exports.ts")).unwrap();
    let result = adapter.parse(&source, "test.ts");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(
        result.exports.len() >= 3,
        "Expected at least 3 exports, got {}",
        result.exports.len()
    );

    let names: Vec<&str> = result.exports.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"UserId"), "Missing export: UserId");
    assert!(names.contains(&"AppConfig"), "Missing export: AppConfig");
    assert!(names.contains(&"Status"), "Missing export: Status");
}

#[test]
fn test_named_imports_extraction() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.ts").unwrap();

    let source = std::fs::read_to_string(fixture_path("ts/named_imports.ts")).unwrap();
    let result = adapter.parse(&source, "test.ts");

    assert!(
        result.imports.len() >= 2,
        "Expected at least 2 imports, got {}",
        result.imports.len()
    );
}

#[test]
fn test_commonjs_require() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.ts").unwrap();

    let source = std::fs::read_to_string(fixture_path("ts/commonjs.ts")).unwrap();
    let result = adapter.parse(&source, "test.ts");

    assert!(
        result.imports.len() >= 2,
        "Expected at least 2 require imports, got {}",
        result.imports.len()
    );
    let sources: Vec<&str> = result.imports.iter().map(|i| i.source.as_str()).collect();
    assert!(sources.contains(&"fs"), "Missing require source: fs");
    assert!(sources.contains(&"path"), "Missing require source: path");
}

#[test]
fn test_side_effects_detection() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.ts").unwrap();

    let source = std::fs::read_to_string(fixture_path("ts/side_effects.ts")).unwrap();
    let result = adapter.parse(&source, "test.ts");

    assert!(
        result.side_effects.len() >= 3,
        "Expected at least 3 side effects, got {}",
        result.side_effects.len()
    );

    let targets: Vec<&str> = result
        .side_effects
        .iter()
        .map(|s| s.target.as_str())
        .collect();
    assert!(
        targets.iter().any(|t| t.contains("fs")),
        "Missing fs side effect"
    );
    assert!(
        targets.iter().any(|t| t.contains("fetch")),
        "Missing fetch side effect"
    );
    assert!(
        targets.iter().any(|t| t.contains("console")),
        "Missing console side effect"
    );
}

#[test]
fn test_broken_syntax_graceful() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.ts").unwrap();

    let source = std::fs::read_to_string(fixture_path("ts/broken_syntax.ts")).unwrap();
    let result = adapter.parse(&source, "test.ts");

    // Should NOT crash, should have degraded confidence
    assert!(
        result.confidence == ConfidenceBand::Low || result.confidence == ConfidenceBand::None,
        "Expected Low or None confidence for broken file, got {:?}",
        result.confidence
    );
}

#[test]
fn test_function_signatures() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.ts").unwrap();

    let source = std::fs::read_to_string(fixture_path("ts/function_signatures.ts")).unwrap();
    let result = adapter.parse(&source, "test.ts");

    assert!(
        result.signatures.len() >= 2,
        "Expected at least 2 signatures, got {}",
        result.signatures.len()
    );

    let add_sig = result
        .signatures
        .iter()
        .find(|s| s.name == "add")
        .expect("Missing 'add' signature");
    assert_eq!(add_sig.parameters.len(), 2, "add should have 2 parameters");
    assert_eq!(add_sig.return_type, "number", "add should return number");
}

#[test]
fn test_js_file_parsing() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.js").unwrap();

    let source = std::fs::read_to_string(fixture_path("js/basic_exports.js")).unwrap();
    let result = adapter.parse(&source, "test.js");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(
        result.exports.len() >= 3,
        "Expected at least 3 JS exports, got {}",
        result.exports.len()
    );
}

#[test]
fn test_json_output_serializable() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("test.ts").unwrap();

    let source = std::fs::read_to_string(fixture_path("ts/basic_exports.ts")).unwrap();
    let file_result = adapter.parse(&source, "test.ts");

    let output = wtcd_core::types::RunOutput {
        api_version: "1".to_string(),
        files: vec![file_result],
        errors: vec![],
        summary: wtcd_core::types::RunSummary {
            total_files: 1,
            parsed_ok: 1,
            confidence_low: 0,
            confidence_none: 0,
            total_exports: 3,
            total_imports: 0,
            elapsed_ms: 0,
        },
    };

    let json = serde_json::to_string(&output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["api_version"], "1");
    assert!(parsed["files"].is_array());
    assert!(parsed["summary"].is_object());
}

#[test]
fn test_python_function_extraction() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.py").unwrap();

    let source = std::fs::read_to_string(fixture_path("python/hello.py")).unwrap();
    let result = adapter.parse(&source, "hello.py");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(result.exports.iter().any(|e| e.name == "greet"));
    assert!(result.exports.iter().any(|e| e.name == "add"));

    let greet = result
        .signatures
        .iter()
        .find(|s| s.name == "greet")
        .expect("Missing greet signature");
    assert!(greet.parameters.iter().any(|p| p.name == "name"));
    assert!(greet.return_type.contains("str") || greet.return_type == "unknown");
}

#[test]
fn test_python_class_and_inheritance() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("classes.py").unwrap();

    let source = std::fs::read_to_string(fixture_path("python/classes.py")).unwrap();
    let result = adapter.parse(&source, "classes.py");

    assert!(result
        .exports
        .iter()
        .any(|e| e.name == "Animal" && e.kind == wtcd_core::types::ExportKind::Class));
    assert!(result
        .exports
        .iter()
        .any(|e| e.name == "Dog" && e.kind == wtcd_core::types::ExportKind::Class));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("py-meta:class_base:Animal")));
}

#[test]
fn test_python_imports_including_relative() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("imports.py").unwrap();

    let source = std::fs::read_to_string(fixture_path("python/imports.py")).unwrap();
    let result = adapter.parse(&source, "imports.py");

    assert!(result.imports.iter().any(|i| i.source == "os"));
    assert!(result.imports.iter().any(|i| i.source == "pathlib"));
    assert!(result.imports.iter().any(|i| i.source == "."));
    assert!(result.imports.iter().any(|i| i.source == "..parent"));
}

#[test]
fn test_python_decorators_and_patterns() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("decorators.py").unwrap();

    let source = std::fs::read_to_string(fixture_path("python/decorators.py")).unwrap();
    let result = adapter.parse(&source, "decorators.py");

    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("py-meta:pattern:dataclass")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("py-meta:pattern:pydantic_basemodel")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("py-meta:method_type:staticmethod")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("py-meta:method_type:classmethod")));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("py-meta:method_type:property")));
}

#[test]
fn test_python_init_and_dunder_all() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("pkg/__init__.py").unwrap();

    let source = std::fs::read_to_string(fixture_path("python/init_package/__init__.py")).unwrap();
    let result = adapter.parse(&source, "pkg/__init__.py");

    assert!(result.exports.iter().any(|e| e.name == "exported_func"));
    assert!(result.exports.iter().any(|e| e.name == "ExportedClass"));
    assert!(!result.exports.iter().any(|e| e.name == "hidden_func"));
    assert!(result.exports.iter().any(|e| e.name == "__package__"));
    assert!(result
        .side_effects
        .iter()
        .any(|s| s.target.contains("py-meta:package_marker:true")));
    assert!(result.side_effects.iter().any(|s| {
        s.target.contains("py-meta:dunder_all:")
            && s.target.contains("exported_func")
            && s.target.contains("ExportedClass")
    }));
}

#[test]
fn test_python_syntax_error_graceful() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("syntax_error.py").unwrap();

    let source = std::fs::read_to_string(fixture_path("python/syntax_error.py")).unwrap();
    let result = adapter.parse(&source, "syntax_error.py");

    assert!(result.confidence == ConfidenceBand::Low || result.confidence == ConfidenceBand::None);
    assert!(result.error_message.is_some());
}
