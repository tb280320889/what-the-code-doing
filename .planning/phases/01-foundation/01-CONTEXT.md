# Phase 1: Foundation - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

用户可以安装 WTCD，初始化仓库配置，配置文件范围规则，并解析 TS/JS 文件提取结构化事实（导出符号、依赖边、函数签名、副作用）。所有 CLI 输出为 JSON 格式，解析失败时优雅降级。

</domain>

<decisions>
## Implementation Decisions

### JSON 输出结构
- **D-01:** 顶层单层扁平结构：`files[]`, `errors`(空数组，保留兼容), `summary`
- **D-02:** 每个文件结果 = 事实层（exports, imports, symbols, signatures, side_effects）+ 元数据（confidence, file_path, parse_time）
- **D-03:** 解析失败的文件在同级 files 数组中表示，confidence=low/none，附带 error_message（不单独拆 errors 数组）
- **D-04:** 带 `api_version: "1"` 版本字段
- **D-05:** 依赖边 = 结构化对象数组 `{source: string, symbols: string[], kind: "named"|"default"|"namespace"}`
- **D-06:** 副作用 = 分类对象数组 `{kind: "io"|"network"|"storage"|"log", target: string, line: number}`
- **D-07:** summary = 运行级统计（total_files, parsed_ok, confidence_low, confidence_none, total_exports, total_imports, elapsed_ms）

### anrsm.yaml 配置格式
- **D-08:** 顶层三块分离：scope（文件范围）、mirror（镜像规则，Phase 1 预留空）、output（输出配置）
- **D-09:** 使用 gitignore 风格标准 glob 语法（`**/*.ts`, `!**/*.test.ts`），与 ignore crate 天然兼容
- **D-10:** 无 anrsm.yaml 时拒绝运行，输出错误提示 `请先运行 wtcd init`
- **D-11:** Phase 1 不定义模块归属规则，modules 块留到 Phase 2

### 错误恢复策略
- **D-12:** 三级 confidence 等级：`high`（完整解析）/ `low`（部分解析，有语法错误）/ `none`（完全失败）
- **D-13:** tree-sitter error recovery 能产出部分 CST → confidence=low，输出能提取的符号；解析完全崩溃 → confidence=none
- **D-14:** confidence=none 的文件输出空事实数组 + error_message（不省略该文件条目）
- **D-15:** 单个文件解析失败不影响 anrsm run 退出码，始终为 0

### tree-sitter 提取范围
- **D-16:** 导出覆盖值导出（function/class/const/let/var）+ 类型导出（type/interface/enum）
- **D-17:** 依赖边覆盖 ESM（import/export）+ CommonJS（require()），不提取动态 import()
- **D-18:** 函数签名提取名称 + 参数（带类型标注）+ 返回类型，类型不可用时标记 unknown
- **D-19:** 副作用检测使用已知 API 集（fs.*, fetch, axios.*, console.*, process.exit, localStorage.* 等），可扩展

### 项目目录架构
- **D-20:** 项目名从 ANRSM 改为 **WTCD**（What The Code Doing），所有 crate 和配置使用 wtcd- 前缀
- **D-21:** Cargo workspace 多 crate 结构，Phase 1 创建 4 个 crate：wtcd-core, wtcd-scope, wtcd-adapters, wtcd-cli
- **D-22:** 每个 crate 内部使用扁平文件布局（非子目录），文件少时更清晰
- **D-23:** wtcd-core 包含：lib.rs, types.rs, adapter.rs, error.rs, config.rs
- **D-24:** wtcd-scope 包含：lib.rs, scanner.rs（文件遍历 + .gitignore）, scope.rs（配置解析 + 范围规则）
- **D-25:** wtcd-adapters 包含：lib.rs（注册所有适配器）, ts.rs（TypeScript/JS 适配器）
- **D-26:** wtcd-cli 包含：main.rs, commands/init.rs, commands/run.rs, output.rs
- **D-27:** 单元测试在各 crate 的 src/ 同级（#[cfg(test)] 模块），集成测试在 workspace root tests/
- **D-28:** 测试 fixture 放在 tests/fixtures/ 目录，按语言分目录（ts/, js/）
- **D-29:** 配置模板用 `include_str!` 嵌入 wtcd-cli crate，单二进制无需外部文件

### the agent's Discretion
- tree-sitter query 的具体实现方式（S-expression 语法细节）
- 各类型的精确字段定义（ArtifactId, ConfidenceBand 等的枚举/结构体细节）
- Cargo.toml 中的精确版本号和 feature flags
- CLI help 文本和 shell completion 生成

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 架构与组件模型
- `docs/04_Architecture_and_Component_Model.md` — 组件定义、Trait 约束、流水线架构、适配器接口规范
- `docs/05_Information_Model_and_Repository_Layout.md` — 信息模型、路径映射、目录布局约定

### 语言适配器
- `docs/06_Semantic_Mirror_Protocol_and_Header_Spec.md` — 镜像协议定义、8 标准段落（Phase 2 范围但理解语义模型需要）
- `docs/02_Normative_Language_and_Glossary.md` — 术语体系、MUST/SHOULD 规范用语

### CLI 与 Agent 集成
- `docs/08_Agent_Runtime_Contract_and_Workflows.md` — Agent 读取工作流、CLI 调用约定

### 技术栈研究
- `.planning/research/STACK.md` — 技术选型理由、版本兼容、MSRV 注意事项
- `.planning/research/ARCHITECTURE.md` — 推荐项目结构、架构模式、反模式

### Schema 与示例
- `schemas/mirror-header.schema.json` — 镜像头部 JSON Schema（Phase 2 依赖，Phase 1 理解输出目标）
- `examples/anrsm.example.yaml` — 配置文件示例
- `examples/file_mirror_example.md` — 镜像文件示例

</canonical_refs>

<code_context>
## Existing Code Insights

### 项目状态
- 绿地项目，无现有 Rust 代码
- 11 份规范文档已完成并冻结
- JSON Schema 已定义（mirror-header, drift-report）
- 示例工件已提供（配置、文件镜像、漂移报告）

### 可复用资产
- `examples/anrsm.example.yaml` — 可作为 anrsm init 生成配置模板的参考
- `schemas/mirror-header.schema.json` — 定义了输出目标结构
- `schemas/drift-report.schema.json` — 定义了漂移报告结构（Phase 3 范围）

### 已锁定的技术模式
- tree-sitter 0.26 + tree-sitter-typescript 0.23 — 多语言统一解析
- gix 0.80 — 纯 Rust Git 操作
- clap 4.6 derive API — CLI 声明式定义
- Trait-based 适配器（编译时注册，&dyn LanguageAdapter）
- Pipeline 架构（Config → Scope → Scan → Parse → ...）

### 注意事项
- clap 4.6 MSRV 是 1.85，若 MSRV 目标 < 1.85 需使用 clap 4.5
- gix 0.80 MSRV 是 1.82
- jsonschema 0.45 MSRV 是 1.83
- serde_yaml 已废弃，使用 yaml_serde 0.10（YAML 官方维护）

</code_context>

<specifics>
## Specific Ideas

- 项目名改为 WTCD（What The Code Doing），所有 crate 和配置使用 wtcd- 前缀
- Phase 1 只做文件级解析，模块归属和镜像生成留到 Phase 2
- anrsm.yaml 中 mirror 块可以放空 {}，为 Phase 2 预留结构
- anrsm.yaml 中 modules 块 Phase 1 不出现，Phase 2 再加

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-21*
