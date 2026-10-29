# auto-ui 计划管理

2026-10-29：按用户裁定，计划在所属代码/内容仓的主检出管理，源码实施仍在独立worktree。

- 活动合同、进度、复审与归档均在本仓docs/plans/；归档目录为docs/plans/archive/。
- 跨仓计划由主责任仓持一份合同，其他仓只留指引，不复制状态机。
- 导入的原编号保留，不重编号、不回收auto-lang已取号；跨仓引用必须带仓名。
- 后续本仓取号以本地.next-id和活动/归档最大号复核，独占分配、创建不覆盖；不再到auto-lang创建本仓新计划。仓间号码可能相同，以“仓名/PLAN-NNN”区分。
- 四阶段沿用new→work→独立review→merge；一Plan每仓一权威实现树，同树顺序复审，worktree禁止链接、清理先guard。
- 计划簿记允许本仓主检出；源码修改/构建/测试在合同指定worktree，禁止把此迁移当作实现授权。
- AG继续固定旧SDK/独立experimental workspace及最新受影响测试政策；完整AG留里程碑，OS抢焦点自动化暂停。
- 原始非Markdown证据保留本地，不以此次文档迁移复制原始日志/截图/target。

## 当前记录

- [882-autographics-runtime-object-lifecycle.md](882-autographics-runtime-object-lifecycle.md)：drafting r2。
- [883-autographics-trigger-events-and-round-restart.md](883-autographics-trigger-events-and-round-restart.md)：drafting r2。

## 来源保全

- `docs/plans/882-autographics-runtime-object-lifecycle.md` ← `auto-lang/docs/plans/882-autographics-runtime-object-lifecycle.md`；原字节SHA-256 `3610afd8cc9569fb7d7e6ad095c32dd7a58dd78c38d26a5e5e8c48b759964333`。
- `docs/plans/883-autographics-trigger-events-and-round-restart.md` ← `auto-lang/docs/plans/883-autographics-trigger-events-and-round-restart.md`；原字节SHA-256 `5ca339b051af0a373fb3fb98664bd60d46329335cbd4dfe8262b040644e3fef5`。
- `docs/plans/attachments/882/planning/baseline.md` ← `auto-lang/docs/plans/attachments/882/planning/baseline.md`；原字节SHA-256 `8b4bb73d5059ac649046f12deeed0ecf15699ec4837d5e6171150fb7803ee2dc`。

旧Specs/设计的当前存放位置由各Plan明确跨仓引用；本次没有批量迁移历史知识，也没有建立第二套同名canonical Specs。
