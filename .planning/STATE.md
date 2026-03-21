---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: shipped
last_updated: "2026-03-21T18:30:00.000Z"
progress:
  total_phases: 5
  completed_phases: 5
  total_plans: 22
  completed_plans: 22
---

# STATE.md — WTCD Project Memory

**Last updated:** 2026-03-21
**Phase:** ALL COMPLETE
**Plan:** ALL COMPLETE
**Status:** ✅ MILESTONE v1.0 SHIPPED — 5 phases, 22 plans, 160 tests, 8 crates, pushed to origin/main

---

## Project Reference

**Core Value:** 让 AI Agent 的默认读取行为从"吞源码"变为"先读镜像"，用最小充分上下文完成工程决策。

**Current Focus:** Milestone v1.0 shipped — pushed to origin/main, ready for v2 planning

---

## Current Position

Phase: 01 (foundation) — COMPLETE
Phase: 02 (mirror + fingerprint) — COMPLETE (5 plans, 106 tests)
Phase: 03 (drift + gate) — COMPLETE (6 plans, 22 new tests, 128 total)
Phase: 04 (index + route) — COMPLETE (3 plans, 32 new tests, 160 total)
Phase: 999.1 (MCP Server) — COMPLETE (3 plans, 12 files, MCP E2E verified)

**MILESTONE v1.0 SHIPPED** — 8 crates, 160 tests, 0 build errors, pushed to origin/main
Audit: .planning/v1.0-MILESTONE-AUDIT.md (status: passed, 38/38 requirements)

## Performance Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| v1 Requirements | 38 | Mapped: 38/38 ✓ |
| Phases | 4 | Coarse granularity |
| Research Confidence | HIGH | Stack, architecture, pitfalls all validated |
| Pitfalls Identified | 6 Critical + 8 Moderate | Mitigations embedded in roadmap |

---
| Phase 01-foundation P03 | 9min | 1 tasks | 1 files |
| Phase 01-foundation P04 | 15min | 2 tasks | 16 files |
| Phase 01-foundation P05 | 6min | 2 tasks | 12 files |
| Phase 02-mirror-fingerprint 02-01 | ~10min | 3 steps | 9 files |
| Phase 02-mirror-fingerprint 02-02 | ~10min | 3 steps | 1 file |
| Phase 02-mirror-fingerprint 02-03 | ~10min | 4 steps | 1 file |
| Phase 02-mirror-fingerprint 02-04 | ~10min | 4 steps | 1 file |
| Phase 02-mirror-fingerprint 02-05 | ~10min | 3 steps | 6 files |
| Phase 03-drift-gate 03-01 | ~5min | 3 steps | 3 files |
| Phase 03-drift-gate 03-02 | ~10min | 3 steps | 4 files |
| Phase 03-drift-gate 03-03 | ~5min | 1 step | 2 files |
| Phase 03-drift-gate 03-04 | ~5min | 1 step | 3 files |
| Phase 03-drift-gate 03-05 | ~5min | 1 step | 2 files |
| Phase 03-drift-gate 03-06 | ~15min | 4 steps | 8 files |
| Phase 04-index-route 04-01 | ~10min | 1 step (TDD) | 2 files |
| Phase 04-index-route 04-02 | included in 04-01 | — | — |
| Phase 04-index-route 04-03 | ~5min | 3 steps | 5 files |
| Phase 999.1-mcp 01 | ~15min | 2 steps | 10 files |
| Phase 999.1-mcp 02 | ~10min | 2 steps | 6 files |
| Phase 999.1-mcp 03 | ~5min | 2 steps | 1 file |

## Accumulated Context

### Key Decisions (from research + context)

| Decision | Status |
|----------|--------|
| 项目名 ANRSM → **WTCD** (What The Code Doing) | Locked — 所有 crate 使用 wtcd- 前缀 |
| tree-sitter 0.26 > SWC > Oxc for parsing | Locked — multi-language unified API, error recovery |
| gix 0.80 > git2 for Git operations | Locked — pure Rust, no C deps, v1 read-only needs covered |
| yaml_serde 0.10 > serde_yaml | Locked — YAML org maintained, serde_yaml deprecated |
| Trait-based adapter (compile-time) > dynamic plugins | Locked — ABI safety, v1 scope doesn't need runtime plugins |
| JSON 输出：单层扁平 + version 字段 | Locked — files[]/summary + api_version |
| Confidence 三级：high/low/none | Locked — tree-sitter partial CST → low |
| anrsm.yaml 三块：scope + mirror + output | Locked — Phase 1 只需 scope 块 |
| 标准 glob 语法（gitignore 风格） | Locked — 与 ignore crate 兼容 |
| Phase 1 四 crate workspace | Locked — wtcd-core, wtcd-scope, wtcd-adapters, wtcd-cli |
| Warn-only gate initially | Required — C2 pitfall prevention |
| Incremental as default path | Required — C6 pitfall prevention |
| tree-sitter positional children traversal for unnamed nodes | Locked — children().find(c.kind()) for import_clause, formal_parameters, variable identifiers |
| Phase 1 五 crate workspace (含 wtcd-tests) | Locked — wtcd-core, wtcd-scope, wtcd-adapters, wtcd-cli, wtcd-tests |
| Integration tests 在 dedicated crate | Locked — Cargo virtual manifest 不支持 dev-dependencies |
| Phase 2: SHA-256 (not blake3) for fingerprints | Locked — D-05, matches mirror-header.schema.json |
| Phase 2: Normalize whitespace/comments/trailing semicolons for semantic fp | Locked — D-06 |
| Phase 2: serde_json canonical serialization → SHA-256 | Locked — D-07 |
| Phase 2: fp_version = "1" | Locked — D-08 |
| Phase 2: Full overwrite mirror body on regeneration | Locked — D-09 |
| Phase 2: Preserve manual_appendix between HTML markers | Locked — D-10 |
| Phase 2: Mirror files git-tracked | Locked — D-11 |
| Phase 2: Auto-delete orphan mirrors | Locked — D-12 |
| Phase 2: Single wtcd-mirror crate | Locked — D-13 |
| Phase 2: Mirror generation in anrsm run | Locked — D-15 |
| Phase 2: wtcd-mirror doesn't depend on wtcd-scope | Locked — D-16 |
| Phase 3: gix 0.63 for pure Rust git diff | Locked — D-01 (0.80 requires Rust 1.88) |
| Phase 3: C0 = source changed, semantic unchanged | Locked — D-05 |
| Phase 3: C1 = semantic changed, exports unchanged; C2 = exports changed | Locked — D-06 |
| Phase 3: C3 = imported by ≥5 files (configurable) | Locked — D-07 |
| Phase 3: Gate config in top-level gate: block | Locked — D-09 |
| Phase 3: Default fail_on: none (warn-only) | Locked — D-10 |
| Phase 3: Default incremental run, --full for rebuild | Locked — D-16 |
| Phase 4: routing_index.json in mirror/ directory | Locked — D-01 |
| Phase 4: Keyword matching on exports + source_path + module_id | Locked — D-05 |
| Phase 4: Freshness weights: fresh=1.0, stale=0.7, invalid=0.3, unknown=0.5 | Locked — D-06 |
| Phase 4: Score = hits/query_tokens × freshness_weight | Locked — D-07 |
| Phase 4: route reads index from disk | Locked — D-13 |
| Phase 4: run auto-rebuilds index | Locked — D-14 |
| Phase 999.1: rmcp 1.2.0 as MCP SDK | Locked — user directive, latest stable |
| Phase 999.1: schemars 1.1.0 for JSON Schema | Locked — rmcp 1.x compatibility |
| Phase 999.1: Rust 1.92 toolchain | Locked — rmcp 1.2.0 requires 1.88+ |
| Phase 999.1: stdio-only transport (D-03) | Locked — local mode, no HTTP/SSE |
| Phase 999.1: 4 MCP tools (parse+run full, check+mirror stub) | Locked — D-06, D-10 |

### Active Todos

- [x] Phase 1 context: capture implementation decisions (01-CONTEXT.md)
- [x] Phase 1 plan: decompose Foundation into executable plans
- [x] wtcd-scope: config loading + file scanning (Plan 02)
- [x] TDD tests for tree-sitter TS/JS parsing (Plan 03)
- [x] CLI init + run commands (Plan 04)
- [x] Golden fixtures + integration tests (Plan 05)
- [x] Phase 02: scaffold wtcd-mirror crate (02-01)
- [x] Phase 02: dual fingerprint engine (02-02)
- [x] Phase 02: mirror template engine (02-03)
- [x] Phase 02: mirror file I/O + orphan cleanup (02-04)
- [x] Phase 02: CLI integration + E2E tests (02-05)
- [x] Phase 03: scaffold wtcd-diff crate (03-01)
- [x] Phase 03: gix git diff (03-02)
- [x] Phase 03: ReverseDepGraph (03-03)
- [x] Phase 03: drift classifier + analyzer (03-04)
- [x] Phase 03: gate evaluator (03-05)
- [x] Phase 03: CLI check + incremental run + integration tests (03-06)
- [x] Phase 04: routing index types + builder (04-01)
- [x] Phase 04: query engine scoring + ranking (04-02)
- [x] Phase 04: CLI route command + run auto-rebuild + integration tests (04-03)
- [x] Phase 999.1 plan: MCP Server phase (3 plans created)
- [x] Phase 999.1: Execute wtcd-mcp crate scaffold + tools (Plan 01)
- [x] Phase 999.1: Execute CLI integration (Plan 02)
- [x] Phase 999.1: Execute verification + docs (Plan 03)
- [ ] MSRV unification (clap 4.5 vs 4.6, jsonschema version)
- [ ] 更新 PROJECT.md/ROADMAP.md/REQUIREMENTS.md 中的项目名 ANRSM → WTCD

### Blockers

None currently.

---

## Session Continuity

**Context load order:**

1. This file (STATE.md) — current position + decisions
2. `.planning/ROADMAP.md` — phase structure + success criteria
3. `.planning/REQUIREMENTS.md` — full requirement list + traceability
4. `.planning/research/` — detailed tech/architecture/pitfall research

**Phase transition checklist (for `/gsd-transition`):**

- [ ] All phase requirements validated
- [ ] Success criteria met
- [ ] PROJECT.md updated (validated requirements)
- [ ] Next phase plan ready

---

*Created: 2026-03-21 after roadmap initialization*
*Last session: 2026-03-21 — Shipped Milestone v1.0 — pushed 38 commits to origin/main*
