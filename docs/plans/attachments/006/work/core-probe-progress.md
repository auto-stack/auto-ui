# PLAN-006 T-02 采样与真实Auto producer进展

2026-10-29；唯一树 `D:/autostack/.wt/ui-006/auto-ui` / `codex/plan-006-dev`；父提交11bc18b，用户完整授权仍有效。前轮T-01是有效进展，本轮重读主Plan/树确认没有阻塞、失踪树或并行writer。未改其它计划WIP。

## 已实施与实际证据

- AGP新增通用`assets/node_animation.rs`：不可变节点/clip/TRS，16/128/8192声明上限、先span/type/count再keys分配、有限严格递增时间、单位quat、正scale、唯一target、唯一父/无环、STEP/LINEAR最短弧slerp、loop/one-shot/zero-duration、保留未动画父matrix、受检finite可逆pose、yaw域与归一、CPU payload+一实例scratch保守计账。没有Bob专用动作或Host玩法。
- driver追加`autographics.node_animation3d/1`（version110/profile21/CLI node-animation3d），从原triggers3d实际装配派生，唯一`ag_visual_pose(kind,entity,clip,phase,looping,yaw)`在同次调用写全部视觉值。static稳定id和实例完整token分域，旧代与错误参数粘性拒绝，槽代次刷新为base初值，一次setter真正消耗共享1credit。
- 公共输出wire和次序已写入`node-animation-contract.md`，实际原件ordinal/phase/radians、矩阵顺序与预算不变。新profile的Host/资源/GPU适配尚未完成，不能作为可交付功能使用，T-02仍开放。

所有Cargo命令在实施树执行，带`--locked --offline`，target独立于主检出：`D:/autostack/plan-artifacts/auto-ui/plan-006/target-agp`；未改SDK/依赖锁。新代码没有compiler warning；SDK已有146 warnings及两个依赖warning如日志，不修无关冻结源。

| 实跑命令（前缀cargo test --manifest-path） | 结果 | 本地证据 |
|---|---|---|
| experimental/autographics-prototype/Cargo.toml --test node_animation -- --test-threads=1 | exit0，4/4；原件四clip、独立30°point oracle区分slerp/nlerp、反号quat/STEP/phase边界、19结构变异+4二进制坏值、matrix parent、8192合法/第三channel超额 | node-core-final.log SHA256 ac4b7a533f27f520c6fa0da15ea9e8b6109062510fb9cfd5f6c65c24cd860747 |
| experimental/autographics-auto-driver/Cargo.toml --test public_node_animation -- --test-threads=1 --nocapture | exit0，3/3；真实Auto callback含移动+垂直位移+grounded+camera basis+event一次读取+视觉setter，连续3tick正确clip/phase/tick；6非法setter+实例正控；9次视觉写+运动触发共享error10负控 | auto-core-final.log（SHA见主Plan本轮收据） |
| experimental/autographics-auto-driver/Cargo.toml --lib authoring::behavior::assembly::tests -- --test-threads=1 | exit0，4/4；managed边界/缺件/derive/顺序变异 | assembly-compat.log |
| experimental/autographics-auto-driver/Cargo.toml --test public_sprite_frames older_profiles_keep_their_exact_assembled_bytes_and_the_new_one_fits -- --test-threads=1 | exit0，1/1；旧profile实际combined/api/bridge hashes逐字节保持 | frozen-profiles.log |

日志均在 `D:/autostack/plan-artifacts/auto-ui/plan-006/`，仅本地保留。两包fmt --check和git diff --check exit0。真实Auto producer实测受管源79567B，user538B，保持96KiB/352KiB包络；正控fault0，特意超额负控error10，非绕过32credits。此为producer可行性，不将缺失的Host/GPU或CPU全链性能称为绿。

## 下一入口与边界

继续同树T-02：保留旧静态/texture明确拒动画，建立新显式animated closure/payload与持久化身份；Host配置target、读/独立验证wire、共同Frame/Prepared，按node sampled world撤销base bake后每mesh生成矩阵，颜色与shadow共享；原纹理character真实Auto整组合→实际像素、一次prepare事务与预算/CPU/步数实测才可关闭T-02。T-03广泛准入矩阵、T-04画面/物理、T-05 owner/20轮、T-06 fresh consumer/identity仍未完成。无用户待答问题，不要求重新授权。
