# G2整链预算规划合同与基线（PLAN-011 r1）

当前合同见[011](../../../011-autographics-g2-full-loop-budget-feasibility.md) §5..8；本附件仅细化研究边界/指纹，不新增另一份任务表。所有probe/route/测量/图片判据都待实施，本轮没有运行实验。

## 1. 版本和并行写面

auto-ui调查HEAD a448d5716bf3a108def48033383d1c1a1f852894；研究只固定已交付004＋008闭包0261ae9dacf786beb6dceb630be6af267f242625，功能delivery e0fce13810f9e1af28182950d62cfce8722aa30a。SDK pin7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、681文件/17本地包，现有G09消费锁记identity1431、479依赖包/29本地路径边，runner c960febaf741e4a896d48be13ad62c8ebea257523c72160e4b97347726d847c4。执行T-01从实际导出/受检保留副本核映射，不消费005 tip或假定master一直没有新源。

可写范围仅本仓`research/autographics-g2-budget/**`与Plan/报告摘要；现有experimental的源码、API、锁、identity生成器/清单、SDK/根Cargo一律不改。harness用新独立workspace/锁及本Plan独立consumer准备/target目录；固定consumer源只读，准备SDK可以写该副本规定生成目录但不能改受管字节。研究挂接SDK/Host的观察不得变成免费的游戏输入。

与005/006、Gallery各自独立树和准备目录；不等其完成。GPU/重型门按机器单实例串行安排，OS抢焦点/置顶暂停；本轮没有建树/agent或启动任何进程型门。

## 2. 价格证据与研究反例

- 源码`character2d_v1.at::ag_char_move`正常路径依次charge1、v5 move_and_slide（24）、charge1做最终支撑，总26；初始重叠等早退路径可能较少。`ag_char_grounded`另1。旧Spec move25需T-01校准，不选较低旧值证明go。
- G09角色每回调固定读jump edge/left/right共3，正常move26、grounded1、sprite1，共31。它没有敌人移动/金币事件/销毁；不能因为31<32就宣布完整玩法够用。
- `ag_object_move/despawn`各charge1，`ag_event_next`即使absent也1，每个getter另1。自动8对有限检测不收credits，但既有读取延迟与未读丢弃不变；next+pair+despawn不是一次免费操作。
- `_ag_spr_tick_begin`只清ops，不重置当前frame/flip；B可研究无变化不写setter的语义等价性，但不能在需要变帧那步也跳过。保持候选A/B相同几何/输入/状态/图像oracle。

T-02必须列每条完整callback账，不只加平均值：普通跑/idle/rise/fall/land、敌人转向、单币enter/read/despawn、三ID同接触、目标未满足/胜利、敌人/坑失败、Enter新局、Pause/Step/Stop。记录自动事件是N产生/N+1读取；新增额外排队、漏事件、减频更新或跨tick分摊来省额度必须使门红。

## 3. 分母和决策的禁止混淆项

核心是冻结原G2-01..08同一真实Auto程序，不是API独立smoke相加。原三段平台、顶平台、三币、巡逻区间与速度/重力/跳跃/相机沿设计原值；原件路线可达性也须实际输入证据。压力fixture另立，不改原关卡后仍引用原分母。

G2-09仍列在矩阵中：公共声音/HUD缺失，只可记录用户Auto产生的事件/状态意图与所需接口/预算。不提供无操作的音效/HUD“免费API”，不把Rust计数/终端日志当声音或实际HUD。决策允许core通过而requires_capability；full_m2_proven不得true。未来公共服务落地后必须重新跑实际消费者，不可复用本次意图计数冒充通过。

go_core须G2-01..08所有必需分支和尖峰都在当前硬额内；任何一条超额/掉语义应no_go_current_profile并定位。8ms目标met/missed/unmeasured与硬额pass/fail分列。负研究结论描述仅限这两种候选和固定支持域，不证明所有可能Auto程序均无法满足。至少包含合法小正控、越额负控、发生问题的完整输入/源、恢复后重放，不能故意加第33个getter就终止研究。

## 4. 计量/输入/关闭条件待T-02冻结

新harness/verify.py与场景尚不存在。T-02冻结每route<=2400逻辑ticks、两候选、3次release/1debug、20个短寿命循环；用固定dt/种子与明确停止条件，超时/无收据/0selected即失败。若无成功route，性能只报告失败前窗口和隔离合法分支，不输出虚构完整route均值。

状态、图像、资源owner、meter/步数/时限四类证据分别观察：读回/断言时间与正常tick分开；实际Host prepare/publish不能从整tick中删去。关键状态像素oracle与新鲜capture绑定，旧截图冒充、空中run、冻结敌人或漏币必须被分辨。语义/预算或准备失败不能发布一半新帧；Stop与fresh Run重建初态，旧Run句柄拒绝。

合法pin/Editor保留不算泄漏，最后owner释放再查weak；固定消费者的底层不变证据可绑定复用，整链需要新鲜状态/像素/预算观察。原始trace/PNG/consumer/输出JSON放本地plan-artifacts/auto-ui/plan-011，Plan/附件只入库摘要与hash。

## 5. 调查指纹（不是测试结果）

G09图/源固定消费ag-examples交付d1b4f9baf6f83575af4c669efa9b17f507d9003d，不等待该Plan清理记录；不从Gallery正在修改的主检出追图。下表绑定规划观察；T-01消费固定0261闭包逐项核其自身身份，不要求005/006将来仍保持当前工作区文件hash。

| 仓/相对路径 | SHA-256 |
|---|---|
| `auto-ui/experimental/autographics-auto-driver/api/character2d_v1.at` | `9f7c1a6cb42d3bd856ac966224eed6487ca6ad69e6f0b1dd11abbe20c1e81c59` |
| `auto-ui/experimental/autographics-auto-driver/api/runtime_objects_v1.at` | `fe9a8948270d6a8756661830aeb1c6ce7f35c18eb2f3e65d5befaf8595408dc5` |
| `auto-ui/experimental/autographics-auto-driver/api/runtime_triggers_v1.at` | `b2b75b168570d8cf86a470b65511d3923a04d0ae0848caa694a2b619fc0b8ce1` |
| `auto-ui/experimental/autographics-auto-driver/api/sprite2d_v1.at` | `a237dbda0f3e162100b3e28eeb34f35f43863be0db6d5e5649d886e2d53ce29b` |
| `auto-ui/experimental/autographics-auto-driver/src/authoring/play.rs` | `4371603540d7f460aa23c6334657f738c1f9dcfe9d41052fd96455a905261bd0` |
| `auto-ui/experimental/autographics-auto-driver/Cargo.lock` | `2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db` |
| `auto-ui/experimental/autographics-prototype/Cargo.lock` | `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf` |
| `ag-examples/examples/g09-sprites-and-frames/character/main.at` | `c051989bfaf353ec870a2e7c92d173103988a2741a60c3fbdb7179c5c26adf06` |
| `ag-examples/checks/g09/gallery_runtime.rs` | `9d7ce60243e3b373f689c458a1f08bd0c5229b5ba55aaf76f908a2e89a3bd7a8` |
| `ag-examples/engine-locks/auto-ui-004.json` | `f8b3bf2047bc0e7d515075cbb27d12d302436dbb938a747879b8fd44e48d5038` |
| `auto-lang/docs/design/autographics/m2-game-sample-matrix.md` | `a9b0b3f2a932ce60411492d521de10e565dd7c91a3ad9f46ea710bc8569306f8` |
| `auto-lang/docs/specs/auto-graphics/game-actions-and-character2d.md` | `e9a4598cb0ced118a40b2f701163a511efd0594dceaa84f47fb6b81a31c493ec` |
