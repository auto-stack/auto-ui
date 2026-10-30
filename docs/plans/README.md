# auto-ui 计划管理

本仓独立编号从001开始；本地下一号见`.next-id`。跨仓身份写作`auto-ui/PLAN-NNN`。

## 当前合同

| 本仓号 | 计划 | 旧仓来源 | 状态 |
|---|---|---|---|
| 001 | [运行对象生命周期](archived/001-autographics-runtime-object-lifecycle.md) | auto-lang/PLAN-882 | archived r4；代码交付537122b，独立收据见归档 |
| 002 | [触发与新局（引擎）](archived/002-autographics-trigger-events-and-round-restart.md) | auto-lang/PLAN-883引擎部分 | archived r5；交付 0ee5ad0，SD/ledger 同步（auto-lang v130） |
| 003 | [Blog原生VM运行与可交互验收前置](archived/003-blog-native-vm-run-and-acceptance.md) | BLOG-004消费前置，非AG工作 | archived/r2; AutoUI prerequisite delivered a2ea08c; BLOG-004 owns full Blog acceptance |
| 004 | [公共图片与2D精灵帧](archived/004-autographics-public-images-and-sprite-frames.md) | AG M2表现 | archived；交付 e0fce13（reviewed 0e6e361），SD/ledger 沉淀 auto-lang v134 |
| 005 | [3D纹理与UV](archived/005-autographics-textured-models-and-uv.md) | AG M2表现 | archived r3；交付351919c，独立R3通过，SD/ledger v141已同步 |
| 006 | [节点动画与视觉朝向](archived/006-autographics-node-animation-and-visual-facing.md) | AG M2表现 | archived/r2；R2 pass；交付c681663，ledger143 |
| 011 | [2D完整玩法整链预算验证](archived/011-autographics-g2-full-loop-budget-feasibility.md) | AG独立研究 | archived/r2；两候选no_go_current_profile（33/32），交付c4143af；知识/ledger v145，完整M2未证实 |
| 012 | [公共音频与接受后一次播放](archived/012-autographics-public-audio-and-accepted-cues.md) | AG公共服务 | archived/r1；45case与完整fresh通过，公共音频交付；G12a内容另仓 |
| 013 | [公共HUD与共同呈现](013-autographics-public-hud-and-presentation-state.md) | AG公共服务 | drafting/r1；硬等012实际交付；G12b内容另仓 |
| 015 | [VM路由页动态加载与原生Blog闭环](archived/015-vm-route-page-dynamic-load.md) | BLOG-004消费前置 | archived/r2；交付df39593；native22/22，规范/ledger148；清理回执见计划 |

下一份本地新计划为016，以独占取号时`.next-id`与本地活动/归档最大号重核；不能因外仓号882/883跳号。003是Blog路线，004..006是AG表现路线，状态以各自交付收据为准；011预算研究已交付，完整游戏仍未证实。设计与并行/后继入口见[本仓设计](../design/autographics/m2-presentation-and-resources.md)。

## 债务/候选

未立项的候选功能与债见 [debts.md](debts.md)（当前：DEBT-001 autoui MCP 批量动作）。

## 管理规则

- 本仓主检出`docs/plans/`持唯一活动合同、进度、复审和归档；归档目录以已交付001/002的实际约定为`archived/`（取号兼扫历史`archive/`，避免遗漏）。
- 取号只扫描本仓活动/归档的本地Plan；外仓历史来源、附件、重定向不计高水位。独占核`.next-id`和本地最大号后创建，禁止覆盖。
- 跨仓计划由主责任仓持一份合同，参与仓只留定位指引；旧auto-lang编号保留为来源，不再作为本地编号。
- 四阶段new/work/独立review/merge沿用；源码修改和构建在合同指定唯一worktree，Plan簿记在本仓主检出；禁止worktree链接，清理先guard。
- 001/002 已归档（002 交付 0ee5ad0）；变更相关测试/全量AG里程碑/OS自动化暂停等原政策保持。
- 旧Specs/设计的现有知识权威仍按Plan跨仓引用，本次没有批量复制历史知识。

r2文档原貌保留于本仓提交`6c7bcdccb08b6320e5920e704990ab6eaa515e41`；r1保留于auto-lang提交`a5d8a4fa8`。编号更正不重写这些历史。

用户批准按仓拆分（2026-10-29）：002仅auto-ui引擎/夹具/消费证据；[ag-examples/001](../../../ag-examples/docs/plans/archived/001-autographics-gallery-runtime-objects.md)独立承接G11，硬等002实际交付。两份分别review/merge，无反向依赖。
