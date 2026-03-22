use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "cpp-meta:";

pub struct CppAdapter {
    parser: Mutex<Parser>,
}

impl CppAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_cpp::LANGUAGE.into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for CppAdapter {
    fn language_name(&self) -> &str {
        "cpp"
    }

    fn file_extensions(&self) -> &[&str] {
        &["cpp", "cc", "cxx", "hpp", "h", "hh", "hxx"]
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
        extract_includes(source, &mut imports);
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
        "function_definition" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(declarator) = node.child_by_field_name("declarator") {
                let name = extract_name_from_declarator(declarator, source);
                if !name.is_empty() {
                    exports.push(ExportedSymbol {
                        name: name.clone(),
                        kind: ExportKind::Function,
                        line,
                        is_generated: false,
                        confidence: ConfidenceBand::High,
                    });

                    let (parameters, return_type) = extract_cpp_signature(node, source);
                    signatures.push(FunctionSignature {
                        name,
                        parameters,
                        return_type,
                    });
                }
            }
        }
        "class_specifier" | "struct_specifier" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                if !name.is_empty() {
                    exports.push(ExportedSymbol {
                        name: name.clone(),
                        kind: ExportKind::Class,
                        line,
                        is_generated: false,
                        confidence: ConfidenceBand::High,
                    });
                    side_effects.push(SideEffect {
                        kind: SideEffectKind::Log,
                        target: format!(
                            "{META_PREFIX}{}:{}",
                            if node.kind() == "class_specifier" {
                                "class"
                            } else {
                                "struct"
                            },
                            name
                        ),
                        line,
                    });
                }
            }
        }
        "enum_specifier" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                if !name.is_empty() {
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
        }
        "namespace_definition" => {
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
        "template_declaration" => {
            let line = node.start_position().row as u32 + 1;
            side_effects.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}template"),
                line,
            });
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        visit_node(child, source, exports, signatures, side_effects);
    }
}

fn extract_name_from_declarator(node: Node, source: &str) -> String {
    match node.kind() {
        "identifier" => node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
        "field_identifier" => node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
        "qualified_identifier" => node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
        "destructor_name" => node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
        _ => {
            for child in node.children(&mut node.walk()) {
                let name = extract_name_from_declarator(child, source);
                if !name.is_empty() {
                    return name;
                }
            }
            String::new()
        }
    }
}

fn extract_cpp_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();
    let mut return_type = "void".to_string();

    if let Some(declarator) = node.child_by_field_name("declarator") {
        if let Some(params) = declarator.child_by_field_name("parameters") {
            for child in params.children(&mut params.walk()) {
                if child.kind() == "parameter_declaration" {
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
    }

    if let Some(type_node) = node.child_by_field_name("type") {
        return_type = type_node
            .utf8_text(source.as_bytes())
            .unwrap_or("void")
            .to_string();
    }

    (parameters, return_type)
}

fn extract_includes(source: &str, out: &mut Vec<DependencyEdge>) {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#include") {
            let include_path = if let Some(angle) = trimmed.find('<') {
                trimmed[angle + 1..].trim_end_matches('>').to_string()
            } else if let Some(quote) = trimmed.find('"') {
                trimmed[quote + 1..].trim_end_matches('"').to_string()
            } else {
                continue;
            };

            out.push(DependencyEdge {
                source: include_path,
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
        "preproc_ifdef" | "preproc_ifndef" | "preproc_if" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(condition) = node.child(1) {
                if let Ok(text) = condition.utf8_text(source.as_bytes()) {
                    out.push(SideEffect {
                        kind: SideEffectKind::Log,
                        target: format!("{META_PREFIX}conditional:{text}"),
                        line,
                    });
                }
            }
        }
        "using_declaration" => {
            let line = node.start_position().row as u32 + 1;
            out.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}using"),
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

    fn parse_cpp(source: &str, file_path: &str) -> FileResult {
        let adapter = CppAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_classes_functions_and_includes() {
        let source = r#"
#include <iostream>
#include <vector>

namespace myapp {
    class Calculator {
    public:
        int add(int a, int b);
        void print(const std::string& msg);
    private:
        int value;
    };

    struct Point {
        double x;
        double y;
    };

    enum Color {
        RED,
        GREEN,
        BLUE
    };
}
"#;

        let result = parse_cpp(source, "demo.cpp");
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
            .any(|e| e.name == "Color" && e.kind == ExportKind::Enum));

        assert!(result.imports.iter().any(|i| i.source == "iostream"));
        assert!(result.imports.iter().any(|i| i.source == "vector"));

        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("cpp-meta:namespace:myapp")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("cpp-meta:class:Calculator")));
    }
}
