# 中文快速开始

先看 [产品教学](tutorial.html) 和 [使用场景](recipes.html)，再照这里跑本地命令。

一行安装：

```bash
curl -fsSL https://dangozhang.github.io/Octopus/install.sh | sh
```

## 安装

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus download
octopus start --check
```

`octopus start --check` 会准备状态并写入 `.octopus/local-app-run.json`，不会占住终端。

需要打开浏览器 app 时再运行：

```bash
octopus start --open
```

`octopus start --open` 会打开 `http://127.0.0.1:8765/app.html` 并持续运行本地 app server。无浏览器环境用 `octopus start`。
`start` 会准备 `.octopus/` 状态、安装可编辑 seed tentacles、触发三类 heartbeat，并加载 `.octopus/llm.env`。
`octopus download` 会打印当前版本的安装命令、更新命令、源码包和文档入口。

## 从源码运行

```bash
git clone https://github.com/dangoZhang/Octopus
cd Octopus
cargo run -p octopus-core --bin octopus -- start
```

本地安装：

```bash
cargo install --path crates/octopus-core --bin octopus --force
octopus start --check
```

## 第一次闭环

```bash
octopus first-run "make this repo easier to use"
```

这个命令会：

- 设置 clean Goal。
- 安装 seed tentacles。
- 运行一次安全 observe Feed。
- 评分最新 Feed trace。
- 触发 heartbeat。
- 返回 Doctor 和 Preflight 证据。

之后只改 Goal，其他状态用于观察：

```bash
octopus chat "make this repo easier to use"
octopus goal refine "prefer small reviewable changes"
octopus pet desktop
octopus start --open
```

Need、Feed、路由、provider、repair 和 harness evolution 都属于 agent 内部工作。app 可以展示这些状态，但用户输入只改变 Goal。

## 领域自进化预览

从并列领域池里跑一个 worker 槽位，观察同一条 Need -> Feed 链路：

```bash
octopus evolve parallel --workers 1 --open "advance the peer field objectives toward v0.2.0"
octopus fields summary
```

八个领域留在同一个并行池里。`--workers 1` 只打开一个执行槽；更大的值会从同一个池里打开更多并发槽。每个槽都会写入自己的 field Need，运行可编辑的 `field-mini-task` harness，记录 Feed 和 verifier signal，桌宠只负责观察状态。

## 可选 LLM

Octopus 可以接 Codex 登录、API key、本地 OpenAI-compatible server 或 LiteLLM 类网关。这是运行时管道，不应该进入 Need 文本。

Codex CLI OAuth：

```bash
codex login
export OCTOPUS_LLM_BACKEND=codex
export OCTOPUS_LLM_CODEX_COMMAND=codex
```

OpenAI-compatible API、router 或本地服务：

```bash
export OCTOPUS_LLM_BACKEND=openai-compatible
export OCTOPUS_LLM_MODEL=gpt-4.1-mini
export OCTOPUS_LLM_BASE_URL=https://api.openai.com/v1
export OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY"
```

本地示例：

```bash
export OCTOPUS_LLM_BACKEND=openai-compatible
export OCTOPUS_LLM_MODEL=local-model
export OCTOPUS_LLM_BASE_URL=http://localhost:1234/v1
export OCTOPUS_LLM_API_KEY=
```

如果通过 `octopus start` 启动，把同样的环境变量放进 `.octopus/llm.env`。先用只读命令检查准备情况：

```bash
octopus providers
octopus provider status
```

Live clean-brain 示例：

```bash
octopus first-run --live "make this repo easier to use"
octopus chat "tighten the current objective with live model"
octopus goal refine "prefer evidence from the latest Feed"
```

`octopus provider status` 会先显示四类覆盖：Goal chat、clean brain、tentacle planning、harness evolution。Codex OAuth 和本地模型可以没有 API key；真实可用性用 `octopus provider check` 或 `octopus preflight --live` 证明。

发布测试前生成 provider 矩阵记录。它也会准备缺失的 `.octopus/providers/*.env` 文件：

```bash
octopus provider matrix
OCTOPUS_LOCAL_OK=1 octopus provider matrix run
octopus provider matrix check
```

`matrix run` 只调用显式启用的目标，例如 `OCTOPUS_LOCAL_OK=1`；跳过或失败的目标会继续挡住发布门。

## Harness 演化

```bash
octopus first-run "让这个仓库更容易使用"
octopus chat "根据最新 Feed 收紧安装目标"
octopus beat 200
octopus report
```

改动会先落到 `.octopus/` 下的可审查计划、patch、repair bundle 和评分记录。

## 发布门槛

```bash
tmp=$(mktemp -d)
octopus --state "$tmp/state.json" first-run "preflight local evidence"
octopus --state "$tmp/state.json" preflight
octopus --state "$tmp/state.json" fields summary
octopus --state "$tmp/state.json" status
octopus benchmark record
# 填写 .octopus/benchmark-evidence.md：SWE/Claw/Wild case id、命令、通过结果、摘要和产物路径。
octopus benchmark check
octopus --state "$tmp/state.json" preflight record "$tmp/real-machine-record.md"
octopus --state "$tmp/state.json" preflight record check "$tmp/real-machine-record.md"
octopus --state "$tmp/state.json" preflight record append "$tmp/real-machine-record.md" docs/real-machine-test.md
```

旧的 `0.1.0` release 产物已撤回。`v0.2.0` 是第一条可用公开版本线；本地闭环、桌宠观察、真实 LLM 驱动的 harness 进化、field pool、live provider、benchmark、GitHub OAuth/PR 路径和真实机器记录都属于发布门禁。
