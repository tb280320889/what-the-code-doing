use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "rust-meta:";

pub struct RustAdapter {
    parser: Mutex<Parser>,
}

impl RustAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for RustAdapter {
    fn language_name(&self) -> &str {
        "rust"
    }

    fn file_extensions(&self) -> &[&str] {
        &["rs"]
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
        extract_use_statements(source, &mut imports);
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
        "function_item" => {
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

                let (parameters, return_type) = extract_rust_signature(node, source);
                signatures.push(FunctionSignature {
                    name,
                    parameters,
                    return_type,
                });
            }
        }
        "struct_item" => {
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
        "enum_item" => {
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
        "trait_item" => {
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
                    target: format!("{META_PREFIX}trait:{name}"),
                    line,
                });
            }
        }
        "type_item" => {
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
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}type_alias:{name}"),
                    line,
                });
            }
        }
        "impl_item" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(type_node) = node.child_by_field_name("type") {
                let type_name = type_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}impl:{type_name}"),
                    line,
                });
            }
        }
        "mod_item" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}mod:{name}"),
                    line,
                });
            }
        }
        "const_item" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Const,
                    line,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
            }
        }
        "static_item" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("?")
                    .to_string();
                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: ExportKind::Var,
                    line,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
            }
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        visit_node(child, source, exports, signatures, side_effects);
    }
}

fn extract_rust_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();
    let mut return_type = "()".to_string();

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
            .unwrap_or("()")
            .to_string();
    }

    (parameters, return_type)
}

fn extract_use_statements(source: &str, out: &mut Vec<DependencyEdge>) {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") && (trimmed.ends_with(';') || trimmed.contains(" as ")) {
            let use_path = if trimmed.ends_with(';') {
                trimmed[4..trimmed.len() - 1].trim()
            } else {
                trimmed[4..].trim()
            };

            let kind = if use_path.contains("::*") {
                ImportKind::Namespace
            } else if use_path.contains(" as ") {
                ImportKind::Default
            } else {
                ImportKind::Named
            };

            out.push(DependencyEdge {
                source: use_path.to_string(),
                symbols: vec![],
                kind,
            });
        }
    }
}

fn extract_side_effects(root: &Node, source: &str, out: &mut Vec<SideEffect>) {
    extract_side_effects_node(*root, source, out);
}

fn extract_side_effects_node(node: Node, source: &str, out: &mut Vec<SideEffect>) {
    match node.kind() {
        "attribute_item" => {
            let line = node.start_position().row as u32 + 1;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            out.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}attribute:{text}"),
                line,
            });
        }
        "macro_invocation" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(macro_name) = node.child_by_field_name("macro") {
                let name = macro_name.utf8_text(source.as_bytes()).unwrap_or("?");
                out.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}macro:{name}"),
                    line,
                });
            }
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

    fn parse_rust(source: &str, file_path: &str) -> FileResult {
        let adapter = RustAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_functions_structs_traits_and_use() {
        let source = r#"
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn add(&mut self, a: i32, b: i32) -> i32 {
        self.value = a + b;
        self.value
    }
}

pub trait Processor {
    fn process(&self, data: &str) -> Result<(), Error>;
}

pub enum Color {
    Red,
    Green,
    Blue,
}

pub type StringMap = HashMap<String, String>;

pub const MAX_SIZE: usize = 100;

static mut COUNTER: u32 = 0;

mod utils;
"#;

        let result = parse_rust(source, "demo.rs");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Calculator" && e.kind == ExportKind::Class));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Processor" && e.kind == ExportKind::Interface));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Color" && e.kind == ExportKind::Enum));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "StringMap" && e.kind == ExportKind::Type));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "MAX_SIZE" && e.kind == ExportKind::Const));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "COUNTER" && e.kind == ExportKind::Var));

        assert!(result
            .imports
            .iter()
            .any(|i| i.source == "std::collections::HashMap"));
        assert!(result
            .imports
            .iter()
            .any(|i| i.source == "serde::{Deserialize, Serialize}"));

        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("rust-meta:struct:Calculator")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("rust-meta:trait:Processor")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("rust-meta:enum:Color")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("rust-meta:impl:Calculator")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("rust-meta:mod:utils")));
    }
}
