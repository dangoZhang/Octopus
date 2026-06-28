# 中文快速开始

## 安装

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start --open
```

`octopus start --open` 会打开 `http://127.0.0.1:8765/app.html`。无浏览器环境用：

```bash
octopus start
```

它会准备 `.octopus/` 状态、安装可编辑 seed tentacles、触发三类 heartbeat，并加载 `.octopus/llm.env`。

## 从源码运行

```bash
git clone https://github.com/dangoZhang/Octopus
cd Octopus
cargo run -p octopus-core --bin octopus -- start
```

本地安装：

```bash
cargo install --path crates/octopus-core --bin octopus --force
octopus start --open
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

开发者/内部演示手动版。产品用户路径保留 `first-run` 和 Goal 修改：

```bash
octopus bootstrap
octopus goal set --constraint "keep tools outside the brain" "make this repo easier to use"
octopus brain --session "what should the brain ask next?"
octopus brain --focus verify --save "what proof matters?"
octopus need observe README.md
octopus feedback latest satisfied "useful evidence"
octopus beat 200
octopus pet
```

## 可选 LLM

Codex CLI OAuth：

```bash
codex login
octopus provider save codex
source .octopus/llm.env
octopus provider check
```

OpenAI-compatible API：

```bash
octopus provider save openai
source .octopus/llm.env
export OPENAI_API_KEY=...
octopus provider check
```

LiteLLM 或本地 OpenAI-compatible gateway：

```bash
octopus provider save litellm
source .octopus/llm.env
octopus provider check
```

Live clean-brain 示例：

```bash
octopus first-run --live "make this repo easier to use"
octopus brain --agenda --save "what matters next?"
octopus brain --focus compare --save "which path should the brain compare?"
octopus brain --council --models OCTOPUS_LLM --save "ask clean brains"
```

`octopus provider status` 会先显示四类覆盖：Goal chat、clean brain、tentacle planning、harness evolution。Codex OAuth 和本地模型可以没有 API key；真实可用性用 `octopus provider check` 或 `octopus preflight --live` 证明。

发布测试前生成 provider 矩阵记录：

```bash
octopus provider matrix
OCTOPUS_LOCAL_OK=1 octopus provider matrix run
octopus provider matrix check
```

`matrix run` 只调用显式启用的目标，例如 `OCTOPUS_LOCAL_OK=1`；跳过或失败的目标会继续挡住发布门。

## Harness 演化

```bash
octopus install swe-agent
octopus need observe README.md
octopus feedback latest partial "feed needs sharper file evidence"
octopus beat 200
octopus repair .
```

改动会先落到 `.octopus/` 下的可审查计划、patch、repair bundle 和评分记录。

## 0.1.0 发布门槛

```bash
tmp=$(mktemp -d)
octopus --state "$tmp/state.json" first-run "preflight local evidence"
octopus --state "$tmp/state.json" preflight
octopus --state "$tmp/state.json" preflight record "$tmp/real-machine-record.md"
octopus --state "$tmp/state.json" preflight record check "$tmp/real-machine-record.md"
octopus --state "$tmp/state.json" preflight record append "$tmp/real-machine-record.md" docs/real-machine-test.md
```

`0.1.0` 需要本地闭环、live provider、GitHub OAuth/PR 路径和真实机器记录都通过。
