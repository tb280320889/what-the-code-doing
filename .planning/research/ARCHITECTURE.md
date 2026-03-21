# Architecture Research: ANRSM

**Domain:** Rust CLI — 代码语义镜像系统（Code Semantic Mirror System）
**Researched:** 2026-03-21
**Confidence:** HIGH

## Executive Summary

ANRSM 的架构应遵循 **控制面/数据面分离 + Trait 驱动语言适配器 + Tree-sitter 增量解析 + 文件系统存储** 的模式。这不是发明，而是对已有成熟模式的组合：

1. **Trait 驱动的适配器架构**（Rust 社区标准实践，ZeroClaw/Litho/dynamic_plugin 等项目验证）——编译时类型安全，无动态加载的 ABI 风险
2. **Tree-sitter 增量解析**（codegraph/NekoCode/mdiff 等 2025-2026 年项目一致采用）——O(changed) 级别的增量更新
3. **文件系统作为镜像存储**（规范已锁定的决策 D1）——可 diff、可版本管理、无外部依赖
4. **CLI 按需运行模式**（非 daemon）——适合 pre-commit/CI 场景

最接近的参考项目是 **codegraph**（optave/codegraph）：同样使用 Rust + Tree-sitter、支持多语言、增量重建、CI 门禁、函数级依赖图。ANRSM 与其不同之处在于：codegraph 做的是通用代码图数据库，ANRSM 专注于生成语义镜像 + 漂移检测。

---

## Standard Architecture

### System Overview

```
┌────────────────────────────────────────────────────────────────────┐
│                          CLI Entry Point                           │
│                    (clap arg parsing, subcommands)                  │
├────────────────────────────────────────────────────────────────────┤
│                        Control Plane                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ Scope Manager │  │ Config Loader│  │ Policy Gate  │             │
│  │ (范围管理)    │  │ (配置加载)    │  │ (策略门禁)    │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │                 │                  │                     │
├─────────┴─────────────────┴──────────────────┴─────────────────────┤
│                        Data Plane Pipeline                        │
│                                                                    │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐     │
│  │ Repo     │───▶│ Language │───▶│ Semantic │───▶│Fingerprint│    │
│  │ Scanner  │    │ Adapter  │    │ Extractor│    │ Engine   │     │
│  │(仓库扫描)│    │(语言适配)│    │(语义提取)│    │(指纹计算)│     │
│  └──────────┘    └──────────┘    └──────────┘    └────┬─────┘     │
│                                                        │           │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐         │           │
│  │ Index    │◀───│ Mirror   │◀───│ Drift    │◀────────┘           │
│  │ Builder  │    │Generator │    │ Analyzer │                     │
│  │(索引构建)│    │(镜像生成)│    │(漂移分析)│                     │
│  └──────────┘    └──────────┘    └──────────┘                     │
├────────────────────────────────────────────────────────────────────┤
│                        Persistence Layer                           │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────┐   │
│  │ .anrsm/cache/    │  │ mirror/file/     │  │ mirror/index/  │  │
│  │ (指纹、解析缓存) │  │ (文件镜像 .md)   │  │ (路由/符号索引)│   │
│  └──────────────────┘  └──────────────────┘  └────────────────┘   │
│  ┌──────────────────┐  ┌──────────────────┐                       │
│  │ mirror/module/   │  │ knowledge/       │                       │
│  │ (模块镜像 .md)   │  │ (知识层文档)     │                       │
│  └──────────────────┘  └──────────────────┘                       │
└────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| CLI Entry | 命令解析、子命令路由、输出格式化 | `clap` derive API，子命令枚举 |
| Config Loader | 加载 `anrsm.yaml`，合并默认值 | `serde_yaml` + 默认值推导 |
| Scope Manager | 确定哪些文件受管，模块归属 | glob 匹配 + 排除规则，确定性 |
| Policy Gate | 根据漂移级别决定 Pass/Fail | 策略矩阵查表，输出失败码 |
| Repo Scanner | 枚举文件、获取 Git diff | `git2` crate 或 shell-out |
| Language Adapter | AST 解析，符号/导入提取 | **Tree-sitter**（核心选型） |
| Semantic Extractor | 语言中立语义模型归一 | Trait 实现，逐语言定制 |
| Fingerprint Engine | 双指纹（source + semantic） | SHA-256/BLAKE3，结构规范化 |
| Mirror Generator | 从语义事实生成 Markdown 镜像 | 模板引擎，章节顺序稳定 |
| Drift Analyzer | 新旧状态对比，C0-C3 分级 | 指纹比对 + 邻域扩展 |
| Index Builder | 路由/符号/依赖索引构建 | JSON 输出，增量分片更新 |
| Knowledge Compiler | 派生人类可读知识文档 | 模板 + 来源追溯 |

---

## Recommended Project Structure

```
anrsm/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── anrsm-core/               # 核心类型、trait 定义、语义模型
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs          # ArtifactId, ModuleId, Fingerprint 等
│   │   │   ├── semantic.rs       # 统一语义模型（8 字段簇）
│   │   │   ├── adapter.rs        # LanguageAdapter trait 定义
│   │   │   ├── error.rs          # 统一错误类型（thiserror）
│   │   │   └── config.rs         # anrsm.yaml 配置结构
│   │   └── Cargo.toml
│   │
│   ├── anrsm-scope/              # Scope Manager 实现
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── scanner.rs        # Repo Scanner（Git 集成）
│   │   │   └── module.rs         # 模块归属规则
│   │   └── Cargo.toml
│   │
│   ├── anrsm-adapters/           # 语言适配器集合
│   │   ├── src/
│   │   │   ├── lib.rs            # 注册所有适配器
│   │   │   ├── ts.rs             # TypeScript/JS 适配器（tree-sitter-typescript）
│   │   │   ├── rust_lang.rs      # Rust 适配器（tree-sitter-rust）
│   │   │   └── fallback.rs       # 正则回退适配器（低置信度）
│   │   └── Cargo.toml
│   │
│   ├── anrsm-fingerprint/        # Fingerprint Engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── source.rs         # Source Fingerprint
│   │   │   └── semantic.rs       # Semantic Fingerprint
│   │   └── Cargo.toml
│   │
│   ├── anrsm-mirror/             # Mirror Generator
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── file_mirror.rs    # 文件镜像生成
│   │   │   ├── module_mirror.rs  # 模块镜像聚合
│   │   │   └── template.rs       # 镜像模板与压缩规则
│   │   └── Cargo.toml
│   │
│   ├── anrsm-index/              # Index Builder
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── routing.rs        # 路由索引
│   │   │   ├── symbol.rs         # 符号索引
│   │   │   └── dependency.rs     # 依赖索引
│   │   └── Cargo.toml
│   │
│   ├── anrsm-drift/              # Drift Analyzer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── analyzer.rs       # 漂移分析核心
│   │   │   ├── classifier.rs     # C0-C3 变更分类
│   │   │   └── report.rs         # 漂移报告生成
│   │   └── Cargo.toml
│   │
│   └── anrsm-cli/                # CLI 二进制入口
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/
│       │   │   ├── init.rs       # anrsm init
│       │   │   ├── build.rs      # anrsm build（全量）
│       │   │   ├── update.rs     # anrsm update（增量）
│       │   │   ├── check.rs      # anrsm check（门禁）
│       │   │   ├── query.rs      # anrsm query（Agent 读取）
│       │   │   └── drift.rs      # anrsm drift（漂移报告）
│       │   └── output.rs         # JSON/TTY 输出格式化
│       └── Cargo.toml
│
└── tests/
    ├── integration/              # 集成测试
    └── fixtures/                 # 测试仓库样本
```

### Structure Rationale

- **`crates/anrsm-core/`:** 所有 crate 共享的类型和 trait。LanguageAdapter trait 定义在此，任何新适配器只需依赖此 crate。
- **`crates/anrsm-adapters/`:** 集中管理所有语言适配器。每个适配器是独立模块，但编译为同一 crate（v1 策略：编译时注册，不做动态加载）。
- **`crates/anrsm-scope/`:** 职责单一：读配置 → 枚举文件 → 输出受管清单。不含解析逻辑。
- **`crates/anrsm-drift/`:** 独立的漂移分析模块，可单独测试变更分级逻辑。
- **`crates/anrsm-cli/`:** 纯入口点，不含业务逻辑，只做命令路由和输出格式化。

---

## Architectural Patterns

### Pattern 1: Trait-Based Adapter（Trait 驱动适配器）

**What:** 定义 `LanguageAdapter` trait，每个语言实现该 trait。编译时注册，通过 `&dyn LanguageAdapter` 或 enum dispatch 调用。

**When to use:** 需要多语言支持但不需要运行时动态加载插件的场景。ANRSM v1 符合此条件。

**Trade-offs:**
- ✅ 编译时类型安全，无 ABI 稳定性问题
- ✅ 无动态库加载的开销和安全风险
- ✅ Rust 编译器保证 trait 实现完整性
- ❌ 添加新语言需重新编译
- ❌ 所有适配器都编译进二进制（可用 feature gate 优化）

**Example:**
```rust
// crates/anrsm-core/src/adapter.rs
pub trait LanguageAdapter: Send + Sync {
    /// 此适配器支持的文件扩展名
    fn supported_extensions(&self) -> &[&str];

    /// 解析单个文件，返回语言特定的解析结果
    fn parse(&self, file: &Path, content: &str, ctx: &ParseContext)
        -> Result<ParseResult, AdapterError>;

    /// 解析能力的置信度等级
    fn confidence(&self) -> ConfidenceBand;

    /// 适配器版本（用于指纹算法升级触发重建）
    fn version(&self) -> &str;
}

// 注册表：编译时确定
pub fn default_registry() -> AdapterRegistry {
    let mut reg = AdapterRegistry::new();
    reg.register(Box::new(TsAdapter::new()));
    reg.register(Box::new(RustAdapter::new()));
    reg.register(Box::new(FallbackAdapter::new())); // 正则回退
    reg
}
```

**Why not dynamic loading?** 研究显示，Rust 生态中动态插件加载（libloading + C ABI）存在严重的 ABI 稳定性风险和安全问题。ZeroClaw 项目（2026-02）明确指出 trait-based 编译时方案比动态插件更安全。Nushell 等项目用 IPC 方案（独立进程 + stdin/stdout），但对 ANRSM 这种 CLI 工具来说，编译时注册是最简洁的方案。

**Confidence:** HIGH — 这是 Rust 社区的标准实践，多个 2025-2026 年项目验证。

---

### Pattern 2: Tree-sitter Incremental Parsing（Tree-sitter 增量解析）

**What:** 使用 Tree-sitter 做 AST 解析，利用其内置的增量解析能力。当文件发生变化时，只重新解析受影响的 AST 节点。

**When to use:** 需要解析多种语言的代码、要求高性能增量更新的场景。ANRSM 完美匹配。

**Trade-offs:**
- ✅ 支持 40+ 语言的 grammar，社区维护
- ✅ 增量解析 O(changed)，typical edit < 1ms
- ✅ 容错解析：即使有语法错误也能产出部分 AST
- ✅ Rust bindings 成熟（`tree-sitter` crate v0.24+）
- ✅ 统一的 query 语言（类 Lisp 语法），适合提取符号/导入
- ❌ 生成的是 CST（具体语法树），需要后处理提取语义
- ❌ 某些语言 grammar 维护质量不一

**Example:**
```rust
use tree_sitter::{Parser, Language};

pub struct TsAdapter {
    parser: Parser,
}

impl LanguageAdapter for TsAdapter {
    fn supported_extensions(&self) -> &[&str] {
        &["ts", "tsx", "js", "jsx", "mjs", "cjs"]
    }

    fn parse(&self, file: &Path, content: &str, ctx: &ParseContext)
        -> Result<ParseResult, AdapterError>
    {
        let tree = self.parser.parse(content, None)
            .ok_or(AdapterError::ParseFailed)?;

        // 使用 tree-sitter query 提取符号
        let exports = self.extract_exports(&tree, content)?;
        let imports = self.extract_imports(&tree, content)?;
        let symbols = self.extract_symbols(&tree, content)?;

        Ok(ParseResult {
            exports,
            imports,
            symbols,
            confidence: ConfidenceBand::High,
            ..Default::default()
        })
    }
}
```

**为什么不用 SWC/其他？** codegraph（2026-03）和 NekoCode（2025-08）都用 Tree-sitter 作为多语言统一解析层。SWC 只支持 JS/TS，ANTLR 需要每语言单独的 grammar 编译。Tree-sitter 是目前唯一同时满足"多语言统一 + 增量解析 + Rust 原生"的方案。

**Confidence:** HIGH — 2025-2026 年多语言代码分析工具的事实标准。

---

### Pattern 3: Pipeline Architecture（流水线架构）

**What:** 数据面组件按流水线顺序执行，每个组件消费上游输出、产出下游输入。支持全量和增量两种模式。

**When to use:** 数据处理有明确的阶段依赖关系，且各阶段可以独立测试。

**Trade-offs:**
- ✅ 职责清晰，每个阶段可独立单元测试
- ✅ 增量模式下可跳过不相关的阶段
- ✅ 可以在任意阶段插入诊断输出
- ❌ 串行执行有性能瓶颈（可并行化缓解）
- ❌ 中间数据的序列化/反序列化开销

**Example:**
```rust
pub struct Pipeline {
    scope: ScopeManager,
    scanner: RepoScanner,
    adapters: AdapterRegistry,
    extractor: SemanticExtractor,
    fingerprint: FingerprintEngine,
    mirror: MirrorGenerator,
    index: IndexBuilder,
    drift: DriftAnalyzer,
    policy: PolicyGate,
}

impl Pipeline {
    /// 全量初始化流水线
    pub fn init(&self, config: &Config) -> Result<InitReport> {
        let scope = self.scope.resolve(config)?;
        let files = self.scanner.full_scan(&scope)?;
        let parse_results = self.adapters.parse_all(&files)?;
        let semantics = self.extractor.normalize_all(&parse_results)?;
        let fingerprints = self.fingerprint.compute_all(&semantics)?;
        let mirrors = self.mirror.generate_all(&semantics, &fingerprints)?;
        let index = self.index.build_all(&mirrors, &semantics)?;
        let baseline = self.drift.establish_baseline(&fingerprints);
        Ok(InitReport { mirrors, index, baseline })
    }

    /// 增量更新流水线
    pub fn update(&self, config: &Config) -> Result<UpdateReport> {
        let diff = self.scanner.git_diff()?;
        let affected = self.scope.resolve_affected(&diff)?;
        let parse_results = self.adapters.parse_changed(&affected)?;
        let semantics = self.extractor.update(&parse_results)?;
        let fingerprints = self.fingerprint.recompute(&semantics)?;
        let drift = self.drift.analyze(&diff, &fingerprints)?;
        let mirrors = self.mirror.update_affected(&drift)?;
        let index = self.index.update_shards(&drift)?;
        let policy = self.policy.evaluate(&drift)?;
        Ok(UpdateReport { drift, mirrors, index, policy })
    }
}
```

**Confidence:** HIGH — 规范文档 04 已明确定义两条流水线。

---

### Pattern 4: Semantic Fingerprinting（语义指纹）

**What:** 对每个文件计算两种指纹——source_fingerprint（基于源码字节）和 semantic_fingerprint（基于归一化后的语义对象）。C0 变更只影响前者，C1+ 影响后者。

**When to use:** 需要区分"文件改了"和"语义变了"的场景。

**Trade-offs:**
- ✅ 有效区分格式化改动 vs 语义改动
- ✅ 指纹比对 O(1)，适合大规模扫描
- ❌ 语义归一化的稳定性要求很高（需排除排序噪声等）
- ❌ 指纹算法升级需要全量重建

**Implementation approach:**
```rust
/// Source Fingerprint: 对源码内容做稳定化后哈希
pub fn source_fingerprint(content: &str) -> String {
    let normalized = normalize_whitespace(content);
    blake3::hash(normalized.as_bytes()).to_hex().to_string()
}

/// Semantic Fingerprint: 对归一化语义对象做确定性序列化后哈希
pub fn semantic_fingerprint(semantic: &SemanticObject) -> String {
    let canonical = serde_json_canonical::to_string(semantic).unwrap();
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}
```

**Confidence:** HIGH — 规范文档 04/07 已定义指纹约束。

---

## Data Flow

### Init Flow (全量初始化)

```
anrsm.yaml
    ↓
Config Loader → Config
    ↓
Scope Manager → ManagedFileList (受管文件清单 + 模块归属)
    ↓
Repo Scanner → FileSet (全量枚举)
    ↓
Language Adapter (per-file, parallel)
  → ParseResult (exports, imports, symbols)
    ↓
Semantic Extractor → SemanticObject (统一语义模型)
    ↓
Fingerprint Engine → (source_fingerprint, semantic_fingerprint)
    ↓
Mirror Generator → FileMirrors (.md) + ModuleMirrors (.md)
    ↓
Index Builder → routing_index.json + symbol_index.json + dependency_index.json
    ↓
Drift Analyzer → BaselineReport (基线一致性)
    ↓
Policy Gate → InitVerdict (覆盖率/完整性检查)
    ↓
Output: mirror/, mirror/index/, .anrsm/cache/
```

### Incremental Flow (增量更新)

```
git diff (HEAD vs cached)
    ↓
Repo Scanner → ChangedFileSet
    ↓
Scope Manager → AffectedArtifactSet
  (直接受影响 + 邻域扩展)
    ↓
Language Adapter (changed + neighbors only)
  → UpdatedParseResult
    ↓
Semantic Extractor → UpdatedSemanticObject
    ↓
Fingerprint Engine → NewFingerprints
    ↓
Drift Analyzer
  ├─ Compare old vs new fingerprints
  ├─ Classify C0/C1/C2/C3
  ├─ Determine drift_level (none/low/material/blocking)
  └─ DriftReport
    ↓
Mirror Generator → Update affected mirrors only
    ↓
Index Builder → Update affected index shards
    ↓
Policy Gate → Pass/Fail + failure codes (ANRSM-001..010)
    ↓
Output: updated mirrors + drift report + gate result
```

### CI Gate Flow (pre-commit / PR)

```
Git hook triggered (pre-commit / CI step)
    ↓
anrsm check --staged
    ↓
Repo Scanner → StagedChanges
    ↓
[Reuse Incremental Flow to compute drift]
    ↓
Policy Gate
  ├─ pre-commit:  blocking → fail, material → warn/fail (configurable)
  ├─ PR CI:       material + blocking → fail, unknown in high-risk → fail
  └─ main/release: blocking + material + invalid → fail, unknown → fail
    ↓
Exit code 0 (pass) or 1 (fail) + diagnostic output
```

### Agent Read Flow

```
Agent: "I need to modify the auth module"
    ↓
anrsm query auth --format json
    ↓
Index Builder (reads routing_index.json)
  ├─ Match keywords "auth" to artifact_ids
  ├─ Return: files, modules, symbols, risk_tags
  └─ Include: freshness_state + confidence_band
    ↓
Agent reads mirror/file/src/auth/login.ts.md
  (gets: responsibilities, contracts, side effects, risks)
    ↓
Agent makes changes
    ↓
Agent runs: anrsm check --staged
    ↓
Drift detected → Agent updates mirrors → Re-check → Commit
```

---

## Recommended Build Order

基于组件依赖关系和风险递增原则，推荐以下构建顺序：

### Phase 1: Foundation（核心类型 + 解析基础）
**Why first:** 所有后续组件都依赖这些类型。必须先冻结。

| Component | Rationale |
|-----------|-----------|
| `anrsm-core` types | 所有 crate 共享，必须先定义 |
| `anrsm-core` adapter trait | 适配器接口契约 |
| `anrsm-core` config | 配置加载，所有组件的输入 |
| `anrsm-adapters` (TS only) | 验证 trait 设计是否合理 |

### Phase 2: Scoping + Parsing（范围 + 解析流水线）
**Why second:** 验证从配置到解析结果的完整链路。

| Component | Rationale |
|-----------|-----------|
| `anrsm-scope` (Scope Manager) | 确定受管文件 |
| `anrsm-scope` (Repo Scanner) | 枚举 + Git 集成 |
| Semantic Extractor | 语义归一化（先做最小版本） |

### Phase 3: Mirror Generation（镜像生成）
**Why third:** 这是用户/Agent 看到的第一个有形产出。

| Component | Rationale |
|-----------|-----------|
| Fingerprint Engine | 双指纹计算 |
| Mirror Generator (file) | 文件镜像生成 |
| Mirror Generator (module) | 模块镜像聚合 |
| `anrsm-cli` init + build | 端到端可运行 |

### Phase 4: Drift + Gate（漂移 + 门禁）
**Why fourth:** 需要先有基线镜像才能检测漂移。

| Component | Rationale |
|-----------|-----------|
| Drift Analyzer | C0-C3 分类 + 漂移报告 |
| Policy Gate | Pass/Fail 决策 + 失败码 |
| `anrsm-cli` update + check | 增量更新 + CI 门禁 |

### Phase 5: Index + Query（索引 + 查询）
**Why fifth:** 索引是镜像的派生物，依赖镜像存在。

| Component | Rationale |
|-----------|-----------|
| Index Builder (routing) | 路由索引 |
| Index Builder (symbol) | 符号索引 |
| Index Builder (dependency) | 依赖索引 |
| `anrsm-cli` query | Agent 查询接口 |

### Phase 6: Knowledge + Polish（知识层 + 打磨）
**Why last:** 知识层是最高层级的派生，依赖所有下层。

| Component | Rationale |
|-----------|-----------|
| Knowledge Compiler | 知识文档生成 |
| 更多语言适配器 | 按需扩展 |
| CLI 输出优化 | JSON schema 稳定化 |

---

## Scaling Considerations

| Scale | Architecture Approach |
|-------|----------------------|
| 100 files | 单线程即可，全量扫描 < 1s |
| 1,000 files | 并行解析（rayon），增量模式默认 |
| 10,000 files | 必须增量模式；Tree-sitter 增量解析优势明显；索引分片 |
| 100,000 files | 需要考虑索引缓存策略、并行 fingerprint 计算、内存分批处理 |

### Scaling Priorities

1. **First bottleneck:** 解析速度 → 用 Tree-sitter（O(changed)）+ rayon 并行
2. **Second bottleneck:** 磁盘 I/O（大量 .md 写入）→ 增量模式只写受影响文件
3. **Third bottleneck:** 索引构建 → 分片索引，只更新受影响分片

---

## Anti-Patterns

### Anti-Pattern 1: Regex-Only Parsing（纯正则解析）

**What people do:** 为了快速支持新语言，用正则提取符号和导入。
**Why it's wrong:** 正则无法处理嵌套结构、模板字符串、泛型等复杂语法。置信度极低，且维护成本随语言复杂度指数增长。
**Do this instead:** 用 Tree-sitter grammar。如果 grammar 不存在，宁可标记 `unsupported` 也不要假装能解析。规范 04 明确要求："仅基于正则的适配器 MUST 标记低置信度"。

### Anti-Pattern 2: Dynamic Plugin Loading（动态插件加载）

**What people do:** 用 `libloading` 加载 .so/.dylib 实现运行时插件扩展。
**Why it's wrong:** Rust 没有稳定的 ABI。不同编译器版本的 .so 可能 crash。安全风险高（2026 年 ClawHub 41.7% 插件含恶意代码）。
**Do this instead:** Trait-based 编译时注册。需要新语言 → 加代码 → 重编译。ANRSM 是 CLI 工具，不是平台，不需要运行时插件。

### Anti-Pattern 3: Full Rebuild on Every Run（每次都全量重建）

**What people do:** 不做增量检测，每次都扫描全部文件、重新解析、重新生成。
**Why it's wrong:** 在 1000+ 文件的仓库中，全量扫描会拖垮 pre-commit 体验（>10s），用户会绕过门禁。
**Do this instead:** 增量模式是默认路径。只在适配器升级、指纹算法变更、模块规则变化时触发全量重建。

### Anti-Pattern 4: Prose-Heavy Mirror Generation（文采优先的镜像生成）

**What people do:** 用 LLM 生成流畅自然语言的镜像文档。
**Why it's wrong:** 每次运行可能生成不同措辞 → semantic_fingerprint 不稳定 → 伪漂移 → 门禁不可信。
**Do this instead:** 先事实、后表达。模板固定段落骨架，压缩规则稳定术语和句式。"稳定性优先于文采"。

---

## Integration Points

### Git Integration

| Integration | Pattern | Notes |
|-------------|---------|------|
| Git diff (变更检测) | `git2` crate 或 shell-out `git diff` | `git2` 更快但需注意跨平台编译；shell-out 更简单 |
| Git hooks (pre-commit) | 生成 hook 脚本或由用户配置 | `anrsm install-hook` 子命令 |
| Git blame (可选) | 用于知识层的变更追踪 | Phase 6+ |

### CI Integration

| Platform | Integration Pattern | Notes |
|----------|---------------------|------|
| GitHub Actions | `anrsm check --staged` as step | 退出码 1 = fail，天然集成 |
| GitLab CI | 同上 | `script: anrsm check` |
| Pre-commit framework | 提供 `.pre-commit-hooks.yaml` | 适配 pre-commit 生态 |

### Agent Integration

| Agent | Integration Pattern | Notes |
|-------|---------------------|------|
| Claude Code | CLI 调用 `anrsm query` + `anrsm check` | JSON 输出，Agent 可直接解析 |
| Cursor/Copilot | 同上 | CLI 是通用接口 |
| 自定义 Agent | JSON schema 约定 | 保证输出结构稳定 |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Core ↔ Adapters | Trait (`LanguageAdapter`) | 松耦合，可独立测试 |
| Scope ↔ Scanner | `ManagedFileList` 结构体 | 确定性输入输出 |
| Extractor ↔ Fingerprint | `SemanticObject` 结构体 | 必须可确定性序列化 |
| Mirror ↔ Drift | `Fingerprint` + `DriftReport` | 漂移驱动镜像更新 |
| Drift ↔ Policy | `DriftReport` → `PolicyVerdict` | 策略可配置 |

---

## Sources

- **规范文档:** `docs/04_Architecture_and_Component_Model.md`（规范性，P0）
- **规范文档:** `docs/05_Information_Model_and_Repository_Layout.md`（规范性，P0）
- **规范文档:** `docs/07_Drift_Detection_Synchronization_and_CI.md`（规范性，P0）
- **参考项目:** [codegraph](https://github.com/optave/codegraph) — Rust + Tree-sitter 多语言代码分析，CI 门禁，增量重建（2026-03，HIGH confidence）
- **参考项目:** [NekoCode](https://github.com/moe-charm/nekocode-rust) — Rust + Tree-sitter 多语言分析，GitHub Actions 集成（2025-08，HIGH confidence）
- **架构模式:** [ZeroClaw trait-driven architecture](https://zeroclaws.io/blog/trait-driven-architecture-extensible-agents/) — 编译时 trait 方案 vs 动态插件安全对比（2026-02，HIGH confidence）
- **架构模式:** [Litho plugin architecture](https://dev.to/sopaco/from-rust-to-multi-language-lithos-plugin-based-extensible-architecture-design-b8b) — `LanguageProcessor` trait + PluginRegistry 模式（2025-10，MEDIUM confidence）
- **解析技术:** [Tree-sitter incremental parsing](https://dasroot.net/posts/2026/02/incremental-parsing-tree-sitter-code-analysis/) — 增量解析原理和 VS Code/Zed 集成案例（2026-02，HIGH confidence）
- **解析技术:** [tree-sitter Rust docs](https://docs.rs/tree-sitter/latest/tree_sitter/) — crate v0.24 API（HIGH confidence）
- **Rust 生态:** [State of Rust 2025](https://blog.jetbrains.com/rust/2026/02/11/state-of-rust-2025/) — 生态成熟度（2026-02，HIGH confidence）

---

*Architecture research for: ANRSM — AI Native Repo Semantic Mirror*
*Researched: 2026-03-21*
*Researcher: GSD Research Phase 6*
