# PLAN-006 T-04画面进展与完整输入预算反例

2026-10-30，r1，parent2739324；唯一树 `D:/autostack/.wt/ui-006/auto-ui` / `codex/plan-006-dev`。前轮T-03为有效进展，本轮无用户决策阻塞；目标/32credits/CPU/源/仓库范围未缩减。

## 已关闭的独立画面证据（T-04仍整体未完）

新增`tests/node_animation_pixels.rs`，从原GLB JSON/BIN直接读取原始f32键与顶点，以独立f64球面插值/层级参考计算，不用生产Animation.sample/channels/mesh_pose生成预期。真实Auto在21个tick选择原4clip，覆盖0/mid/exact end/beyond end/loop end及非零phase立即切clip；同asset静态与runtime slot独立clip/phase/yaw，消费同一immutable原件。实际Host发布的矩阵逐点比较43596个源顶点，最大误差2.4978897e-7（阈值2e-5）；实际GPU轮廓的2074207个稳定像素探针通过（排除1px边缘，每帧最多2错）。

static保持静态；idle/walk/jump的mid图实际变化12480/18733/13338个颜色通道。one-shot超过末端保持end图、精确loop末端回first图。只擦除node delta（位置/相机/yaw/mesh/纹理固定）的故障控制使独立轮廓oracle分别错644/1277/215像素，均被拒。21张原始PNG在`D:/autostack/plan-artifacts/auto-ui/plan-006/t04-pixels-final/`，只本地保留；walk原件画面已实看，为原纹理机器人及明确肢体姿势，没有Bob或替代素材。

发现并修复非均匀scale法线错误：旧shader直接model*normal，缩放(2,1,.5)的倾斜平面真实GPU灰度119，而独立S^-T法线预期163。当前color/shadow共享shader用cofactor/determinant计算inverse transpose，结果163±2；旧005深度/双面朝向定向1/1、4朝向/2绘制顺序/1reveal控制通过，max_error0。

精确命令（包目录运行，target-stack32，--locked --offline，--test-threads=1）：

- `cargo test --test node_animation_pixels -- --nocapture`，最终2/2，日志`t04-pixels-final.log` SHA25600ef4c3bb76ece0017ecaa915c358fb1685ef7a7d1585fa7d03fdff53534f1b4。
- 修复前`... --test node_animation_pixels nonuniform_node_scale_uses_inverse_transpose_normals_in_actual_gpu_lighting`，预期红119≠163，日志`t04-normal-red.log` SHA256d327f84c5e7262e7b2a989bba125e84b83dc6d3a6d0a770df976389aab1fa253。
- AGP `cargo test --test textured_models textured_depth_and_double_sided_normal_orientation_are_visible_in_real_pixels`，1/1，日志`t04-legacy-normal.log`（hash见主Plan）。

日志在`D:/autostack/plan-artifacts/auto-ui/plan-006/`。两包fmt/diff干净，新产品/测试无warning，旧SDK146 warnings未改。初始fixture使用glam旧投影API及保留名tick被正式入口拒绝，已改为仓内Camera3d和非保留名ticks，不改产品准入规则。

## F-006-W1-INPUT-BUDGET（needs_replan，AC-05/T-02/T-04）

**现有不变量**：真实Auto同时完成四向/跳跃输入、运动/最终支撑、相机、触发/一次读取、原clip与视觉setter，并保持共享32credits及旧profile/旧函数价格。不得用固定运动向量代替实际输入后宣布整组合预算完成。

**实测反例**：`public_node_animation::full_input_movement_trigger_and_visual_combination_exposes_scalar_read_budget_limit`，真实SourceSnapshot/Session/现有new profile装配，4次ag_action_held+1次jump pressed→ag_char3_move→grounded→event_next→visual setter；未加事件pair/phase getter的更弱组合已经返回粘性error10（32credit超额）。正式包目录命令`cargo test --locked --offline --test public_node_animation full_input_movement_trigger_and_visual_combination_exposes_scalar_read_budget_limit -- --test-threads=1 --nocapture`，exit0（断言预期error10），受管79567B/用户480B，日志`t04-scalar-input-budget-counterexample.log` SHA256ac7ae59e56d0c733e8c35fdf8f655ccffb87cd1cd103d2d946aa9c5bf3b0e030。

**原因与先前漏检**：T-02完整路径读了相机basis，却使用固定运动向量而没有4方向+跳键读取；公开单键getter每次1credit的既有价格被漏算。不是GPU/计时噪声、缺件或用户授权问题。Canonical `game-character3d.md`明确move26、ground/support各1、每held/pressed1；`runtime-triggers-and-rounds.md`明确event_next/每getter1。更完整含event pair/phase的组合需36，不能靠本来32的额度完成。

Spec锚：auto-lang `docs/specs/auto-graphics/game-character3d.md` SHA2566dc38be4bfbeaef096ea1a7e15ece37743c9140d2aad14bdba19c2c0b5c26cc4；`runtime-triggers-and-rounds.md` SHA2561d0e37f3a641d33fd6f4408bbcf039f28cc41b90c6765127bd04ad8585eb2e44；`behavior-api.md` SHA256ea0c9f6c6752f015af6e0235728edcd7873f166790f4493780a5d03b1443695e。只读取，未发布修改。

**bounded new候选，尚未实施或当作现合同**：仅在尚未发布的node_animation3d/1增加一次有限`ag_actions_snapshot() int`读取现有6 held+6 pressed，packed=`held + 64*pressed`，域0..4095，共享成本1；读取消耗当前tick所有pressed，与旧pressed getter共用同一消耗状态，重复读取held保留/pressed0，Pause/Step/fresh Run继续旧input snapshot规则。Host继续只传原输入，不负责方向、动作或玩法。旧14个profile、逐键getter定价和装配字节零改；无预算增加、无新的host wire。

这样真实Auto用一次有限快照自行解码四向/跳/重启，1 input +26 move +1 grounded +1 event_next +1 pair +1 phase +1 visual=32。这是修正设计以达到原AC，并非省输入、相机、触发、原clip或移动Rust玩法；仍需new按证据正式修订合同后work实现。关闭oracle必须含原逐键反例仍error10、新完整真实组合fault0且传入按键确实改变运动/clip/yaw、混用/重复pressed消费、neutral Step/fresh Run、再添一credit必须error10、旧profile hash保持。

## 当前交接

work outcome=needs_replan，next=new有界修订。T-02重开（原素材/采样/同帧GPU/wire/存储证据保留；其“整组合预算完成”结论失效）；T-01/T-03保持完成，计数2/10。T-04画面部分已有上证，但相机相对四向/物理独立/实际阴影仍待完成，不能勾选。T-05..10未完成。没有需要用户重新批准的目标/预算/范围变化，也没有外部阻塞。
