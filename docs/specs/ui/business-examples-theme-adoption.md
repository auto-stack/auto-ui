# 业务示例主题接入（business examples theme adoption）

> PLAN-037 交付合同（SD-01 草稿，worktree 内备稿；merge 落地时发布）。
> B2 十二个业务/教学示例（011-calculator、012-clock、013-todo、014-weather、
> 015-notes、016-calendar、017-chat、021-blog-viewer、022-kanban、
> 023-realworld、028-launcher、038-minesweeper）接入官方主题包的**逐例可
> 验收**规则：接入形态、逐例默认/映射与豁免、双端通道、隔离/优先级、
> 原业务守恒、T-06 渠道与外部消费、B1+B2 收口与后继。分类：feature ｜
> 状态：active ｜ 来源计划：docs/plans/037-business-examples-theme-migration.md

## 1. 接入形态（C-01）

- 十二例统一：pac `dep themes { path: "../../../packages/themes" }` +
  app `use themes.themes: Auto, Solarized` + `theme_name`/`dark_mode`
  模型态 + `design { theme: .theme_name, mode: if .dark_mode … }`
  （031 C-14 形态；不复制调色板，不新增菜单/业务 handler）。
- 显式默认逐例保留（C-14：不暗改未声明 App 的默认）：
  - pac 显式键：015 `theme: "dark"`（dark-first）、016 `theme: "light"`
    （light-first）原样保留；
  - App 内显式 dark 默认：011（原暗支常量臂）、014（dark_mode=true +
    ToggleTheme）、022（dark_mode=true + 切换器，并在 pac 层显式化
    `theme: "dark"`——迁移前双轨启动不一致 [VM 链 light/Vue 模型 dark]，
    统一为 App 声明的 dark-first，同 006/008/009 先例）；
  - 无主题接线、无显式默认 → 官方 Auto/light 独立默认：012、013、017、
    021、023、028、038。
- 014 pac 维持不声明 theme 键（其自身"跟随宿主/CLI"契约）：VM/Vue 两轨
  模式由主题链驱动（本机 host=light 时与迁移前 HEAD 逐位一致，V 证据
  见 Plan §9）。

## 2. 逐例默认、映射与合法豁免（C-02）

| 例 | 默认 | 映射/豁免要点 |
|---|---|---|
| 011 | App dark-first（原暗支外观） | 原 dark/light 双臂样式保留（design 绑定后即随主题实时切换）；运算符 bg-orange-500/科学键 bg-indigo-700 为类目身份色豁免（registry 无语义键） |
| 012 | Auto/light | 表盘 SVG hex 为钟面内容插画豁免（B1 r2 口径）；其余全语义 role |
| 013 | Auto/light | 全语义 token（PLAN-637 已映射），零新豁免 |
| 014 | 链驱动（ pac 不声明） | 7 条件天气渐变 + AQI emerald/amber 徽章为类目内容色豁免；hero 文字白为渐变配对字 |
| 015 | pac dark（显式） | store 主权 dark_mode/accent_color + applyAccent 重放链路原样（T12-DARK 契约）；npm_deps 修正见 C-06 |
| 016 | pac light（显式） | 今天/选中/其他月身份由 day_style 语义槽区分；设置弹层 Dark/Light 按钮原 handler |
| 017 | Auto/light | 全语义 token；SSE/typing/消息业务零改动 |
| 021 | Auto/light | 3 路由共享 App design 绑定（跨路由主题保持）；无新豁免 |
| 022 | App+pac dark-first | 卡片/列语义 role；Done 列 emerald 点为状态身份色豁免 |
| 023 | Auto/light | brand-green 品牌绿 + text-white×5 维持既有字面豁免（app.at 注入口径）；表单 input_field 消费不变 |
| 028 | Auto/light | 品牌 hex `bg-[<hex>21]`/`text-[<hex>]` 为宿主注入/mock 数据内容色豁免（动态拼接，双端同源） |
| 038 | Auto/light | 锌灰壳 84 处逐字段映射公共 role（background/card/muted/secondary/border 系）；数字 1-8（blue/emerald/rose/purple/amber/cyan/zinc-200/pink 双编码）、雷 rose-950/rose-500、旗 rose-400、LCD rose/amber 为身份内容色字面豁免（不归并语义槽） |

## 3. 双端驱动通道（C-03；测试/宿主切换不新增菜单）

- **VM**：`autoui_fixture`（AUTOUI_TEST_FIXTURES=1，PLAN-623）状态写
  theme_name/dark_mode + 现有 MCP 驱动；生效值经 autoui_state 断言。
- **Vue**：setupState 模型驱动（`#app.__vue_app__._instance.setupState`）
  + `[data-auto-theme-root]` CSS-var 逐值 oracle（Auto/light `0 0% 100%`、
  Solarized/light `44 87% 94%`、Solarized/dark `192 100% 11%`、
  Auto/dark `240 10% 4%`）。
- 逐例主题腿落在**原门文件内**（desktop_mcp/vm_smoke/vm-smoke/
  smoke.spec.ts 扩展段或新增同目录入口）：016 此前无任何测试，本包新建
  `tests/test_016_vm.py`（15 断言）+ `tests/test_016_vue.mjs` 作为其首批
  真实回归入口；012/013/014/021/022/023/038 另建 VM 主题腿或 Vue 主题腿
  补齐双轨（原文件未覆盖的轨）。

## 4. 隔离、优先级与生命周期（C-04）

- 同进程多 App 各持 scope：`plan033_b1_scope_isolation`（两例相反主题
  同帧隔离 + 宿主 override 生效/释放/重开）在本包基线重跑 2/2 green。
- 多组件/路由：015 store 主权 + App 镜像链路原样（T11/T11b/T12 全绿）；
  021/023 跨路由共享 App 绑定（主题跨路由保持断言）；017 多子件
  （ChatSidebar/MessageThread/Composer）主题随 App 根。
- 既有投影面（033 已修）不回归：窗口投影路与 view 构建路同持组件 scope。

## 5. 原业务守恒（C-05；主题迁移不是业务重写）

- 逐例原业务 oracle 全部在迁移后重跑通过（数字/命令见 Plan §9 分例
  行）：011 运算/模式、012 计时/闹钟/storage 重启、013 增删改筛、
  014 城市/预报/布局、015 笔记全流程 + accent、016 月历/选中/Today、
  017 会话/发送/计时（VM 轨）、021 分类/详情/clap/发布/删除、022 卡片
  增删移/拖拽、023 注册/登录/feed（T9/T10 见 C-07 首红）、028 键盘流/
  recent/mock 注册表、038 确定性雷盘 (4,4)→29 展开/旗标/重开。
- 活跃状态换肤不丢业务：012 走表中换肤（Tick 连续 600≤Δ≤2200ms）、
  011 挂起表达式跨换肤求值 12+34=46、015 草稿/编辑、017 未发草稿、
  021 详情路由、023 登录态、028 查询中换肤 nres 恒定、038 揭示盘面。

## 6. T-01 具名阻点与本包有界修复（C-06）

- **015 AutoDown 依赖**：`npm_deps: ["@autodown/editor:link:…", …]` 数组
  串在结构化解析臂不识别（`ERR_PNPM_INVALID_DEPENDENCY_NAME` 实证，
  `npm_deps_from_value` 的 scoped 名/:link: 解析缺陷——**具名交 owner，
  本包不改生成器**），且 editor/core 包已被 auto-down e7d079e 物理退役
  （PLAN-040 并入 @autodown/engine）。修定：对象语法 +
  `{ "@autodown/engine": "link:<幸存 engine 包>" }`；本仓 .at 未消费
  autodown 组件（editor 为原生 textarea），engine 链接维持 main.ts 的
  engine style.css 消费面；修定后 pnpm install/`auto run` 真实绿。
- **stylekit 共享配方 VM fatal（有界修复，原反例+回归钉住）**：
  `collect_module_imports` 投机解析路径缺 stylekit 配方预注册
  （build_dynamic_component_inner A-02 的对应面）——自带
  `use stylekit.styles: <name>` 的子模块（PLAN-637 sidebar.at/editor.at
  实案，015/017 均中）在 plan-446 C1 引导门 fatal（**冻结基线复现**，
  主检出另有他方未提交同诊断 WIP，未消费、合并时对账）。修复=UI 场景
  模块解析前按 A-02 同型预注册；回归
  `plan037_stylekit_import_parse_tests`（基线红/修复绿双向验证）。
- **023 双店同名 Init（有界修定）**：App 双 store 裸 `store.Init()` 命中
  plan-446 A1 歧义 → VM 引导 fatal（冻结基线复现；Vue 轨 codegen 恰好
  双店展开故未暴露）。按 A1 处方限定 `AuthStore.Init()` +
  `ArticleStore.Init()`（home 同）；语义=会话恢复+feed 加载不变。

## 7. 首红与支持轨边界（C-07；不冒迁移完成、不缩样）

| 面 | 状态 | owner/去向 |
|---|---|---|
| 017 Vue T3/T6/T7/T9（发送气泡/SSE/typing） | 基线首红：codegen 把 `json.from_value(…)` 原样透传进 TS（浏览器 ReferenceError，POST 静默失败）；SSE EventSource 接线未生成 | auto-ui codegen（表达式转译/SSE 注线）；本包不改通用 codegen |
| 023 Vue T9/T10（鉴权发文/改文） | 基线首红：生成 api client 把 `meta` 放进 JSON body 而非 Authorization 头（后端 bearer 提取落空） | auto-ui codegen（api client meta 发射）；T5/T6 的种子口令不一致已按现源修测试 |
| 028 VM `__focus_input` 清位 | 聚焦请求正常消费（AUTO_DEBUG_FOCUS 实证 5× consumed、目标聚焦成功），design 块帧路径下清位 shadow-write 不反映到 VM 状态读回 | auto-ui runtime（host.rs focus 消费清位回写）；基线同点位直接进程崩溃（无法对比） |
| 028 VM palette 打开间歇崩溃 | 冻结基线同点位确定性崩溃（T2 后连接重置）；迁移后间歇（多数轮次全绿，偶尔 T2 后进程消失） | auto-ui runtime（RC canary/overlay 重建族，与 038 T3 已知 master 缺陷同族嫌疑） |
| 016/021/022/023 VM 原无测试 | 本包新建 VM 回归入口（test_016/021/022/023_vm.py） | 本包交付 |
| B3 十二例 | 原样登记后继（038-plan/研究文档），不以"通用控件已绿"宣称媒体/编辑能力完成 | UI038 |
| U036 新渠道外部最终门（vue-render app 的 SDK/delivery-root 消费） | `--delivery-root` 仅支持 rust 后端（具名拒绝实证）；B2 十二例均为 render:"vue" → 新渠道外部最终消费**保持待办**，owner=Man 发行/U036 谱系 | 渠道 owner |

## 8. 渠道与外部消费（C-08；910 隔离保持）

- **path 腿（已验）**：受管快照独立根（vendor/themes：清单+清单列文件）
  被仓外 scratch 消费者（012 拷贝、dep path 指向外部根）真实生成，
  receipt 内容摘要 `9fa07742…bef838e` 与仓内规范摘要逐位一致（摘要与
  绝对根无关）；篡改快照 → 生成具名拒绝（"主题包生成准入失败"）；
  还原 → 恢复生成。仓内同步验：篡改 packages/themes → 012 gen 具名
  拒绝；还原 → 恢复。
- **生成准入**：`rust_ui::plan036_theme_input_tests`
  （identity/tamper 拒绝族）在本包基线全绿。

## 9. 收口与后继（C-09；不冒称全量）

- **B1+B2 收口**：B1 十例（002/005/003/010+033 六例）+ 本包十二例
  = **22** 个示例接入官方包；**B3 十二例**（018/019/020/024/025/026/
  027/029/030/031-image-viewer/031-paint/041）仍未迁移，逐 app 独立
  状态归 UI038；共享 stylekit、remote-frame-e2e、032 验证例按各自来源
  保持义务（本包未触碰其源）。
- OS053/055、auto-clock 产品、OS054 装配不在本包；教学例不等产品
  （012 教学例不是 auto-clock，011 不是 auto-calc）。

## 10. 验收锚点

- 逐例双轨主题腿 + 原业务门：各例 tests/（见 Plan §9 分例行，命令/退出
  码/关键数全录）。
- 隔离/优先级：`plan033_b1_scope_isolation`（runtime）。
- 准入/篡改/外部根：`rust_ui::plan036_theme_input_tests` + Plan §9 的
  V-05/V-06 实录。
- 有界修复回归：`plan037_stylekit_import_parse_tests`（runtime）。
