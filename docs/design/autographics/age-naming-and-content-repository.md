# Auto Graphics Engine（AGE）：正式命名与内容仓路径

2026-11-04，决议v1。用户确认采用AGE命名；本轮只写设计、路线和更名计划，不实施目录迁移。

## 名称与稳定身份

| 对象 | 本轮决定 |
|---|---|
| 正式名称 | Auto Graphics Engine |
| 简称/路线名 | AGE；与Auto Compile Engine（ACE）并列 |
| 品牌写法 | AutoGraphics Engine |
| 历史名称 | AG、AutoGraphics保留为历史/检索别名；AG-NNN旧计划引用仍有效 |
| 引擎权威仓 | auto-ui；此次命名不搬引擎、不重编包名 |
| 当前内容仓 | `D:/autostack/ag-examples`；Demo在examples/，完整游戏在games/ |
| 计划中的内容仓名 | `D:/autostack/age-examples`，同一个Git仓迁移，不创建替代仓 |
| 历史与协议 | Plan号/plan_id/交付SHA/标签/归档原合同保留；`ag-examples.*`、`autographics.*`等schema/API不是此次重命名对象 |

设计/路线的新正文使用AGE，首次出现写全称。`experimental/autographics-*`、`docs/specs/auto-graphics/`及已有文件名、SDK锁、资源ID、历史收据路径继续保持；不能用全局替换修改机器协议或声称旧二进制因改名得到新认证。历史M1标签映射差异仍按原记录待核，不重打标签或推定等价。

## 迁移时机与编号

唯一目录迁移合同为[ag-examples/020](../../../../ag-examples/docs/plans/020-age-examples-repository-rename.md)，内容侧设计见[更名设计](../../../../ag-examples/docs/design/age-examples-rename.md)。默认在**019＋021实际交付且扩展M2必需门收口后**选择维护窗口，再按后续执行指令实施；020是019/021的后继，不能反过来成为它们的前置。旧M1已经有历史交付，不能据此现在自动迁移。若后续明确选择其它里程碑窗口，再修订020的时机。

迁移保留同一`.git`、refs/交付对象、所有用户工作内容和计划号源。旧`ag-examples/PLAN-NNN`/AG-NNN作为历史身份保留；新责任路径可写`age-examples/PLAN-NNN`，别名指向同一计划，不另建一份合同。本仓001..020不重编号，`.next-id`连续；auto-ui编号独立，互不重置。

实施时只更新当前有效路径、导航、工具和必要项目配置；历史档案中的旧绝对路径保留为当时来源，不保证旧路径仍可点击。禁止以junction/symlink维持旧根，禁止复制重初始化、覆盖目标仓、改写历史/旧锁或顺带更名远端。

## 维护窗口与验收

迁移前清点内容仓全部树、任务/进程、用户未提交与未跟踪内容、目标占用和外部消费者；先在隔离临时副本演练路径/应用项目配置更新和两阶段回退，再独立复审冻结补丁。清理已合自有内容树先guard，确认只剩主检出且无活动owner后，从目录外执行更名。不能迁/删其它owner树或私改应用数据库；现有项目路径更新方式需在020先证，尽量保持项目/聊天关联身份。

迁后从新根核Git历史身份、启动/验证/冷再生/打包/离线A→B/坏件修复和跨仓当前引用，按真实触面保原oracle/预算；不借更名重跑无关全套GPU，也不拿目录检查代替真实消费。失败按记录恢复同一仓根/配置并只回退本计划补丁，禁止reset/clean覆盖用户工作。实际执行仍须work/独立review/merge冻结门。

## 当前路线快照

2026-11-04：016/017/018与019均已交付，Gallery22/22；019只形成原合同边界内本地候选。用户新增明确M2-10：新语法基础UI/2D/3D同应用组合，见[补齐设计](m2-mixed-ui-2d-3d-composition.md)，048/049/AG021仅规划；020已修订r2/0-10，目录仍ag-examples，默认扩展M2收口后的维护窗口。
