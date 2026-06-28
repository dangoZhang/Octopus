# v0.2.0 Field Adaptation Harness

目标：让 Octopus 为不同领域任务长出合适触手，而不是人工为每个领域写死流程。

v0.2.0 的核心不是补一堆工具列表，而是建设一套领域适配基建：

```text
Goal -> field signal -> Need -> tentacle selection/generation
     -> action trajectory -> Feed -> score/error
     -> harness repair/evolution -> reuse
```

## Field Packs

每个领域需要一个 Field Pack，描述任务形状，而不是固定实现：

- task schema：领域任务怎样被描述。
- capability hints：需要观察、搜索、计算、执行、引用、仿真、屏幕操作等哪类能力。
- permission boundary：哪些动作需要授权，哪些动作只能观察。
- verifier：结果如何检查，失败如何归因。
- trajectory labels：哪些轨迹片段值得进入学习和复用。

首批领域：

- math：证明、计算、符号推理、数值校验。
- search：检索、去重、引用、可信度判断。
- code：局部代码读写改测。
- SWE：跨文件 issue 修复、回归测试、benchmark 对齐。
- research：论文、资料综合、claim grounding、引用。
- computer-use：屏幕、浏览器、文件、系统动作闭环。
- IB work：金融分析、表格、memo、market data、合规边界。
- robotics：仿真优先的感知、动作计划、执行反馈。

## Trajectory Store

Octopus 需要把每次任务当成可学习轨迹：

- Goal and Need
- selected tentacle and field pack
- tool metadata visible to the tentacle
- action sequence
- observations
- errors
- Feed
- verifier result
- repair attempt
- reused harness patch

轨迹是触手进化的主要数据源。

## Evolution Loop

失败后优先让 Octopus 修复触手基建：

1. 读取失败轨迹和 verifier 结果。
2. 判断错误来自 prompt、meta、tool code、permission、router、verifier 还是环境。
3. 生成 harness patch 候选。
4. 复跑同类 mini task。
5. 成功后沉淀为 field-specific route signal 或 tentacle update。
6. 失败超过阈值时交给人修基建。

人类兜底的重点是修复自修复能力，而不是长期手写领域工具。

## Router

Router 学习谁擅长供应什么 Feed：

- field match score
- tentacle historical success
- verifier pass rate
- error recovery rate
- permission cost
- latency and tool cost

Need 稳定，实现可替换。相同 Need 可以路由到不同触手组合。

## Minimal v0.2.0 Gate

每个领域先放一个最小真实任务：

- math：一个可校验计算或证明任务。
- search：一个需要引用和去重的检索任务。
- code：一个小 repo 修改和测试任务。
- SWE：一个最小 issue 修复任务。
- research：一个带来源的 claim synthesis 任务。
- computer-use：一个浏览器或桌面观察动作任务。
- IB work：一个小型表格分析和 memo 输出任务。
- robotics：一个 simulator-only 动作计划任务。

通过条件：

- 能识别领域。
- 能选或生成触手计划。
- 能记录轨迹。
- 能得到 verifier 结果。
- 失败能产出 harness 改进候选。
- 至少部分领域能复跑后变好。

## Files To Add Later

```text
field-packs/
  math/
  search/
  code/
  swe/
  research/
  computer-use/
  ib/
  robotics/

.octopus/trajectories/
.octopus/field-scores/
```

稳定 Rust kernel 只保留 Field Pack 读取、轨迹记录、路由信号、Need/Feed 传输和 heartbeat。领域实现继续留在可进化 harness 中。
