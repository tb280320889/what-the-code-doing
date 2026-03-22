use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "swift-meta:";

pub struct SwiftAdapter {
    parser: Mutex<Parser>,
}

impl SwiftAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_swift::LANGUAGE.into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for SwiftAdapter {
    fn language_name(&self) -> &str {
        "swift"
    }

    fn file_extensions(&self) -> &[&str] {
        &["swift"]
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
        "class_declaration" => {
            let line = node.start_position().row as u32 + 1;
            // Check declaration_kind to determine if it's class, enum, or struct
            let mut declaration_kind = "class";
            if let Some(kind_node) = node.child_by_field_name("declaration_kind") {
                declaration_kind = kind_node.utf8_text(source.as_bytes()).unwrap_or("class");
            }

            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();

                let (export_kind, meta_kind) = match declaration_kind {
                    "enum" => (ExportKind::Enum, "enum"),
                    "struct" => (ExportKind::Class, "struct"),
                    "class" => (ExportKind::Class, "class"),
                    "extension" => (ExportKind::Class, "extension"),
                    _ => (ExportKind::Class, "class"),
                };

                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: export_kind,
                    line,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}{meta_kind}:{name}"),
                    line,
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
        "protocol_declaration" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Interface,
                    line,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}protocol:{name}"),
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

                let (parameters, return_type) = extract_swift_signature(node, source);
                signatures.push(FunctionSignature {
                    name,
                    parameters,
                    return_type,
                });
            }
        }
        "typealias_declaration" => {
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
                    target: format!("{META_PREFIX}typealias:{name}"),
                    line,
                });
            }
        }
        "extension_declaration" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("type") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}extension:{name}"),
                    line,
                });
            }
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        visit_node(child, source, exports, signatures, side_effects);
    }
}

fn extract_swift_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();
    let mut return_type = "Void".to_string();

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

    if let Some(type_node) = node.child_by_field_name("return_type") {
        return_type = type_node
            .utf8_text(source.as_bytes())
            .unwrap_or("Void")
            .to_string();
    }

    (parameters, return_type)
}

fn extract_imports(source: &str, out: &mut Vec<DependencyEdge>) {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            let import_path = trimmed[7..].trim();
            out.push(DependencyEdge {
                source: import_path.to_string(),
                symbols: vec![],
                kind: ImportKind::Named,
            });
        }
    }
}

fn extract_side_effects(root: &Node, source: &str, out: &mut Vec<SideEffect>) {
    extract_side_effects_node(*root, source, out);
}

fn extract_side_effects_node(node: Node, source: &str, out: &mut Vec<SideEffect>) {
    match node.kind() {
        "attribute" => {
            let line = node.start_position().row as u32 + 1;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            out.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}attribute:{text}"),
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

    fn parse_swift(source: &str, file_path: &str) -> FileResult {
        let adapter = SwiftAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_classes_protocols_and_imports() {
        let source = r#"
import Foundation
import UIKit

class Calculator {
    func add(a: Int, b: Int) -> Int {
        return a + b
    }

    func print(message: String) {
        print(message)
    }
}

protocol IProcessor {
    func process(data: String)
}

enum Color {
    case red
    case green
    case blue
}

struct Point {
    let x: Double
    let y: Double
}

typealias StringMap = Dictionary<String, String>

extension String {
    func trimmed() -> String {
        return self.trimmingCharacters(in: .whitespaces)
    }
}
"#;

        let result = parse_swift(source, "demo.swift");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Calculator" && e.kind == ExportKind::Class));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "IProcessor" && e.kind == ExportKind::Interface));
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
            .any(|e| e.name == "StringMap" && e.kind == ExportKind::Type));

        assert!(result.imports.iter().any(|i| i.source == "Foundation"));
        assert!(result.imports.iter().any(|i| i.source == "UIKit"));

        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("swift-meta:class:Calculator")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("swift-meta:protocol:IProcessor")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("swift-meta:enum:Color")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("swift-meta:typealias:StringMap")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("swift-meta:extension:String")));
    }
}
