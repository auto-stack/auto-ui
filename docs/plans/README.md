# auto-ui 计划管理

本仓独立编号从001开始；本地下一号见`.next-id`。跨仓身份写作`auto-ui/PLAN-NNN`。

## 当前合同

| 本仓号 | 计划 | 旧仓来源 | 状态 |
|---|---|---|---|
| 001 | [运行对象生命周期](001-autographics-runtime-object-lifecycle.md) | auto-lang/PLAN-882 | drafting r3，未执行 |
| 002 | [触发、新局与G11](002-autographics-trigger-events-and-round-restart.md) | auto-lang/PLAN-883 | drafting r3，等本仓001交付 |

下一份新计划为003，之后004…；不能因导入历史号882/883而跳过本地001/002。

## 管理规则

- 本仓主检出`docs/plans/`持唯一活动合同、进度、复审和归档；归档目录为`archive/`。
- 取号只扫描本仓活动/归档的本地Plan；外仓历史来源、附件、重定向不计高水位。独占核`.next-id`和本地最大号后创建，禁止覆盖。
- 跨仓计划由主责任仓持一份合同，参与仓只留定位指引；旧auto-lang编号保留为来源，不再作为本地编号。
- 四阶段new/work/独立review/merge沿用；源码修改和构建在合同指定唯一worktree，Plan簿记在本仓主检出；禁止worktree链接，清理先guard。
- 当前两份仅规划；变更相关测试/全量AG里程碑/OS自动化暂停等原政策保持。
- 旧Specs/设计的现有知识权威仍按Plan跨仓引用，本次没有批量复制历史知识。

r2文档原貌保留于本仓提交`6c7bcdccb08b6320e5920e704990ab6eaa515e41`；r1保留于auto-lang提交`a5d8a4fa8`。编号更正不重写这些历史。
