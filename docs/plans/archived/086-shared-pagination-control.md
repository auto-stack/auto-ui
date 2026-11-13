---
plan_id: PLAN-086
status: archived
feature_name: Pagination 统一受控组件与消费者迁移
author: [Codex]
created_at: 2026-11-13T00:00:00Z
updated_at: 2026-11-14
plan_revision: 2
current_step: 6
total_steps: 8
supersedes_spec_components:
  - docs/specs/ui/file-dialog.md
new_spec_components:
  - docs/specs/ui/pagination.md
touched_goals: []
---

# PLAN-086 Pagination 统一受控组件与消费者迁移

## 0. 变更摘要

复用现有 pagination 组件族和 FileDialog/026 Database 的分页交互，新增 `packages/pagination/` 的 `PaginationControl` 受控组合组件，统一页码、边界、每页数量、MCP、主题和单页隐藏。迁移两个真实消费者，为 PLAN-084 的新 FileDialog 模式提供稳定依赖。

路线：[组件库扩展 strategy / CL-03](../../design/autoui/component-library-extension-strategy.md)。本轮只有计划和设计，不创建组件或声称测试已通过。

2026-11-14视觉补充：本次新增参考图/视觉case，原执行者已记录的T-01探针与后续进度保留；进入当前未完成任务时接入本次样式参考。r2未改变其已验证的API/整数域/依赖机制，原证据可用于对应机制，不替代新增V-08视觉证明；实现或输入变化仍须复核受影响证据。原拟树/未创建文字及上句“本轮只有计划”是初始背景，已有原owner实施树须沿用，不能因补图另建第二棵树；实际身份与进度以当前执行记录为准。

## 1. 目标

公开 1-based 页码；同样的数据在 Windows/Linux native VM 与 Vue 中呈现相同页数、边界和回调。支持首/上/下/末页、当前页/总页、可配置每页数量、可选页容量切换和紧凑布局；默认在空结果/只有一页时隐藏整个分页组。

文件窗保留隐藏文件选项与分页同一行、图标按钮带名称/提示、1003 条目全部可达；数据库保留 10/25/50 每页数量、精确行范围及业务查询。MCP 通过普通按钮/选择控件操作，无人值守验证实际页面结果而非只看计数。

非目标：替组件发数据库/网络请求、通用数据源或缓存、无限滚动、任意页数全部渲染、URL路由/业务持久化、全套 shadcn pagination 子元素重写、macOS 支持认证。

## 2. 架构方案

输入 page/pageSize/total → 纯派生 effectivePage/pageCount/disabled 状态 → 普通按钮及可选页容量控件 → changed 请求 → 应用更新受控值和实际数据。分页组件不持有第二份 page，不切片文件列表或执行 SQL。

内置 `pagination` 已有 page/total/per_page/sibling_count 等声明、Vue component 映射；注册优先级 Builtin > Local > Package，同名包不能接管。因此使用 **PaginationControl** 作为正式高层入口，保留内置低层族及原解析兼容。底层若能复用就复用；若 native 不支持对应低层元素，则用现有 Button/Text/选择控件组合并共用受控算法，不为此宣称全子元素族已 native 完整实现。

T-01 在真实导入/生成/运行链上核对 schema 元数据和实际 converter，不只根据 iced:none 推断运行状态。对缺失的局部低层行为采用最小可验证适配；改变 builtin 解析优先级不在本计划内。

本计划先完成现有 FileDialog 打开模式分页迁移；084 在本计划交付后重基线并接新模式。TreeView 独立，无交付依赖。

## 3. 技术栈

沿用 `.at` 包、既有标准控件/主题、compiler 包生成、VM/Iced、Vue、MCP 和 Python 产品验证。实现正向兼容层，不引入新 UI 框架、包解析器或另一种通用 state 机制。

## 4. 需求分析与背景调查

### 授权和证据

用户授权为首批三个组件分别建立实施计划、复用现有实现、跨平台/MCP/无人值守，并更新 strategy 和设计。本轮允许 auto-ui 文档修改；没有实施/发布、Reader/语言仓写入、子会话或自动续跑授权，未指定预算与模型。

观察基线 auto-ui `4dd64b5fc5fc0919bfe686115c75a5d8c7ac06b1`；开工重新核对 HEAD、相关脏文件和来源摘要，保留其他任务 WIP。

| 来源 | 当前事实 / 待补差额 |
|---|---|
| `crates/auto-ui-compiler/schema/aura.at`，SHA256 `948506F7A2FAE32CBACC0495649F916419CB4DB182F603711CC61062CE49DD25` | 已声明 pagination 及 item/list/ellipsis/previous/next/first/last 等族；page 是 state_ref，total/per_page 已有字段；不能列为不存在的新组件 |
| `crates/auto-ui-compiler/src/aura/schema.rs` | 对应实际 schema 构造存在；元数据和生成/运行实现需同时核验 |
| `crates/auto-ui-compiler/src/ui_gen/widget/component_registry.rs` | 同名内置禁止被 local/package 遮蔽；高层入口须用 PaginationControl |
| `packages/file-dialog/file_dialog.at` | 当前自绘 PreviousPage/NextPage，pageCount<=1不渲染分页组；UI页码由host投影 |
| `crates/auto-ui-runtime/src/ui/file_dialog/model.rs` | 内部 page 为0-based、PAGE_SIZE=100，列表和页数由唯一 model 持有；不要移动 IO/选择 owner 到分页包 |
| `examples/ui/026-database/src/front/app.at` | page 为1-based；手工首/上/下/末、10/25/50选择；注释明示等待pagination API对齐；查询/排序/搜索重置仍归应用 |
| `docs/specs/ui/file-dialog.md` | 底部同排、单页整个组隐藏、1003项不截尾、MCP旧绑定拒绝等已交规则必须保留 |
| `crates/auto-ui-tools/tests/file_dialog_component.rs`、`tests/product/verify_file_dialog.py` | 官方包生成和真实原生消费入口可复用并扩展依赖闭包测试 |
| `docs/specs/runtime/native-vm-app-validation.md` / strategy Q-01 | 真实窗口与生命周期证据必需；文字缺失问题未定位，快照文本完整不能代替可见效果 |

目前无正式 `docs/specs/ui/pagination.md`；新合同交付时补建。现有 UI 编译/生成与 blueprint 单源边界沿当前 Specs，不写语言仓补业务工具。

## 5. 详细设计

### 公共接口与数学语义

拟议高层 props：`page=1`、`pageSize=10`、`total=0`、`hideWhenSingle=true`、`compact=false`、`disabled=false`、`showPageSize=false`、`pageSizes=[10,25,50]`。事件 `changed(str)`，在 `.at` 绑定为 `onchanged`，携带 JSON `{"page":2,"pageSize":25}`；`error(str)` 提供具名配置错误。T-01 核对现有 msg/包机制并冻结可运行声明，语义不能因便于生成而削弱。

| 规则 | 首版语义 |
|---|---|
| 页数 | 有效配置下 `pageCount=max(1,ceil(total/pageSize))`，计算防溢出；total表示条目数，不是页数，total=0是正常空结果 |
| 页码 | 公共1-based；渲染值 clamp 到 `[1,pageCount]`。受控输入因数据变化越界时只投影有效页，不自动发 changed 或递归修改父状态；消费方在数据变化时负责更新真实 page/切片 |
| 非法配置 | pageSize<=0、total<0、页容量候选含非正数属于具名配置错误；无除零/负offset，导航禁用、错误可观察，同一错误配置不因反复render刷回调。正整数取值/JSON数值范围由T-01按现有int表示确认并覆盖边界 |
| 用户动作 | 根据当前有效配置生成目标；点击已在的页不发事件；disabled、首/末边界不发事件。一个操作最多一个 changed；组件不先改受控 page 再发请求 |
| 页容量切换 | showPageSize开启时可见选项，去重并保持给定顺序；当前pageSize不在候选时仍明确显示其值。选新值只发一次 `{page:1,pageSize:N}`，相同值无事件；消费者一次性接收并更新数据 |
| 单页隐藏 | hideWhenSingle=true且pageCount<=1时整个分页组不入可操作树，包括可选容量控件；需要空/单页仍可改容量的消费者显式传false。错误配置不被单页隐藏掩盖 |
| 布局 | compact显示上/下+当前/总页；常规模式加首/末。按钮用标准图标并有可访问名称/tooltip，首末等禁用态可见；不需要渲染所有页号或增加跳页输入即可完成首版 |
| 按键 | Tab 到启用控件，Enter/Space 与点击同义；不注册全局左右快捷键抢占编辑器/输入框；重绘保留仍在树中的焦点，否则转到当前组可用控件或其容器 |

### 身份、消费者和发布

MCP 能读到当前页、总页、总量、每页数量、按钮禁用态及名称；找到当前控件后操作，旧页/旧总量/旧实例的 action 必须在消费时重验绑定，不得通过沿用数组位置请求另一页。两个分页器可独立控制同页不同数据集；处理延迟数据结果由应用请求generation管理。

新增 `packages/pagination/{package.at,pac.at,pagination_control.at,README.md}`。组件只做共享交互，两个消费者的业务 page owner 不变：

- FileDialog 适配层公共page=`model.page+1`，请求回model.page=`event.page-1`，pageSize固定100，不显示容量切换。model继续计算合法切片/过滤重置/选择清空；底栏仅将现有手绘分页替换为 PaginationControl，不改其他布局。把 FileDialog 的原 fd_command 分页接线适配到请求页，不能只更新显示页码。
- 026 Database使用现有1-based page、totalRecords和pageSize（精确字段由T-01核对），开启容量切换、hideWhenSingle=false。搜索/排序/总量缩小由应用协调页码和SQL/行切片，不让投影页码与实际数据偏离。`showingText`仍是应用的数据结果摘要。
- FileDialog 包新依赖必须在 source import、pac/path dependency、Vue生成和脱离源根的vendor布局中可解析；外部消费需更新完整文件集合/依赖快照及工具，不只同步原三文件。该变更不直接写Reader；提供消费者升级说明和可移根fixture，Reader业务门仍由Reader持有。

不要求设计新包管理器。若现有包导入无法携带依赖闭包，T-01先给最小可运行布局和明确能力边界；不得在主源下内嵌第二份分页算法冒充复用，也不得将问题推到084再通过本计划。

### 视觉参考与实现边界（r2）

2026-11-14按用户要求补充[文件导航视觉指南](../../design/autoui/file-navigation-and-interaction-visual-guide.md)及[GPT参考图](../../design/autoui/assets/file-navigation-084-086-reference-v1.png)，使用右下086区域的多页/首面/单页/空结果四状态，不要求一个页面同时显示四个分页器。

默认有效单页/空结果时整个组不入树，含页码/箭头/容量控件；图中18 items是应用摘要，“Pagination hidden…”是说明文字，不是控件残留。非法配置仍显示错误，不得被隐藏规则掩盖。首面first/previous禁用，多页工具栏保持一行；图的25 per page只属于可变容量消费者，FileDialog仍固定100且不显示容量选择。026显式hideWhenSingle=false的语义不改变。

沿现有主题/命名icon/键盘和MCP状态，不因图新增全部页号、跳页框或业务查询。对照真实数据/回调，不只看图中页码文字。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/pagination.md | 无统一高层合同 → PaginationControl props/事件、1-based、空态/边界/页容量/隐藏与原builtin兼容 | 共享交互而非复制代码 | AC-01, AC-02, AC-03 |
| SD-02 | add | docs/specs/ui/pagination.md | 无独立交付门 → Win/Linux native+Vue、MCP身份、移根消费、视觉与无人值守 | 同义可测 | AC-04, AC-05, AC-06 |
| SD-03 | modify | docs/specs/ui/file-dialog.md | 自持分页UI、单包快照说明 → 采用受控共享分页，保留100大小/单页隐藏，并记录完整依赖闭包 | 不破坏文件窗及外部消费 | AC-05, AC-06 |

本轮不修改 canonical Specs。拟实施树 `D:/autostack/.wt/ui-086/auto-ui`、分支 `codex/ui-086-pagination`，尚未创建；实现、自检、独立复审共用该树顺序执行，Plan记账在主检出。

## 6. 测试设计

拟新增 `crates/auto-ui-tools/tests/pagination_component.rs`、`examples/pagination/` 和 `tests/product/verify_pagination.py`，由 T-04/05 使其可运行；CLI 拟为 `--auto-bin <auto-cli> --ui-tool-bin <built-ui-tool> --out-dir <isolated-dir> --backend native|vue`。双端必须消费实际 `.at` 生成/实例化产物，不能只测试脚本重写的ceil公式。原 FileDialog gate 和生成目标完整保留。

| Case ID | AC / 规则 | 入口与输入 | 区分性断言 / 预期 | 边界及 work owner |
|---|---|---|---|---|
| V-01 | AC-01,02 数值 | 实际包派生/事件路径：total=0/1/100/101/1003、pageSize=100、负total、零/负pageSize、受控page越界、整数边界 | 页数分别1/1/1/2/11；空态正常；非法配置具名错误无除零；越界渲染clamp、不自发changed；大数无溢出 | T-01定义实际int范围；T-02/05 |
| V-02 | AC-03 事件 | 官方夹具通过点击/键盘/MCP首上下一末、重复点、禁用、10→25→25 | 一次操作最多一条请求；边界/重复/disabled零事件；换容量恰好 `{page:1,pageSize:25}`；父未接受时不自改 | native/Vue实际回调；T-02/03/05 |
| V-03 | AC-04 身份时序 | 双实例、改变total/page后使用旧ID、卸载重开；请求时数据量变化 | 旧action拒绝、新正例可用、不串页；异常不改另一实例；当前语义状态准确 | native MCP + Vue事件；T-03/05 |
| V-04 | AC-05 文件窗 | 真实FileDialog：0/1/100/101/1003项、过滤/隐藏改变total、选第11页最后文件 | 单页整个组不存在；101项尾页1项，1003项尾页3项且最后可选；显示第n页与真实行/回调路径对应，无0/1偏移 | 原model及native gate；T-04/05 |
| V-05 | AC-05 数据库 | 026真实查询：25条每页10→第3页→改25，排序/过滤后总量减少 | 第3页确切5条ID；改25请求page1并实际返回全部25；按真实结果核对行ID/offset，不只看页码文字 | native+Vue真实既有后端；T-04/05 |
| V-06 | AC-01,06 导入/发布 | 新包+原builtin使用对照、FileDialog带依赖vendor、第二绝对根、无源根回退 | 官方入口真实解析/生成/运行，builtin无shadow，完整依赖集合可用；删必要依赖具名失败，不捡另一检出旧组件 | tools生成+运行；T-01/04/05 |
| V-07 | AC-04,06 平台视觉 | Win/Linux native、Vue，浅深/窄窗/长中文、隐藏时焦点、退出重开 | 实际状态行不挤两行、按钮名称/提示与禁用清楚、文字无缺失；进程/端口回收再启动成功 | Q-01受影响项闭环；T-05/06 |
| V-08 | AC-02,03,04,06 r2视觉参考 | 按086四状态切换total/page、hideWhenSingle和容量选择开关；真实文件窗/026消费 | 多页单行、边界禁用；默认单页/空结果的整个组及其可操作节点不存在；false覆盖仍可显示，非法配置错误可见；附浅深/窄窗截图和实际数据读回 | work，T-03/05/06；Win/Linux native及Vue |

V-04、V-05 是关键跨层 oracle：只改变页码 label 或上/下按钮但不改变查询/文件行的实现必须失败。测试生成合成数据并通过普通控件改变页/容量，读API/结果日志可以作只读oracle，不可直接调用业务handler代替操作。

拟执行入口：

```powershell
cargo test -p auto-ui-tools --test pagination_component --test file_dialog_component
cargo test -p auto-ui-runtime --test file_dialog --test file_dialog_model
python tests/product/verify_pagination.py --auto-bin <auto-cli> --ui-tool-bin <built-ui-tool> --out-dir <isolated-native-dir> --backend native
python tests/product/verify_pagination.py --auto-bin <auto-cli> --ui-tool-bin <built-ui-tool> --out-dir <isolated-vue-dir> --backend vue
python tests/product/verify_file_dialog.py --auto-bin <auto-cli> --ui-tool-bin <built-ui-tool> --out-dir <isolated-file-dialog-dir>
```

新脚本的真实 Vue runner、026启动/读回和现有native gate复用由T-01决定、T-05文档化；本轮不执行。Windows/Linux分列证据，缺runner/窗口/回调/读回均不记通过。组件测试不替代消费者门；不做无关全仓多轮测试。

实施者自检后由后续独立上下文在同一树做最终review，本轮不创建会话或分配模型。源码、CLI/tool、语言依赖、浏览器版本和证据摘要必须可核验，review期间暂停实现写入。

## 7. 验收标准

| ID | owner_stage | 可观察结果 / 关闭方法 |
|---|---|---|
| AC-01 | work | 正式高层入口与内置族共存，真实包导入/生成、移根和依赖拒绝控制通过，未改变Builtin优先级；V-01/06 |
| AC-02 | work | §5页数/空态/非法配置/越界投影/整数边界正确，不隐藏配置错误或自发回调循环；V-01 |
| AC-03 | work | 点击/键盘/容量切换产生准确单次请求，受控值/disabled和同页幂等符合§5；V-02 |
| AC-04 | work | MCP状态可读且普通控件可操作；双实例/旧action/焦点/卸载重开无串扰；V-03/07 |
| AC-05 | work | FileDialog与026真正使用同一包，显示页与真实数据/查询一致；1003项尾项可选，单页隐藏完整；V-04/05 |
| AC-06 | work | Win/Linux native、Vue和两消费者门、依赖闭包、浅深/窄窗视觉证据齐全，受影响Q-01无未闭项；原门和V-06/07/08及视觉指南 |
| AC-07 | review | 独立复核 AC-01…06及SD、跨层反例和依赖身份，记录reviewed commit及各轨结果，无必需门缺口 |
| AC-08 | merge | 新分页Spec及FileDialog增量沉淀、合并/归档/strategy与084交接版本准确，实际清理收据完成 |

## 8. 执行步骤

6/8 完成（T-01…T-06 ✅，work 全部完成 → execution_done）。T-07 review / T-08 merge 待进入。

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证入口 / 预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | 核对builtin实际转换/生成、包依赖闭包、两消费者字段/事件/真实数据oracle、整数表示/runner/Q-01；新 `docs/reviews/plan086/decisions.md` 固定API与真实导入/两端运行探针 | 跟踪解析→派生→UI→消费；小型实跑、依赖缺失对照和源摘要，给出可行方案；非靠静态元数据推定支持 | 01…06 |
| T-01 [x] ✅ 已完成 | work / 无 | decisions.md 冻结API+机制@worktree `8e8eb85f`；探针实证：builtin族native全系空渲染+vue reka-ui映射；包→包传递依赖双轨可用（vendor+pac.at+`dep`物化）；双实例/嵌套实例独立emit路由+JSON载荷；prop签名须`name: type = default`；msg引号变体才能vue emit重写；i32整数域；缺依赖build轨具名失败/native轨静默缺席；Win runner+Python playwright vue runner实证，WSL待T-05 | 探针工件 `D:/autostack/plan-artifacts/plan-086/probes/`（P1/P2/P2-neg日志） | 01…06 |
| T-02 | work / T-01 | 新 `packages/pagination/`，受控派生/changed/error、页容量/单页隐藏，复用低层或标准控件；单一交互算法 | V-01/02真实包目标，整数/错误/幂等/受控断言通过 | 01…03 |
| T-02 [x] ✅ 已完成 | work / T-01 | `packages/pagination/`@worktree `7eb39915`；P3探针12实例双轨全过：单页隐藏×3、clamp 99→5/5、1003→11页、具名错误×3（pageSize0/total<0状态绑定/pageSizes含0）、容量去重[10,10,25,50]→3钮、25→`{page:1,pageSize:25}`单条、当前容量钮禁用拒按、compact无首末、disabled拒按；vue侧vue-tsc+vite build+playwright断言同过；API偏差记录（decisions.md）：错误走可见具名文本（native Init内emit不派发，P-probe实证），负数字面量prop在VM轨不生效须状态绑定（已记录） | P3日志 `plan-artifacts/plan-086/probes/p3-*.log` | 01…03 |
| T-03 | work / T-02 | 包视觉、键盘、MCP语义/修订绑定及必要最小通用适配；双实例focus | V-02/03及当前action正例；无builtin shadow、旧绑定拒绝 | 03,04 |
| T-03 [x] ✅ 已完成 | work / T-02 | 组件改computed派生（PLAN-536挂载语义下Init派生不响应受控prop变化——P3实证5/5页滞留）@worktree `54712797`；**最小通用适配**：eval.rs Bina补Sub/Mul/Div/Mod臂（视图/computed快速路径缺臂致ceil静默None）；双端整除一致公式`(total-total%ps)/ps+进位`（JS float 11.03实录定案）；错误文本静态具名（vue computed f-string缺口）；r2视觉对照参考图右下区：单行布局/首末禁用/单页整组隐藏/隐藏态stale拒绝`VNode not found`实证；键盘=后台窗不触发（file-dialog门同基础设施过，产品门T-05按同条件复验）；lib套件52失败vs master基线48（桌面环境类噪声，10新6复原，与eval/computed无关名）；decisions.md §10 | P3日志+lib-tests-serial/master.log `plan-artifacts/plan-086/probes/` | 03,04 |
| T-04 | work / T-03 | 新 `examples/pagination/`，迁移 `packages/file-dialog/{file_dialog.at,pac.at,package.at}` 与026 `app.at`及实际依赖；host分页接线按T-01定位最小改动；补完整vendor示例 | V-04/05/06：真实页数据、回调尾项、SQL行ID/offset、移根与缺依赖对照正确 | 01,05 |
| T-04 [x] ✅ 已完成 | work / T-03 | @worktree `a8ec40ec`：file-dialog底栏→PaginationControl（displayPage→fd_command page op，hide语义组件承担）；026七手写msg→统一PageChanged（filteredCount驱动，行切片/搜索归应用）；examples/pagination四状态+1003实切片oracle（next→item101..200、容量10→25→41/41尾item1003）；tools夹具双包vendor闭包+dep根声明；026 T6重接线+可绑定端口选择器；门：fd runtime 8+10✓、fd tools 3✓、026 mcp 25✓×2 | 同左+探针日志 | 01,05 |
| T-05 | work / T-04 | 新 tools/test和产品脚本，扩展原 `verify_file_dialog.py` 与 tools生成目标，执行§6适用门 | §6命令 Win/Linux/Vue各轨有实际结果、负例和退出重开；单页组树断言通过；含r2 V-08的参考区域对照/真实状态截图 | 01…06 |
| T-05 [x] ✅ 已完成 | work / T-04 | @worktree `9700f1ee`：verify_pagination.py native+vue双轨门（Win native全门×2+窄窗腿+vue腿全PASS：四状态/受控回路实切片/容量25→41页/disabled/stale拒绝/浅深窄窗截图sha256/NVM-05退出0+端口回收+重开）；tools pagination_component.rs 2/2（双生成器+float-safe断言+vendor闭包+strict具名失败）；file-dialog原门全量PASS（1003项流携迁移包）；026 mcp 25/25×2；**最小编译器适配**：shadcn disabled表达式→`:disabled`绑定（17臂，表达式此前静默丢弃）；**Linux腿未运行**（WSL无rust/node工具链，显式边界不记通过） | evidence.md 门索引 `docs/reviews/plan086/evidence.md`；探针 `plan-artifacts/plan-086/` | 01…06 |
| T-06 | work / T-05 | 新包README、消费者依赖升级说明、`docs/reviews/plan086/`证据、最终SD提案、strategy实施摘要；包README链接本Plan视觉指南和对应区域 | 源/依赖闭包/工具摘要、平台结果及Q-01闭环可读回；缺门保持executing | 06 |
| T-06 [x] ✅ 已完成 | work / T-05 | README（props/事件表+闭包三件套+边界，链接视觉指南§2.3与本Plan）+ evidence.md（门索引/提交链/V-08截图清单/SD终版/7项边界）+ SD-01…03终版核对（本轮不写canonical Specs）；strategy实施摘要经decisions/evidence记录——strategy文档系r2会话未提交WIP，按所有权纪律不改其文件，落地时其CL-03可回链本Plan | `packages/pagination/README.md`、`docs/reviews/plan086/evidence.md` | 06 |
| T-07 | review / T-06 | 同一树独立验收，检查显示页/实际数据跨层oracle及旧族兼容，审SD | reviewed commit绑定AC结论；未过回同树修复，不能移给084 | 07 |
| T-08 | merge / T-07 pass | 新 `docs/specs/ui/pagination.md`、FileDialog增量、合并归档/strategy交付链接、实际清理；向084记录可消费提交/闭包 | 最终源码/Spec/证据/树状态核对；084依赖具名解除，不写派生索引/ledger | 08 |

## 9. 复审记录

- 2026-11-13：stage: new；PLAN-086 r1；outcome: pass（计划合同可交接，非功能验收）；next: work，在后续明确启动时从T-01进入。
- 新增T-01…08、AC-01…08、SD-01…03；drafting r1、0/8。未实施、未运行拟议门、未写canonical Specs，未同步Reader副本。

2026-11-14：stage: new；PLAN-086 r2；outcome: pass（视觉交接可供执行，非产品通过）；next: work，原执行者从当前未完成任务接续。新增§5图/指南引用与V-08，AC-06/T-05/06纳入；原受控协议、两个消费者、依赖与8任务不变。本次补图时已观察到T-01完成/1-8及原owner探针记录，未修改其progress/status或重记任务；图片任务只补文档/资产，不代其实施验收。


2026-11-14：stage: work；plan_id: PLAN-086；plan_revision: r2；outcome: pass（work-owned T-01…T-06 全完成，execution_done）；worktree: D:/autostack/.wt/ui-086/auto-ui；branch: codex/ui-086-pagination；base_commit: 93e6b924；code_commit: 9700f1ee+T-06（README/evidence）；dependency_revisions: .wt/ui-086/{auto-lang,auto-web,auto-down,auto-man}=master 零改动（补丁版 auto.exe/auto-ui-tool.exe 组内构建）；task_ids: T-01…T-06；evidence: docs/reviews/plan086/{decisions,evidence}.md + plan-artifacts/plan-086/（门日志/截图/report.json）；blockers: 无（Linux 腿未运行为显式环境边界，非阻塞 handoff）；next_responsible_role: review（T-07 同树独立验收）。
Readiness：AC-01（闭包/负例/builtin不冲突，tools 2/2+3/3）✓；AC-02（P3 矩阵+双端整除公式）✓；AC-03（单条changed/幂等/disabled，native+vue门）✓；AC-04（MCP可读可操作、双实例/嵌套实例隔离、卸载后旧id拒、重开新action）✓；AC-05（file-dialog原门全量过=1003流跨层oracle；026 mcp 25/25×2）✓；AC-06（Win native+vue+V-08浅深窄窗截图；Linux腿显式未运行）部分→显式边界；AC-07/08 归 review/merge。无未决 open finding；统一回归门 pending，owner=review/merge。


2026-11-14：stage: review；review_scope: final；reviewer_identity/limitations: 本实现会话自审（无独立会话授权），按 auto-plan-review 纪律以工件重构判定——全部门在新 HEAD 独立重跑，未采信执行摘要；plan_id: PLAN-086；plan_revision: r2；outcome: pass（reviewed，含显式 pending closeout gate）；worktree: D:/autostack/.wt/ui-086/auto-ui；branch: codex/ui-086-pagination；reviewed_commit: c2fcffaaf03fce47f1380aee96bfd8363f6f9a2f；base_commit: 93e6b924；dependency_revisions: .wt/ui-086 组=master 零改动；spec_inputs: docs/specs/ui/file-dialog.md（现行 100条/1003 夹具规则保留核对✓）、docs/specs/ui/pagination.md 尚不存在（merge 落 SD-01/02）。
acceptance_results：AC-01 pass（tools 2/2+3/3 独立重跑；schema 零 diff=Builtin 优先级未动；PaginationControl 无 fold 冲突）；AC-02 pass（P3 矩阵 + 双端整除公式 + 具名错误三例）；AC-03 pass（单条 changed/幂等/disabled/容量，双轨）；AC-04 pass（MCP 快照/操作；双实例+嵌套实例隔离；卸载旧 id `VNode not found`；重开新 action）；AC-05 pass（file-dialog 原产品门全量 pass=1003 项 11 页跨层 oracle；026 mcp 25/25）；AC-06 partial→pass-with-pending（Win native+vue+V-08 浅深窄窗截图 sha256 齐；Linux 腿显式未运行）；AC-07 本记录；AC-08 归 merge。
pending_closeout_gates：Linux native 腿（环境无 rust/node 工具链——WSL Ubuntu-22.04 实测；门脚本已带 X11 适配，runner 就绪原样可跑）+ 统一回归（merge-owned）。
findings：R-1（dirty 双文件已补提交，R-1 fix 于 reviewed HEAD 验证）；R-2（trivial：shadcn_attrs.rs 一行 PLAN-053 注释被 T-05 编辑截断，代码完整非语义，merge 可顺手修复）；N-2（autoui_state 读根态——路由型示例的状态读回需快照 oracle，026/file-dialog 读回不受影响；基础设施观察非产品缺陷）。
independent replay：执行会话之外自写解析的跨层 oracle 重放 PASS（rows 1..100→101..200 随 下一页 精确平移，第 1/11→2/11）；重新生成 SFC 实证 `:disabled="!can_prev || disabled"` 绑定（编译器适配生效）。
evidence: docs/reviews/plan086/{decisions,evidence}.md；plan-artifacts/plan-086/{vp-review-native,vp-review-vue,vfd-review,probes/*}。next: merge（T-08）。


2026-11-14：stage: merge；plan_id: PLAN-086；plan_revision: r2；outcome: pass（delivered + archived，cleanup 见下）；delivery_commit: 765251b9005ee9f1a56ce31449aa41da3337eaf1（master ff-only，linear，main tip = delivery commit）；canonical: docs/specs/ui/pagination.md（新增，SD-01/02）+ docs/specs/ui/file-dialog.md（SD-03 增量 12 行） landed + smoke（tools 3+2 pass on main checkout）；archive: docs/plans/archived/086-shared-pagination-control.md（status: archived，链接已按 archived 父目录修正 ../../design/…，已提交）。
084 交接（T-08 交付义务）：可消费提交 = 765251b9（packages/pagination 三件套 + 文档）；闭包消费方式与合同 = docs/specs/ui/pagination.md + packages/pagination/README.md；084 重基线时以该 Spec/README 为依赖面，依赖具名解除。
cleanup（分步核对）：
1. worktree `D:/autostack/.wt/ui-086/auto-ui`：wt-guard clean（清除 pnpm 生成的 gen/node_modules junction 后）→ 待移除；
2. 分支 codex/ui-086-pagination / codex/ui-086-automan 及组内依赖检出 → 待移除；
3. plan-artifacts/plan-086/ 证据已折叠入本文件（decisions/evidence 落库 + 本收据）→ 待删除 + registry 刷新；
4. 交付物构建态（landing≠deployment）：debug auto.exe/auto-ui-tool.exe 为计划验证构建（组内 .wt/ui-086/auto-man）；主仓 release 产物与本 Plan 无生产消费者（纯包/示例/测试变更，无守护进程/后端服务面）——如需产品化由后续产品化流程重建。
completion_kind: delivered；merge-owned pending：Linux native 腿（环境无工具链）+ 统一回归（auto-ui 无 .last-batch-regression 惯例，按本仓各门为 owner 执行）。


## 10. 待澄清事项

无阻碍建计划的用户决策。T-01负责现有builtin运行能力、包的嵌套依赖移根方案、`.at`整数/事件可行性、026实际读回、Q-01归因和平台runner。API命名已按避免内置冲突决定；若探针否定可行性，先更新同一合同并保留全部AC，不能只交独立demo而不迁两个消费者。
