use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "zig-meta:";

pub struct ZigAdapter {
    parser: Mutex<Parser>,
}

impl ZigAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_zig::LANGUAGE.into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for ZigAdapter {
    fn language_name(&self) -> &str {
        "zig"
    }

    fn file_extensions(&self) -> &[&str] {
        &["zig"]
    }

    fn parse(&self, source: &str, file_path: &str) -> FileResult {
        let start = Instant::now();
        let tree = self.parser.lock().unwrap().parse(source, None);

        let tree = match tree {
            Some(t) => t,
            None => {
                return FileResult {
                    file_path: file_path.to_string(),
                    confidence: ConfidenceBand::None,
                    exports: vec![],
                    imports: vec![],
                    signatures: vec![],
                    side_effects: vec![],
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error_message: Some("Parser returned no tree".to_string()),
                };
            }
        };

        let root = tree.root_node();
        let has_errors = root.has_error();

        let confidence = if has_errors {
            if root.child_count() > 0 {
                ConfidenceBand::Low
            } else {
                ConfidenceBand::None
            }
        } else {
            ConfidenceBand::High
        };

        let mut exports = Vec::new();
        let mut imports = Vec::new();
        let mut signatures = Vec::new();
        let mut side_effects = Vec::new();

        extract_exports_and_signatures(
            &root,
            source,
            &mut exports,
            &mut signatures,
            &mut side_effects,
        );
        extract_imports(source, &mut imports);
        extract_side_effects(&root, source, &mut side_effects);

        FileResult {
            file_path: file_path.to_string(),
            confidence,
            exports,
            imports,
            signatures,
            side_effects,
            parse_time_ms: start.elapsed().as_millis() as u64,
            error_message: if has_errors {
                Some("Parse errors detected — partial extraction".to_string())
            } else {
                None
            },
        }
    }
}

fn extract_exports_and_signatures(
    root: &Node,
    source: &str,
    exports: &mut Vec<ExportedSymbol>,
    signatures: &mut Vec<FunctionSignature>,
    side_effects: &mut Vec<SideEffect>,
) {
    visit_node(*root, source, exports, signatures, side_effects);
}

fn visit_node(
    node: Node,
    source: &str,
    exports: &mut Vec<ExportedSymbol>,
    signatures: &mut Vec<FunctionSignature>,
    side_effects: &mut Vec<SideEffect>,
) {
    match node.kind() {
        "function_declaration" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Function,
                    line,
                });

                let (parameters, return_type) = extract_zig_signature(node, source);
                signatures.push(FunctionSignature {
                    name,
                    parameters,
                    return_type,
                });
            }
        }
        "struct_declaration" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Class,
                    line,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}struct:{name}"),
                    line,
                });
            }
        }
        "enum_declaration" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Enum,
                    line,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}enum:{name}"),
                    line,
                });
            }
        }
        "union_declaration" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Type,
                    line,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}union:{name}"),
                    line,
                });
            }
        }
        "opaque_declaration" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Type,
                    line,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}opaque:{name}"),
                    line,
                });
            }
        }
        "variable_declaration" => {
            let line = node.start_position().row as u32 + 1;
            // tree-sitter-zig doesn't assign a "name" field to variable_declaration;
            // find the identifier child by position (after "pub"/"var"/"const" keywords)
            let mut var_name: Option<String> = None;
            let mut container_kind: Option<&str> = None;

            for child in node.children(&mut node.walk()) {
                match child.kind() {
                    "identifier" if var_name.is_none() => {
                        var_name = Some(
                            child
                                .utf8_text(source.as_bytes())
                                .unwrap_or("?")
                                .to_string(),
                        );
                    }
                    "struct_declaration" => {
                        container_kind = Some("struct");
                    }
                    "enum_declaration" => {
                        container_kind = Some("enum");
                    }
                    "union_declaration" => {
                        container_kind = Some("union");
                    }
                    "opaque_declaration" => {
                        container_kind = Some("opaque");
                    }
                    _ => {}
                }
            }

            if let Some(name) = var_name {
                let (export_kind, maybe_meta) = match container_kind {
                    Some("struct") => (ExportKind::Class, Some("struct")),
                    Some("enum") => (ExportKind::Enum, Some("enum")),
                    Some("union") => (ExportKind::Type, Some("union")),
                    Some("opaque") => (ExportKind::Type, Some("opaque")),
                    _ => (ExportKind::Var, None),
                };

                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: export_kind,
                    line,
                });

                if let Some(meta) = maybe_meta {
                    side_effects.push(SideEffect {
                        kind: SideEffectKind::Log,
                        target: format!("{META_PREFIX}{meta}:{name}"),
                        line,
                    });
                }
            }
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        visit_node(child, source, exports, signatures, side_effects);
    }
}

fn extract_zig_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();
    let mut return_type = "void".to_string();

    if let Some(params) = node.child_by_field_name("parameters") {
        for child in params.children(&mut params.walk()) {
            if child.kind() == "parameter" {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                let parts: Vec<&str> = text.split(':').collect();
                if parts.len() >= 2 {
                    let param_name = parts[0].trim().to_string();
                    let param_type = parts[1].trim().to_string();
                    parameters.push(Parameter {
                        name: param_name,
                        type_annotation: param_type,
                    });
                }
            }
        }
    }

    if let Some(return_type_node) = node.child_by_field_name("return_type") {
        return_type = return_type_node
            .utf8_text(source.as_bytes())
            .unwrap_or("void")
            .to_string();
    }

    (parameters, return_type)
}

fn extract_imports(source: &str, out: &mut Vec<DependencyEdge>) {
    for line in source.lines() {
        // Find @import("...") anywhere in the line
        let mut rest = line;
        while let Some(start) = rest.find("@import(") {
            let after_prefix = &rest[start + 8..];
            if let Some(end) = after_prefix.find(')') {
                let import_path = after_prefix[..end].trim().trim_matches('"');
                if !import_path.is_empty() {
                    out.push(DependencyEdge {
                        source: import_path.to_string(),
                        symbols: vec![],
                        kind: ImportKind::Named,
                    });
                }
                rest = &after_prefix[end + 1..];
            } else {
                break;
            }
        }
    }
}

fn extract_side_effects(root: &Node, source: &str, out: &mut Vec<SideEffect>) {
    extract_side_effects_node(*root, source, out);
}

fn extract_side_effects_node(node: Node, source: &str, out: &mut Vec<SideEffect>) {
    match node.kind() {
        "builtin_call_expression" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child(0) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("?");
                out.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}builtin:{name}"),
                    line,
                });
            }
        }
        "test_declaration" => {
            let line = node.start_position().row as u32 + 1;
            out.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}test"),
                line,
            });
        }
        "comptime_expression" => {
            let line = node.start_position().row as u32 + 1;
            out.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}comptime"),
                line,
            });
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        extract_side_effects_node(child, source, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_zig(source: &str, file_path: &str) -> FileResult {
        let adapter = ZigAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_functions_structs_and_imports() {
        let source = r#"
const std = @import("std");
const math = @import("math.zig");

pub const Calculator = struct {
    value: i32,

    pub fn init() Calculator {
        return Calculator{ .value = 0 };
    }

    pub fn add(self: *Calculator, a: i32, b: i32) i32 {
        self.value = a + b;
        return self.value;
    }
};

pub const Color = enum {
    Red,
    Green,
    Blue,
};

pub const Point = struct {
    x: f64,
    y: f64,
};

pub const MAX_SIZE: usize = 100;

test "addition" {
    var calc = Calculator.init();
    const result = calc.add(2, 3);
    try std.testing.expectEqual(5, result);
}
"#;

        let result = parse_zig(source, "demo.zig");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Calculator" && e.kind == ExportKind::Class));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Color" && e.kind == ExportKind::Enum));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Point" && e.kind == ExportKind::Class));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "MAX_SIZE" && e.kind == ExportKind::Var));

        assert!(result.imports.iter().any(|i| i.source == "std"));
        assert!(result.imports.iter().any(|i| i.source == "math.zig"));

        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("zig-meta:struct:Calculator")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("zig-meta:enum:Color")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("zig-meta:test")));
    }
}
