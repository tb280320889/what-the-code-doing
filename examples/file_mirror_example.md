---
anrsm_version: 1
artifact_type: file_mirror
artifact_id: file_mirror:src/auth/login.ts
source_path: src/auth/login.ts
source_language: typescript
module_id: auth
source_commit: 9f8c2ab
source_fingerprint: sha256:9eb9ce7bd36e4982fa0b04d647d2f0e2f8d1b9f176ef2f2ce20b2fd8b4d695e9
semantic_fingerprint: sha256:7a2d49a0f317a9dc8aa8650e9d0ab5feff9cb5fb7461a6f0b03f4a2f53deab40
freshness_state: fresh
confidence_band: high
generator_name: anrsm
generator_version: 0.1.0
generated_at: 2026-03-20T10:30:00Z
exports:
  - login
dependencies:
  - src/core/http/client.ts
  - src/core/auth/session.ts
risk_tags:
  - auth
  - external_api
  - session
---

## 职责
处理账号登录主流程，负责凭证校验、风控验证、鉴权服务调用和本地会话建立。

## 对外契约
对外暴露 `login(credentials)`；成功时返回会话对象，失败时抛出鉴权或输入验证错误。

## 输入与前置条件
调用方必须传入标准化凭证对象；当风控开关开启时必须提供验证码令牌；本文件默认依赖可用的 HTTP Client 和会话存储。

## 状态与控制流
主流程为参数校验 -> 验证码检查 -> 请求鉴权服务 -> 建立本地会话 -> 返回会话。鉴权失败和风控失败都不会创建会话。

## 副作用与外部依赖
读取风控配置；调用鉴权 API；写入本地会话；记录失败日志；可能触发失败指标上报。

## 关键不变量与风险
不得在鉴权失败时创建会话；验证码验证与鉴权调用必须共享同一风控上下文；错误分类影响前端提示与审计。

## 变更影响
会影响登录页提交流程、错误展示、会话恢复逻辑以及相关契约测试和端到端测试。

## 何时必须展开源码
当修改重试策略、错误映射、会话持久化细节、风控上下文或并发登录处理时，必须展开源码与调用方。
