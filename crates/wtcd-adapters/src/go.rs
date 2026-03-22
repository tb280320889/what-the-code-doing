use std::sync::Mutex;
use std::time::Instant;

use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "go-meta:";

pub struct GoAdapter {
    parser: Mutex<Parser>,
}

impl GoAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_go::LANGUAGE.into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for GoAdapter {
    fn language_name(&self) -> &str {
        "go"
    }

    fn file_extensions(&self) -> &[&str] {
        &["go"]
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
                }
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
        extract_side_effects(source, &mut side_effects);

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

fn is_exported(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

fn push_visibility_meta(name: &str, line: u32, side_effects: &mut Vec<SideEffect>) {
    let visibility = if is_exported(name) {
        "public"
    } else {
        "private"
    };
    side_effects.push(SideEffect {
        kind: SideEffectKind::Log,
        target: format!("{META_PREFIX}visibility:{name}:{visibility}"),
        line,
    });
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
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
                push_visibility_meta(&name, line, side_effects);

                let (parameters, return_type) = extract_go_signature(node, source);
                signatures.push(FunctionSignature {
                    name,
                    parameters,
                    return_type,
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
                push_visibility_meta(&name, line, side_effects);

                if let Some(receiver) = node.child_by_field_name("receiver") {
                    if let Ok(receiver_text) = receiver.utf8_text(source.as_bytes()) {
                        side_effects.push(SideEffect {
                            kind: SideEffectKind::Log,
                            target: format!("{META_PREFIX}receiver:{}", receiver_text.trim()),
                            line,
                        });
                    }
                }

                let (parameters, return_type) = extract_go_signature(node, source);
                signatures.push(FunctionSignature {
                    name,
                    parameters,
                    return_type,
                });
            }
        }
        "type_declaration" => {
            let line = node.start_position().row as u32 + 1;
            for spec in node.children(&mut node.walk()) {
                if spec.kind() != "type_spec" {
                    continue;
                }

                let name = spec
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("?")
                    .to_string();

                let kind = spec
                    .child_by_field_name("type")
                    .map(|n| n.kind())
                    .unwrap_or("");

                let export_kind = match kind {
                    "struct_type" => ExportKind::Class,
                    "interface_type" => ExportKind::Interface,
                    _ => ExportKind::Type,
                };

                exports.push(ExportedSymbol {
                    name: name.clone(),
                    kind: export_kind,
                    line,
                    is_generated: false,
                    confidence: ConfidenceBand::High,
                });
                push_visibility_meta(&name, line, side_effects);

                if kind == "struct_type" {
                    collect_struct_meta(spec, source, line, side_effects);
                }
                if kind == "interface_type" {
                    collect_interface_meta(spec, source, line, side_effects);
                }
            }
        }
        "const_declaration" | "var_declaration" => {
            let line = node.start_position().row as u32 + 1;
            let export_kind = if node.kind() == "const_declaration" {
                ExportKind::Const
            } else {
                ExportKind::Var
            };

            for spec in node.children(&mut node.walk()) {
                if spec.kind() != "const_spec" && spec.kind() != "var_spec" {
                    continue;
                }
                for n in spec.children(&mut spec.walk()) {
                    if n.kind() == "identifier" {
                        let name = n.utf8_text(source.as_bytes()).unwrap_or("?").to_string();
                        exports.push(ExportedSymbol {
                            name: name.clone(),
                            kind: export_kind.clone(),
                            line,
                            is_generated: false,
                            confidence: ConfidenceBand::High,
                        });
                        push_visibility_meta(&name, line, side_effects);
                    }
                }
            }
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        visit_node(child, source, exports, signatures, side_effects);
    }
}

fn collect_struct_meta(spec: Node, source: &str, line: u32, side_effects: &mut Vec<SideEffect>) {
    if let Some(struct_type) = spec.child_by_field_name("type") {
        for child in struct_type.children(&mut struct_type.walk()) {
            if child.kind() != "field_declaration" && child.kind() != "embedded_field" {
                continue;
            }

            let text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();
            if text.is_empty() {
                continue;
            }
            side_effects.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}struct_field:{}", text),
                line,
            });

            let token_count = text.split_whitespace().filter(|s| !s.is_empty()).count();
            if child.kind() == "embedded_field" || token_count == 1 {
                side_effects.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}embedded_struct:{}", text),
                    line,
                });
            }
        }
    }
}

fn collect_interface_meta(spec: Node, source: &str, line: u32, side_effects: &mut Vec<SideEffect>) {
    if let Some(interface_type) = spec.child_by_field_name("type") {
        for child in interface_type.children(&mut interface_type.walk()) {
            if child.kind() != "method_spec" {
                continue;
            }
            let text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();
            if text.is_empty() {
                continue;
            }
            side_effects.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!("{META_PREFIX}interface_method:{}", text),
                line,
            });
        }
    }
}

fn extract_go_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut parameters = Vec::new();

    if let Some(params) = node.child_by_field_name("parameters") {
        for child in params.children(&mut params.walk()) {
            if child.kind() != "parameter_declaration" {
                continue;
            }

            let text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();
            if text.is_empty() {
                continue;
            }

            let parts = text.split_whitespace().collect::<Vec<_>>();
            if parts.len() >= 2 {
                for param_name in &parts[..parts.len() - 1] {
                    parameters.push(Parameter {
                        name: (*param_name).to_string(),
                        type_annotation: parts[parts.len() - 1].to_string(),
                    });
                }
            } else {
                parameters.push(Parameter {
                    name: text.to_string(),
                    type_annotation: "unknown".to_string(),
                });
            }
        }
    }

    let return_type = node
        .child_by_field_name("result")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("void")
        .trim()
        .to_string();

    (parameters, return_type)
}

fn extract_imports(source: &str, out: &mut Vec<DependencyEdge>) {
    let mut in_block = false;
    for line in source.lines() {
        let clean = line.split("//").next().unwrap_or("").trim();
        if clean.is_empty() {
            continue;
        }

        if clean == "import (" {
            in_block = true;
            continue;
        }
        if in_block && clean == ")" {
            in_block = false;
            continue;
        }

        let import_line = if in_block {
            clean
        } else if let Some(rest) = clean.strip_prefix("import ") {
            rest.trim()
        } else {
            continue;
        };

        let mut kind = ImportKind::Named;
        let mut source_path = String::new();
        let mut symbols = Vec::new();

        if import_line.starts_with('"') {
            source_path = import_line.trim_matches('"').to_string();
        } else {
            let parts = import_line.split_whitespace().collect::<Vec<_>>();
            if parts.len() >= 2 {
                let alias = parts[0];
                source_path = parts[1].trim_matches('"').to_string();
                if alias == "." {
                    kind = ImportKind::Namespace;
                    symbols.push(".".to_string());
                } else if alias != "_" {
                    kind = ImportKind::Default;
                    symbols.push(alias.to_string());
                }
            }
        }

        if !source_path.is_empty() {
            out.push(DependencyEdge {
                source: source_path,
                symbols,
                kind,
            });
        }
    }
}

fn extract_side_effects(source: &str, out: &mut Vec<SideEffect>) {
    let mut in_struct = false;
    let mut in_interface = false;
    for (idx, line) in source.lines().enumerate() {
        let line_no = idx as u32 + 1;
        let clean = line.trim();

        if clean.contains("struct {") {
            in_struct = true;
        } else if clean.contains("interface {") {
            in_interface = true;
        } else if in_struct && clean == "}" {
            in_struct = false;
        } else if in_interface && clean == "}" {
            in_interface = false;
        }

        if in_struct {
            let field_part = clean.split('`').next().unwrap_or("").trim();
            let tokens = field_part
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            if tokens.len() >= 2
                && !tokens[0].starts_with("//")
                && tokens[0] != "type"
                && tokens[0] != "struct"
            {
                out.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!(
                        "{META_PREFIX}struct_field:{} {}",
                        tokens[0],
                        tokens[1..].join(" ")
                    ),
                    line: line_no,
                });
            }
            if tokens.len() == 1 && !tokens[0].starts_with("//") && tokens[0] != "{" {
                out.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}embedded_struct:{}", tokens[0]),
                    line: line_no,
                });
            }
        }

        if in_interface
            && clean.contains('(')
            && clean.contains(')')
            && !clean.starts_with("type ")
            && clean != "interface {"
        {
            let method_name = clean
                .split('(')
                .next()
                .unwrap_or("")
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim();
            if !method_name.is_empty() {
                out.push(SideEffect {
                    kind: SideEffectKind::Log,
                    target: format!("{META_PREFIX}interface_method:{method_name}"),
                    line: line_no,
                });
            }
        }

        if clean.contains("go ") {
            out.push(SideEffect {
                kind: SideEffectKind::Network,
                target: format!("{META_PREFIX}goroutine"),
                line: line_no,
            });
        }
        if clean.contains("<-") || clean.contains("chan ") {
            out.push(SideEffect {
                kind: SideEffectKind::Io,
                target: format!("{META_PREFIX}channel"),
                line: line_no,
            });
        }
        if clean.starts_with("//go:") {
            out.push(SideEffect {
                kind: SideEffectKind::Log,
                target: format!(
                    "{META_PREFIX}directive:{}",
                    clean.trim_start_matches("//go:")
                ),
                line: line_no,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_go(source: &str, file_path: &str) -> FileResult {
        let adapter = GoAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_functions_methods_structs_interfaces_vars_and_consts() {
        let source = r#"
package demo

import (
  "fmt"
  ioalias "io"
)

type Person struct {
  Name string `json:"name"`
  Age int
  Base
}

type Base struct{}

type Reader interface {
  Read(p []byte) (n int, err error)
}

const Version = "1"
var internalCounter = 1

func Add(a int, b int) int { return a + b }
func (p *Person) Greet() string { return p.Name }
"#;

        let result = parse_go(source, "demo.go");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Add" && e.kind == ExportKind::Function));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Greet" && e.kind == ExportKind::Function));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Person" && e.kind == ExportKind::Class));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Reader" && e.kind == ExportKind::Interface));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Version" && e.kind == ExportKind::Const));
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "internalCounter" && e.kind == ExportKind::Var));

        assert!(result.imports.iter().any(|i| i.source == "fmt"));
        assert!(result.imports.iter().any(|i| i.source == "io"));

        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("go-meta:receiver:(p *Person)")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("go-meta:embedded_struct:Base")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("go-meta:interface_method:Read")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("go-meta:struct_field:Name string")));
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
    fn extracts_goroutine_channel_and_directive_side_effects() {
        let source = r#"
package demo

//go:generate stringer -type=Status
func Work(ch chan int) {
  go func() { ch <- 1 }()
}
"#;

        let result = parse_go(source, "effect.go");
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
            .any(|s| s.target.contains("go-meta:directive:generate")));
    }

    #[test]
    fn syntax_error_degrades_confidence() {
        let result = parse_go("package demo\nfunc Broken( {\n", "broken.go");
        assert!(
            result.confidence == ConfidenceBand::Low || result.confidence == ConfidenceBand::None
        );
    }
}
