# Octopus 🐙 中文 README

让主模型专注目标，让本地触手处理工具过程。

[安装](#快速安装和启动) · [本地 App](https://dangozhang.github.io/Octopus/app.html) · [网页教程](https://dangozhang.github.io/Octopus/docs.html) · [English](README.md)

章鱼把控制分散在身体里。中枢给出方向，腕足在离环境最近的地方完成大量感知和调整。

Octopus 把这个结构放进 agent。主模型保留较小的目标上下文；触手靠近工具，处理嘈杂步骤，再把一段可用结果带回来。

这就是产品核心：更干净的大脑，更聪明的工具，以及一个可以进化但不污染主上下文的 harness。

```text
Goal -> Brain -> Need -> Tentacle -> Tool work -> Feed -> Brain
Heartbeat -> run data -> memory and harness updates
```

## 为什么是 Octopus

**工具成为局部神经系统。**

很多 agent 把工具当被动调用。Octopus 把每条工具流程当成本地工作单元：它可以观察环境，选择下一步，检查结果，再返回紧凑 Feed。

**需求和实现分开。**

大脑只说它需要什么。shell 语法、浏览器步骤、repo 命令、provider 配置都留给触手。

这种分离是 runtime 里的一级设计：Goal、Need、Feed、触手执行是不同表面，不只是一段 prompt 约定。

**触手可改，大脑保持干净。**

种子触手放在 `tentacles/`。prompt、manifest、tools、repair policy 可以被检查和修改，而核心 Goal -> Need -> Feed 链路保持稳定。

**领域是并列池。**

`v0.2.0` 的目标是让 Octopus 适应 math、search、code、SWE、research、computer-use、IB work、robotics。八个领域留在同一个 Goal pool 里；`--workers n` 只改变同时打开几个执行槽，不把领域变成队列。

`v0.2.0` 之后，`0.2.x` 会继续为 `v0.3.0` 预进化更多可安装 field pack。第一批扩展是写作和翻译。

**章鱼会变色。**

像素章鱼是只读桌面观察器。它读取 `.octopus/state.json`：Need 出现时头部变色，触手执行时挥动，code-as-harness 运行时吐泡泡，蓝色表示进化，红色表示卡住，Feed 返回后变绿。

## 快速安装和启动

当前状态：`v0.2.0`。旧的 `v0.1.0` release 产物已删除；这是第一条可用公开版本线，已带桌宠观察和真实 harness 进化证据。

```bash
curl -fsSL https://dangozhang.github.io/Octopus/install.sh | sh
octopus --version
```

跑第一次本地 Goal/Need/Feed 闭环，并打开 App：

```bash
octopus first-run "make this repo easier to use"
octopus start --open
```

你应该看到 `.octopus/state.json`、一段 Feed summary，以及本地 Goal/Need/Feed app：`http://127.0.0.1:8765/app.html`。`start --open` 会持续运行本地 app server；只需要证据时用 `start --check`。

直接从 GitHub 安装：

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus first-run "make this repo easier to use"
octopus start --open
```

Live 模型是可选项。Octopus 可以接 Codex 登录、API key、本地 OpenAI 兼容服务或网关路由，但 provider setup 只是运行时管道。用户主路径仍然只改 Goal。模型设置见 [快速开始](docs/zh/quickstart.md)。

## License

MIT.
