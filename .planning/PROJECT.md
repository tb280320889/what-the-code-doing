# ANRSM — AI Native Repo Semantic Mirror

## What This Is

ANRSM 是一套以源码为唯一真相源的仓库语义镜像系统，面向 AI Agent 和开发团队。
它通过 Rust CLI 为代码仓库生成机器优先的语义镜像（文件级 → 模块级 → 知识层），让 Agent 不必直扫全量源码即可完成定位、理解和修改任务。
同时提供漂移检测和 CI 门禁，确保镜像层与代码演化保持可验证同步。

## Core Value

**让 AI Agent 的默认读取行为从"吞源码"变为"先读镜像"，用最小充分上下文完成工程决策。**

## Current Milestone: v0.2.0 Polyglot Adapters

**Goal:** 将 WTCD 语言适配能力从 TS/JS/Python/Go 扩展到 9 种新增语言，并保持与现有镜像、漂移、路由链路的完整一致性。

**Target features:**
- Rust 语言适配器（解析/提取/测试/CLI 集成）
- Dart 语言适配器（解析/提取/测试/CLI 集成）
- Java 语言适配器（解析/提取/测试/CLI 集成）
- Kotlin 语言适配器（解析/提取/测试/CLI 集成）
- Swift 语言适配器（解析/提取/测试/CLI 集成）
- C++ 语言适配器（解析/提取/测试/CLI 集成）
- C# 语言适配器（解析/提取/测试/CLI 集成）
- C 语言适配器（解析/提取/测试/CLI 集成）
- Zig 语言适配器（解析/提取/测试/CLI 集成）

## Requirements

### Validated

- [x] TypeScript/JS 适配器：首批目标语言，AST 级结构化提取 — `Validated in Phase 01: Foundation`
- [x] 规范冻结：术语、路径映射、文档头 Schema、Material Change 分类已锁定 — `Validated in Phase 01`
- [x] 文件级语义镜像生成：Scope Manager + Language Adapter + Mirror Generator 输出标准镜像 — `Validated in Phase 02`
- [x] 语义指纹与漂移检测：区分 C0/C1/C2/C3，输出漂移报告 — `Validated in Phase 03`
- [x] CI 门禁：pre-commit / PR 级阻断未同步的 Material Change — `Validated in Phase 03`
- [x] 路由索引：从任务描述映射到候选模块和文件集合 — `Validated in Phase 04`
- [x] Agent 集成：CLI 驱动的读取顺序引导，JSON 结构化输出 — `Validated in Phase 01+04`
- [x] Rust CLI 分发：crates.io + GitHub Releases 预编译二进制 — `Validated in Phase 01`
- [x] MCP Server 集成：通过 MCP 协议暴露 WTCD 核心能力给 Agent — `Validated in Phase 999.1`
- [x] Python 语言适配器：支持函数/类/import/装饰器/`__all__`/`__init__.py` 提取与低置信降级 — `Validated in Phase 05`
- [x] Go 语言适配器：支持函数/方法/类型/可见性/指令与并发模式提取 — `Validated in Phase 06`
- [x] 模块级镜像聚合：跨文件导出/依赖/副作用聚合，模块指纹与 fan-in/fan-out 统计 — `Validated in Phase 07`
- [x] 知识层文档生成：仓库总览、模块依赖图、全局导出索引、热点与 Agent 读取路径建议 — `Validated in Phase 08`

### Active

**v0.2.0: Polyglot Adapters milestone scope**

- [ ] 完成 Rust / Dart / Java / Kotlin / Swift / C++ / C# / C / Zig 9 种语言适配器
- [ ] 每种语言达到完整适配标准：tree-sitter 解析 + 结构化语义提取 + fixtures + 单元/集成回归
- [ ] 保持与现有 LanguageAdapter 管线、镜像输出、drift/check/route 命令的兼容

### Out of Scope (v1)

- 复杂 UI 平台 — 先做 CLI，不做 Web UI
- 多仓库全局图 — v1 聚焦单仓
- 实时后台常驻服务 — CLI 按需运行
- 任意语言零成本支持 — 先 TS/JS，后续按需扩展
- 自动修复所有镜像质量问题 — 低置信度人工复核
- 替代编译、测试、审查和回归验证 — ANRSM 是补充，不是替代

## Context

### 技术环境

- 核心引擎：Rust（CLI 工具，单二进制分发）
- 首批适配器：TypeScript/JS（AST 解析）
- 镜像格式：Markdown + YAML Front Matter（文本化，可 diff，可版本管理）
- 索引格式：JSON（机器可读）
- 配置格式：YAML（anrsm.yaml）
- 分发渠道：crates.io + GitHub Releases

### 现有工作

- 11 份核心规范文档已完成（docs/01-11）
- JSON Schema 已定义（mirror-header、drift-report）
- 示例工件已提供（配置、文件镜像、漂移报告）
- 术语体系、架构模型、M0-M5 里程碑已锁定

### 目标用户

- 开源社区
- 日常开发大量依赖 AI Agent 的团队
- 中小型到中大型跨端仓库
- 需要同时优化 Agent 效率和人类维护成本的项目

## Constraints

- **技术栈**：Rust 核心引擎，不接受其他语言重写核心
- **语言适配**：从一开始支持多语言适配器架构，首批 TS/JS
- **Agent 集成**：CLI 驱动模式 + MCP Server 双轨制，JSON 结构化输出
- **分发**：必须支持 crates.io 和 GitHub Releases 两种渠道
- **规范先行**：实现必须符合已冻结的 11 份核心规范，规范变更需走 ADR
- **试点**：本仓库自身作为第一个 dogfood 试点

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust 作为核心引擎 | 性能好，编译为单二进制，CI 集成简单，生态成熟 | ✅ 8 crates, 160 tests |
| CLI 驱动 Agent 集成 | 比 MCP Server 更简洁通用，任何 Agent 都能调用 CLI | ✅ init/run/check/route 命令 |
| JSON 作为 CLI 输出格式 | Agent 可直接解析，人类可用 jq 处理 | ✅ 所有命令 JSON 输出 |
| TypeScript/JS 作为首批适配器 | 前端/全栈项目量大，AST 工具链成熟（tree-sitter） | ✅ TS/JS tree-sitter 适配器 |
| Markdown + YAML Front Matter 作为镜像格式 | 文本化、可 diff、可版本管理、人类可读、机器可解析 | ✅ wtcd-mirror crate |
| 本仓库自身作为试点 | Dogfooding 验证可行性，发现规范盲点 | ✅ Phase 1 试点完成 |
| 多语言适配器从一开始设计 | 避免后期架构返工，适配器插件化 | ✅ trait-based adapter registry |
| MCP Server 集成 | Agent 自动发现 WTCD 能力，无需手动调用 CLI | ✅ rmcp 1.2.0, 4 MCP Tools |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**Current State**: Milestone v0.2.0 initialized — preparing full adapter coverage for 9 additional languages.

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-22 after v0.2.0 milestone initialization*
