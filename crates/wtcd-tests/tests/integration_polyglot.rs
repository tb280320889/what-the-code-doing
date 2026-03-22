use wtcd_adapters::register_all_adapters;
use wtcd_core::types::{ConfidenceBand, ExportKind};

fn fixture_path(relative: &str) -> String {
    format!(
        "{}/../../tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        relative
    )
}

/// Verify that every new language adapter can be found by extension and parses hello fixture.
#[test]
fn test_all_polyglot_adapters_registered() {
    let registry = register_all_adapters().unwrap();

    let languages: &[(&str, &str)] = &[
        ("hello.c", "c"),
        ("hello.cpp", "cpp"),
        ("hello.cs", "csharp"),
        ("hello.dart", "dart"),
        ("hello.java", "java"),
        ("hello.kt", "kotlin"),
        ("hello.rs", "rust"),
        ("hello.swift", "swift"),
        ("hello.zig", "zig"),
    ];

    for (filename, lang_dir) in languages {
        let adapter = registry
            .find_adapter(filename)
            .unwrap_or_else(|| panic!("No adapter found for {}", filename));
        let source = std::fs::read_to_string(fixture_path(&format!(
            "{}/hello.{}",
            lang_dir,
            filename.split('.').last().unwrap()
        )))
        .unwrap();
        let result = adapter.parse(&source, filename);

        assert!(
            result.confidence == ConfidenceBand::High || result.confidence == ConfidenceBand::Low,
            "{}: expected High or Low confidence, got {:?}",
            filename,
            result.confidence
        );
        assert!(
            !result.exports.is_empty(),
            "{}: expected at least 1 export",
            filename
        );
    }
}

// ---------------------------------------------------------------------------
// Per-language integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_c_hello_exports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.c").unwrap();
    let source = std::fs::read_to_string(fixture_path("c/hello.c")).unwrap();
    let result = adapter.parse(&source, "hello.c");

    assert_eq!(result.confidence, ConfidenceBand::High);
    let names: Vec<&str> = result.exports.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"add"), "Missing C export: add");
    assert!(names.contains(&"greet"), "Missing C export: greet");
}

#[test]
fn test_cpp_hello_exports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.cpp").unwrap();
    let source = std::fs::read_to_string(fixture_path("cpp/hello.cpp")).unwrap();
    let result = adapter.parse(&source, "hello.cpp");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(!result.exports.is_empty(), "Expected C++ exports");
}

#[test]
fn test_csharp_hello_exports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.cs").unwrap();
    let source = std::fs::read_to_string(fixture_path("csharp/hello.cs")).unwrap();
    let result = adapter.parse(&source, "hello.cs");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(
        result
            .exports
            .iter()
            .any(|e| e.kind == ExportKind::Class || e.kind == ExportKind::Function),
        "Expected C# class or function exports"
    );
}

#[test]
fn test_dart_hello_exports_and_imports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.dart").unwrap();
    let source = std::fs::read_to_string(fixture_path("dart/hello.dart")).unwrap();
    let result = adapter.parse(&source, "hello.dart");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(
        result
            .exports
            .iter()
            .any(|e| e.name == "Calculator" && e.kind == ExportKind::Class),
        "Missing Dart export: Calculator"
    );
    assert!(
        result
            .exports
            .iter()
            .any(|e| e.name == "Color" && e.kind == ExportKind::Enum),
        "Missing Dart export: Color"
    );
    assert!(
        result.imports.iter().any(|i| i.source == "dart:math"),
        "Missing Dart import: dart:math"
    );
    assert!(
        result.imports.iter().any(|i| i.source == "dart:async"),
        "Missing Dart import: dart:async"
    );
}

#[test]
fn test_java_hello_exports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.java").unwrap();
    let source = std::fs::read_to_string(fixture_path("java/hello.java")).unwrap();
    let result = adapter.parse(&source, "hello.java");

    assert_eq!(result.confidence, ConfidenceBand::High);
    let names: Vec<&str> = result.exports.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"Calculator"), "Missing Java: Calculator");
    assert!(names.contains(&"Point"), "Missing Java: Point");
    assert!(names.contains(&"IProcessor"), "Missing Java: IProcessor");
    assert!(names.contains(&"Color"), "Missing Java: Color");
    assert!(names.contains(&"add"), "Missing Java: add");
}

#[test]
fn test_kotlin_hello_exports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.kt").unwrap();
    let source = std::fs::read_to_string(fixture_path("kotlin/hello.kt")).unwrap();
    let result = adapter.parse(&source, "hello.kt");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(
        result
            .exports
            .iter()
            .any(|e| e.kind == ExportKind::Class || e.kind == ExportKind::Function),
        "Expected Kotlin class or function exports"
    );
}

#[test]
fn test_rust_hello_exports_and_imports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.rs").unwrap();
    let source = std::fs::read_to_string(fixture_path("rust/hello.rs")).unwrap();
    let result = adapter.parse(&source, "hello.rs");

    assert_eq!(result.confidence, ConfidenceBand::High);
    let names: Vec<&str> = result.exports.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"add"), "Missing Rust export: add");
    assert!(names.contains(&"greet"), "Missing Rust export: greet");
    assert!(
        result.imports.iter().any(|i| i.source.contains("std")),
        "Missing Rust import: std"
    );
}

#[test]
fn test_swift_hello_exports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.swift").unwrap();
    let source = std::fs::read_to_string(fixture_path("swift/hello.swift")).unwrap();
    let result = adapter.parse(&source, "hello.swift");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(
        result
            .exports
            .iter()
            .any(|e| e.kind == ExportKind::Class || e.kind == ExportKind::Function),
        "Expected Swift class or function exports"
    );
}

#[test]
fn test_zig_hello_exports_and_imports() {
    let registry = register_all_adapters().unwrap();
    let adapter = registry.find_adapter("hello.zig").unwrap();
    let source = std::fs::read_to_string(fixture_path("zig/hello.zig")).unwrap();
    let result = adapter.parse(&source, "hello.zig");

    assert_eq!(result.confidence, ConfidenceBand::High);
    assert!(!result.exports.is_empty(), "Expected Zig exports, got none");
    assert!(
        result.imports.iter().any(|i| i.source == "std"),
        "Missing Zig import: std"
    );
}

// ---------------------------------------------------------------------------
// Syntax error graceful degradation
// ---------------------------------------------------------------------------

#[test]
fn test_syntax_error_graceful_all_languages() {
    let registry = register_all_adapters().unwrap();

    let test_cases: &[(&str, &str)] = &[
        ("c/syntax_error.c", "error.c"),
        ("cpp/syntax_error.cpp", "error.cpp"),
        ("csharp/syntax_error.cs", "error.cs"),
        ("dart/syntax_error.dart", "error.dart"),
        ("java/syntax_error.java", "error.java"),
        ("kotlin/syntax_error.kt", "error.kt"),
        ("rust/syntax_error.rs", "error.rs"),
        ("swift/syntax_error.swift", "error.swift"),
        ("zig/syntax_error.zig", "error.zig"),
    ];

    for (fixture, filename) in test_cases {
        let adapter = registry
            .find_adapter(filename)
            .unwrap_or_else(|| panic!("No adapter for {}", filename));
        let source = std::fs::read_to_string(fixture_path(fixture))
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", fixture, e));
        let result = adapter.parse(&source, filename);

        assert!(
            result.confidence == ConfidenceBand::Low || result.confidence == ConfidenceBand::None,
            "{}: expected degraded confidence for broken syntax, got {:?}",
            filename,
            result.confidence
        );
        assert!(
            result.error_message.is_some(),
            "{}: expected error_message for broken syntax",
            filename
        );
    }
}

// ---------------------------------------------------------------------------
// Parse time sanity
// ---------------------------------------------------------------------------

#[test]
fn test_parse_time_sanity() {
    let registry = register_all_adapters().unwrap();

    let files: &[(&str, &str)] = &[
        ("c/hello.c", "hello.c"),
        ("cpp/hello.cpp", "hello.cpp"),
        ("csharp/hello.cs", "hello.cs"),
        ("dart/hello.dart", "hello.dart"),
        ("java/hello.java", "hello.java"),
        ("kotlin/hello.kt", "hello.kt"),
        ("rust/hello.rs", "hello.rs"),
        ("swift/hello.swift", "hello.swift"),
        ("zig/hello.zig", "hello.zig"),
    ];

    for (fixture, filename) in files {
        let adapter = registry.find_adapter(filename).unwrap();
        let source = std::fs::read_to_string(fixture_path(fixture)).unwrap();
        let result = adapter.parse(&source, filename);

        assert!(
            result.parse_time_ms < 100,
            "{}: parse took {}ms (expected <100ms)",
            filename,
            result.parse_time_ms
        );
    }
}

// ---------------------------------------------------------------------------
// Scanner-registry consistency (STAB-04)
// ---------------------------------------------------------------------------

#[test]
fn test_scanner_registry_extension_consistency() {
    let registry = register_all_adapters().unwrap();

    // All extensions that adapters claim
    let mut adapter_extensions: Vec<String> = Vec::new();
    // We can't directly iterate adapters, but we know the expected set
    let expected_extensions = &[
        "ts", "tsx", "js", "jsx", // TS/JS
        "py", "pyi",  // Python
        "go",   // Go
        "rs",   // Rust
        "dart", // Dart
        "java", // Java
        "kt", "kts",   // Kotlin
        "swift", // Swift
        "c", "h", // C
        "cpp", "cc", "cxx", "hpp", "hh", "hxx", // C++
        "cs",  // C#
        "zig", // Zig
    ];

    for ext in expected_extensions {
        let test_file = format!("test.{}", ext);
        let adapter = registry.find_adapter(&test_file);
        assert!(
            adapter.is_some(),
            "No adapter registered for extension .{} — scanner/registry mismatch",
            ext
        );
    }
}

// ---------------------------------------------------------------------------
// Existing adapter regression (ROUT-04)
// ---------------------------------------------------------------------------

#[test]
fn test_existing_adapters_no_regression() {
    let registry = register_all_adapters().unwrap();

    // TS/JS
    let ts_adapter = registry.find_adapter("test.ts").unwrap();
    let ts_result = ts_adapter.parse("export const x = 1;", "test.ts");
    assert_eq!(ts_result.confidence, ConfidenceBand::High);
    assert!(!ts_result.exports.is_empty());

    // Python
    let py_adapter = registry.find_adapter("test.py").unwrap();
    let py_result = py_adapter.parse("def hello():\n    pass\n", "test.py");
    assert_eq!(py_result.confidence, ConfidenceBand::High);
    assert!(!py_result.exports.is_empty());

    // Go
    let go_adapter = registry.find_adapter("test.go").unwrap();
    let go_result = go_adapter.parse("package main\n\nfunc Hello() {}\n", "test.go");
    assert_eq!(go_result.confidence, ConfidenceBand::High);
    assert!(!go_result.exports.is_empty());
}

// ---------------------------------------------------------------------------
// Full performance budget (STAB-02) — all 13 languages
// ---------------------------------------------------------------------------

#[test]
fn test_parse_time_budget_all_languages() {
    let registry = register_all_adapters().unwrap();

    let all_fixtures: &[(&str, &str)] = &[
        // Existing languages
        ("ts/basic_exports.ts", "basic.ts"),
        ("python/hello.py", "hello.py"),
        ("go/sample.go", "sample.go"),
        // New polyglot languages
        ("c/hello.c", "hello.c"),
        ("cpp/hello.cpp", "hello.cpp"),
        ("csharp/hello.cs", "hello.cs"),
        ("dart/hello.dart", "hello.dart"),
        ("java/hello.java", "hello.java"),
        ("kotlin/hello.kt", "hello.kt"),
        ("rust/hello.rs", "hello.rs"),
        ("swift/hello.swift", "hello.swift"),
        ("zig/hello.zig", "hello.zig"),
    ];

    for (fixture, filename) in all_fixtures {
        let adapter = registry.find_adapter(filename).unwrap();
        let source = std::fs::read_to_string(fixture_path(fixture))
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", fixture, e));
        let result = adapter.parse(&source, filename);

        assert!(
            result.parse_time_ms < 100,
            "{}: parse took {}ms (budget: 100ms)",
            filename,
            result.parse_time_ms
        );
    }
}
