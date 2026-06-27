# Octopus 🐙 中文 README

默认入口仍是 [README.md](README.md)。这份是中文阅读版。

## Intro

Clean brain. Independent tentacles.

生物章鱼不会把所有控制信号都压进一个大脑。
它们的腕足有局部神经系统。
行为来自意图、局部控制和反馈。
心跳服务于供血和环境适应。

Octopus 把这个想法带到 agent。
大脑只拥有目标和需求。
带智能的触手拥有实现。
心跳驱动自发动作，也驱动 harness 自我演化。

```text
Goal -> Brain -> Need -> Tentacle Intelligence -> Action -> Feed -> Brain
Heartbeat -> Action Data -> Tentacle harness change
```

## 安装

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start --open
```

`start --open` 会准备本地状态、种子触手、心跳和 HTML app，然后打开 `http://127.0.0.1:8765/app.html`。

更新：

```bash
octopus update
octopus update --run
```

## 第一次闭环

```bash
octopus first-run "make this repo easier to use"
```

它会设置干净 Goal，安装 seed tentacles，跑一次安全 observe Feed，记录反馈，触发 heartbeat，并返回 Doctor 与 Preflight 证据。

## 现在能做什么

- 干净大脑：`Goal + Mem + Need + Feed`。
- 触手执行：`Need + Tool + Action + Tool + Action -> Feed`。
- Seed tentacles：SWE、computer-use、repo-maintainer、harness-repair、bash-only、json-feed、visual。
- Provider：Codex CLI OAuth、OpenAI-compatible API、本地模型、LiteLLM 等。
- 本地 app：用户只改 Goal；provider、preflight、pet、trace、repair 和 harness 状态作为观察面。
- Harness evolution：基于 Feed trace、check history、repair outcome 生成可审查的改动计划。

## 中文文档

- [中文快速开始](docs/zh/quickstart.md)
- [中文架构说明](docs/zh/architecture.md)
- [英文架构](docs/architecture.md)
- [产品 gap log](docs/product-gap.md)

当前版本线是 `0.0.16`。`0.1.0` 需要完整真实机器记录、live provider gate、GitHub OAuth/PR 路径和发布前 preflight 全部通过。
