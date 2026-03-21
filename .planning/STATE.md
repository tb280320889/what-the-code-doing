# STATE.md — WTCD Project Memory

**Last updated:** 2026-03-21
**Phase:** 1 (Foundation)
**Plan:** Not started
**Status:** Phase 1 context captured, awaiting plan

---

## Project Reference

**Core Value:** 让 AI Agent 的默认读取行为从"吞源码"变为"先读镜像"，用最小充分上下文完成工程决策。

**Current Focus:** Phase 1 — Foundation (core types, CLI framework, TS/JS parsing, scope management)

---

## Current Position

```
Phase:  1. Foundation                    [not started]
        2. Mirror + Fingerprint          [not started]
        3. Drift + Gate                  [not started]
        4. Index + Route                 [not started]

Progress: ░░░░░░░░░░░░░░░░░░░░ 0%
```

**Active Phase:** Phase 1 — Foundation
**Next Milestone:** Phase 1 plan completion → all core types frozen, TS adapter parsing, scope manager working
**Blocking:** None

---

## Performance Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| v1 Requirements | 38 | Mapped: 38/38 ✓ |
| Phases | 4 | Coarse granularity |
| Research Confidence | HIGH | Stack, architecture, pitfalls all validated |
| Pitfalls Identified | 6 Critical + 8 Moderate | Mitigations embedded in roadmap |

---

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

### Active Todos

- [x] Phase 1 context: capture implementation decisions (01-CONTEXT.md)
- [ ] Phase 1 plan: decompose Foundation into executable plans
- [ ] Golden test suite setup for tree-sitter TS/JS parsing
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
