use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "csharp-meta:";

pub struct CSharpAdapter {
    parser: Mutex<Parser>,
}

impl CSharpAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c_sharp::LANGUAGE.into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for CSharpAdapter {
    fn language_name(&self) -> &str {
        "csharp"
    }

    fn file_extensions(&self) -> &[&str] {
        &["cs"]
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
        extract_using_directives(source, &mut imports);
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
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Class,
                    line,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}class:{name}"),
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
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}struct:{name}"),
                    line,
                });
            }
        }
        "interface_declaration" => {
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
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}interface:{name}"),
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
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}enum:{name}"),
                    line,
                });
            }
        }
        "method_declaration" => {
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
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });

                let (parameters, return_type) = extract_csharp_signature(node, source);
                signatures.push(FunctionSignature {
                    name,
                    parameters,
                    return_type,
                });
            }
        }
        "constructor_declaration" => {
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
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });

                let (parameters, _) = extract_csharp_signature(node, source);
                signatures.push(FunctionSignature {
                    name,
                    parameters,
                    return_type: "void".to_string(),
                });
            }
        }
        "namespace_declaration" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}namespace:{name}"),
                    line,
                });
            }
        }
        "attribute_list" => {
            let line = node.start_position().row as u32 + 1;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            side_effects.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}attribute:{text}"),
                line,
            });
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        visit_node(child, source, exports, signatures, side_effects);
    }
}

fn extract_csharp_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();
    let mut return_type = "void".to_string();

    if let Some(params) = node.child_by_field_name("parameters") {
        for child in params.children(&mut params.walk()) {
            if child.kind() == "parameter" {
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

    if let Some(type_node) = node.child_by_field_name("type") {
        return_type = type_node
            .utf8_text(source.as_bytes())
            .unwrap_or("void")
            .to_string();
    }

    (parameters, return_type)
}

fn extract_using_directives(source: &str, out: &mut Vec<DependencyEdge>) {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("using ") && trimmed.ends_with(';') {
            let namespace = trimmed[6..trimmed.len() - 1].trim();
            out.push(DependencyEdge {
                source: namespace.to_string(),
                symbols: vec![],
                kind: ImportKind::Named,
            });
        }
    }
}

fn extract_side_effects(root: &Node, _source: &str, out: &mut Vec<SideEffect>) {
    extract_side_effects_node(*root, out);
}

fn extract_side_effects_node(node: Node, out: &mut Vec<SideEffect>) {
    if node.kind() == "preprocessor_directive" {
        let line = node.start_position().row as u32 + 1;
        out.push(SideEffect {
            kind: SideEffectKind::Log,
            target: format!("{META_PREFIX}preprocessor"),
            line,
        });
    }

    for child in node.children(&mut node.walk()) {
        extract_side_effects_node(child, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_csharp(source: &str, file_path: &str) -> FileResult {
        let adapter = CSharpAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_classes_methods_and_using() {
        let source = r#"
using System;
using System.Collections.Generic;

namespace MyApp {
    public class Calculator {
        public int Add(int a, int b) {
            return a + b;
        }

        public void Print(string message) {
            Console.WriteLine(message);
        }
    }

    public struct Point {
        public double X;
        public double Y;
    }

    public interface IProcessor {
        void Process(string data);
    }

    public enum Color {
        Red,
        Green,
        Blue
    }
}
"#;

        let result = parse_csharp(source, "demo.cs");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Calculator" && e.kind == ExportKind::Class));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Point" && e.kind == ExportKind::Class));
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
            .any(|e| e.name == "Add" && e.kind == ExportKind::Function));

        assert!(result.imports.iter().any(|i| i.source == "System"));
        assert!(result
            .imports
            .iter()
            .any(|i| i.source == "System.Collections.Generic"));

        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("csharp-meta:namespace:MyApp")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("csharp-meta:class:Calculator")));
    }
}
