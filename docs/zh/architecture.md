# 中文架构说明

## 核心边界

Octopus 把 agent 拆成两个上下文：

```text
Clean brain: Goal + Mem + Need + Feed
Tentacle: Need + Tool + Action + Tool + Action -> Feed
```

大脑只表达目标、记忆、需求和反馈。
触手负责计划、工具选择、执行、证据压缩和可演化 harness。

## 文件结构

```text
crates/octopus-core/
  src/lib.rs      Rust kernel：合同、状态、路由、Feed、heartbeat、tentacle 执行
  src/main.rs     CLI 分发、provider、preflight、repair、product report
  src/app_bridge.rs  本地 app server、HTTP/SSE、输入策略、拒绝响应、provider env overlay
  src/core_boundary.rs  稳定 Rust / 可变 harness 边界诊断
  src/release_gate.rs  preflight check、真实机器记录解析、release gate 脚本命令

tentacles/
  profile-registry/default.json  seed profile 注册表：prompt、tool meta、check、权限、演化策略
  */manifest.json  触手声明：brain prompt、tool meta、runtime、权限、演化面
  */tools/*        可编辑运行时工具

docs/
  *.md / *.html    英文文档、本地 app、pixel pet、release gate 记录
  zh/              中文文档入口

structure.md       正式仓库结构和能力记录
```

```text
cowork/
  structure.md     被忽略的跨线程同步稿
```

## 当前能力

产品面只允许用户写入 Goal：`chat`、`goal set/refine`、`brain --goal` 和 `first-run` 的目标输入。`download`、`doctor`、`report`、`preflight` 属于观察面。Need、Feed、feedback、repair、evolve、install、check、provider 写入由 agent loop 或开发者流程驱动。
本地 app bridge 已拆到 `app_bridge.rs`，包括 HTTP/SSE、静态 app fallback、命令流式执行、输入策略和 `bridge_goal_surface` preflight 证据。
`report` 和 `preflight` 会显示稳定 Rust 层、产品 app 层和可变 code-as-harness 层，避免触手/feed 实现重新混进核心。

- `bootstrap`：初始化状态，安装 seed tentacles，触发 heartbeat。
- `first-run`：跑一次完整本地闭环并生成 preflight 证据。
- `brain`：导出或运行 clean-brain 会话，支持 intent、brief、clarify、agenda、deliberate、reflect、memory、focus、council、synthesize、rewrite。
- `needs`：维护可审查 Need Queue，不直接执行工具。
- `think`：只看触手计划，不执行工具。
- `need`：把 Need 交给 harness，执行触手并记录 Feed trace。
- `feedback`：人工评分 Feed trace，更新 route learning 和 pet 状态。
- `repair`：让 harness-repair-agent 诊断状态、trace、check、adapter，并写可审查 repair bundle。
- `evolve` / `beat`：从失败 trace、check history、repair outcome 生成 harness 演化建议。
- `provider`：生成、保存、检查 Codex/OpenAI-compatible/local/LiteLLM 配置，同一套 env 可供 clean brain、触手规划和 harness evolution 使用；`provider status` 会先显示 Goal chat、clean brain、tentacle planning、harness evolution 四类覆盖，避免把 Codex OAuth 或本地模型误判成缺少 API key。
- `start`：启动本地 HTML app，通过受限本地 API 运行 Octopus 子命令。
- `preflight`：检查 `0.1.0` 发布门槛，并显示必需/可选通过数和必需阻塞项。

## Seed tentacles

- `swe-agent`：读、改、patch、测试 repo。
- `computer-use-agent`：桌面、浏览器、剪贴板、MCP 和 shell adapter。
- `repo-maintainer`：repo 检查、PR draft、GitHub 发布路径。
- `harness-repair-agent`：harness 诊断、repair session、outcome memory。
- `bash-only`：透明 write-and-run harness。
- `json-feed`：`octopus-json-v1` Python runtime seed。
- `visual`：pixel pet 状态层。

## 0.1.0 缺口

- 需要对当前 head 追加真实机器记录。
- 需要 live provider gate 通过。
- 需要 GitHub OAuth grant 和 PR dry-run/publish 路径证据。
- 需要发布前 `preflight --live` 全部 required check 通过。
- 旧 Python SDK 已移除；Python 继续作为 tentacle runtime 存在，例如 `json-feed` 和 repair tools。
- Seed profiles 已从 Rust kernel 源码移到 `tentacles/profile-registry/default.json`；启动会写出 `.octopus/profile-registry/default.json`，也可用 `OCTOPUS_PROFILE_REGISTRY` 指向其他 registry。Registry 属于 developer/harness 数据面；app 用户写入口只保留 Goal。
- Doctor、Product Report 和 Preflight 会显示 registry 来源、路径、解析状态和 profile 数量。
