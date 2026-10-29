# PLAN-006 T-03：动画准入/持久化/资源计账

2026-10-30；唯一树 `D:/autostack/.wt/ui-006/auto-ui` / `codex/plan-006-dev`，parent9354e9e；前轮T-02有实际整链进展，本轮重读主Plan与Git核实，无006并行writer/中断状态。主检出其它计划docs WIP保留，SDK/两锁/固定原件未改。执行合同仍为r1，无范围/阈值变化。

## 实现与区分性修正

1. `Animation::preflight`先检查全部metadata、跨度、计数、时间/输出数值与base层级，计算精确payload+必要scratch字节而不分配key/output向量。`textured_models`先收全PNG依赖header/资源union预算，过门后才构建不可变轨道；直接AGP `from_parts`也先验证所有轨道值，不会因后轨道坏值留下早轨道载荷。`channel_allocation_count`在实际分配key/output向量对的唯一位置计数，预检/采样不增加；这是本任务的准入分配观测，不是用计数代替行为。
2. 新profile支持合法无纹理节点TRS（glTF缺省可选数组按空数组处理），保留原节点/primitive关系；旧纹理profile的id与规则不变。无纹理动画保存/冷重开通过。所有声明sampler的插值受检，未引用CUBIC也unsupported；空clip invalid，skin/morph/weights/其它目标保持unsupported。
3. 必要pose资源账补齐Visual/asset key/Arc header、目标表与额外wire，按实际Rust类型大小预留last-good/candidate/prepare有界副本。每静态target与8潜在槽都计价，隐藏不减分母。真实合法资源构造反例：旧公式21888B，asset剩余25501B，新完整reserve29112B；旧公式放行、新公式拒，增加65536B余量接受。受控恢复旧公式实际运行该回归exit101，再逐字节还原后exit0；不是只计算一个理论数。
4. LINEAR translation/scale用凸组合避免`b-a`中间溢出。[-3e38,+3e38]的合法中点必须为0，独立core断言通过；quat仍最短弧正规化slerp。未松任何非有限/负scale/奇异pose拒绝。

## 生产入口与持久化证据

新增 `tests/node_animation_resources.rs`，最终5个独立case均通过（定向运行与明确未变证据复用，未重复全套）。

- 23结构反例：CUBIC（含未引用）、weights/skin/声明morph、重复target、错count/type/normalized/stride/span、错node、cycle/多父、负/零scale、错插值类型、nodes129/clips17/channels129；6二进制反例：重复/负/Inf时间、NaN输出、坏quat、负scale输出。生产`import_animated`逐项正确分类，均0 key/output分配、0图像/geometry decode；恢复原件真正分配44 channel，原四clip真接受。
- 8192累计samples合法/8193 resource limit；16 clips、128 nodes/128 channels真合法；STEP与多层无纹理层级准入/持久化。preflight计费与实际Animation `cpu_bytes`一致。
- 4个合法2048² RGBA resident图片恰满原64MiB CPU预算时，导入character在完整union门拒绝，轨道分配与解码均0；移除一图留下48MiB时真接受并分配44channel；同包暖导入共享同一Asset/Animation Arc，无新分配/解码。
- 5保存故障：AssetAfterPrepare/AssetAfterInstall/BeforeReplace/AfterReplaceRead/ClosureAfterReplace，真实retry或pending→损坏PNG verify拒绝→修复verify确认；同样板暖加载缺/坏/修复PNG、暖GLB原字节变异/恢复均受检；改profile flag无法复用id，连旧域id和resource一起重算仍因真实动画root显式unsupported拒绝，恢复后冷重开原4clip。

## 精确运行与日志

全部日志位于 `D:/autostack/plan-artifacts/auto-ui/plan-006/`，仅本地保留；Cargo均`--locked --offline`，单线程。driver运行工作目录固定为包目录，启用其原`.cargo/config.toml`（主线程PE reserve33554432B、worker配置16777216B），target=`target-stack32`。PE头实际读回通过，审计`t03-stack-profile.json`；此前T-02功能probe从仓根使用manifest启动，未用它声称此PE设置，当前宿主整链已在正式包目录重放。

| 命令（cargo test前缀） | 结果 | 日志/sha256 |
|---|---|---|
| --test node_animation_resources -- --test-threads=1 --nocapture | 4/4原资源case通过；后续loader/core/输入未变，复用3个未变case，storage case补充root损坏后定向更新 | t03-resources-final.log / 27a90289b68091e24eff7569c7b517fd4193597e3a14214fef8cddc9261a5945 |
| --manifest-path experimental/autographics-prototype/Cargo.toml --test node_animation -- --test-threads=1（仓根） | core5/5；新增预检/实际字节一致、直接API坏值零channel分配、有限大坐标中点；未作性能声明 | t03-core-final.log / 49e1f4ca359a9ffb2a47def20435369bb114f38730d334d2da566880289cfbaf |
| --test node_animation_resources pose_metadata_wire_and_hidden_potential_slots_share_the_existing_cpu_union -- --test-threads=1 --nocapture | 受控旧公式必须红exit101；原字节恢复后1/1绿exit0 | t03-reservation-negative.log / 0180f05b317aa6feb8233d5d6033148ef7cbc91c60eb820d6d0d9ab37a207092；t03-reservation-restored.log（hash见主Plan） |
| --test node_animation_resources animated_closure_save_faults_cached_dependencies_and_profile_forgery_recover -- --test-threads=1 --nocapture | 最终storage1/1含补充暖GLB坏字节/修复 | t03-storage-final.log（hash见主Plan） |
| --test public_node_animation original_textured_character_auto_host_same_frame_pose_and_gpu_pixels -- --test-threads=1 --nocapture | 当前核心/资源账、正式栈配置下Host1/1；19wire负控、原角色/同帧/真正去轨道GPU负控、保存搬迁冷重开；tick3617us，GPU像素与T-02相同 | t03-host-replay.log（hash见主Plan） |

两包fmt/diff检查通过，新产品无compiler warning，冻结SDK既有146 warnings未改。首次资源test编译因Storage无Debug而误用unwrap_err，已改为显式err读取，未改变产品规则或降低断言。没有跑整AG/根UI/语言批回归或OS窗口自动化。

## handoff

T-03关闭，3/10。AC-01/V-01的生产准入/支持边界及AC-04中的资源/持久化部分已有当前断言证据；其余AC不提前关闭。T-04继续四clip实际画面/独立矩阵、相机相对面向、碰撞支撑独立及阴影；T-05完整事务/暂停单步/新局/20轮owner；T-06完整消费/identity与最终性能。独立final review与merge冻结门仍pending，无用户决策阻塞。
