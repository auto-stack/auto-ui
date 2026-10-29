# PLAN-006 r2 T-06：消费、身份与复审交接

2026-10-30，实施树 `D:/autostack/.wt/ui-006/auto-ui`，分支
`codex/plan-006-dev`。最终源码提交
`475d376978c6af5acc87a73b277fc68c018c21be`；本Markdown是树外源码范围的交接记录。
源身份 `50a87a3c973b93f4a961fc00b81b39788cbd785f3775fba01811f6edc192a5da`，
1477个文件全部在Git中，覆盖driver/AGP完整源码、资源、SDK快照、测试/工具/配置和SD。
005交付/004修复/SDK及两锁仍按T-01冻结，未改历史001..005身份收据。

## 当前交付

- 公开 `samples/node_animation/scene.agscene` 绑定 profile110/21 的项目内Auto脚本；
  原始character87624B、四clip、PNG保持原字节。内容寻址GLB/PNG/Auto均在项目内。
  `generate_project.py --check` 重现身份、文档和规范化碰撞shape，不剥离动画。
- Auto自己解码聚合输入、算相机相对四向/对角线移动、重力/跳、grounded、事件消费、
  clip/phase/yaw。真实公开source速度4→1变异使实际位移改变4倍，证明不是Host玩法。
- `node_animation_consumer`通过Storage.load_project→Host.start_selected_play跑60tick，
  保存A、搬迁B、全新Registry冷重开，保留四clip/原Auto身份并恢复初pose。公开CLI选择、
  profile/domain/价格、wire和支持表见样例README与SD；不以内部Session夹具替代消费。
- `plan006_source_policy`完整集合/文件夹/内容身份以及source verifier/exporter的006入口。
  普通文本规范化换行，SDK冻结文件和内容寻址Auto按原字节；身份JSON确定生成LF。
  producer工作副本、Git blob与最终仓外consumer的身份文件逐字节相同。
- `research/node-animation-spec-delta.md`备好SD-01新节点动画规范与SD-02/03两个追加段落。
  canonical/ledger尚未发布，归review后merge。新增Session只保留两个标量高水位观测，
  用于实测CPU steps/slices；未更改原CPU guard、调度或值/owner模型。

## 最终冻结检查

所有日志/报告在 `D:/autostack/plan-artifacts/auto-ui/plan-006/`，不入Git。
正式Cargo工作目录为各包目录；外置target `target-stack32`保留32MiB主栈/16MiB worker。

`python tools/verify_plan006.py --controls` exit0，1477集合/源变异、删除、未列入文件、
稳定scene、错误pins/路径逃逸/PID错误列入均拒；合法PID输出忽略；新增实际Auto blob
LF→CRLF变异也拒，恢复后通过。身份生成和校验工具本身纳入完整集合。

`python tools/check_node_animation.py --controls --reports <.../t06-closed-gate>` exit0：

| 当前runner | 通过 | 覆盖 |
|---|---:|---|
| public_node_animation | 8 | 原件Host/搬迁、19wire、真实GPU擦轨道；S-06/原逐键反例/32与33；槽复用初值 |
| node_animation_resources | 5 | 预检先于分配、精确额度、完整resident union、存储/修复/降级拒绝、pose预留负控 |
| node_animation_pixels | 5 | 独立源键/顶点、原四clip21状态与共享实例、STEP旋转、四向/stop/±pi、同pose shadow、逆转置法线 |
| node_animation_lifecycle | 5 | 真输入/事件/neutral Step、last-good、旧代/prepare错、20轮owner、240tick物理对照、四向真实位移 |
| node_animation_consumer | 2 | 公开项目、真实Auto变异、SaveAs/搬迁/冷重开与下表CPU矩阵 |
| AGP node_animation | 5 | 层级、最短弧slerp/反号、STEP、phase边界、准入分类/跨度/额度/大有限端点 |
| 3个继承定向case | 3 | 旧14profile精确hash、004真实Auto精灵GPU组合、005公开纹理项目保存搬迁重开 |

合计33通过、0失败/忽略；新模块无过滤，继承选择明确为1/17、1/17、1/21。
检查器绑定实际exe/hash、确切runner/count、完整源身份、log、CPU/资源/像素结果与栈；
stale/partial/foreign runner、zero count、像素错误、warm上传、CPU guard/内存篡改均拒，
实际执行不存在的测试名返回0tests也拒。不能只靠绿色横幅或零选择冒充通过。
实际exe PE主栈均33554432B。当前shader/采样/旧组合正反例一起验证，完整root/UI/OS门
不在本范围；没有执行窗口抢焦点自动化。两包fmt及`git diff --check 80d9bbe` exit0。
146条SDK旧warning保留，本轮交付代码无新增warning。

| 布局 | 基础节点 | steps / slices高水位 | producer tick中位/最大us | 结果 |
|---|---:|---:|---:|---|
| regular | 3 | 7025 / 8 | 4106 / 4559 | 接受 |
| wide | 120（+8潜在实例=128） | 91382 / 92 | 42772 / 46616 | 接受 |
| deep/contact | 64 | 68304 / 69 | 26949 / 36868 | 接受 |
| deep/contact | 120 | 原100000步guard触发 | 不发布候选 | Fault，完整last-good保持 |

CPU profile仍1000steps/slice、100000累计、120resumptions、250ms callback、2s whole tick。
这些是debug CPU测量及原包络，不声称所有布局60FPS。例中raw2/97732B，geometry+animation
1102359B、pose29112B、CPU总1131471B、GPU预期1101700B；仍原64MiB。managed80021B。

## 最终仓外A→B

`package_consumer.py <.../t06-closed-a> --snapshot-only`只导出1477清单文件+006身份；
原始 `.sdk-source`/build/evidence不作为依赖。经检查绝对路径后原生Move到
`t06-closed-b`，在B运行 `prepare_sdk.py --from-snapshot` 与 `verify_source.py` exit0：
SDK681/17本地packages/29路径边、479 metadata packages、禁用UI/Man/旧生产包0、两锁保持。
再运行 `cargo test --locked --offline --test node_animation_consumer -- --test-threads=1 --nocapture`
exit0、2/2；实际重新编译51.28s，执行10.79s，编译内置manifest_dir就是仓外B，
不是生产者exe。B中用户source/controller、保存搬迁冷重开与CPU包络全部通过。

B的regular/wide/deep64分别7025/91382/68304steps，中位4482/47196/26717us；deep120
仍100000步拒绝。公开60tick高水位6370steps/7slices，暖mesh/texture上传0增量。
实际consumer exe `node_animation_consumer-2798423048da7500.exe` SHA256
`b725028992bc4539ba354573d81bf97de2fada616f29987036dc2f7af90cc99a`，PE栈33554432。
B运行之后，producer冻结报告再次validate exit0，原runner二进制与源身份未漂移。

最终证据hash（下列只用closed版本；早期同类报告不替代当前冻结身份）：

| 证据 | SHA256 |
|---|---|
| t06-identity-closed.log | 67adac4f4dc312e302c50cebe4c2a49035adda1592a3999e0ef9b44a8b85a76e |
| t06-package-closed.log | 46ef49be8352507b198ee2f596f84ef4d87354df5942e9074cadf2fea097a513 |
| t06-sdk-closed-b.log | e6b6a968f5289271d5f4074741316842d8a1ac6613eaa092012dba4da6abbdbd |
| t06-closed-gate.log | edb730fe825f06cb17257f543d3ccb261ed1d989826ff82590311aeb2b8b4876 |
| t06-closed-gate/manifest.json | b3d41a65903ac6f1980f8f3a2fa0c47a315dcd61059e7232ac90ce50b5272fec |
| t06-closed-b.log | cbc1b862c44b3a9b90bad41f725f10e9314b8bfefb06febe499eeb96028d1195 |
| t06-revalidate.log | d86898960d799e05e91ef18093bc9b0cbff21db1cf2f5ca6a3f37b8dac51c133 |
| t06-portability-audit.json | 00904e3ad24ee1d72629b65ed4f7521f2924c4fb0d5a122ff2544b04bbbc826c |

收尾曾发现Windows生成身份CRLF与Git LF不一致，另有generator检查错误规范化内容寻址
Auto的问题；分别修复为确定LF生成、raw-exact SDK/blob规则与blob原字节检查，并增加
LF→CRLF真实拒绝控制。修正后重新生成身份、重跑完整声明的新模块门和全新仓外消费，
不是给旧报告改标签。此前positive仅作历史诊断。

## Readiness

AC-01：T-01/T-03原件/准入/存储证据由当前resources/core冻结门重放；通过。
AC-02/03：当前pixels/core/lifecycle独立实际GPU/物理、所有区分负控通过。
AC-04：当前public/lifecycle/resources完整事务/代次/neutral/fresh/20轮owner/额度通过。
AC-05：真实32/33、旧profile/精灵/纹理定向、全新仓外消费/CPU/source/current identity通过。
work-owned开放finding、覆盖缺口、未解决合同决策：空。没有缩减本期AC或提高预算。

T-01..06 work完成；后续T-07独立上下文逐AC/V/S/SD复审与revision绑定、T-08 merge冻结门、
T-09 canonical/ledger、T-10归档/delivery/guard清树仍待。已执行的work声明门不是独立
review verdict，也不提前授权跳过merge门；原全链用户授权继续有效。规范未发布，
全AG/G3游戏/Gallery/G10与语言批量回归均不在本交付完成声明内。
