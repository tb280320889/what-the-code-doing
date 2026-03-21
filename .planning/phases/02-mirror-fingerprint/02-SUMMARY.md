---
phase: 02-mirror-fingerprint
plan: "02-mirror-fingerprint"
subsystem: mirror
tags: [mirror, fingerprint, cli, template]
dependency_graph:
  requires: [wtcd-core, wtcd-scope, wtcd-adapters]
  provides: [wtcd-mirror]
  affects: [wtcd-cli]
tech_stack:
  added: [sha2, chrono, yaml_serde]
  patterns: [pipeline-architecture, rule-based-template, dual-fingerprint]
key_files:
  created:
    - crates/wtcd-mirror/Cargo.toml
    - crates/wtcd-mirror/src/lib.rs
    - crates/wtcd-mirror/src/types.rs
    - crates/wtcd-mirror/src/config.rs
    - crates/wtcd-mirror/src/fingerprint.rs
    - crates/wtcd-mirror/src/template.rs
    - crates/wtcd-mirror/src/io.rs
    - crates/wtcd-tests/tests/integration_mirror.rs
  modified:
    - Cargo.toml
    - crates/wtcd-core/src/config.rs
    - crates/wtcd-cli/Cargo.toml
    - crates/wtcd-cli/src/commands/run.rs
    - crates/wtcd-cli/src/commands/init.rs
    - crates/wtcd-tests/Cargo.toml
decisions:
  - "SHA-256 via sha2 crate for both source and semantic fingerprints (D-05)"
  - "Canonical JSON serialization via serde_json → SHA-256 for semantic fingerprint (D-07)"
  - "fp_version = '1' in all FingerprintResult outputs (D-08)"
  - "Full overwrite of mirror body on regeneration (D-09)"
  - "manual_appendix preserved between <!-- manual_appendix_start/end --> markers (D-10)"
  - "Mirror files are git-tracked version artifacts (D-11)"
  - "Auto-delete orphan mirror files on run (D-12)"
  - "Single wtcd-mirror crate for fingerprint + template + I/O (D-13)"
  - "Mirror generation built into anrsm run pipeline (D-15)"
  - "wtcd-mirror receives Vec<FileResult>, doesn't depend on wtcd-scope (D-16)"
metrics:
  duration: "~45min"
  completed: "2026-03-21"
  tasks: 5 plans, 14 steps
  files: 13 (8 created, 6 modified)
  tests: 106 passing (16 suites)
---

# Phase 02: Mirror + Fingerprint — Summary

**One-liner:** Complete mirror generation system — dual SHA-256 fingerprints, rule-based 8-section template, file I/O with appendix preservation, orphan cleanup, integrated into `anrsm run` pipeline.

## What Was Built

### Wave 1: Crate Scaffold (02-01)
- Created `wtcd-mirror` crate with module structure (types, fingerprint, template, config, io)
- Defined core types: `MirrorHeader` (19 fields), `MirrorBody` (8 sections), `FingerprintResult`, `FreshnessState`, `MirrorConfidenceBand`
- Expanded `MirrorConfig` from empty struct to include `output_dir`, `fp_version`, `include_manual_appendix`
- Added `MirrorSettings` with `From<MirrorConfig>` conversion (D-16: no wtcd-scope dependency)

### Wave 2: Fingerprint Engine (02-02) + Template Engine (02-03)

**Fingerprint Engine:**
- Source fingerprint: SHA-256 of raw bytes — any byte change produces different hash
- Semantic fingerprint: canonical JSON of FileResult fields → SHA-256
  - Exports/imports/signatures sorted alphabetically
  - Line numbers excluded (semantic stability across formatting changes)
  - Side effects sorted by kind, then target
- Public API: `compute_fingerprints()` and `compute_fingerprints_from_path()`
- 13 unit tests for determinism, stability, and format validation

**Template Engine:**
- Deterministic path mapping: `mirror/file/<source_path>.md`
- YAML frontmatter generation with 13 required + 6 optional fields matching `mirror-header.schema.json`
- Rule-based 8-section body generation from FileResult fields:
  1. 职责 — derived from exports + file path
  2. 对外契约 — function signatures with types
  3. 输入与前置条件 — import dependencies
  4. 状态与控制流 — function flows + side effects
  5. 副作用与外部依赖 — categorized side effects
  6. 关键不变量与风险 — inferred from exports + side effects
  7. 变更影响 — import dependency graph
  8. 何时必须展开源码 — generic rules + confidence-based
- 16 unit tests for path mapping, header derivation, body sections

### Wave 3: File I/O (02-04)
- `write_mirror`: creates parent dirs, writes content
- `write_mirror_with_appendix`: preserves `manual_appendix` between HTML comment markers (D-10)
- `read_mirror_header`: parses YAML frontmatter from existing mirrors
- `extract_appendix`: extracts content between `manual_appendix_start`/`end` markers
- `find_orphan_mirrors`: detects mirror files with deleted sources (D-12)
- `delete_orphans`: removes orphan mirror files
- `generate_and_write_mirror`: full pipeline (fingerprint → template → write)
- 23 unit tests for write, appendix, read, orphan, and pipeline

### Wave 4: CLI Integration (02-05)
- Extended `run_analysis` with full mirror pipeline: parse → fingerprint → mirror → write → orphan cleanup
- Added `get_git_commit_hash` helper for `source_commit` header field
- Added `derive_module_id` helper for `module_id` header field
- Updated init config template with mirror defaults
- 6 integration tests: file creation, header fields, orphan cleanup, fingerprint stability, appendix preservation, body exports

## Decisions Followed

All 12 context decisions (D-05 through D-16) were implemented exactly as specified:

| Decision | Implementation |
|----------|---------------|
| D-05 | SHA-256 via `sha2` crate |
| D-06 | Line numbers excluded, collections sorted for semantic stability |
| D-07 | `serde_json::to_string` → SHA-256 for canonical serialization |
| D-08 | `fp_version: "1"` in all fingerprint outputs |
| D-09 | Full overwrite of mirror body on regeneration |
| D-10 | manual_appendix preserved between HTML comment markers |
| D-11 | Mirror files git-tracked (not added to .gitignore) |
| D-12 | Orphan mirrors auto-deleted on `anrsm run` |
| D-13 | Single `wtcd-mirror` crate |
| D-15 | Mirror generation built into `anrsm run` |
| D-16 | `wtcd-mirror` receives `Vec<FileResult>`, no wtcd-scope dependency |

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

```bash
cargo build --workspace    # 0 errors, 0 warnings
cargo test --workspace     # 106 passed, 0 failed, 16 suites
```

## Known Stubs

None — all modules fully implemented.

## Self-Check: PASSED

- [x] All 5 plans executed and committed
- [x] 8 new files created
- [x] 6 existing files modified
- [x] 106 tests passing across 16 test suites
- [x] Workspace builds cleanly with 0 warnings
- [x] All D-05 through D-16 decisions implemented
- [x] All commits follow `feat(02-XX):` format
