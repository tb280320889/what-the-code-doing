use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "dart-meta:";

pub struct DartAdapter {
    parser: Mutex<Parser>,
}

impl DartAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_dart::language().into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for DartAdapter {
    fn language_name(&self) -> &str {
        "dart"
    }

    fn file_extensions(&self) -> &[&str] {
        &["dart"]
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
        "class_definition" => {
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
                    target: format!("{META_PREFIX}class:{name}"),
                    line,
                });
            }
        }
        "function_signature" => {
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

                let (parameters, return_type) = extract_dart_signature(node, source);
                signatures.push(FunctionSignature {
                    name,
                    parameters,
                    return_type,
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
        "mixin_declaration" | "mixin_definition" => {
            let line = node.start_position().row as u32 + 1;
            // Try to find name from children (identifier or type_identifier)
            for child in node.children(&mut node.walk()) {
                if child.kind() == "identifier" || child.kind() == "type_identifier" {
                    let name = child
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
                        target: format!("{META_PREFIX}mixin:{name}"),
                        line,
                    });
                    break;
                }
            }
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        visit_node(child, source, exports, signatures, side_effects);
    }
}

fn extract_dart_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();
    let mut return_type = "dynamic".to_string();

    if let Some(params) = node.child_by_field_name("parameters") {
        for child in params.children(&mut params.walk()) {
            if child.kind() == "formal_parameter" {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() >= 2 {
                    let param_type = parts[..parts.len() - 1].join(" ");
                    let param_name = parts[parts.len() - 1].to_string();
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
            .unwrap_or("dynamic")
            .to_string();
    }

    (parameters, return_type)
}

fn extract_imports(source: &str, out: &mut Vec<DependencyEdge>) {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") && trimmed.ends_with(';') {
            let import_path = trimmed[7..trimmed.len() - 1].trim();
            let import_path = import_path.trim_matches(|c| c == '\'' || c == '"');
            out.push(DependencyEdge {
                source: import_path.to_string(),
                symbols: vec![],
                kind: ImportKind::Named,
            });
        } else if trimmed.starts_with("export ") && trimmed.ends_with(';') {
            let export_path = trimmed[7..trimmed.len() - 1].trim();
            let export_path = export_path.trim_matches(|c| c == '\'' || c == '"');
            out.push(DependencyEdge {
                source: export_path.to_string(),
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
        "annotation" => {
            let line = node.start_position().row as u32 + 1;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            out.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}annotation:{text}"),
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

    fn parse_dart(source: &str, file_path: &str) -> FileResult {
        let adapter = DartAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_classes_functions_and_imports() {
        let source = r#"
import 'dart:math';
import 'package:flutter/material.dart';

class Calculator {
  int add(int a, int b) => a + b;
  
  void print(String message) {
    print(message);
  }
}

enum Color {
  red,
  green,
  blue
}

mixin Printable {
  void printMessage();
}
"#;

        let result = parse_dart(source, "demo.dart");
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
            .any(|e| e.name == "Printable" && e.kind == ExportKind::Type));

        assert!(result.imports.iter().any(|i| i.source == "dart:math"));
        assert!(result
            .imports
            .iter()
            .any(|i| i.source == "package:flutter/material.dart"));

        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("dart-meta:class:Calculator")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("dart-meta:enum:Color")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("dart-meta:mixin:Printable")));
    }
}
