# 12. MCP Server Contract

状态：Normative
版本：v1
优先级：P0

---

## 1. 目的

定义 WTCD 通过 MCP 协议暴露给 Agent 的工具契约、传输约束和服务生命周期。

## 2. 传输层

- 仅支持 stdio 传输（D-03）
- 日志输出 stderr，JSON-RPC 走 stdout
- Agent 作为 parent process 管理 MCP Server 生命周期
- Server 启动命令：`wtcd mcp`

## 3. 工具定义

### 3.1 wtcd_parse

- 功能：解析单个文件，提取语义事实
- 参数：`{ file_path: string, language?: string }`
  - `file_path`（必填）：文件路径，相对于项目根目录
  - `language`（可选）：语言适配器，支持 `auto`、`ts`、`js`，默认 `auto`
- 返回：FileResult JSON
  - `file_path`：解析文件路径
  - `confidence`：置信度 `high` | `low` | `none`
  - `exports`：导出符号列表 `{ name, kind, line }`
  - `imports`：依赖边列表 `{ source, symbols[], kind }`
  - `signatures`：函数签名列表 `{ name, parameters[], return_type }`
  - `side_effects`：副作用列表 `{ kind, target, line }`
  - `parse_time_ms`：解析耗时
  - `error_message`：错误信息（如有）
- 错误：文件不存在或无适配器时返回 confidence: none + error_message

### 3.2 wtcd_run

- 功能：扫描项目，执行全量语义分析
- 参数：`{ project_root?: string }`
  - `project_root`（可选）：项目根目录路径，默认使用服务器启动时的工作目录
- 返回：RunOutput JSON
  - `api_version`：API 版本号
  - `files[]`：FileResult 列表
  - `errors[]`：错误列表
  - `summary`：汇总统计 `{ total_files, parsed_ok, confidence_low, confidence_none, total_exports, total_imports, elapsed_ms }`

### 3.3 wtcd_check (stub)

- 功能：检测镜像漂移（Phase 3 实现）
- 参数：无
- 返回：`"not implemented"` error
- 后续实现将检测源码与镜像的一致性，返回 DriftReport

### 3.4 wtcd_mirror (stub)

- 功能：生成镜像文件（Phase 2 实现）
- 参数：无
- 返回：`"not implemented"` error
- 后续实现将根据解析结果生成语义镜像文件

## 4. 配置发现

- 首次工具调用时自动查找 `anrsm.yaml`
- 支持 `project_root` 参数显式指定
- 配置加载失败时工具返回错误信息

## 5. Agent 集成

### 5.1 Claude Code

```json
// ~/.claude.json 中添加
{
  "mcpServers": {
    "wtcd": {
      "command": "wtcd",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

或使用 CLI：
```bash
claude mcp add --transport stdio wtcd -- wtcd mcp
```

配置生成：
```bash
wtcd mcp-install claude
```

### 5.2 Cursor

```json
// .cursor/mcp.json
{
  "mcpServers": {
    "wtcd": {
      "command": "wtcd",
      "args": ["mcp"]
    }
  }
}
```

配置生成：
```bash
wtcd mcp-install cursor
```

### 5.3 通用配置

任何支持 MCP stdio 传输的 Agent 均可通过 `wtcd mcp-install --agent <name>` 获取配置片段。

## 6. 与 CLI 的关系

- MCP 和 CLI 共享 wtcd-core/wtcd-adapters/wtcd-scope（D-01）
- MCP 直接调用 Rust API，不走 CLI 子进程（D-02）
- MCP 和 CLI 功能对等（D-01）
- 新增 CLI 子命令：`wtcd mcp`、`wtcd mcp-install`

## 7. 实现细节

### 7.1 依赖

- rmcp 1.2.0（MCP 协议官方 Rust SDK）
- schemars 1.1.0（JSON Schema 生成）
- tokio 1.x（异步运行时）

### 7.2 crate 结构

```
crates/wtcd-mcp/
├── Cargo.toml
├── src/
│   ├── lib.rs          # 模块声明
│   ├── server.rs       # WtcdMcpServer + ServerHandler
│   ├── tools/
│   │   ├── mod.rs      # 工具模块声明
│   │   ├── parse.rs    # wtcd_parse 实现
│   │   ├── run.rs      # wtcd_run 实现
│   │   ├── check.rs    # wtcd_check (stub)
│   │   └── mirror.rs   # wtcd_mirror (stub)
│   └── error.rs        # 错误映射
```

### 7.3 同步-异步桥接

wtcd-core/wtcd-adapters/wtcd-scope 为同步代码。MCP Server 使用 `tokio::task::spawn_blocking()` 包装同步调用。

---

*参考决策：D-01 至 D-18（见 999.1-CONTEXT.md）*
*规范版本：v1 — 2026-03-21*
