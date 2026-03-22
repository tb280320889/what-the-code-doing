---
phase: 07-module-aggregation
status: ready
updated_at: 2026-03-22T00:00:00Z
---

# Phase 7 Context — Module Aggregation

## Goal

把 file-level 结果聚合成 module-level 语义镜像，补充依赖关系、责任描述、模块指纹、fan-in/fan-out 与漂移聚合。

## Inputs

- `Vec<FileResult>`
- 源码内容映射（用于边界推断）
- drift classes（用于 C0-C3 rollup）

## Outputs

- `ModuleResult`
- `mirror/module/*.md`
