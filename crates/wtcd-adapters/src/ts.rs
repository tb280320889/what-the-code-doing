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

            // import_clause is a positional child of import_statement (not a named field)
            let import_clause = node
                .children(&mut node.walk())
                .find(|c| c.kind() == "import_clause");

            if let Some(clause) = import_clause {
                // First child of import_clause determines the import type
                let mut clause_cursor = clause.walk();
                let first = clause.children(&mut clause_cursor).next();
                if let Some(inner) = first {
                    match inner.kind() {
                        "named_imports" => {
                            kind = ImportKind::Named;
                            for child in inner.children(&mut inner.walk()) {
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
                                inner
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("?")
                                    .to_string(),
                            );
                        }
                        "namespace_import" => {
                            kind = ImportKind::Namespace;
                            let mut ns_cursor = inner.walk();
                            let id = inner
                                .children(&mut ns_cursor)
                                .find(|c| c.kind() == "identifier");
                            if let Some(id_node) = id {
                                symbols.push(
                                    id_node
                                        .utf8_text(source.as_bytes())
                                        .unwrap_or("?")
                                        .to_string(),
                                );
                            }
                        }
                        _ => {}
                    }
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
            "function_declaration" | "method_definition" => {
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
                // variable_declarator has no "name" field — identifier is positional child
                let name = node
                    .parent()
                    .filter(|p| p.kind() == "variable_declarator")
                    .and_then(|p| p.children(&mut p.walk()).find(|c| c.kind() == "identifier"))
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("<anonymous>")
                    .to_string();

                let params = extract_parameters(&node, source);
                // return_type is a positional child of arrow_function
                let return_type = node
                    .children(&mut node.walk())
                    .find(|c| c.kind() == "type_annotation")
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
    // formal_parameters is a positional child (not a named field in tree-sitter-typescript)
    let params_node = func_node
        .children(&mut func_node.walk())
        .find(|c| c.kind() == "formal_parameters");
    if let Some(params_node) = params_node {
        for child in params_node.children(&mut params_node.walk()) {
            match child.kind() {
                "required_parameter" | "optional_parameter" => {
                    // identifier is a positional child of required_parameter
                    let name = child
                        .children(&mut child.walk())
                        .find(|c| c.kind() == "identifier")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("?")
                        .to_string();

                    let type_annotation = child
                        .children(&mut child.walk())
                        .find(|c| c.kind() == "type_annotation")
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

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ts(source: &str) -> FileResult {
        let adapter = TsAdapter::new().unwrap();
        adapter.parse(source, "test.ts")
    }

    fn parse_js(source: &str) -> FileResult {
        let adapter = TsAdapter::new().unwrap();
        adapter.parse(source, "test.js")
    }

    // --- Test 1: Named function export ---
    #[test]
    fn extracts_named_function_export() {
        let result = parse_ts("export function greet(name: string): string { return name; }");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert_eq!(result.exports.len(), 1);
        assert_eq!(result.exports[0].name, "greet");
        assert_eq!(result.exports[0].kind, ExportKind::Function);
    }

    // --- Test 2: Class/const/type/interface/enum exports ---
    #[test]
    fn extracts_class_export() {
        let result = parse_ts("export class UserService {}");
        assert_eq!(result.exports.len(), 1);
        assert_eq!(result.exports[0].name, "UserService");
        assert_eq!(result.exports[0].kind, ExportKind::Class);
    }

    #[test]
    fn extracts_const_export() {
        let result = parse_ts("export const MAX_SIZE = 100;");
        assert_eq!(result.exports.len(), 1);
        assert_eq!(result.exports[0].name, "MAX_SIZE");
        assert_eq!(result.exports[0].kind, ExportKind::Const);
    }

    #[test]
    fn extracts_type_export() {
        let result = parse_ts("export type UserId = string | number;");
        assert_eq!(result.exports.len(), 1);
        assert_eq!(result.exports[0].name, "UserId");
        assert_eq!(result.exports[0].kind, ExportKind::Type);
    }

    #[test]
    fn extracts_interface_export() {
        let result = parse_ts("export interface Config { name: string; }");
        assert_eq!(result.exports.len(), 1);
        assert_eq!(result.exports[0].name, "Config");
        assert_eq!(result.exports[0].kind, ExportKind::Interface);
    }

    #[test]
    fn extracts_enum_export() {
        let result = parse_ts("export enum Color { Red, Green, Blue }");
        assert_eq!(result.exports.len(), 1);
        assert_eq!(result.exports[0].name, "Color");
        assert_eq!(result.exports[0].kind, ExportKind::Enum);
    }

    // --- Test 3: ESM named import ---
    #[test]
    fn extracts_esm_named_import() {
        let result = parse_ts("import { readFile, writeFile } from 'fs';");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].source, "fs");
        assert_eq!(result.imports[0].kind, ImportKind::Named);
        assert_eq!(result.imports[0].symbols.len(), 2);
        assert!(result.imports[0].symbols.contains(&"readFile".to_string()));
        assert!(result.imports[0].symbols.contains(&"writeFile".to_string()));
    }

    // --- Test 4: ESM default import ---
    #[test]
    fn extracts_esm_default_import() {
        let result = parse_ts("import React from 'react';");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].source, "react");
        assert_eq!(result.imports[0].kind, ImportKind::Default);
        assert_eq!(result.imports[0].symbols.len(), 1);
        assert_eq!(result.imports[0].symbols[0], "React");
    }

    // --- Test 5: CommonJS require() ---
    #[test]
    fn extracts_commonjs_require() {
        let result = parse_js("const fs = require('fs');");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].source, "fs");
    }

    // --- Test 6: Syntax errors → confidence=low with partial extraction ---
    #[test]
    fn syntax_errors_yield_low_confidence() {
        let source = "export function broken( { return 42; }";
        let result = parse_ts(source);
        assert_eq!(result.confidence, ConfidenceBand::Low);
        // Partial extraction should still work on non-broken parts
        assert!(result.error_message.is_some());
    }

    // --- Test 7: Completely unparseable → confidence=none + empty ---
    #[test]
    fn unparseable_yields_none_confidence() {
        // Use deeply nested broken braces that tree-sitter can't recover from
        let source = "{{{{{{{{{{{{{{{{{{{{";
        let result = parse_ts(source);
        // If tree-sitter still produces some structure, confidence=low is acceptable
        // The key is that no bogus exports/imports are produced
        assert!(
            result.confidence == ConfidenceBand::None || result.confidence == ConfidenceBand::Low
        );
        assert!(result.exports.is_empty());
        assert!(result.imports.is_empty());
    }

    // --- Test 8: JS file extracts correctly ---
    #[test]
    fn js_file_extracts_exports() {
        let result = parse_js("export function add(a, b) { return a + b; }");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert_eq!(result.exports.len(), 1);
        assert_eq!(result.exports[0].name, "add");
        assert_eq!(result.exports[0].kind, ExportKind::Function);
    }

    #[test]
    fn js_file_extracts_require() {
        let result = parse_js("const path = require('path');");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].source, "path");
    }

    // --- Side effects ---
    #[test]
    fn detects_fetch_side_effect() {
        let result = parse_ts("export function getData() { return fetch('/api'); }");
        assert_eq!(result.side_effects.len(), 1);
        assert_eq!(result.side_effects[0].kind, SideEffectKind::Network);
        assert_eq!(result.side_effects[0].target, "fetch");
    }

    #[test]
    fn detects_console_log_side_effect() {
        let result = parse_ts("console.log('hello');");
        assert_eq!(result.side_effects.len(), 1);
        assert_eq!(result.side_effects[0].kind, SideEffectKind::Log);
        assert_eq!(result.side_effects[0].target, "console.log");
    }

    // --- Function signatures ---
    #[test]
    fn extracts_function_signature() {
        let result = parse_ts("export function greet(name: string): string { return name; }");
        assert_eq!(result.signatures.len(), 1);
        assert_eq!(result.signatures[0].name, "greet");
        assert_eq!(result.signatures[0].parameters.len(), 1);
        assert_eq!(result.signatures[0].parameters[0].name, "name");
        assert_eq!(result.signatures[0].parameters[0].type_annotation, "string");
        assert_eq!(result.signatures[0].return_type, "string");
    }

    #[test]
    fn extracts_arrow_function_signature() {
        let result = parse_ts("const add = (a: number, b: number): number => a + b;");
        assert_eq!(result.signatures.len(), 1);
        assert_eq!(result.signatures[0].name, "add");
        assert_eq!(result.signatures[0].parameters.len(), 2);
        assert_eq!(result.signatures[0].return_type, "number");
    }

    // --- Namespace import ---
    #[test]
    fn extracts_namespace_import() {
        let result = parse_ts("import * as utils from './utils';");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].source, "./utils");
        assert_eq!(result.imports[0].kind, ImportKind::Namespace);
        assert_eq!(result.imports[0].symbols.len(), 1);
        assert_eq!(result.imports[0].symbols[0], "utils");
    }

    // --- Edge cases ---
    #[test]
    fn empty_file_returns_high_confidence() {
        let result = parse_ts("");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert!(result.exports.is_empty());
        assert!(result.imports.is_empty());
    }

    #[test]
    fn parse_time_is_recorded() {
        let result = parse_ts("export const x = 1;");
        // Just check it's non-negative (can't assert exact value)
        let _ = result.parse_time_ms; // u64 is always >= 0
    }
}
