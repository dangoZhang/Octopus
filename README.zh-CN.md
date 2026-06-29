# Octopus 🐙 中文 README

让主模型专注目标，让本地触手处理工具过程。

[安装](#快速安装和启动) · [产品 Demo](https://dangozhang.github.io/Octopus/demo.html) · [网页教程](https://dangozhang.github.io/Octopus/docs.html) · [试用 App](https://dangozhang.github.io/Octopus/app.html?demo=hello) · [English](README.md)

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

**章鱼会变色。**

像素章鱼是只读桌面观察器。它读取 `.octopus/state.json`：Need 出现时头部变色，触手执行时挥动，code-as-harness 运行时吐泡泡，蓝色表示进化，红色表示卡住，Feed 返回后变绿。

## 快速安装和启动

```bash
curl -fsSL https://dangozhang.github.io/Octopus/install.sh | sh
octopus --version
octopus start --open
```

跑第一次本地闭环：

```bash
octopus first-run "make this repo easier to use"
octopus chat "prefer one small evidence-backed improvement"
octopus pet desktop
```

你应该看到本地 app：`http://127.0.0.1:8765/app.html`，一个 `.octopus/state.json` 文件，一段 Feed summary，以及一个观察真实状态的原生桌面章鱼。

直接从 GitHub 安装：

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start --check
octopus start --open
```

## 模型接入

Codex 登录：

```bash
octopus provider save codex
source .octopus/llm.env
octopus provider check
octopus first-run --live "make this repo easier to use"
```

API key：

```bash
octopus provider save openai
source .octopus/llm.env
export OPENAI_API_KEY=...
octopus provider check
```

本地 OpenAI 兼容模型、网关或路由服务也走同一套 provider 配置。

## 当前形态

Octopus 已经是一个本地产品：app、pet、docs、recipes、installer page、provider checks、seed tentacles 和 release evidence 都在同一条启动路径里。

发布证据记录在 [docs/real-machine-test.md](docs/real-machine-test.md)。v0.1.0 已记录 installed binary、本地 app、provider matrix、最小 SWE/Claw/Wild benchmark 证据。当前 `0.1.x` release gate 也会记录 field pool 证据。

GitHub Pages app 用来零安装体验想法。它直接请求你填写的 endpoint；项目不代理 API key。

## 文档

- [产品 Demo](docs/demo.html)
- [网页教程](docs/docs.html)
- [5 分钟使用教学](docs/use.html)
- [Recipes](docs/recipes.html)
- [架构](docs/architecture.md)
- [领域适配 TODO](docs/field-adaptation.md)
- [产品 gap log](docs/product-gap.md)

## TODO

`0.1.x` 先把 `v0.2.0` 的领域进化基建做稳。八个 field slot 属于同一个并行矩阵，worker 只决定一次并发几个槽位。

平级槽位：`math`、`search`、`code`、`SWE`、`research`、`computer-use`、`IB`、`robotics`。
每个槽位共用同一条 Need -> Feed 链路、verifier trace、repair template 和 harness evolution 闭环。

- [x] 原生只读桌宠显示 Need 气泡、执行泡泡、Goal 展开、蓝色进化、红色卡住。
- [x] Feed trace 记录 `field_pack`、verifier 结果、轨迹摘要和 repair signal。
- [x] 领域修复代码放在 Rust kernel 外部的可编辑 harness template。
- [x] 失败轨迹可以产出可审查 harness patch，授权后 apply、rerun、score。
- [x] `mini-1/2/3` 是单个 field 内部训练阶梯，不给八个领域排序。

## License

MIT.
