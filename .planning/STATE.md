---
gsd_state_version: 1.0
milestone: v0.1.1
milestone_name: multi-language-and-knowledge
status: defining_requirements
last_updated: "2026-03-21T18:45:00.000Z"
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
---

# STATE.md — WTCD Project Memory

**Last updated:** 2026-03-21
**Phase:** Not started (defining requirements)
**Plan:** —
**Status:** ◆ DEFINING REQUIREMENTS — Milestone v0.1.1

---

## Project Reference

**Core Value:** 让 AI Agent 的默认读取行为从"吞源码"变为"先读镜像"，用最小充分上下文完成工程决策。

**Current Focus:** Milestone v0.1.1 — extending to multi-language support (Python, Go) and knowledge layer generation

---

## Current Position

**Previous Milestone:** v1.0 — 5 phases, 22 plans, 160 tests, 8 crates, pushed to origin/main
Audit: .planning/v1.0-MILESTONE-AUDIT.md (status: passed, 38/38 requirements)

**This Milestone:** v0.1.1 — Multi-Language & Knowledge Layer
- Python 语言适配器
- Go 语言适配器
- 模块级镜像聚合
- 知识层文档生成

---

## Accumulated Context

### Key Decisions (from v1.0 + research)

| Decision | Status |
|----------|--------|
| 项目名 ANRSM → **WTCD** (What The Code Doing) | Locked — 所有 crate 使用 wtcd- 前缀 |
| tree-sitter 0.26 > SWC > Oxc for parsing | Locked — multi-language unified API, error recovery |
| gix 0.80 > git2 for Git operations | Locked — pure Rust, no C deps, v1 read-only needs covered |
| Trait-based adapter (compile-time) > dynamic plugins | Locked — ABI safety, v1 scope doesn't need runtime plugins |
| JSON 输出：单层扁平 + version 字段 | Locked — files[]/summary + api_version |
| Confidence 三级：high/low/none | Locked — tree-sitter partial CST → low |
| Phase 1 五 crate workspace (含 wtcd-tests) | Locked — wtcd-core, wtcd-scope, wtcd-adapters, wtcd-cli, wtcd-tests |
| Integration tests 在 dedicated crate | Locked — Cargo virtual manifest 不支持 dev-dependencies |
| Phase 2: SHA-256 for fingerprints | Locked — matches mirror-header.schema.json |
| Phase 3: gix 0.63 for pure Rust git diff | Locked — D-01 (0.80 requires Rust 1.88) |
| Phase 4: routing_index.json in mirror/ directory | Locked — D-01 |
| Phase 999.1: rmcp 1.2.0 as MCP SDK | Locked — user directive, latest stable |

### Active Todos

- [ ] Define requirements for v0.1.1
- [ ] Research Python tree-sitter integration
- [ ] Research Go tree-sitter integration
- [ ] Design module-level mirror aggregation
- [ ] Design knowledge layer document generation
- [ ] Create roadmap for v0.1.1

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
*Last session: 2026-03-21 — Started Milestone v0.1.1 — Multi-Language & Knowledge Layer*
