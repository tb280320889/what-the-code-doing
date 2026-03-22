use std::collections::BTreeSet;
use std::sync::Mutex;
use std::time::Instant;
use tree_sitter::{Node, Parser};
use wtcd_core::adapter::LanguageAdapter;
use wtcd_core::types::*;

const META_PREFIX: &str = "py-meta:";

pub struct PyAdapter {
    parser: Mutex<Parser>,
}

impl PyAdapter {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())?;
        Ok(Self {
            parser: Mutex::new(parser),
        })
    }
}

impl LanguageAdapter for PyAdapter {
    fn language_name(&self) -> &str {
        "python"
    }

    fn file_extensions(&self) -> &[&str] {
        &["py", "pyi"]
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

        let all_symbols = extract_dunder_all(source);

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

        if let Some(all_set) = &all_symbols {
            exports.retain(|e| all_set.contains(e.name.as_str()));
            side_effects.push(meta_line(
                SideEffectKind::Log,
                format!(
                    "{META_PREFIX}dunder_all:{}",
                    all_set.iter().cloned().collect::<Vec<_>>().join(",")
                ),
                1,
            ));
        }

        if file_path.ends_with("__init__.py") {
            exports.push(ExportedSymbol {
                name: "__package__".to_string(),
                kind: ExportKind::Const,
                line: 1,
                is_generated: false,
                confidence: ConfidenceBand::High,
            });
            side_effects.push(meta_line(
                SideEffectKind::Log,
                format!("{META_PREFIX}package_marker:true"),
                1,
            ));
        }

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

fn meta_line(kind: SideEffectKind, target: String, line: u32) -> SideEffect {
    SideEffect { kind, target, line }
}

fn extract_dunder_all(source: &str) -> Option<BTreeSet<String>> {
    let mut merged = String::new();
    let mut capture = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("__all__") && trimmed.contains('=') {
            capture = true;
        }
        if capture {
            merged.push_str(trimmed);
            if trimmed.contains(']') || trimmed.contains(')') {
                break;
            }
        }
    }

    if merged.is_empty() {
        return None;
    }

    let rhs = merged.split_once('=')?.1;
    let mut out = BTreeSet::new();
    let mut cur = String::new();
    let mut in_string = false;
    let mut quote = '\0';
    for ch in rhs.chars() {
        if !in_string {
            if ch == '"' || ch == '\'' {
                in_string = true;
                quote = ch;
                cur.clear();
            }
            continue;
        }
        if ch == quote {
            if !cur.is_empty() {
                out.insert(cur.clone());
            }
            in_string = false;
            quote = '\0';
            continue;
        }
        cur.push(ch);
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn extract_exports_and_signatures(
    root: &Node,
    source: &str,
    exports: &mut Vec<ExportedSymbol>,
    signatures: &mut Vec<FunctionSignature>,
    side_effects: &mut Vec<SideEffect>,
) {
    visit_node(*root, source, exports, signatures, side_effects, &[]);
}

fn visit_node(
    node: Node,
    source: &str,
    exports: &mut Vec<ExportedSymbol>,
    signatures: &mut Vec<FunctionSignature>,
    side_effects: &mut Vec<SideEffect>,
    inherited_decorators: &[String],
) {
    let decorators = if node.kind() == "decorated_definition" {
        let mut d = Vec::new();
        for child in node.children(&mut node.walk()) {
            if child.kind() == "decorator" {
                let text = child
                    .utf8_text(source.as_bytes())
                    .unwrap_or("")
                    .trim()
                    .trim_start_matches('@')
                    .to_string();
                if !text.is_empty() {
                    d.push(text);
                }
            }
        }
        d
    } else {
        inherited_decorators.to_vec()
    };

    if node.kind() == "function_definition" {
        let name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("<anonymous>")
            .to_string();

        exports.push(ExportedSymbol {
            name: name.clone(),
            kind: ExportKind::Function,
            line: node.start_position().row as u32 + 1,
            is_generated: false,
            confidence: ConfidenceBand::High,
        });

        let (parameters, return_type) = extract_py_signature(node, source);
        signatures.push(FunctionSignature {
            name,
            parameters,
            return_type,
        });

        for deco in &decorators {
            side_effects.push(meta_line(
                SideEffectKind::Log,
                format!("{META_PREFIX}decorator:function:{deco}"),
                node.start_position().row as u32 + 1,
            ));
            if deco.ends_with("staticmethod") {
                side_effects.push(meta_line(
                    SideEffectKind::Log,
                    format!("{META_PREFIX}method_type:staticmethod"),
                    node.start_position().row as u32 + 1,
                ));
            }
            if deco.ends_with("classmethod") {
                side_effects.push(meta_line(
                    SideEffectKind::Log,
                    format!("{META_PREFIX}method_type:classmethod"),
                    node.start_position().row as u32 + 1,
                ));
            }
            if deco.ends_with("property") {
                side_effects.push(meta_line(
                    SideEffectKind::Log,
                    format!("{META_PREFIX}method_type:property"),
                    node.start_position().row as u32 + 1,
                ));
            }
        }
    }

    if node.kind() == "class_definition" {
        let name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("<anonymous>")
            .to_string();
        exports.push(ExportedSymbol {
            name,
            kind: ExportKind::Class,
            line: node.start_position().row as u32 + 1,
            is_generated: false,
            confidence: ConfidenceBand::High,
        });

        let bases = extract_class_bases(node, source);
        for base in &bases {
            side_effects.push(meta_line(
                SideEffectKind::Log,
                format!("{META_PREFIX}class_base:{base}"),
                node.start_position().row as u32 + 1,
            ));
            if base.ends_with("BaseModel") {
                side_effects.push(meta_line(
                    SideEffectKind::Log,
                    format!("{META_PREFIX}pattern:pydantic_basemodel"),
                    node.start_position().row as u32 + 1,
                ));
            }
        }

        for deco in &decorators {
            side_effects.push(meta_line(
                SideEffectKind::Log,
                format!("{META_PREFIX}decorator:class:{deco}"),
                node.start_position().row as u32 + 1,
            ));
            if deco.ends_with("dataclass") {
                side_effects.push(meta_line(
                    SideEffectKind::Log,
                    format!("{META_PREFIX}pattern:dataclass"),
                    node.start_position().row as u32 + 1,
                ));
            }
        }
    }

    for child in node.children(&mut node.walk()) {
        visit_node(
            child,
            source,
            exports,
            signatures,
            side_effects,
            &decorators,
        );
    }
}

fn extract_py_signature(node: Node, source: &str) -> (Vec<Parameter>, String) {
    let mut out = Vec::new();
    if let Some(params) = node.child_by_field_name("parameters") {
        for p in params.children(&mut params.walk()) {
            if p.kind() == "identifier" {
                let name = p.utf8_text(source.as_bytes()).unwrap_or("?").to_string();
                out.push(Parameter {
                    name,
                    type_annotation: "unknown".to_string(),
                });
            }

            if p.kind() == "typed_parameter"
                || p.kind() == "default_parameter"
                || p.kind() == "typed_default_parameter"
                || p.kind() == "list_splat_pattern"
                || p.kind() == "dictionary_splat_pattern"
            {
                let name = p
                    .child_by_field_name("name")
                    .or_else(|| p.children(&mut p.walk()).find(|c| c.kind() == "identifier"))
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("?")
                    .to_string();

                let annotation = p
                    .child_by_field_name("type")
                    .or_else(|| {
                        p.children(&mut p.walk()).find(|c| {
                            c.kind().contains("type")
                                || c.kind() == "generic_type"
                                || c.kind() == "identifier"
                        })
                    })
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("unknown")
                    .to_string();

                out.push(Parameter {
                    name,
                    type_annotation: annotation,
                });
            }
        }
    }

    let return_type = node
        .child_by_field_name("return_type")
        .or_else(|| {
            node.children(&mut node.walk())
                .find(|c| c.kind() == "type" || c.kind() == "generic_type")
        })
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("unknown")
        .trim_start_matches("->")
        .trim()
        .to_string();

    (out, return_type)
}

fn extract_class_bases(node: Node, source: &str) -> Vec<String> {
    if let Some(superclasses) = node.child_by_field_name("superclasses") {
        let text = superclasses.utf8_text(source.as_bytes()).unwrap_or("");
        return text
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    Vec::new()
}

fn extract_imports(source: &str, out: &mut Vec<DependencyEdge>) {
    for line in source.lines() {
        let clean = line.split('#').next().unwrap_or("").trim();
        if clean.is_empty() {
            continue;
        }

        if let Some(rest) = clean.strip_prefix("import ") {
            for part in rest.split(',') {
                let module = part.split_whitespace().next().unwrap_or("").trim();
                if module.is_empty() {
                    continue;
                }
                out.push(DependencyEdge {
                    source: module.to_string(),
                    symbols: vec![],
                    kind: ImportKind::Namespace,
                });
            }
            continue;
        }

        if let Some(rest) = clean.strip_prefix("from ") {
            if let Some((module, imported)) = rest.split_once(" import ") {
                let module = module.trim();
                if module.is_empty() {
                    continue;
                }
                let symbols = imported
                    .trim()
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .split(',')
                    .map(|s| s.split_whitespace().next().unwrap_or("").trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                out.push(DependencyEdge {
                    source: module.to_string(),
                    symbols,
                    kind: ImportKind::Named,
                });
            }
        }
    }
}

const PY_SIDE_EFFECT_APIS: &[(&str, SideEffectKind)] = &[
    ("open(", SideEffectKind::Io),
    ("requests.", SideEffectKind::Network),
    ("urllib.", SideEffectKind::Network),
    ("sqlite3.", SideEffectKind::Storage),
    ("psycopg2.", SideEffectKind::Storage),
    ("print(", SideEffectKind::Log),
    ("logging.", SideEffectKind::Log),
    ("subprocess.", SideEffectKind::Io),
    ("sys.exit(", SideEffectKind::Io),
    ("os.environ", SideEffectKind::Io),
];

fn extract_side_effects(root: &Node, source: &str, out: &mut Vec<SideEffect>) {
    let mut cursor = root.walk();
    let mut stack = vec![*root];
    while let Some(node) = stack.pop() {
        if node.kind() == "call" || node.kind() == "call_expression" {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            for (api, kind) in PY_SIDE_EFFECT_APIS {
                if text.contains(api) {
                    out.push(SideEffect {
                        kind: kind.clone(),
                        target: api.trim_end_matches('(').to_string(),
                        line: node.start_position().row as u32 + 1,
                    });
                    break;
                }
            }
        }
        stack.extend(node.children(&mut cursor));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_py(source: &str, file_path: &str) -> FileResult {
        let adapter = PyAdapter::new().unwrap();
        adapter.parse(source, file_path)
    }

    #[test]
    fn extracts_function_with_types() {
        let result = parse_py("def greet(name: str) -> str:\n    return name\n", "a.py");
        assert_eq!(result.confidence, ConfidenceBand::High);
        assert_eq!(
            result.exports.iter().filter(|e| e.name == "greet").count(),
            1
        );
        let sig = result
            .signatures
            .iter()
            .find(|s| s.name == "greet")
            .unwrap();
        assert_eq!(sig.parameters[0].name, "name");
        assert!(sig.parameters[0].type_annotation.contains("str"));
    }

    #[test]
    fn extracts_class_and_bases_meta() {
        let result = parse_py("class Dog(Animal):\n    pass\n", "c.py");
        assert!(result
            .exports
            .iter()
            .any(|e| e.name == "Dog" && e.kind == ExportKind::Class));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("class_base:Animal")));
    }

    #[test]
    fn extracts_imports_and_relative() {
        let result = parse_py(
            "import os\nfrom pathlib import Path\nfrom .core import x\n",
            "i.py",
        );
        assert!(result.imports.iter().any(|i| i.source == "os"));
        assert!(result.imports.iter().any(|i| i.source == "pathlib"));
        assert!(result.imports.iter().any(|i| i.source == ".core"));
    }

    #[test]
    fn captures_decorator_meta() {
        let result = parse_py("@dataclass\nclass Point:\n    x: int\n", "d.py");
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("decorator:class:dataclass")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("pattern:dataclass")));
    }

    #[test]
    fn detects_package_marker_and_dunder_all() {
        let result = parse_py(
            "__all__ = [\"public\"]\n\ndef public():\n    pass\n\ndef hidden():\n    pass\n",
            "pkg/__init__.py",
        );
        assert!(result.exports.iter().any(|e| e.name == "public"));
        assert!(!result.exports.iter().any(|e| e.name == "hidden"));
        assert!(result.exports.iter().any(|e| e.name == "__package__"));
    }

    #[test]
    fn syntax_error_is_low_or_none() {
        let result = parse_py("def broken(\n", "broken.py");
        assert!(
            result.confidence == ConfidenceBand::Low || result.confidence == ConfidenceBand::None
        );
    }

    #[test]
    fn detects_method_kinds_and_pydantic() {
        let source = "
from pydantic import BaseModel

class User(BaseModel):
    @staticmethod
    def s():
        return 1

    @classmethod
    def c(cls):
        return cls

    @property
    def p(self):
        return 42
";
        let result = parse_py(source, "model.py");
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("pattern:pydantic_basemodel")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("method_type:staticmethod")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("method_type:classmethod")));
        assert!(result
            .side_effects
            .iter()
            .any(|s| s.target.contains("method_type:property")));
    }
}
