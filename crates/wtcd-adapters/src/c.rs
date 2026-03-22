use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "c-meta:";

pub struct CAdapter {
    parser: Mutex<Parser>,
}

impl CAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for CAdapter {
    fn language_name(&self) -> &str {
        "c"
    }

    fn file_extensions(&self) -> &[&str] {
        &["c", "h"]
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
                if let Some(name_node) = declarator.child_by_field_name("declarator") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("?")
                        .to_string();
                    exports.push(ExportedSymbol {
                        name: name.clone(),
                        kind: ExportKind::Function,
                        line,
                    });

                    let (parameters, return_type) = extract_c_signature(node, source);
                    signatures.push(FunctionSignature {
                        name,
                        parameters,
                        return_type,
                    });
                }
            }
        }
        "declaration" => {
            let line = node.start_position().row as u32 + 1;
            // Check for function declarations (prototypes)
            if let Some(declarator) = node.child_by_field_name("declarator") {
                if declarator.kind() == "function_declarator" {
                    if let Some(name_node) = declarator.child_by_field_name("declarator") {
                        let name = name_node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("?")
                            .to_string();
                        exports.push(ExportedSymbol {
                            name: name.clone(),
                            kind: ExportKind::Function,
                            line,
                        });

                        let (parameters, return_type) =
                            extract_c_signature_from_declaration(node, source);
                        signatures.push(FunctionSignature {
                            name,
                            parameters,
                            return_type,
                        });
                    }
                }
            }
        }
        "struct_specifier" => {
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
                    });
                    side_effects.push(SideEffect {
                        kind: SideEffectKind::Log,
                        target: format!("{META_PREFIX}struct:{name}"),
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
                    });
                    side_effects.push(SideEffect {
                        kind: SideEffectKind::Log,
                        target: format!("{META_PREFIX}enum:{name}"),
                        line,
                    });
                }
            }
        }
        "type_definition" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(declarator) = node.child_by_field_name("declarator") {
                let name = declarator
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
                    target: format!("{META_PREFIX}typedef:{name}"),
                    line,
                });
            }
        }
        "preproc_def" | "preproc_function_def" => {
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
                });
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}define:{name}"),
                    line,
                });
            }
        }
        "linkage_specification" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(value_node) = node.child_by_field_name("value") {
                if let Ok(lang) = value_node.utf8_text(source.as_bytes()) {
                    side_effects.push(SideEffect {
                        kind: SideEffectKind::Log,
                        target: format!("{META_PREFIX}extern:{lang}"),
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

fn extract_c_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();
    let mut return_type = "void".to_string();

    if let Some(declarator) = node.child_by_field_name("declarator") {
        if let Some(func_declarator) = declarator.child_by_field_name("declarator") {
            if func_declarator.kind() == "function_declarator" {
                if let Some(params) = func_declarator.child_by_field_name("parameters") {
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
        }

        if let Some(type_node) = node.child_by_field_name("type") {
            return_type = type_node
                .utf8_text(source.as_bytes())
                .unwrap_or("void")
                .to_string();
        }
    }

    (parameters, return_type)
}

fn extract_c_signature_from_declaration(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();
    let mut return_type = "void".to_string();

    if let Some(declarator) = node.child_by_field_name("declarator") {
        if declarator.kind() == "function_declarator" {
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
        "preproc_include" => {
            let line = node.start_position().row as u32 + 1;
            out.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}include"),
                line,
            });
        }
        "call_expression" => {
            let line = node.start_position().row as u32 + 1;
            if let Some(func) = node.child_by_field_name("function") {
                if let Ok(name) = func.utf8_text(source.as_bytes()) {
                    match name {
                        "printf" | "fprintf" | "sprintf" | "snprintf" => {
                            out.push(SideEffect {
                                kind: SideEffectKind::Log,
                                target: format!("{META_PREFIX}io:{name}"),
                                line,
                            });
                        }
                        "malloc" | "calloc" | "realloc" | "free" => {
                            out.push(SideEffect {
                                kind: SideEffectKind::Storage,
                                target: format!("{META_PREFIX}memory:{name}"),
                                line,
                            });
                        }
                        "fopen" | "fclose" | "fread" | "fwrite" => {
                            out.push(SideEffect {
                                kind: SideEffectKind::Io,
                                target: format!("{META_PREFIX}file:{name}"),
                                line,
                            });
                        }
                        _ => {}
                    }
                }
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

    fn parse_c(source: &str, file_path: &str) -> FileResult {
        let adapter = CAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_functions_structs_and_includes() {
        let source = r#"
#include <stdio.h>
#include "myheader.h"

struct Point {
    int x;
    int y;
};

enum Color {
    RED,
    GREEN,
    BLUE
};

typedef struct Point Point;

int add(int a, int b) {
    return a + b;
}

void print_point(struct Point p) {
    printf("(%d, %d)", p.x, p.y);
}
"#;

        let result = parse_c(source, "demo.c");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "add" && e.kind == ExportKind::Function));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "print_point" && e.kind == ExportKind::Function));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Point" && e.kind == ExportKind::Class));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Color" && e.kind == ExportKind::Enum));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Point" && e.kind == ExportKind::Type));

        assert!(result.imports.iter().any(|i| i.source == "stdio.h"));
        assert!(result.imports.iter().any(|i| i.source == "myheader.h"));

        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("c-meta:struct:Point")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("c-meta:enum:Color")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("c-meta:typedef:Point")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("c-meta:io:printf")));
    }

    #[test]
    fn extracts_preprocessor_directives() {
        let source = r#"
#define MAX_SIZE 100
#define MIN(a, b) ((a) < (b) ? (a) : (b))

#ifdef DEBUG
void debug_print(const char* msg);
#endif

#ifndef HEADER_H
#define HEADER_H
#endif
"#;

        let result = parse_c(source, "header.h");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "MAX_SIZE" && e.kind == ExportKind::Const));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "MIN" && e.kind == ExportKind::Const));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("c-meta:define:MAX_SIZE")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("c-meta:conditional:DEBUG")));
    }

    #[test]
    fn extracts_function_prototypes() {
        let source = r#"
int add(int a, int b);
void process(const char* data, size_t len);
"#;

        let result = parse_c(source, "prototypes.h");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "add" && e.kind == ExportKind::Function));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "process" && e.kind == ExportKind::Function));
    }
}
