# Requirements: ANRSM

**Defined:** 2026-03-21
**Core Value:** 让 AI Agent 的默认读取行为从"吞源码"变为"先读镜像"，用最小充分上下文完成工程决策。

## v1 Requirements

Requirements for initial release (M0-M3). Each maps to roadmap phases.

### Core Engine

- [x] **CORE-01**: CLI 可通过 `cargo install anrsm` 或 GitHub Releases 安装为单二进制
- [x] **CORE-02**: CLI 支持 `anrsm init` 命令，在目标仓库初始化 ANRSM 配置和目录结构 — Phase 1
- [x] **CORE-03**: CLI 支持 `anrsm run` 命令，对目标仓库执行全量扫描和镜像生成 — Phase 1+2
- [x] **CORE-04**: CLI 支持 `anrsm check` 命令，检测当前变更集的镜像漂移 — Phase 3
- [x] **CORE-05**: CLI 支持 `anrsm route <task>` 命令，将任务路由到候选模块和文件 — Phase 4
- [x] **CORE-06**: CLI 输出 JSON 格式，可被 Agent 直接解析

### Scope Manager

- [x] **SCOP-01**: 从 `anrsm.yaml` 配置文件读取源码根目录、排除规则和模块映射
- [x] **SCOP-02**: 确定性地枚举纳入范围的文件清单
- [x] **SCOP-03**: 支持 glob 模式的纳入/排除规则

### Language Adapter (TypeScript/JS)

- [x] **LANG-01**: 使用 tree-sitter 解析 TypeScript 和 JavaScript 文件
- [x] **LANG-02**: 提取导出符号（函数、类、类型、常量）
- [x] **LANG-03**: 提取依赖边（import/require 语句）
- [x] **LANG-04**: 提取函数签名和参数类型
- [x] **LANG-05**: 识别副作用（I/O、网络、存储、日志调用）
- [x] **LANG-06**: 对解析失败的文件标记低置信度并优雅降级

### Mirror Generator

- [x] **MIRR-01**: 生成符合 `mirror-header.schema.json` 的文件镜像（YAML Front Matter + Markdown 正文） — Phase 2
- [x] **MIRR-02**: 镜像正文包含规范定义的 8 个标准段落（职责、对外契约、输入与前置条件、状态与控制流、副作用与外部依赖、关键不变量与风险、变更影响、何时必须展开源码） — Phase 2
- [x] **MIRR-03**: 镜像路径按规则映射：`mirror/file/<source_relative_path>.md` — Phase 2
- [x] **MIRR-04**: 计算 source_fingerprint（基于源码内容的 SHA-256） — Phase 2
- [x] **MIRR-05**: 计算 semantic_fingerprint（基于结构化语义对象的 SHA-256，排除非语义噪声） — Phase 2

### Fingerprint Engine

- [x] **FP-01**: source_fingerprint 在源码任何文本变化时变化 — Phase 2
- [x] **FP-02**: semantic_fingerprint 在纯格式化/注释/空白变化时保持稳定 — Phase 2
- [x] **FP-03**: 指纹算法可版本化，支持 `fp_version` 字段 — Phase 2

### Drift Detection

- [x] **DRFT-01**: 基于 Git diff 获取变更集 — Phase 3
- [x] **DRFT-02**: 区分 C0（Non-material）、C1（Local Material）、C2（Contract Material）、C3（Systemic Material）四类变更 — Phase 3
- [x] **DRFT-03**: 输出符合 `drift-report.schema.json` 的漂移报告 — Phase 3
- [x] **DRFT-04**: 对变更文件的受影响邻域进行扩展分析 — Phase 3

### CI Gate

- [x] **GATE-01**: 支持 pre-commit hook 模式 — Phase 3
- [x] **GATE-02**: 支持 `anrsm gate` 命令，可被 CI pipeline 调用 — Phase 3
- [x] **GATE-03**: 可配置的门禁策略（fail_on / warn_on 按漂移级别） — Phase 3
- [x] **GATE-04**: 输出标准失败码（ANRSM-001 到 ANRSM-010） — Phase 3
- [x] **GATE-05**: 初期支持 warn-only 模式，避免误报导致 adoption 失败 — Phase 3

### Routing Index

- [x] **RTIX-01**: 生成 routing_index，包含 artifact_id、module_id、semantic_keywords、exported_symbols — Phase 4
- [x] **RTIX-02**: `anrsm route <task>` 命令从自然语言任务映射到候选模块和文件 — Phase 4
- [x] **RTIX-03**: 支持 top_k 限制返回数量 — Phase 4

### Incremental Update

- [x] **INCR-01**: 基于 Git diff 驱动的增量更新，不默认全量重扫 — Phase 3
- [x] **INCR-02**: 仅重新解析变更文件及其受影响邻域 — Phase 3
- [x] **INCR-03**: 支持 `anrsm run --full` 触发全量重建 — Phase 3

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Module Mirror

- **MODL-01**: 从文件镜像聚合生成模块级镜像
- **MODL-02**: 模块镜像包含职责、边界、契约、依赖、状态、入口文件
- **MODL-03**: 模块规则可配置（module_rules in anrsm.yaml）

### Multi-Language

- **MULT-01**: Python 语言适配器
- **MULT-02**: Go 语言适配器
- **MULT-03**: 适配器插件化架构（trait-based registry）

### Knowledge Layer

- **KNOW-01**: 从镜像和索引编译人类知识层文档
- **KNOW-02**: 支持 architecture、features、operations、onboarding 四类知识文档
- **KNOW-03**: 知识文档保留来源追溯

### Agent Integration

- **AGNT-01**: Agent 读取顺序验证工具
- **AGNT-02**: 路由基准测试集
- **AGNT-03**: Agent 行为度量和报告

## Out of Scope

| Feature | Reason |
|---------|--------|
| Web UI / Dashboard | CLI 优先，UI 是后续增强 |
| 多仓库全局图 | v1 聚焦单仓场景 |
| 实时后台常驻服务 | CLI 按需运行，不做 daemon |
| 任意语言零成本支持 | 先 TS/JS，后续按需扩展 |
| 自动修复镜像质量问题 | 低置信度人工复核 |
| 替代编译/测试/Lint/静态分析 | ANRSM 是补充层，不是替代 |
| LLM 直接生成镜像事实 | 必须先结构化提取，LLM 只做压缩 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1: Foundation | ✅ Complete |
| CORE-02 | Phase 1: Foundation | ✅ Complete |
| CORE-03 | Phase 1+2: Foundation + Mirror | ✅ Complete |
| CORE-04 | Phase 3: Drift + Gate | ✅ Complete |
| CORE-05 | Phase 4: Index + Route | ✅ Complete |
| CORE-06 | Phase 1: Foundation | ✅ Complete |
| SCOP-01 | Phase 1: Foundation | ✅ Complete |
| SCOP-02 | Phase 1: Foundation | ✅ Complete |
| SCOP-03 | Phase 1: Foundation | ✅ Complete |
| LANG-01 | Phase 1: Foundation | ✅ Complete |
| LANG-02 | Phase 1: Foundation | ✅ Complete |
| LANG-03 | Phase 1: Foundation | ✅ Complete |
| LANG-04 | Phase 1: Foundation | ✅ Complete |
| LANG-05 | Phase 1: Foundation | ✅ Complete |
| LANG-06 | Phase 1: Foundation | ✅ Complete |
| MIRR-01 | Phase 2: Mirror + Fingerprint | ✅ Complete |
| MIRR-02 | Phase 2: Mirror + Fingerprint | ✅ Complete |
| MIRR-03 | Phase 2: Mirror + Fingerprint | ✅ Complete |
| MIRR-04 | Phase 2: Mirror + Fingerprint | ✅ Complete |
| MIRR-05 | Phase 2: Mirror + Fingerprint | ✅ Complete |
| FP-01 | Phase 2: Mirror + Fingerprint | ✅ Complete |
| FP-02 | Phase 2: Mirror + Fingerprint | ✅ Complete |
| FP-03 | Phase 2: Mirror + Fingerprint | ✅ Complete |
| DRFT-01 | Phase 3: Drift + Gate | ✅ Complete |
| DRFT-02 | Phase 3: Drift + Gate | ✅ Complete |
| DRFT-03 | Phase 3: Drift + Gate | ✅ Complete |
| DRFT-04 | Phase 3: Drift + Gate | ✅ Complete |
| GATE-01 | Phase 3: Drift + Gate | ✅ Complete |
| GATE-02 | Phase 3: Drift + Gate | ✅ Complete |
| GATE-03 | Phase 3: Drift + Gate | ✅ Complete |
| GATE-04 | Phase 3: Drift + Gate | ✅ Complete |
| GATE-05 | Phase 3: Drift + Gate | ✅ Complete |
| RTIX-01 | Phase 4: Index + Route | ✅ Complete |
| RTIX-02 | Phase 4: Index + Route | ✅ Complete |
| RTIX-03 | Phase 4: Index + Route | ✅ Complete |
| INCR-01 | Phase 3: Drift + Gate | ✅ Complete |
| INCR-02 | Phase 3: Drift + Gate | ✅ Complete |
| INCR-03 | Phase 3: Drift + Gate | ✅ Complete |

**Coverage:**
- v1 requirements: 38 total
- Satisfied: 38/38 (100%)
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-21*
*Last updated: 2026-03-21 after initial definition*
