# PLAN-006 r2 R1修复交接

2026-10-30，唯一树 `D:/autostack/.wt/ui-006/auto-ui`，分支
`codex/plan-006-dev`；R1基线3a6404e，修复源码提交
`82ba57e8e51b83a3715aabd03ca0b9f4f38b08b7`。
新源身份 `e2994efa1b1ee6d55f79de7bd194bf59c1745c41f60b6c8bbaac7a67197e7243`，
仍1477文件、SDK681/17本地包/29路径边、两锁不变。此为work修复，不是独立R2 verdict。

## Finding closure

| Finding → invariant | 影响面/原因 | 修复与永久断言 | 当前结果/剩余 |
|---|---|---|---|
| F-006-R1-UNUSED-SAMPLER → 所有声明的sampler均在payload分配/decode前做共同准入 | 原代码只对全部sampler检查interpolation，时间/output检查只跟随channel引用，故unused重复/负/NaN时间及output type/count/NaN漏过；不是素材路径或GLB读失败 | 先为全部sampler建立经过type/span/count检查的只读metadata view；对所有声明读取严格非负递增有限时间与有限FLOAT VEC3/VEC4 output，再做已有channel target-specific校验和retention；used/unused共用sampler_key，未把unused提升为channel | 原R1 14份冻结fixture重放13拒/1接受；13拒均0channel allocation/0decode。新22个used/unused损坏对照同样拒于分配/解码前；无本finding残余 |
| 共因兄弟路径/合法控制 | LINEAR/STEP、output VEC3/VEC4、同一sampler供不同node/translation与scale、有效轨道累计8192 | 合法unused的负VEC3和非unit VEC4接受，不凭没有target的数据臆造scale/rotation角色；一旦绑定scale/rotation则原正值/unit规则仍拒坏值。unused不留track、不改变4clip duration/pose/CPU账，44真实channel allocations；8192有效channel samples之外存在unused keys仍接受 | 4个unused LINEAR/STEP/VEC3/4正控、2个共享sampler正控、8192边界、原4clip成功；不改预算/支持域/8192按channel累加口径 |

核心变更仅在AGP `assets/node_animation.rs`。Auto受管单元、Host、sample插值/世界矩阵
算法、renderer、物理、输入、VM/owner调度、SDK与锁均未改。target-specific的正scale、
单位quat、node/path唯一性仍由channel提供语义。未引用metadata view仅是有界GLB解析
工作内存，不保留为动画载荷，也没有新增unused样本quota或提高任何资源额度。

新增永久case在driver `node_animation_resources`：
`every_declared_sampler_is_checked_without_allocating_unused_tracks_or_changing_channel_quota`。
内部生成22个used/unused反例并直接调用preflight/from_parts及正式import，检查分类、
真实分配/解码计数；正控覆盖上述角色/模式/共享/额度/pose不变。设置
`UI006_R1_INPUT_DIR=D:/autostack/plan-artifacts/auto-ui/plan-006/review-r1` 时，还重放独立R1
保存的14份原始GLB，不靠新生成器重建审查输入。

SD-01增加unused共同检查/不留track、不改duration或pose、不计channel样本及target角色说明，
对应已约定规则；不删弱“每个声明sampler”要求。新SD原文件SHA256
`3d6af002bef2c30fb84417287be8422f13468f2b82f6b18f7d2734bbf28e7621`。
当前checker资源runner计数随新增case从5更新6，总声明门从新30+旧3变为新31+旧3=34；
旧33case报告保留原身份/历史，不给新身份换标签。canonical/ledger未发布。

## 局部验证及复用范围

所有命令在对应包目录，以外置target-stack32、原32MiB主栈/16MiB worker，
`cargo test --locked --offline ... -- --test-threads=1 --nocapture`。只运行本finding及
共同admission入口受影响的检查，未在repair重跑整个GPU/20轮owner/CPU矩阵。

| 命令后缀/入口 | 实际结果 | 日志SHA256（根D:/autostack/plan-artifacts/auto-ui/plan-006） |
|---|---|---|
| driver `--test node_animation_resources`，R1_INPUT_DIR设原始fixture根 | 6/6；新22反例+原始14重放、原29准入反例、8192/8193/16clip/128node/channel、64MiB resident/48MiB接受、pose隐形预留、保存/冷开修复全通过 | r1-repair-resources.log `c3a62692d58538d63c0cfd4bb6eb7da1d0a10a472eefe9a000bd5448646a4dc4` |
| driver `--test node_animation_resources every_declared_sampler` | 最后补充unused duration/pose严格相同断言后，新增case1/1（5filtered）；R1 14原件仍13拒/1接受、22内部反例及所有正控通过，其它5case未变复用 | r1-repair-final-sampler.log `2455b3ec1e8ce04a9eb948c41419d5c4e613bded31e978ac1a63d67b18fa48dd` |
| AGP `--test node_animation` | 5/5；固定四clip、独立slerp/STEP/层级/边界、分类、矩阵parent、预检账、极大有限端点不溢出 | r1-repair-core.log `c96fcba4c8fac75067d3fa6901a8e33b611dd014d01c34d86b8bdc83e8e054fa` |
| `verify_plan006.py --controls` | 1477集合/目录/hash、错误pins/escape/unlisted/PID与Auto blob原字节LF→CRLF负控均过；恢复后接受 | r1-repair-identity.log `67adac4f4dc312e302c50cebe4c2a49035adda1592a3999e0ef9b44a8b85a76e` |
| `verify_source.py` | 新身份/SDK681、17本地包/29路径边、479 metadata、禁用包0、两锁保持 | r1-repair-source.log `f81ad6ece91ae25acf5b1020bf2703b9012a6f66ae18fddf1d997e0eda43b4c2` |

两包fmt与git diff --check exit0。SDK原146warning保留，无新增交付warning。
生产代码自82ba57e冻结，后续本Markdown不在实验源码集合中。

本轮保留R1已独立审过的AC-02/03/04与AC-05输入/VM/CPU/旧兼容证据：对应Auto/Host/
renderer/physics/owner/SDK/锁/输入和固定原件均无改动；新增共同admission已由上述原件
及冷开/资源cases重放。未把整份旧source identity报告当成当前新身份的冻结门。

为覆盖确实受影响的公开asset载入/完整当前消费身份，又做了一次新的source-only A→B：
`package_consumer.py <.../r1-repair-a> --snapshot-only`，检查绝对移动目标后Move到
`r1-repair-b`；从包内SDK快照prepare并verify_source。B的1477身份文件与producer/Git
blob逐字节相同（e2994efa...），不是旧B或原producer构建。
在B执行 `--test node_animation_consumer bound_original_project` 只重放受影响公开项目case，
不重复未变的CPU矩阵：1/1（1filtered），离线locked重新编译41.07s/执行4.82s，
内置manifest_dir指向r1-repair-b；60tick、速度4→1真实Auto变异、SaveAs/搬迁/新Registry
冷开4clip/初pose通过，6370steps/7slices、managed80021B、暖mesh/texture上传0。

消费证据SHA256：r1-repair-package.log
`d69df965a3ba44c2e34f68a08f39e6f0a57443cb1fcda48034d0ada4a66f5ced`；
r1-repair-consumer-sdk.log
`cc07e20bb564fecab35cc759a2e40c2f21cdb1ca5fdf824727e93e4a0214faf1`；
r1-repair-consumer.log
`928cfd5672e797a7b8e669666e44f31661139392b5f199fb7edffb2f9164d9da`。

## Ready for affected final re-review

F-006-R1-UNUSED-SAMPLER的现有反例、共同机制与代表性兄弟路径已关闭；work-owned
开放finding/覆盖缺口/合同决策为空，T03/T06恢复完成。没有变更原Plan r2的AC、预算、
支持模式、仓库或动作范围。下一步由原独立上下文进行R2受影响复审；其后merge owner
仍需在新冻结版本核34case门及控制/有效复用，不能用旧身份报告或本work pass替代。
T07..10、canonical/ledger/归档/delivery/guard清理仍待，目标未完成。
