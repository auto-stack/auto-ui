# PLAN-006 r2：输入预算、朝向/阴影、事务与所有权

2026-10-30，唯一实施树 `D:/autostack/.wt/ui-006/auto-ui`，分支 `codex/plan-006-dev`，实施基线 `ccfe6d9`。本记录关闭 T-02 r2、T-04、T-05；T-06 fresh consumer/current identity、SD 与最终复审/合入仍待完成。SDK、两锁、原始素材未变，原 T-01/T-03 与未变采样代码的既有证据继续有效。

## 变更与实际路径

新 profile 的受管单元增加 `ag_actions_snapshot()`，收费1，返回 held+64*unreadPressed（各6bit）。它与逐键 pressed getter 共用消费状态，重复读只保留 held。未改旧受管单元、旧 API 价格、32credits 或运行限额。受管源80021B，低于96KiB。

真实 Host 路径为 GameInput/camera dispatch → Auto 解码输入/算四向移动、跳、grounded、clip、phase、yaw → 输出读取 → 独立准入/prepare → 共同 Frame。完整事件分支读取 event_next/pair/phase 后设置一次视觉，总成本32。下一次 snapshot 触发 error10。坏 clip46、负 phase47、旧实例代次33 与资源 prepare 失败均保留整组旧位置、相机、事件、pose 和实际 GPU 画面；Stop 可恢复并再运行。

资源检查增加 `asset_union.pose_reserved_cpu_bytes`、`cpu_total_with_pose_bytes`、模型纹理上传与数量，来自实际准入的同一保守公式。实测 raw2/97732B、共享 geometry+animation1102359B、pose32240B、CPU合计1134599B、GPU预期1101700B；均保留原64MiB额度。检查项为诊断数据，不添加玩法。

## 执行证据

所有命令在 driver 包目录执行，`CARGO_TARGET_DIR=D:/autostack/plan-artifacts/auto-ui/plan-006/target-stack32`，`--locked --offline`，真实 GPU 单实例、测试单线程。日志位于该 plan-006 外置目录。146条 SDK 原有 warning 保留；本轮无新增 warning。

| 命令（`cargo test --locked --offline` 后缀） | 实际结果 / 区分性断言 | 日志 SHA256 |
|---|---|---|
| `--test public_node_animation -- --test-threads=1 --nocapture` | 8/8；聚合所有bit/63/混合/重复与旧 getter 消费；完整32成功、额外1失败；旧逐键反例仍error10；槽旧代设置clip3/phase.4/yaw1.2后销毁重用，新代2恢复base/-1/0/0；原Host搬迁/19wire/擦轨道GPU负控通过 | r2-public-final.log `c6acd0532717bf47a070808f9e05f7228f013459d23261130ed54895664016e4` |
| `--test node_animation_lifecycle -- --test-threads=1 --nocapture` | 当时4/4；真实packet130/1040、真实enter101/stay102分支、neutral Step仅一次且无暂停边沿/相机泄漏、新Run初pose；全组last-good；20轮Play/restart/Stop，40个VM及pose释放、descriptor释放；mesh/texture暖上传与decode零增量、最后外部pin释放后GPU/纹理weak失效；240tick开关动画/yaw，地面/墙/平台位置、grounded/support、events逐项一致 | r2-lifecycle-final.log `d6aff3c09d2b60d9717f0137470a279848fc66353a4103213a0199c33f67dbb9` |
| `--test node_animation_lifecycle four_direction_motion -- --test-threads=1 --nocapture` | 新增第5case 1/1；四向真实输入和30度相机下位移与独立sin/cos一致；模型+Z面向与移动一致；停止位置/yaw保持并选idle | r2-four-motion.log `62b2044a233d2fa44a95ff50ee1b180124c237f8f67f20eae60a214bc0a2f99d` |
| `--test node_animation_pixels camera_relative -- --test-threads=1 --nocapture` | 1/1；真实四向输入、停止、±pi跨界，独立期望的固定相机轮廓<=2误差；错yaw负控1341/1380/1486/1458像素错误 | r2-facing.log `b477f9165005949d4a68374ed073afc02ed0f12f94f139006e8cb7326e43318b` |
| `--test node_animation_pixels step_rotation -- --test-threads=1 --nocapture` | 1/1；补充STEP root quaternion fixture，.5保持第一key、1切换、1.5保持第二key；原件不改，错误45度插值被独立轮廓608像素拒绝 | r2-step.log `33c9a8d83c2dfc7e3658200f6771b76e848e8dcf6c6b19ba26e9e99c5d614e78` |
| `--test node_animation_pixels actual_original_walk_shadow -- --test-threads=1 --nocapture` | 1/1；原walk .19/yaw.45实际Host矩阵→同色图/阴影；独立raw-key采样并投影到receiver，87196稳定ROI、2171阴影内部、错误0；冻结节点pose负控263、关闭阴影负控2171。已实际查看PNG，角色与影子可辨认 | r2-shadow.log `678afeb0bd298ae09756839f744b77b886e7b8828dafbfb6bb07e717b2969b5b` |
| `--test public_sprite_frames older_profiles_keep -- --test-threads=1` | 1/1；旧14profile精确装配hash保持 | r2-old-profiles.log `91fd7c162707244431e5d86cf1b33f493c58b00e8f04d507b7642e7a7bb54b95` |

图片为 `r2-pixels/*.png`，不入Git。新增测试与检查字段不改变旧采样/渲染算法；T-04前轮原四clip21状态/43596源顶点与2074207轮廓probe、逆转置法线修正和旧005定向证据按代码/输入/依赖不变复用。新增物理case后未重跑昂贵20轮，因为该case不改共享helpers/生产代码；已运行的4case与新增定向case共同构成当前5case证据，不声称曾执行5/5整文件。

最初Session探针重叠模型使移动提前退出，修正为分离模型后正式32/33控制通过。Host触发事件按冻结语义在tick末evaluate、下一callback消费；测试按此区分start无事件与随后真实enter/stay，没有伪造事件。两处测试脚本拼写/保留名编译失败已修正；失败日志不是通过证据。

`cargo fmt --check` 与定向 `git diff --check` exit0。完整离线消费/current identity、固定源变异、性能矩阵与SD归T-06；全局冻结门归review/merge。此记录不声称完整G3游戏、Gallery、G10或AG里程碑已交付。
