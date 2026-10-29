---
plan_id: PLAN-003
status: drafting
feature_name: blog-native-vm-run-and-acceptance
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
current_step: 0
total_steps: 7
depends_on: []
supersedes_spec_components: []
new_spec_components: ["docs/specs/runtime/native-vm-app-validation.md"]
touched_goals: []
---

# PLAN-003：Blog原生VM窗口运行与可交互验收前置

## 0. 变更摘要

由[BLOG-004](D:/autostack/auto-blog/docs/plans/004-dual-runtime-acceptance.md)登记的原生UI前置：诊断真实Blog作者窗口的启动/驱动缺口，证明框架的真实原生窗口能接收输入并可重复退出重开；完整Blog业务任务由004闭合。与本仓001/002的AutoGraphics生命周期/触发工作无依赖，不重开它们。不是通用桌面自动化大改，也不替代Blog最终Rust组合门。

## 1. 目标

区分交互桌面环境、CLI/工具交接、MCP驱动、UI runtime与应用.at接缝；仅修实际owner内的最小阻断，提供可由Blog复用的原生窗口启动/交互/清理合同。缺可交互桌面时保留blocked条件，不能以HTTP可达或窗口截图当作可操作通过。

## 2. 架构方案

主改面为auto-ui `crates/auto-ui-tools/src/{run_vm,proc_host}.rs`、bin/auto_ui_tool.rs及auto-ui-runtime动态UI/真实MCP路径（后者具体符号先调查）；不在UI复制应用草稿存储或修改Rust后端合同。Man编排根因由其真实owner另立最小前置，不在UI假接线。

主检出master只持Plan簿记；唯一实现/顺序复审树拟为 `D:/autostack/.wt/ui-003/auto-ui`，分支 `codex/plan-003-dev`。框架同组只读依赖按Cargo真实兄弟布局固定，需依赖worktree时按owner规范建立，禁止junction/symlink。Blog开发/验收仍在 `D:/autostack/auto-blog` 单写者clone，UI计划不写其业务代码。

## 3. 技术栈

现有AutoUI原生VM/iced、auto-ui-tool、proc_host与可用MCP。已读run_vm.rs说明托管时VM主线程hook、未托管时CLI主线程；不得未实测就归因为“环境问题”。隔离使用已确认存在的CLI参数 `auto run -r vm --server vm` 与独立数据根/端口，后端为真VM HTTP；实际Blog作诊断消费者，必要时用T-02约简的非空原生控件/API夹具验证框架正控（不是mock窗口）。Blog最终仍须完整实际应用＋`--server rust`，夹具不能替代。

不硬编码猜测的MCP方法名。先读取实际工具协议/驱动入口，固定探活、窗口身份、动作与状态观察、超时/清理的可重放序列；原生窗口生命周期要有有界等待，失败保留日志。

## 4. 需求分析与背景调查

用户2026-10-29明确授权“建立Blog后续计划，专门补齐Rust后端和原生VM界面的运行与验收，关联必要框架前置计划，并更新总跟踪表”。本轮只编制合同、必要反链与组合pin，不实施代码、不运行全局Cargo门、不宣布运行缺口已关闭。后续执行须按四技能进入work；未指定模型、独立会话、自动续跑或并行agent。

UI基线 `78ee3b70c38697e51181e296eb91e88ea1e761b4`；现有attachments/002/在途材料保持。知识权威沿语言仓 `docs/specs/auto-ui/project.md`（生产owner迁出）、`docs/specs/auto-ui-compiler/project.md`；本仓当前无docs/specs，拟新增的是小范围原生应用验收合同，不搬迁所有历史知识。

- `crates/auto-ui-tools/src/run_vm.rs` — SHA256 `075b17fdf4dc12d883f8a1e78c50d23cfb1d6b286043afda8faf5c72abd6235c`。
- `crates/auto-ui-tools/src/proc_host.rs` — SHA256 `bc3dc50ecdcd54e434a04e698909b0b63b9b90d5184b8e5cd918358ff7cb0d76`。
- `crates/auto-ui-tools/src/bin/auto_ui_tool.rs` — SHA256 `198838b2931d2228ccfb216b28e501d6d6fb23fe10cecd186d0a715d8967fbe3`。
- `crates/auto-ui-runtime/src/ui/dynamic.rs` — SHA256 `62e560deae710a2e878d5c7bab8b15476f3b7930447c1c25a861b47b82d02ac2`。

001历史原生VM四轮不可驱动为调查输入；本次只核对源与CLI --help，没有新跑原生窗口。不能从Notes/Todo成功推导Blog路由/作者窗口必然可用，也不能把Blog使用VM后端的HTTP记录当原生UI证据。

## 5. 详细设计

### 首任务与关闭合同

T-01无Rust交付前置：固定应用/CLI/UI/lang版本、可交互会话条件和驱动协议，分段检查CLI→tool进程→VM主线程→窗口/root→实际输入事件→store/API。拟新增 `docs/plans/attachments/003/native-path-decision.md` 记录实际路径/owner、日志和可重复操作。将“进程启动即退”“驱动发现不到”“事件不进widget”“应用路由不兼容”等分别归因，不用无界重启掩盖。

若当前无框架缺陷：以有效真实窗口正控＋退出重开和故障传播证据形成不需代码修复回执；若有缺陷，T-02先约简，T-03只修UI owner。问题落到Blog作者widget/路由时由BLOG-004 T-03接入，前置回执必须明确哪个是待应用闭合路径，以真实非空原生控件/API正控关闭框架项，不要求BLOG-004先完成。完整Blog任务的失败继续由004阻断，不能随框架pass消失。

### 规范增量

| delta_id | action | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/native-vm-app-validation.md（拟新增，本仓） | 无明确可复验原生应用合同 → 实际启动/驱动/身份/事件到达/失败/退出与环境边界 | HTTP或空窗口不能冒充原生用户任务通过 | AC-01..04 |

已有语言侧UI知识保留权威；若实际改动要求修改其现有语义，先明确增量与owner，不复制一份冲突规范。本轮不发布canonical。

## 6. 测试设计

证据政策：原始日志/编译诊断/截图/探针输出放本地 `docs/plans/attachments/`（不提交非md证据）；Plan内保留命令、退出码、版本、关键结果及本地路径+内容hash摘要。不得以日志里的pass字样代替断言。所有测试使用独立数据目录与明确端口；先检查占用，只关闭本次创建的PID，不批量杀同仓或其他用户进程。

| case_id | owner / AC | 输入/实际路径 | 关闭oracle |
|---|---|---|---|
| U-01 | T-01/02/04，AC-01/02 | 真Blog源码诊断＋T-02最小原生正控，`auto run -r vm --server vm`，可交互桌面/固定工具版本 | 窗口与驱动关联到本次PID/应用会话，正控native root非空、真实控件可操作；Blog接缝缺项单列004，不仅后端健康 |
| U-02 | T-02/04，AC-02 | 经实际协议在原生正控中选择/点击/输入中文、换行，触发真实API并重开 | 事件到达真实widget/store，真实后端/文件与输入对应；仅关闭框架输入链，不能直接POST后截图冒充，也不作为Blog作者任务已过 |
| U-03 | T-02/04，AC-03 | 不可用独立MCP端口/目标已退出/后端不可达，附合法同输入控制 | 有界非零失败、错误归属准确；不能得到0场景pass，合法控制可重复成功 |
| U-04 | T-04/05，AC-04 | 正常关闭/仅终止本次拥有进程，重启同隔离目录 | 会话与驱动释放、不误连前实例或残留PID；重开可再次交互、已确认数据仍在 |

永久驱动/fixture由T-02根据真实协议建立，优先复用现有AutoUI验证入口，不先编造方法/硬编码vnode编号。实现阶段只跑受影响定向检查；冻结代码与最终review后由merge持本仓适用t/tu/ti/tn门，具体触面和原生人工门结果必须记录，不套用语言仓档位。

## 7. 验收标准

| ID | owner_stage | 结果 |
|---|---|---|
| AC-01 | work | 失败位置/真实owner、桌面条件、CLI/UI依赖与驱动协议明确，历史环境结论有新鲜调查；必需条件欠缺时明确阻塞 |
| AC-02 | work | 实际Blog诊断的归属明确，非空原生正控窗口可识别并经实际输入触发真实API；U-01/02事件与后端证据相连，所有UI owner阻断闭合；完整Blog任务仍由004验证 |
| AC-03 | work | U-03故障控制有界非零且正控通过；不能以HTTP健康/单截图/mock/跳过替代原生交互 |
| AC-04 | work | U-04退出/重启与进程/驱动身份闭环，形成Blog可复用的精确版本/命令/操作回执；仅解除UI前置，不声称Rust已通过 |
| AC-05 | review | 冻结实现/依赖与Spec增量，复核真实事件链和负控，记录上下文限制，无未关闭必需环境门 |
| AC-06 | merge | 适用最终门通过后交付UI、Spec/ledger归属与归档/guard清理完成，BLOG-004可消费精确版本；不等待Blog最终Rust双端通过 |

## 8. 执行步骤

- [ ] T-01（owner_stage=work；依赖无；AC-01）：在唯一框架树调查实际启动/驱动路径，产出native-path-decision.md；明确环境缺项与owner。
- [ ] T-02（owner_stage=work；依赖T-01；AC-01..03）：约简原生输入/生命周期问题，按真实协议建立可重复驱动/正反控，记录永久入口命令。
- [ ] T-03（owner_stage=work；依赖T-02；AC-02/03）：只修UI owner确认缺陷；不重现时以有效正控/版本解释关闭，无需空补丁；Man/app问题分别登记消费者待办，禁止越owner。
- [ ] T-04（owner_stage=work；依赖T-03；AC-02..04）：U-01..04实际窗口、事件、失败与退出重开回放；只清理本次进程与隔离数据。
- [ ] T-05（owner_stage=work；依赖T-04；AC-04）：形成Blog可消费回执和拟SD-01，提交实现/测试/摘要，转execution_done；Rust组合不在本前置的通过声明。
- [ ] T-06（owner_stage=review；依赖T-05；AC-05）：同一框架树暂停实现，最终复核真实原生输入/退出/负控与Spec，不通过则回work。
- [ ] T-07（owner_stage=merge；依赖T-06；AC-06）：完成适用本仓门、交付/规范/索引/归档/guard清理，把精确UI版本交BLOG-004接入。

## 9. 复审记录

stage: new | plan_id: PLAN-003 | plan_revision: 1 | outcome: pass（有界原生路径调查就绪；环境/运行能力未验证） | next: work/T-01（另行执行）。7项未勾选，本轮未启动原生窗口或修改UI源码。

## 10. 待澄清事项

无须再次确认原生VM是否必须验收。可交互桌面、实际MCP协议/可用工具、进程退出原因由T-01核实；若当前环境无法提供实际交互，给出具体缺失条件并保持任务未完成，不因是环境问题就放行最终pass。
