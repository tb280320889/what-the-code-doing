# ANRSM — Technology Stack Research

**Domain:** Rust CLI tool for code repository semantic mirror generation
**Researched:** 2026-03-21
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **clap** | 4.6 | CLI argument parsing & subcommands | Rust CLI 的事实标准，722M+ 下载，33K+ 反向依赖。Derive API 声明式定义子命令，自动生成 help/shell completions。powers ripgrep, bat, fd 等一线工具 |
| **tree-sitter** | 0.26 | TS/JS 语法解析（CST） | 多语言统一解析框架，天然适配 ANRSM 的多语言适配器架构。增量解析、容错恢复、无 serde 开销（直接在 Rust 中遍历）。tree-sitter-typescript 0.23 提供 TS/TSX 支持 |
| **serde** | 1.0 | 序列化/反序列化框架 | Rust 生态的统一序列化层。YAML 配置、JSON Schema、JSON 输出全部经过 serde |
| **gix** (gitoxide) | 0.80 | Git 仓库操作（纯 Rust） | 纯 Rust 实现，无 C/OpenSSL 依赖，跨平台编译简单。读操作（status, diff, commit SHA, refs）已 production-grade。7.4M+ 最近下载量，趋势明确 |
| **sha2** | 0.10 | SHA-256 指纹计算 | RustCrypto 项目，512M+ 下载，API 稳定。`Sha256::digest()` 一行计算指纹。支持增量 hashing |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **serde_json** | 1.0 | JSON 读写 | CLI JSON 输出、Index 文件生成、Schema 验证输入 |
| **yaml_serde** | 0.10 | YAML 读写（serde_yaml 正统继任） | `anrsm.yaml` 配置解析、YAML Front Matter 生成。由 YAML 官方组织维护 |
| **jsonschema** | 0.45 | JSON Schema 验证 | mirror-header schema 验证、drift-report schema 验证。支持 draft 2020-12 |
| **pulldown-cmark** | 0.13 | Markdown 解析/生成 | 镜像文件（.md）的读取、写入和结构化处理。CommonMark 100% 合规，纯 Rust |
| **ignore** | 0.4 | 目录遍历（尊重 .gitignore） | Repository Scanner 组件 — 递归扫描仓库文件，自动过滤 .gitignore/.ignore 规则。来自 ripgrep 作者，支持并行遍历 |
| **anyhow** | 1.0 | 灵活错误处理 | 应用层错误传播，`?` 操作符链式调用 |
| **thiserror** | 2.0 | 自定义 Error 类型派生 | 库层错误类型定义，`#[derive(Error)]` 自动生成 Display/From 实现 |
| **tracing** | 0.1 | 结构化日志 & 诊断 | 可观测性 — 记录适配器置信度、漂移原因、全量重建触发条件等诊断信息 |
| **tracing-subscriber** | 0.3 | tracing 输出格式化 | CLI 终端输出配置（verbose 模式等） |
| **tempfile** | 3 | 临时文件管理 | 镜像生成时的原子写入（先写临时文件再 rename） |
| **colored** | 2 | 终端彩色输出 | 漂移报告、Policy Gate 结果的人类友好展示 |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| **cargo-nextest** | 测试运行器 | 比 `cargo test` 更快，更好的并行和输出 |
| **cargo-watch** | 热重载开发 | `cargo watch -x check -x test` |
| **cargo-deny** | 依赖审计 | 许可证检查、安全漏洞扫描、重复依赖检测 |
| **insta** | 快照测试 | 镜像输出的 golden file 测试 — 验证生成结果稳定性 |

## Installation

```toml
# Cargo.toml
[package]
name = "anrsm"
version = "0.1.0"
edition = "2021"

[dependencies]
# CLI
clap = { version = "4.6", features = ["derive", "env"] }

# AST Parsing (TypeScript/JavaScript)
tree-sitter = "0.26"
tree-sitter-typescript = "0.23"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
yaml_serde = "0.10"

# JSON Schema
jsonschema = "0.45"

# Git
gix = { version = "0.80", features = ["max-performance-safe"] }

# Hashing
sha2 = "0.10"

# Markdown
pulldown-cmark = "0.13"

# File traversal
ignore = "0.4"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Logging / diagnostics
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
tempfile = "3"
colored = "2"

[dev-dependencies]
insta = "1.46"
tempfile = "3"
```

## Key Architectural Decisions

### 1. tree-sitter vs SWC vs Oxc — 选择了 tree-sitter

这是整个技术栈最关键的决策。ANRSM 需要从 TS/JS 源码中提取 **结构化事实**（exports, imports, symbols, types），而非做代码转换。

| 候选 | 优势 | 劣势 | 判定 |
|------|------|------|------|
| **tree-sitter 0.26** | 多语言统一 API；容错解析（部分错误不影响整体）；增量解析；无 serde 序列化开销；tree-sitter-typescript 已成熟 | CST 而非 AST（节点类型需手动匹配）；提取比 SWC 需要更多代码 | ✅ **选用** |
| **SWC (swc_ecma_parser)** | 完整 AST；ECMA 标准节点类型；25M+ npm 下载；生产验证（Next.js） | ~37MB 二进制体积；serde 序列化开销；单一语言（需为每种语言独立集成） | 备选 — 若 tree-sitter 提取不足时切换 |
| **Oxc (oxc_parser)** | 最快（3x SWC）；~2MB 体积；99.86% TS 兼容 | MSRV 1.92（过高）；API 仍在快速迭代（0.120，每 3 天一版）；生态尚不成熟 | 关注 — 2026 H2 重新评估 |

**选择 tree-sitter 的核心理由：**
1. **多语言适配器架构**：tree-sitter 的统一 API 意味着添加 Python/Go/Rust 适配器时，解析层无需重构
2. **容错解析**：ANRSM 必须处理可能不完整的源码（用户正在编辑中的文件），tree-sitter 的 error recovery 保证即使有语法错误也能产出部分 CST
3. **无序列化开销**：在 Rust 进程内直接遍历 CST，无需像 SWC/Oxc 那样经过 serde 序列化层
4. **可靠的生态**：tree-sitter 是 Neovim, Helix, GitHub (code navigation), ast-grep 等工具的底层引擎

**信心等级：HIGH** — 基于 Context7 文档 + 多源验证 + 实际项目参考

### 2. gix vs git2 — 选择了 gix

| 候选 | 优势 | 劣势 | 判定 |
|------|------|------|------|
| **gix 0.80** | 纯 Rust，无 C 依赖；OpenSSL 不是必需；跨平台编译简单；API 现代 | 部分写操作不完整（merge, hooks）；API 仍在演进 | ✅ **选用** |
| **git2** | 功能完整（libgit2 全量绑定）；生产验证十年+ | 需要 OpenSSL/libssh2 C 依赖；跨平台交叉编译痛苦；二进制体积更大 | 备选 — 若 gix 功能不足时回退 |

**ANRSM v1 的 Git 需求分析：**
- ✅ `gix` 支持：打开仓库、读取 refs、获取 HEAD commit SHA、status 检测、blob/tree diff、读取 .gitignore
- ⚠️ `gix` 部分支持：stash、worktree（可用但不够成熟）
- ❌ `gix` 不需要：push, pull, merge, hooks（v1 不涉及）

**结论：ANRSM v1 的 Git 需求完全落在 gix 的 production-grade 范围内。**

**信心等级：HIGH** — gix 7.4M+ 最近下载量，是 Rust 社区公认的 Git 未来方向

### 3. yaml_serde — serde_yaml 的正统继任

serde_yaml 已停止维护。社区出现多个 fork，质量参差不齐：
- ❌ `serde_yml` — 被怀疑是 AI 生成的代码，nushell 等项目已移除
- ⚠️ `serde_yaml_ng` — 社区 fork，质量尚可但维护者单一
- ✅ `yaml_serde` 0.10 — **YAML 官方组织维护**，API 兼容 serde_yaml，可无缝迁移

**信心等级：HIGH** — 基于官方组织背书 + 社区共识

## Alternatives Considered

| 推荐选择 | 替代方案 | 何时使用替代 |
|----------|----------|-------------|
| tree-sitter | SWC (swc_ecma_parser) | 当 tree-sitter 的 CST 提取无法满足精确的类型签名/泛型信息提取需求时 |
| gix | git2 | 当需要完整的 Git 写操作（merge, hooks）且可接受 C 依赖时 |
| yaml_serde | serde_yaml_ng | 若 yaml_serde 更新滞后，serde_yaml_ng 是第二选择 |
| pulldown-cmark | comrak | 需要完整的 GitHub Flavored Markdown 扩展（task lists, tables）时 |
| ignore | walkdir | 简单场景不需要 .gitignore 过滤时（但 ANRSM 需要） |
| sha2 0.10 | blake3 | 若需要更快的非加密 hash（语义指纹的性能优化路径） |

## What NOT to Use

| 避免使用 | 原因 | 替代方案 |
|----------|------|----------|
| **serde_yaml**（原版） | 已停止维护，无安全更新 | yaml_serde 0.10 |
| **serde_yml** | 被怀疑 AI 生成，nushell 等项目已移除，安全风险 | yaml_serde 0.10 |
| **structopt** | 已废弃，功能已合并入 clap derive | clap 4.6 with `derive` feature |
| **error-chain** | 已废弃 | thiserror + anyhow |
| **rustc-serialize** | 已废弃 | serde |
| **yaml-rust** | 原始 YAML 1.1 实现，不维护 | yaml_serde 0.10 |
| **hyper**（直接使用） | ANRSM v1 不需要 HTTP（CLI 工具） | 不需要；若未来需要 HTTP 用 reqwest |
| **tokio**（直接使用） | ANRSM 是同步 CLI，不需要 async runtime | 标准库 + rayon（若需并行） |
| **regex** 解析 TS/JS | 架构规范明确禁止：正则适配器 MUST 标记低置信度 | tree-sitter |

## Version Compatibility

| 注意事项 | 详情 |
|----------|------|
| tree-sitter 0.26 ↔ tree-sitter-typescript 0.23 | typescript crate 使用 `tree-sitter-language ^0.1` 抽象层，兼容 tree-sitter 0.24+。需验证实际编译兼容性 |
| gix 0.80 feature flags | ANRSM 只需要 `max-performance-safe` 子集，不需要 HTTP transport（纯本地操作） |
| sha2 0.10 vs 0.11-rc | 0.11 尚在 RC 阶段，**使用 0.10.9 稳定版**。0.11 MSRV 1.85 过高 |
| jsonschema 0.45 | MSRV 1.83。ANRSM 使用 draft 2020-12 schema，需启用对应 feature |
| clap 4.6 | MSRV 1.85（4.5 系列是 1.74）。若整体 MSRV 目标 < 1.85，使用 clap 4.5 |
| pulldown-cmark 0.13 | MSRV 1.71.1，无兼容问题 |

### MSRV 统一建议

**推荐 MSRV: 1.82**（gix 要求 1.82 是当前最高门槛）
- clap 4.5 系列支持 1.74 ✅
- gix 0.80 要求 1.82 ✅
- tree-sitter 0.26 要求 1.77 ✅
- jsonschema 0.45 要求 1.83 ⚠️ → 若需 1.82，使用 jsonschema 0.41 或确认实际 MSRV
- sha2 0.10 无 MSRV 限制 ✅

**注意：** clap 4.6 的 MSRV 是 1.85。若坚持 1.82 MSRV，使用 `clap = "~4.5"`。

## Future Considerations

### Oxc 重新评估窗口

Oxc 是值得关注的新兴选择：
- 2026 H2 重新评估：MSRV 是否降低到合理范围？API 是否稳定？
- 若 ANRSM 需要 **代码转换** 能力（不仅仅是提取），Oxc/SWC 比 tree-sitter 更合适
- 若 benchmark 显示 tree-sitter 提取性能不足，Oxc 是首选替代

### gix 功能扩展

当 ANRSM 需要以下能力时，检查 gix 是否已实现：
- [ ] Git hooks 集成（pre-commit gate）
- [ ] Worktree 支持
- [ ] Merge base 计算

### blake3 作为语义指纹算法

若 SHA-256 在大规模仓库中成为性能瓶颈：
- `blake3` 比 SHA-256 快 5-10x
- 同样是密码学安全的
- 可作为 fingerprint algorithm v2 升级路径

## Sources

- Context7: `/websites/rs_clap` — clap 4.x derive API, features (HIGH)
- Context7: `/websites/rs_tree-sitter` — tree-sitter Rust bindings, parser usage (HIGH)
- Context7: `/websites/rs_gix` — gix API, feature flags, git2 migration guide (HIGH)
- Context7: `/websites/rs_git2` — git2-rs API coverage (HIGH)
- Context7: `/websites/rs_crate_serde_yml` — YAML serde ecosystem (HIGH)
- Context7: `/pulldown-cmark/pulldown-cmark` — Markdown parsing (HIGH)
- Context7: `/stranger6667/jsonschema` — JSON Schema validation (HIGH)
- Context7: `/swc-project/swc` — SWC parser (HIGH)
- Context7: `/websites/rustdoc_swc_rs` — SWC ecma parser API (HIGH)
- crates.io: `yaml_serde` 0.10.4 — YAML org maintained fork (HIGH)
- crates.io: `oxc_parser` 0.120.0, `oxc_ast` 0.121.0 — Oxc current state (HIGH)
- crates.io: `gix` 0.80.0 — gitoxide current version (HIGH)
- crates.io: `sha2` 0.10.9 — RustCrypto stable (HIGH)
- crates.io: `jsonschema` 0.45.0 — JSON Schema validator (HIGH)
- crates.io: `ignore` 0.4.25 — BurntSushi directory walker (HIGH)
- crates.io: `tree-sitter` 0.26.7 — current version (HIGH)
- crates.io: `tree-sitter-typescript` 0.23.2 — TS grammar (HIGH)
- WebSearch: "gix vs git2 2025" — ecosystem trend toward gix (MEDIUM)
- WebSearch: "SWC vs tree-sitter" — benchmark comparisons (MEDIUM)
- WebSearch: "Oxc vs SWC 2026" — PkgPulse comparison (MEDIUM)
- WebSearch: "serde-yml yaml_serde" — YAML crate ecosystem (MEDIUM)
- GitHub: nushell/nushell#14985 — serde_yml removal discussion (MEDIUM)
- GitHub: MaaAssistantArknights/maa-cli#399 — git2→gix migration RFC (MEDIUM)

---
*Stack research for: ANRSM — AI Native Repo Semantic Mirror*
*Researched: 2026-03-21*
*Next review: After M0 dogfooding, validate tree-sitter extraction quality against real TS/JS codebases*
