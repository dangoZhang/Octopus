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
  src/main.rs     CLI、本地 app server、provider、preflight、repair、release gate

src/octopus/
  *.py            Python SDK/原型层：Need、Feed、Harness、Memory、LLM planner

tentacles/
  */manifest.json  触手声明：brain prompt、tool meta、runtime、权限、演化面
  */tools/*        可编辑运行时工具

docs/
  *.md / *.html    英文文档、本地 app、pixel pet、release gate 记录
  zh/              中文文档入口

cowork/
  structure.md     跨线程结构记录和当前 gap 记录
```

## 当前能力

- `bootstrap`：初始化状态，安装 seed tentacles，触发 heartbeat。
- `first-run`：跑一次完整本地闭环并生成 preflight 证据。
- `brain`：导出或运行 clean-brain 会话，支持 intent、brief、clarify、agenda、deliberate、reflect、memory、focus、council、synthesize、rewrite。
- `needs`：维护可审查 Need Queue，不直接执行工具。
- `think`：只看触手计划，不执行工具。
- `need`：把 Need 交给 harness，执行触手并记录 Feed trace。
- `feedback`：人工评分 Feed trace，更新 route learning 和 pet 状态。
- `repair`：让 harness-repair-agent 诊断状态、trace、check、adapter，并写可审查 repair bundle。
- `evolve` / `beat`：从失败 trace、check history、repair outcome 生成 harness 演化建议。
- `provider`：生成、保存、检查 Codex/OpenAI-compatible/local/LiteLLM 配置。
- `start`：启动本地 HTML app，通过受限本地 API 运行 Octopus 子命令。
- `preflight`：检查 `0.1.0` 发布门槛。

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
- Python 测试依赖需要整理：`pytest` 在当前机器缺失，但 `PYTHONPATH=src python3 -m unittest discover -s tests` 可通过。
