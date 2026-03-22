---
phase: 06-go-adapter
status: ready
updated_at: 2026-03-22T00:00:00Z
---

# Phase 6 Context — Go Adapter

## Goal

在现有 WTCD 解析流水线内新增 Go 语言支持，抽取函数/方法/类型/import/可见性/并发与编译指令元信息。

## Reuse

- `LanguageAdapter` trait + `AdapterRegistry`
- `FileResult` 统一输出
- `wtcd run` 既有扫描与解析流程

## Key constraints

- 不执行 Go 代码，仅静态解析。
- 错误恢复沿用 confidence 带宽，不新增等级。
