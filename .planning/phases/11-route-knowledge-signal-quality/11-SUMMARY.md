# Phase 11 Summary: Route & Knowledge Signal Quality

## What Was Done

Route and knowledge layer enhancements for language coverage and confidence signals:

### Type Changes (`crates/wtcd-core/src/types.rs`)
- Added `LanguageCapability` struct with per-language extraction info (file_count, module_count, has_exports/imports/signatures/side_effects, confidence_distribution)
- Added `language_matrix: BTreeMap<String, LanguageCapability>` to KnowledgeResult
- Added `low_confidence_modules: Vec<String>` to KnowledgeResult

### Route Confidence Filtering (`crates/wtcd-core/src/index.rs`)
- Added `confidence_weight()` function: High=1.0, Low=0.5, None=0.0
- Route query scoring now includes confidence weight: `score = relevance * freshness * confidence`
- Low-confidence results are deprioritized (50% score penalty)

### Knowledge Layer Language Matrix (`crates/wtcd-mirror/src/knowledge.rs`)
- `build_knowledge_result()` now computes language_matrix and low_confidence_modules
- New `generate_language_matrix_md()` generates markdown table with language capabilities
- Low-confidence modules are explicitly listed in knowledge output

## Files Modified

- `crates/wtcd-core/src/types.rs` — LanguageCapability struct, KnowledgeResult fields
- `crates/wtcd-core/src/index.rs` — confidence_weight, route scoring
- `crates/wtcd-mirror/src/knowledge.rs` — language matrix computation, markdown generation

## Verification

- `cargo check` — ✓ compiles
- `cargo test` — ✓ 207 tests pass (24 suites, 0.25s)
