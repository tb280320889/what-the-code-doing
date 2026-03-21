use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

/// TypeScript/JavaScript adapter using tree-sitter (LANG-01)
pub struct TsAdapter {
    ts_parser: Mutex<Parser>,
    js_parser: Mutex<Parser>,
}

impl TsAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut ts_parser = Parser::new();
        ts_parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;

        let mut js_parser = Parser::new();
        js_parser.set_language(&tree_sitter_javascript::LANGUAGE.into())?;

        Ok(Self {
            ts_parser: Mutex::new(ts_parser),
            js_parser: Mutex::new(js_parser),
        })
    }

    fn is_js_file(path: &str) -> bool {
        path.ends_with(".js") || path.ends_with(".jsx")
    }
}

impl LanguageAdapter for TsAdapter {
    fn language_name(&self) -> &str {
        "typescript"
    }

    fn file_extensions(&self) -> &[&str] {
        &["ts", "tsx", "js", "jsx"]
    }

    fn parse(&self, source: &str, file_path: &str) -> FileResult {
        let start = Instant::now();

        // Choose parser based on file extension
        let tree = if Self::is_js_file(file_path) {
            self.js_parser.lock().unwrap().parse(source, None)
        } else {
            self.ts_parser.lock().unwrap().parse(source, None)
        };

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

        // Determine confidence (D-12, D-13, D-14)
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

        extract_exports(&root, source, &mut exports);
        extract_imports(&root, source, &mut imports);
        extract_signatures(&root, source, &mut signatures);
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

// ========== Export Extraction (D-16) ==========

fn extract_exports(root: &Node, source: &str, out: &mut Vec<ExportedSymbol>) {
    let mut cursor = root.walk();
    let mut reached_root = false;
    loop {
        let node = cursor.node();

        if node.kind() == "export_statement" {
            if let Some(decl) = node.child_by_field_name("declaration") {
                match decl.kind() {
                    "function_declaration" => {
                        if let Some(name_node) = decl.child_by_field_name("name") {
                            out.push(ExportedSymbol {
                                name: name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("?")
                                    .to_string(),
                                kind: ExportKind::Function,
                                line: node.start_position().row as u32 + 1,
                            });
                        }
                    }
                    "class_declaration" => {
                        if let Some(name_node) = decl.child_by_field_name("name") {
                            out.push(ExportedSymbol {
                                name: name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("?")
                                    .to_string(),
                                kind: ExportKind::Class,
                                line: node.start_position().row as u32 + 1,
                            });
                        }
                    }
                    "lexical_declaration" => {
                        let is_let = decl
                            .utf8_text(source.as_bytes())
                            .unwrap_or("const")
                            .starts_with("let");
                        for child in decl.children(&mut decl.walk()) {
                            if child.kind() == "variable_declarator" {
                                if let Some(name_node) = child.child_by_field_name("name") {
                                    let kind = if is_let {
                                        ExportKind::Let
                                    } else {
                                        ExportKind::Const
                                    };
                                    out.push(ExportedSymbol {
                                        name: name_node
                                            .utf8_text(source.as_bytes())
                                            .unwrap_or("?")
                                            .to_string(),
                                        kind,
                                        line: node.start_position().row as u32 + 1,
                                    });
                                }
                            }
                        }
                    }
                    "variable_declaration" => {
                        for child in decl.children(&mut decl.walk()) {
                            if child.kind() == "variable_declarator" {
                                if let Some(name_node) = child.child_by_field_name("name") {
                                    out.push(ExportedSymbol {
                                        name: name_node
                                            .utf8_text(source.as_bytes())
                                            .unwrap_or("?")
                                            .to_string(),
                                        kind: ExportKind::Var,
                                        line: node.start_position().row as u32 + 1,
                                    });
                                }
                            }
                        }
                    }
                    "type_alias_declaration" => {
                        if let Some(name_node) = decl.child_by_field_name("name") {
                            out.push(ExportedSymbol {
                                name: name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("?")
                                    .to_string(),
                                kind: ExportKind::Type,
                                line: node.start_position().row as u32 + 1,
                            });
                        }
                    }
                    "interface_declaration" => {
                        if let Some(name_node) = decl.child_by_field_name("name") {
                            out.push(ExportedSymbol {
                                name: name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("?")
                                    .to_string(),
                                kind: ExportKind::Interface,
                                line: node.start_position().row as u32 + 1,
                            });
                        }
                    }
                    "enum_declaration" => {
                        if let Some(name_node) = decl.child_by_field_name("name") {
                            out.push(ExportedSymbol {
                                name: name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("?")
                                    .to_string(),
                                kind: ExportKind::Enum,
                                line: node.start_position().row as u32 + 1,
                            });
                        }
                    }
                    _ => {}
                }
            }

            // Handle re-exports: export { X } from "Y"
            if let Some(source_node) = node.child_by_field_name("source") {
                let module_source = source_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("\"\"")
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                if let Some(export_clause) = node
                    .children(&mut node.walk())
                    .find(|c| c.kind() == "export_clause")
                {
                    let symbols: Vec<String> = export_clause
                        .children(&mut export_clause.walk())
                        .filter(|c| c.kind() == "export_specifier")
                        .filter_map(|spec| spec.child_by_field_name("name"))
                        .map(|n| n.utf8_text(source.as_bytes()).unwrap_or("?").to_string())
                        .collect();

                    for sym in &symbols {
                        out.push(ExportedSymbol {
                            name: sym.clone(),
                            kind: ExportKind::Const,
                            line: node.start_position().row as u32 + 1,
                        });
                    }
                    let _ = module_source;
                }
            }
        }

        // Traverse
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if !reached_root && cursor.node().id() == root.id() {
                reached_root = true;
            }
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
            if cursor.node().id() == root.id() {
                return;
            }
        }
    }
}

// ========== Import Extraction (D-17) ==========

fn extract_imports(root: &Node, source: &str, out: &mut Vec<DependencyEdge>) {
    let mut cursor = root.walk();
    let mut reached_root = false;
    loop {
        let node = cursor.node();

        // ESM imports
        if node.kind() == "import_statement" {
            let module_source = node
                .child_by_field_name("source")
                .and_then(|s| s.utf8_text(source.as_bytes()).ok())
                .unwrap_or("\"\"")
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            let mut symbols = Vec::new();
            let mut kind = ImportKind::Named;

            if let Some(clause) = node.child_by_field_name("import") {
                match clause.kind() {
                    "named_imports" => {
                        kind = ImportKind::Named;
                        for child in clause.children(&mut clause.walk()) {
                            if child.kind() == "import_specifier" {
                                if let Some(name) = child.child_by_field_name("name") {
                                    symbols.push(
                                        name.utf8_text(source.as_bytes())
                                            .unwrap_or("?")
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                    "identifier" => {
                        kind = ImportKind::Default;
                        symbols.push(
                            clause
                                .utf8_text(source.as_bytes())
                                .unwrap_or("?")
                                .to_string(),
                        );
                    }
                    "namespace_import" => {
                        kind = ImportKind::Namespace;
                        if let Some(id) = clause.named_child(0) {
                            symbols
                                .push(id.utf8_text(source.as_bytes()).unwrap_or("?").to_string());
                        }
                    }
                    _ => {}
                }
            }

            if !symbols.is_empty() || !module_source.is_empty() {
                out.push(DependencyEdge {
                    source: module_source,
                    symbols,
                    kind,
                });
            }
        }

        // CommonJS require() (D-17)
        if node.kind() == "call_expression" {
            let is_require = node
                .child_by_field_name("function")
                .and_then(|f| f.utf8_text(source.as_bytes()).ok())
                .map(|f| f == "require")
                .unwrap_or(false);

            if is_require {
                if let Some(args) = node.child_by_field_name("arguments") {
                    let source_str = args
                        .children(&mut args.walk())
                        .find(|c| c.kind() == "string")
                        .and_then(|s| s.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("\"\"")
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();

                    out.push(DependencyEdge {
                        source: source_str,
                        symbols: vec![],
                        kind: ImportKind::Named,
                    });
                }
            }
        }

        // Traverse
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if !reached_root && cursor.node().id() == root.id() {
                reached_root = true;
            }
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
            if cursor.node().id() == root.id() {
                return;
            }
        }
    }
}

// ========== Function Signature Extraction (D-18) ==========

fn extract_signatures(root: &Node, source: &str, out: &mut Vec<FunctionSignature>) {
    let mut cursor = root.walk();
    let mut reached_root = false;
    loop {
        let node = cursor.node();

        match node.kind() {
            "function_declaration" | "method_definition" | "function" => {
                let name = node
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("<anonymous>")
                    .to_string();

                let params = extract_parameters(&node, source);
                let return_type = node
                    .child_by_field_name("return_type")
                    .and_then(|rt| rt.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("unknown")
                    .trim_start_matches(':')
                    .trim()
                    .to_string();

                out.push(FunctionSignature {
                    name,
                    parameters: params,
                    return_type,
                });
            }
            "arrow_function" => {
                let name = node
                    .parent()
                    .and_then(|p| p.parent())
                    .filter(|gp| gp.kind() == "variable_declarator")
                    .and_then(|gp| gp.child_by_field_name("name"))
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("<anonymous>")
                    .to_string();

                let params = extract_parameters(&node, source);
                let return_type = node
                    .child_by_field_name("return_type")
                    .and_then(|rt| rt.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("unknown")
                    .trim_start_matches(':')
                    .trim()
                    .to_string();

                out.push(FunctionSignature {
                    name,
                    parameters: params,
                    return_type,
                });
            }
            _ => {}
        }

        // Traverse
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if !reached_root && cursor.node().id() == root.id() {
                reached_root = true;
            }
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
            if cursor.node().id() == root.id() {
                return;
            }
        }
    }
}

fn extract_parameters(func_node: &Node, source: &str) -> Vec<Parameter> {
    let mut params = Vec::new();
    if let Some(params_node) = func_node.child_by_field_name("parameters") {
        for child in params_node.children(&mut params_node.walk()) {
            match child.kind() {
                "required_parameter" | "optional_parameter" => {
                    let name = child
                        .child_by_field_name("pattern")
                        .or_else(|| child.child_by_field_name("name"))
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("?")
                        .to_string();

                    let type_annotation = child
                        .child_by_field_name("type")
                        .and_then(|t| t.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("unknown")
                        .trim_start_matches(':')
                        .trim()
                        .to_string();

                    params.push(Parameter {
                        name,
                        type_annotation,
                    });
                }
                "identifier" => {
                    params.push(Parameter {
                        name: child
                            .utf8_text(source.as_bytes())
                            .unwrap_or("?")
                            .to_string(),
                        type_annotation: "unknown".to_string(),
                    });
                }
                _ => {}
            }
        }
    }
    params
}

// ========== Side Effect Detection (D-19) ==========

const SIDE_EFFECT_APIS: &[(&str, SideEffectKind)] = &[
    ("fs.", SideEffectKind::Io),
    ("fetch", SideEffectKind::Network),
    ("axios.", SideEffectKind::Network),
    ("axios", SideEffectKind::Network),
    ("console.", SideEffectKind::Log),
    ("process.exit", SideEffectKind::Io),
    ("localStorage.", SideEffectKind::Storage),
    ("sessionStorage.", SideEffectKind::Storage),
];

fn extract_side_effects(root: &Node, source: &str, out: &mut Vec<SideEffect>) {
    let mut cursor = root.walk();
    let mut reached_root = false;
    loop {
        let node = cursor.node();

        if node.kind() == "call_expression" {
            if let Some(func) = node.child_by_field_name("function") {
                let text = func.utf8_text(source.as_bytes()).unwrap_or("");

                for (api_prefix, kind) in SIDE_EFFECT_APIS {
                    if text.starts_with(api_prefix) || text == *api_prefix {
                        out.push(SideEffect {
                            kind: kind.clone(),
                            target: text.to_string(),
                            line: node.start_position().row as u32 + 1,
                        });
                        break;
                    }
                }
            }
        }

        // Traverse
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if !reached_root && cursor.node().id() == root.id() {
                reached_root = true;
            }
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
            if cursor.node().id() == root.id() {
                return;
            }
        }
    }
}
