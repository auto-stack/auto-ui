# PLAN-006 r2：完整输入组合的有界修订

2026-10-30；new阶段，修订依据F-006-W1-INPUT-BUDGET，代码基线`ccfe6d9c0e80616af344ce8898a674a380903ffe`，唯一树`D:/autostack/.wt/ui-006/auto-ui`、分支`codex/plan-006-dev`。此文件是执行设计合同，不是实现或验收通过声明。主Plan§5–8持当前任务/AC，原r1及反例保留于Git与`t04-pixels-and-input-diagnosis.md`。

## 不变量与授权

原目标、8项AC、10任务、素材来源、仓库与work/review/merge职责保持；共享32credits、96KiB受管/352KiB总源、256KiB用户源、100000-step/120-resume/250ms callback、原整tick/CPU/内存/图片/几何上限均不变。用户最初`/goal 完成AG计划006`授权涵盖同目标/验收/预算/范围的有界设计修订；不追加用户审批或扩限。旧14profile和逐键输入getter的价格、字节、消费语义不变。

## 证据及实际生产路径

现有`game_input::GameInput::fold`输出6bit held/pressed，经Play.dispatch→有界`_ag_char3_input(held,pressed)`写入真实Auto的6个held与6个pressed私有标量；`_ag_char3_tick_begin`给本次pressed绑定tick；公开`ag_action_held/pressed`各1credit，pressed消费对应bit。相机basis是同tick候选的只读输入，Auto自行算方向/动作。

r1反例已实际运行：4次held+1次jump pressed、move、grounded、event_next、visual setter，未读event pair/phase已经error10；不是只做静态加法。日志`t04-scalar-input-budget-counterexample.log` SHA256ac7ae59e56d0c733e8c35fdf8f655ccffb87cd1cd103d2d946aa9c5bf3b0e030。T-02旧probe的“整组合预算完成”结论失效，固定向量不能代替实际键控输入。

权威Spec：auto-lang `docs/specs/auto-graphics/game-character3d.md`（SHA2566dc38be4bfbeaef096ea1a7e15ece37743c9140d2aad14bdba19c2c0b5c26cc4），`runtime-triggers-and-rounds.md`（1d0e37f3a641d33fd6f4408bbcf039f28cc41b90c6765127bd04ad8585eb2e44），`behavior-api.md`（ea0c9f6c6752f015af6e0235728edcd7873f166790f4493780a5d03b1443695e）。代码事实已逐项核实，不用当前实现覆盖原32credit约束。

## r2 唯一设计变更

仅在尚未发布的`autographics.node_animation3d/1`受管单元`api/node_animation3d_v1.at`新增`ag_actions_snapshot() int`（待work实现）。沿用version110/profile21/CLI node-animation3d；旧profile不引入该函数，不改SDK/Host输入wire/character3d原始单元。该helper只是有限读取既有输入，不选动作、不计算运动、不解算碰撞，也不绕过CPU或总源预算。

返回值 `held_mask + 64*unread_pressed_mask`，int域0..4095。两组bit的0..5顺序均为Left、Right、Forward、Back、Jump、Restart。一次成功调用消耗共享池1credit并消费当前tick所有尚未消费的pressed；held不消费。同tick再次调用返回同held+pressed0；调用前被旧`ag_action_pressed(i)`消费的bit不得重新出现，调用后旧pressed getter也不得重放已消费bit。旧held getter仍读同held。app_start按原尚无运行输入的状态返回0；超额/error沿原粘性错误与整帧拒绝路径处理，不另加旁路。

不缓存跨tick/跨Run packet、不引入第二输入路由。Host仍每dispatch送原held/pressed；Pause不调用用户回调，neutral Step的两组mask为0；真实新局/fresh Run重新建VM和输入状态，不继承上次pressed。合法外部held恢复与fresh-press规则沿既有实现。

Auto用整数除法/余数解码后自行算相机相对方向、跳跃速度、状态、phase与yaw。完整活动回调的目标组合为1 snapshot+26 move+1 grounded+1 event_next+1 event_pair+1 event_phase+1 visual=32；setup/initialization可有不同调用，但关闭证据不得靠跨tick省略正常输入、movement、camera、trigger/event字段或visual。输入变化必须实际改变运动和clip/yaw，不能再用固定向量。这个和式只是实现目标，实际通过仍归work实测。

## S-06 关闭合同（owner=work，T-02；关联AC-05/V-05，影响T-04输入路径）

| 状态/反例 | 实际路径与断言 | 覆盖边界 |
|---|---|---|
| 原逐键负控 | 保留已有真实Session回归，原五读取组合仍error10；旧价格不能被偷偷降低 | 不要求旧API不可能的34/36credit组合变绿 |
| 聚合编码与混读 | 6个action bit逐项、0/63极值、held与pressed混合；packet→packet、legacy pressed→packet、packet→legacy pressed；每个已消费pressed只出现一次，held保持 | 代表性域覆盖，不要求4096种笛卡尔组合 |
| 真实完整组合正控 | 当前正式profile的真实Auto/Host/原character，真实4向/跳键输入、相机、move/grounded、event_next/pair/phase、visual setter同callback运行；存在实际事件的tick必须走读取分支并fault0，运动/clip/yaw/phase/相机在同tick发布 | 同原G3前置，不冒充完整关卡；CPU/步数/源/内存仍须既定门 |
| 超额负控与合法恢复 | 确认32credit全路径正控后再添一实际charged调用必须error10；Host保持last-good整帧，Stop/fresh Run恢复 | 不用“无事件所以少读”当作满额证明 |
| neutral与fresh身份 | 真实Pause/neutral Step/fresh Run输入状态；6 held/pressed的neutral=0、无跨Run packet/旧pressed重放 | T-02作输入定向；20轮新局/owner仍归T-05，不提前关闭 |
| 兼容与字节 | 旧14profile的combined/api/bridge hash保持；新profile源仍<=96KiB；旧getter各1credit，未支持的profile不可获得此新API | 只更新006当前身份，历史001..005 identity不重写 |

计划命令：driver包目录`cargo test --locked --offline --test public_node_animation -- --test-threads=1 --nocapture`；已有反例入口可立即重放，新的snapshot/完整组合case由T-02新增并记录精确名称；Host朝向/物理后续case仍T-04。新case尚未运行，不能标绿。先验证实现与上述新路径，再复用明确未变的采样/准入/原件/存储/GPU数值证据；不因新HEAD或managed hash变化全盘重跑未受影响门。

## 证据重绑定与交接

T-01输入和T-03准入/持久化/计账保持完成；r1旧probe的完整预算结论失效。原四clip独立f64顶点/GPU轮廓、法线修复、资源边界/分配/保存证据在其算法/素材/配置不变时可复用；新getter可能影响managed source与真实输入消费，必须新验证，T-06当前source/consumer身份需重新冻结。整计划仍executing、2/10，T-02重开；T-04朝向/物理/阴影、T-05 owner、T-06 consumer、T-07..10 review/merge全部保留。

下一责任work：进入同一clean树ccfe6d9，先同步主检出的r2当前合同文档，追加有限getter与S-06测试，实测修复F-006-W1-INPUT-BUDGET后才恢复T-02完成标记，再继续T-04..06。当前无待用户决定事项；若新设计实测仍达不到原约束，带新反例回new或按真正范围/预算变化请求用户裁定，不能预称成功。
