---
phase: 08-knowledge-layer
status: ready
updated_at: 2026-03-22T00:00:00Z
---

# Phase 8 Context — Knowledge Layer

## Goal

基于 module mirrors 生成仓库级知识文档，支持 Agent 读路径决策与架构漂移跟踪。

## Inputs

- `Vec<ModuleResult>`
- `RoutingIndex`
- drift history（窗口聚合）

## Outputs

- `mirror/knowledge/overview.md`
- `mirror/knowledge/module-deps.mmd`
- `mirror/knowledge/export-index.md`
- `mirror/knowledge/stats.md`
- `mirror/knowledge/clusters.md`
- `mirror/knowledge/hotspots.md`
- `mirror/knowledge/read-paths.md`
- `mirror/knowledge/adr/*.md`
