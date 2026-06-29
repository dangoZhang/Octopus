# Field Adaptation TODO

Target: `v0.2.0`.

当前 `0.1.x` 只做基建：让 Octopus 识别任务领域、保存轨迹、记录验证结果，并把失败交给 harness evolution。具体触手功能代码优先由 Octopus 通过轨迹自己迭代；人类只在它无法自修复时补基建。

目标：让 Octopus 在八个同级 field slot 里长出合适触手，同时保持 Brain 只表达 Need。`--workers 1` 只限制当前执行槽数量，Goal 层始终保留同一组并行领域。

八个 field 是同级槽位。`mini-1/2/3` 是单个 field 内部的训练阶梯，不给 math、search、code、SWE、research、computer-use、IB work、robotics 排序。

```text
Goal -> field signal -> Need -> tentacle selection/generation
     -> action trajectory -> Feed -> score/error
     -> harness repair/evolution -> reuse
```

## Field Packs

每个领域需要一个 Field Pack，描述任务形状、权限、验证和学习信号：

- task schema：领域任务怎样被描述。
- capability hints：需要观察、搜索、计算、执行、引用、仿真、屏幕操作等哪类能力。
- permission boundary：哪些动作需要授权，哪些动作只能观察。
- verifier：结果如何检查，失败如何归因。
- trajectory labels：哪些轨迹片段值得进入学习和复用。

并行 field matrix：

| Field | Task shape |
| --- | --- |
| math | 证明、计算、符号推理、数值校验 |
| search | 检索、去重、引用、可信度判断 |
| code | 局部代码读写改测 |
| SWE | 跨文件 issue 修复、回归测试、benchmark 对齐 |
| research | 论文、资料综合、claim grounding、引用 |
| computer-use | 屏幕、浏览器、文件、系统动作闭环 |
| IB work | 金融分析、表格、memo、market data、合规边界 |
| robotics | 仿真优先的感知、动作计划、执行反馈 |

这些 pack 只写 task shape，避免把工具流程写死。Octopus 后续从 pack、轨迹和 verifier 结果里选择或修复触手。

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

当前核心已经会加载 `field-packs/`，为 Need 匹配 `field_pack`，把 mini task 和 expected Feed 注入 Need context，并把选择写入 Feed trace。这样 Octopus 后续可以按领域读取失败轨迹，生成 harness patch 候选。

## Evolution Loop

失败后优先让 Octopus 修复触手基建：

1. 读取失败轨迹和 verifier 结果。
2. 判断错误来自 prompt、meta、tool code、permission、router、verifier 还是环境。
3. 生成 harness patch 候选。
4. 复跑同类 mini task。
5. 成功后沉淀为 field-specific route signal 或 tentacle update。
6. 失败超过阈值时交给人修基建。

人类兜底的重点是修复自修复能力，避免长期手写领域工具。

## Router

Router 学习谁擅长供应什么 Feed：

- field match score
- tentacle historical success
- verifier pass rate
- error recovery rate
- permission cost
- latency and tool cost

Need 稳定，实现可替换。相同 Need 可以路由到不同触手组合。

## TODO

- [x] 加入 math、search、code、SWE、research、computer-use、IB work、robotics 的 Field Pack。
- [x] 核心加载 `field-packs/`。
- [x] `octopus fields` 可检查和匹配领域。
- [x] Need context 和 Feed trace 写入 `field_pack`。
- [x] 原生桌宠只读观察 `.octopus/state.json`，可显示 Need、action、Feed、evolution、blocked。
- [x] 记录 verifier result：pass、partial、fail、error category、artifact 字段。
- [x] 自动运行 0.1.x sampled field slot：parallel Goal pool -> Need -> Feed -> trace -> verifier signal。
- [x] verifier 未通过时留在当前领域，并把 `status.next_action` 指向 harness repair/evolution。
- [x] 从 Field Pack 注入 mini task 和 expected Feed 到 Need/Feed trace。
- [x] 加入 `field-mini-task` 触手基座：field mini task 先进入可进化执行面，失败后再触发 harness repair。
- [x] `repair_session` 读取最新 field trajectory，写入 `FIELD_TRAJECTORY.md`，并把修复目标对准失败触手。
- [x] `heartbeat_repair` 会把 `FIELD_TRAJECTORY.md`、field、mini task、verifier 状态暴露到 repair plan Feed 和下一步 review 命令。
- [x] `evolve recommend field-mini-task` 能从 math field trace 生成可审查的 runtime patch draft；临时应用后可返回 `verifier_status=satisfied`。
- [x] 授权后 `evolve apply field-mini-task 03-runtime-code` 已把 math runtime patch 应用到 live harness，复跑后 `math-mini-1` 变为 `satisfied`，并记录 evolution outcome。
- [x] search 失败轨迹能生成 field-specific runtime patch draft；授权应用后 `search-mini-1` 自动复跑为 `satisfied`。
- [x] 并列池采样的 queued Need 写入 field/mini task 强标记，避免 `ib-mini-1` 被 “table math” 误路由到 math。
- [x] IB 失败轨迹已生成 field-specific runtime patch；授权应用后 `ib-mini-1` 自动复跑为 `satisfied`，并记录 evolution outcome。
- [x] code 失败轨迹已生成 scoped diff runtime patch；授权应用后 `code-mini-1` 自动复跑为 `satisfied`，并记录 evolution outcome。
- [x] SWE 失败轨迹已生成 reproduction -> patch -> regression-test runtime patch；修复 Python bytecode cache 后 `swe-mini-1` 自动复跑为 `satisfied`，并记录 evolution outcome。
- [x] robotics 失败轨迹已生成 simulator-only path runtime patch；授权应用后 `robotics-mini-1` 自动复跑为 `satisfied`，并记录 evolution outcome。
- [x] 修复 `research-mini-1` 被 `search-mini-` 子串误路由的问题；research 失败轨迹已生成 source coverage + uncertainty runtime patch，授权应用后 `research-mini-1` 自动复跑为 `satisfied`，并记录 evolution outcome。
- [x] computer-use 失败轨迹已生成 local page observation runtime patch；授权应用后 `computer-use-mini-1` 自动复跑为 `satisfied`，并记录 evolution outcome。
- [x] `octopus fields summary` 按 field 汇总成功/失败轨迹、latest verifier、pass evidence、repair signal 和下一步动作。
- [x] 让同一套 draft -> grant -> apply -> rerun -> score 闭环覆盖八个并列领域；调度器按最新 verifier 信号选择当前执行槽。
- [x] 每个领域准备一个可重复 mini task 定义。
- [x] 让 Octopus 自己跑 mini task、修触手、复跑，并记录是否变好。
- [x] 为八个并列领域定义第二层 mini task；调度器会选择每个 field 的第一个未满足任务。
- [x] 八个第二层 mini task 已通过失败轨迹 -> runtime patch -> apply -> rerun -> score。
- [x] `octopus fields summary` 可报告 all-pack task 完成状态。
- [x] 为八个并列领域定义第三层 mini task；调度器从同一个并列池采样未满足任务。
- [x] 八个第三层 mini task 已全部进入失败轨迹 -> runtime patch -> apply -> rerun -> score 闭环，并变为 satisfied。
- [x] `field-mini-task` 开始支持 `repair-templates/` 外部模板；24 个 field mini task 修复模板已从 Rust core 迁到可进化触手目录，三层模板已全部外置。
- [x] `check field-mini-task` 会运行 repair-template 覆盖检查，确认 8 个并列 field 的 24 个 mini task 都有匹配模板。
- [x] `run_field_mini_task` 优先执行外部 `repair-templates/*.pyfrag`，并在 Feed metadata 里记录 `runtime_template=repair-template` 和模板路径。
- [x] field-mini-task 的 runtime evolution 候选现在指向 `repair-templates/<field>/<mini>.pyfrag`，不再向 runner 插入领域分支。
- [x] Rust 核心测试只断言模板边界，不再复制 8 个领域的具体模板实现。
- [x] 24 个 repair template 都已改成独立 `if field/mini_task` 文件，loader 不再把 `elif` 转成 `if`。
- [x] `check field-mini-task` 会编译并执行 24 个 repair template，要求每个模板返回 `status=satisfied` 的 `field_result`。
- [x] 多领域 objective 只定义候选池；`evolve parallel --workers 1` 会按 field 完成度和最近采样状态选择一个执行槽，避免固定从列表第一个领域开始。
- [x] `evolve parallel --workers n` 会为每个 sampled worker slot 写入一个独立 Need Queue 项，执行仍走 Need -> Feed 链路。
- [x] 内部 `needs run --workers n` 可以连续消费多个 queued Need，让并列 sampled slots 自动进入 Feed，而不是手工逐条运行。
- [x] `evolve parallel --workers n` 会记录每个 worker 对应的 queued Need，并自动运行本次 sampled slots，避免只跑最后一个 Need。
- [x] sampled worker 会在 Feed 返回后回填实际 trace index、verifier index 和状态，parallel run 不再停留在排队前视图。
- [x] `evolve parallel` 在新 state 中会先准备 `field-mini-task` seed 触手，再让 sampled Needs 进入真实 Feed。
- [x] installed binary 会物化 `field-mini-task` runner、checker 和 24 个 repair template，下载运行路径不依赖源码树。
- [x] 如果当前项目已有自己的 `tentacles/` 但缺少 `field-mini-task`，parallel evolution 会回退到 bundled seed，不让项目触手目录遮掉 field 基建。
- [x] bundled seed 同时物化 field-packs 和最小文档 fixture；`check field-mini-task` 在无源码树或本地触手遮挡时也能执行 24/24 模板。
- [x] 直接点名触手的 `install`、`probe`、`think` 入口也复用 bundled seed fallback，避免用户项目自定义 `tentacles/` 遮住内置触手。
- [x] `evolve recommend/apply` 和 provider matrix 的触手规划/进化检查也复用同一个 manifest resolver，下载版自进化路径不再依赖源码树触手目录。
- [x] `starter` 和 `skills` 使用本地 manifest 加 bundled seed 的合并视图；用户自定义触手不会覆盖掉 Octopus 自带起步能力。
- [x] `bootstrap`、`adapt` 和默认 `manifests` 也保留本地触手优先级，同时补齐缺失 bundled seeds。
- [x] `repair` 自修复入口也使用 bundled seed resolver，partial `tentacles/` 不会阻断 harness-repair-agent。
- [x] `beat` 的 harness evolution 先用本地触手，缺失时再用 bundled seed 写 evolution/apply artifact。

## v0.2.0 Gate

每个领域至少有一个最小真实任务：

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

这些条件按 field 独立统计。一个 field 的成功不能替代另一个 field；worker 只决定一次跑几个槽位。

## Current Foundation

```text
field-packs/
  README.md
  index.json
  field-pack.schema.json
  _template/
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

`field-packs/` contains the first reusable pack template plus eight field skeletons. Rust core loads these packs, `octopus fields` exposes them, and Feed traces now carry the selected `field_pack`.

Current infrastructure step: keep all eight fields in the same parallel Goal pool while the verifier fills sampled execution slots. All first-pass, second-layer, and third-layer mini tasks are satisfied. Each third-layer task passed through failure, runtime patch, authorized apply, rerun, verifier score, and evolution outcome. `octopus fields summary` now reports `all_pack_tasks_satisfied=true` and no sampled slot when the pool is complete. `field-mini-task/repair-templates/` has moved all 24 task-specific implementation fragments out of Rust core and into editable harness. The live runner now only records the task, loads the matching standalone template, returns its compact Feed, or falls back to a generic partial result when no template exists. It no longer carries hard-coded math/search/code/SWE/research/computer-use/IB/robotics branches or `elif` conversion compatibility. Runtime evolution candidates now target template files directly. The manifest check now verifies 24/24 template coverage and executes every template in a temporary session. `--workers 1` means one active worker slot; higher worker counts expand concurrent slots from the same pool and enqueue one Need per sampled slot. `evolve parallel --workers n` prepares the `field-mini-task` seed tentacle when a fresh state has not installed it yet, records each worker's queued Need index, runs those sampled Needs through the same Need -> Feed verifier path, then writes trace/verifier/status back onto the worker. If an objective names multiple fields, those names constrain the candidate pool; the sampled slot still follows field status and recent-run fairness rather than textual order.

稳定 Rust kernel 只保留 Field Pack 读取、轨迹记录、路由信号、Need/Feed 传输和 heartbeat。领域实现继续留在可进化 harness 中。
