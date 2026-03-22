---
gsd_state_version: 1.0
milestone: v0.2.0
milestone_name: polyglot-adapters
status: complete
last_updated: "2026-03-22"
progress:
  total_phases: 4
  completed_phases: 4
  total_plans: 2
  completed_plans: 2
---

# STATE.md — WTCD Project Memory

**Last updated:** 2026-03-22
**Phase:** All complete
**Plan:** v0.2.0 milestone complete — 24/24 requirements verified, 210 tests pass
**Status:** ✓ Milestone complete

---

## Project Reference

**Core Value:** 让 AI Agent 的默认读取行为从"吞源码"变为"先读镜像"，用最小充分上下文完成工程决策。

**Current Focus:** Milestone v0.2.0 — Polyglot Adapters (Rust/Dart/Java/Kotlin/Swift/C++/C#/C/Zig)

---

## Current Position

**Previous Milestones:**
- v1.0 — 5 phases, 22 plans, 160 tests, 8 crates
- v0.1.1 — 4 phases, 40 requirements (fully complete)

**Current Milestone:** v0.2.0 — Polyglot Adapters
目标：新增 Rust、Dart、Java、Kotlin、Swift、C++、C#、C、Zig 9 种语言完整适配器。

**Roadmap (Phase 9–12):**
- Phase 9: Polyglot Adapter Coverage
- Phase 10: Aggregation Consistency Hardening
- Phase 11: Route & Knowledge Signal Quality
- Phase 12: Stability Gates & Regression Reliability

Progress: [██████████] 100%

---

## Accumulated Context

### Key Decisions (from v1.0 + research)

| Decision | Status |
|----------|--------|
| 项目名 ANRSM → **WTCD** (What The Code Doing) | Locked |
| tree-sitter 0.26 + tree-sitter-language ^0.1 桥接 | Locked — Python/Go 语法库通过此桥接兼容 |
| Trait-based adapter (compile-time) > dynamic plugins | Locked — PyAdapter/GoAdapter 实现 LanguageAdapter |
| Python 先行：tree-sitter-python 下载量 5.2M > Go 3.8M | Phase 5 作为 tree-sitter 多语法集成验证 |
| 模块边界：auto-detect + anrsm.yaml 可选覆盖 | Phase 7 需要 ADR 确认 |
| 知识层只生成可验证事实，不生成 "why" | Locked — 避免 EXT-C5 泛化无用内容 |

### Active Todos

- [x] Execute Phase 9 — 9 adapters verified, 207 tests pass
- [x] Execute Phase 10 — cross-language aggregation consistency verified
- [x] Execute Phase 11 — route/knowledge signal quality verified
- [x] Execute Phase 12 — stability gates + regression verified
- [x] Milestone audit — 24/24 requirements verified

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
*Last session: 2026-03-22 — v0.2.0 roadmap created with phases 9–12*
