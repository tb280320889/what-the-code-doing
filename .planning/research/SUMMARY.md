# Project Research Summary

**Project:** ANRSM v0.1.1 — Multi-Language & Knowledge Layer
**Domain:** Rust CLI for code repository semantic mirror generation
**Researched:** 2026-03-21
**Confidence:** HIGH

## Executive Summary

ANRSM 是一个面向 Agent 的代码语义镜像工具，通过 tree-sitter 解析源码生成结构化镜像，供 AI Agent 高效消费。当前版本仅支持 TypeScript/JavaScript，v0.1.1 里程碑的核心目标是扩展到 Python 和 Go 语言，并新增模块级聚合和知识层两个抽象层级，形成**文件镜像 → 模块镜像 → 知识层**的三层架构。

推荐方案是**纯增量式扩展**：Python 和 Go 适配器通过实现已有的 `LanguageAdapter` trait 加入现有流水线，无需修改核心架构。模块聚合在文件镜像之上新增中间层，知识层作为最顶层消费者从模块镜像编译仓库级文档。整个方案零新增 LLM 依赖，tree-sitter 生态通过 `tree-sitter-language ^0.1` 桥接层确保版本兼容。构建顺序受硬依赖约束：适配器先行（并行开发），模块聚合次之，知识层最后。

最大风险来自三个方向：**tree-sitter 版本冲突**（多语法库编译时的链接器符号碰撞）、**Python 缩进解析边缘情况**（外部 C scanner 的复杂性）、以及**知识层生成无用的泛化内容**（只有语法结构没有语义意图）。预防策略已在 PITFALLS.md 中详细记录，核心原则是"只生成可验证的事实，不生成无法验证的 why"。

## Key Findings

### Recommended Stack

详见 [STACK.md](./STACK.md)。

**Core technologies:**
- **tree-sitter-python 0.25.0**: Python 语法解析 — 最新稳定版，与项目现有 tree-sitter 0.26.7 通过 `tree-sitter-language ^0.1` 桥接兼容，5.2M+ 下载验证
- **tree-sitter-go 0.25.0**: Go 语法解析 — 同一桥接层，3.8M+ 下载，Go AST 结构清晰（function_declaration vs method_declaration）
- **tree-sitter-md 0.5.3** (optional): Markdown 结构化解析 — 用于知识层 ADR/文档提取，默认不启用，用户可通过 feature flag 按需开启
- **pulldown-cmark 0.13** (已有): 知识层默认 Markdown 解析方案 — 零新依赖，提取标题/段落/前端数据足够
- **yaml_serde 0.10** (已有): YAML Front Matter 解析 — 复用于镜像头部和知识层

**关键兼容性决策：** tree-sitter 0.23+ 通过 `tree-sitter-language ^0.1` 桥接 crate 统一版本，语法库不再直接依赖特定 tree-sitter 版本。项目已验证此模式（tree-sitter-typescript 0.23 + tree-sitter 0.26 正常工作），Python 和 Go 适配器遵循相同模式。

**不需要新增的依赖：** rustpython-parser（过重）、syn（不适用）、pest/nom（tree-sitter 已覆盖）、rayon（当前性能足够）、walkdir（已有 ignore crate）。

### Expected Features

详见 [FEATURES.md](./FEATURES.md)。

**Must have (table stakes):**
- **Python 适配器** — 函数/类定义提取、import 语句（含相对导入）、装饰器检测、类型注解提取、语法错误容忍、`__init__.py` 处理
- **Go 适配器** — 函数/方法声明提取、struct/interface/type 提取、import 语句、常量/变量提取、首字母大小写可见性判断、语法错误容忍
- **模块级镜像聚合** — 模块级导出汇总、依赖汇总、职责描述、文件列表索引、副作用汇总
- **知识层文档** — 仓库总览、模块关系图（Mermaid）、导出索引、语言/文件统计

**Should have (competitive):**
- Python: `__all__` 解析、相对导入解析、`@property`/`@staticmethod` 方法类型识别、dataclass/Pydantic 检测
- Go: Method Receiver 提取、struct 字段提取、interface 方法列表、嵌入式结构体识别、goroutine/channel 检测、`//go:embed` 指令
- 模块: 模块内依赖图、模块边界自动检测、模块级指纹、模块级漂移检测、扇入/扇出统计
- 知识层: 语义聚类（社区发现）、变更热点图、Token 压缩报告、Agent 读取路径建议、ADR 自动骨架

**Defer (v2+):**
- CGo 分析（复杂度过高）
- 泛型约束深度分析
- 跨语言类型引用关联（protobuf/OpenAPI 桥接）
- LLM 生成自由文本总结（违反 A4 公理）
- 完整 UML 图生成
- Web UI（v1 不做）

### Architecture Approach

详见 [ARCHITECTURE.md](./ARCHITECTURE.md)。

现有流水线 `scan → parse → mirror → index → diff/gate` 设计良好，四个新特性均能干净融入。关键架构决策：

1. **Python & Go 适配器** — 纯增量，扩展 `wtcd-adapters` crate（单 crate 多模块），每个语言一个 `py.rs`/`go.rs` 文件实现 `LanguageAdapter` trait
2. **模块级聚合** — 文件镜像与路由索引之间的新中间层，新增 `ModuleResult` 类型、`group_by_module_id()` 和 `aggregate_module()` 函数
3. **知识层** — `wtcd-mirror::knowledge` 模块，三阶段流水线：Collect（读取模块镜像+索引）→ Analyze（依赖图、风险聚合）→ Generate（结构化文档）

**输出目录结构：**
```
mirror/
├── file/          # 已有 — 文件级镜像
├── module/        # 新增 — 模块级镜像
├── knowledge/     # 新增 — 知识层文档
└── routing_index.json  # 增强（含模块条目）
```

**Major components:**
1. `wtcd-adapters` — 新增 `PyAdapter` 和 `GoAdapter`，注册到 `register_all_adapters()`
2. `wtcd-core::types` — 扩展 `ExportKind`（Module, Decorator, Method, Struct）、新增 `ModuleResult`
3. `wtcd-mirror` — 新增模块镜像生成 + `knowledge` 子模块
4. `wtcd-scope` — 扩展 `SUPPORTED_EXTENSIONS` 加入 `"py"`, `"pyi"`, `"go"`
5. `wtcd-cli` — 流水线插入模块聚合和知识生成步骤

### Critical Pitfalls

详见 [PITFALLS.md](./PITFALLS.md)。v0.1.1 最关键的 5 个陷阱：

1. **tree-sitter 版本锁冲突 (EXT-C1)** — 多语法库编译时产生链接器符号碰撞。**预防：** 所有 tree-sitter crate 锁定相同 minor 版本，CI 编译集成测试
2. **Python 缩进解析边缘情况 (EXT-C2)** — Python 外部 C scanner 在混合 tab/space、dedent 错误恢复时产生异常。**预防：** 用 Django/Flask/FastAPI 等真实仓库测试，ERROR 节点优雅降级为 `confidence: low`
3. **Go 包语义不匹配 (EXT-C3)** — Go 的 package 声明、`internal/` 可见性、首字母大小写导出与 TS/Python 模式完全不同。**预防：** 先解析 go.mod，按 package 声明分组（非目录），过滤导出
4. **知识层生成泛化无用内容 (EXT-C5)** — 只有语法结构没有语义意图，生成的文档"比没有更糟"。**预防：** 只生成可验证事实（exports、dependencies、structure），绝不生成 "why"
5. **镜像成为第二真相源 (C5)** — 工程师先改镜像再改代码。**预防：** 每个镜像必须有 `source_artifacts` 字段，门禁阻断"只改镜像不改源码"的提交

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Python Language Adapter
**Rationale:** Python 适配器与 Go 适配器完全独立，可并行开发。Python 社区更大，tree-sitter-python 下载量更高（5.2M vs 3.8M），作为先行验证 tree-sitter 多语法集成模式。同时验证 EXT-C1（版本锁冲突）的预防策略。
**Delivers:** `PyAdapter` 实现 `LanguageAdapter` trait，Python 文件解析产出 `FileResult`，扩展 `ExportKind` 和 `SUPPORTED_EXTENSIONS`
**Addresses:** FEATURES.md §1 Python 适配器（所有 table stakes）
**Avoids:** EXT-C1（建立版本锁定模式）、EXT-C2（处理缩进边缘情况）、PITFALLS m2（避免适配器过度工程化）
**Risk:** MEDIUM — Python 装饰器包裹定义（`decorated_definition`）需要额外处理

### Phase 2: Go Language Adapter
**Rationale:** 与 Phase 1 并行或紧随其后，复用 Phase 1 建立的 tree-sitter 集成模式。Go 适配器的独特挑战在于包语义（EXT-C3），需要独立验证。
**Delivers:** `GoAdapter` 实现 `LanguageAdapter` trait，Go 文件解析产出 `FileResult`，Go 特殊处理（首字母大写导出、init() 标记、method vs function）
**Addresses:** FEATURES.md §2 Go 适配器（所有 table stakes）
**Avoids:** EXT-C3（Go 包语义正确处理）、EXT-M1（cgo 构建验证）、PITFALLS m2
**Risk:** MEDIUM — `method_declaration` vs `function_declaration` 的 receiver 提取需要验证

### Phase 3: Module-Level Mirror Aggregation
**Rationale:** 依赖 Phase 1+2 完成（需要多语言 FileResults 验证聚合逻辑）。这是架构上最大的新增层，需要新类型、新模板、新流水线步骤。
**Delivers:** `ModuleResult` 类型、`group_by_module_id()`/`aggregate_module()` 聚合逻辑、`ModuleMirrorHeader`/Body、`mirror/module/*.md` 输出、增强 routing_index
**Addresses:** FEATURES.md §3 模块级聚合（所有 table stakes + 模块内依赖图、模块级指纹等 differentiators）
**Avoids:** EXT-C4（模块定义歧义 — 需要 per-adapter boundary function）、EXT-M4（聚合大小爆炸 — 硬限制 2-3KB）、PITFALLS C6（增量更新）
**Risk:** HIGH — 模块边界定义（Python 包 vs Go package vs TS 目录）有歧义，需要设计决策

### Phase 4: Knowledge Layer
**Rationale:** 最后构建，依赖 Phase 3 的模块镜像和增强路由索引。知识层是纯派生品，风险在于内容质量而非技术实现。
**Delivers:** `wtcd-mirror::knowledge` 模块、`KnowledgeHeader` 类型、`mirror/knowledge/*.md` 输出（architecture.md、api-surface.md、risk-map.md）、`anrsm.yaml` 扩展 `knowledge:` 配置块
**Addresses:** FEATURES.md §4 知识层（所有 table stakes）
**Avoids:** EXT-C5（只生成可验证事实）、PITFALLS M4（不提前平台化）、PITFALLS C5（知识层从镜像编译，不允许独立事实）
**Risk:** MEDIUM — 输出格式和内容深度需要与实际 Agent 消费需求对齐

### Phase Ordering Rationale

- **适配器先行（Phase 1-2 并行）：** 产出 FileResult 供所有后续消费，无下游依赖，纯增量
- **模块聚合其次（Phase 3）：** 依赖多语言 FileResults 验证，是知识层的硬依赖
- **知识层最后（Phase 4）：** 纯派生品，依赖模块镜像 + 增强路由索引，提前做会产生"泛化无用内容"陷阱
- **此顺序遵循 PITFALLS M4 的警告：** 严格遵循阶段顺序，先适配器 → 镜像 → 聚合 → 知识

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3 (Module Aggregation):** 模块边界定义策略需要设计决策 — Python `__init__.py` vs Go `package` vs TS 目录的统一方案。需要 ADR。
- **Phase 4 (Knowledge Layer):** 输出内容深度和格式需要与 Agent 实际消费需求对齐。可能需要 `/gsd-research-phase` 验证最佳实践。

Phases with standard patterns (skip research-phase):
- **Phase 1 (Python Adapter):** tree-sitter-python 成熟，TS 适配器可作模板，模式明确
- **Phase 2 (Go Adapter):** tree-sitter-go 成熟，同上

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | tree-sitter 版本兼容性已验证（tree-sitter-language 桥接 + 项目已有 TS 适配器），所有依赖在 crates.io 可用且下载量验证 |
| Features | HIGH | Python/Go 的 AST 节点类型从 node-types.json 验证，tree-sitter 生态成熟，映射表完整 |
| Architecture | HIGH | 现有流水线设计良好，trait-based 扩展模式清晰，集成点明确，无架构重构需求 |
| Pitfalls | HIGH | 项目自身反模式文档 + 外部研究（tree-sitter issues、DevTool 采用率、CI 门禁模式）高度一致 |

**Overall confidence:** HIGH

### Gaps to Address

- **模块边界配置 vs 自动发现：** Python `__init__.py` 是自然边界，Go `package` 声明是自然边界，但 TS/JS 的目录约定需要设计决策。**处理方式：** Phase 3 开始时写 ADR，建议"自动发现 + anrsm.yaml 可选覆盖"
- **模块镜像指纹计算：** 聚合文件指纹 vs 重新计算语义指纹。**处理方式：** 建议聚合（`hash(sorted(child_fingerprints))`），确定性好
- **知识层是否需要独立指纹：** 完全由下层派生，建议 v0.1.1 不做独立指纹
- **Python `__all__` 语义：** 有 `__all__` 时只提取列表中的符号，无则全量导出。**处理方式：** Phase 1 实现时验证
- **Cross-language type reference gap:** 单仓 TS+Python+Go 中跨语言类型引用（protobuf/OpenAPI）当前无法关联。**处理方式：** v0.1.1 不解决，v2 评估

## Sources

### Primary (HIGH confidence)
- crates.io: `tree-sitter-python` 0.25.0, `tree-sitter-go` 0.25.0, `tree-sitter-md` 0.5.3, `tree-sitter` 0.26.7 — 版本、依赖、下载量
- Context7: `/tree-sitter/tree-sitter-python`, `/tree-sitter/tree-sitter-go` — AST 节点类型、语法结构
- GitHub: tree-sitter/tree-sitter#3069 — tree-sitter-language crate 架构
- GitHub: tree-sitter/tree-sitter-python/node-types.json — Python 完整节点类型
- GitHub: tree-sitter/tree-sitter-go/test/corpus — Go AST 结构示例
- 现有代码库: `wtcd-core::adapter`, `wtcd-adapters::ts`, `wtcd-mirror::template`, `wtcd-cli::commands::run`

### Secondary (MEDIUM confidence)
- tree-sitter issues: #4209（链接器符号冲突）、#5421（Go binding include path）、#4001（增量解析不一致）、#322（解析器无限循环）
- DevTools 采用率研究 — 95% 放弃率，time-to-first-value 关键
- AI Documentation Debt — 泛化内容、stale-on-arrival 模式
- CI 门禁失败模式 — 误报导致绕过
- AutomaDocs / Context+ MCP / Knowledge as Code — 架构参考模式

### Tertiary (LOW confidence)
- WebSearch: "tree-sitter 0.26 compatible tree-sitter-python 0.25" — ABI 兼容性验证
- Cross-language type reference gap — v2 问题，当前无成熟方案

---

*Research completed: 2026-03-21*
*Ready for roadmap: yes*
