# auto-ui/PLAN-052 T-00 诊断决策（codegen-families）

日期：2026-11-05。输入：候选组合 auto-notes @ 38c7c87 `auto build -r rust`（锁定工具链），全量日志 `notes-build-full.log`（50 errors）。对照：002-counter 同组合 PASS。

## 错误分族 → 根因 → 生成器锚点

| 族 | 计数 | 表征 | 根因 | 锚点（ui-052 master） |
|---|---|---|---|---|
| F1 实体字段点访问 | 24× E0609 | store 逻辑 `n.pinned`/`n.folder`/`n.tags`（`n: &serde_json::Value`） | user type（`var notes []Note`）在 merged-JSON 世界被降链为 `pub type Note = serde_json::Value;`，但 store 发射面仍按强类型 struct 语义发点访问；缺 Value 访问器降链（`n["pinned"].as_bool()` 族）或实体 struct 直编 | rust_ui.rs store/表达式发射面（`NotesStoreMsg::ApplyFilter` 一行式发射） |
| F2 用户型双重定义 | 1× E0428 + 5× E0117 + 部分 E0433/E0277 | `pub type Note = serde_json::Value;` 与 `pub struct Note { id, revision, ... }`（derive Serialize/Deserialize）同名并存 | 两条发射通道（"User types aliased to Value" 旧通道 + 新实体 struct 通道）未互斥去重；alias 使 impl/derive 落 E0117/E0433 | main.rs:605-618 并置可见；alias 发射点与 struct 发射点需对同一 type-name 做唯一仲裁 |
| F3 子组件 Msg 名 | 7× E0425 | `NavTree(NavTreeMsg)`/`EditorPanel(EditorPanelMsg)` 引用未定义类型 | 子 widget 的 Msg enum 未随父 AppMsg 发射（view fn 片段/子组件编译集缺口，对照 PLAN-681 F2 依赖解析面同类） | rust_ui.rs:871 "Register view fn fragments BEFORE extracting widgets" 面；子组件 msg 收集 |
| F4 api 桩缺省型 | 3× E0308 | `fn get_note(id) -> Option<Note>` 体尾 `serde_json::Value::Null` | GET 桩 `default_expr` 只收口 String/bool/数值/()/Vec<>,`_` 臂恒 Null；`Option<…>`（auto_type_to_rust 已正确产 `Option<Note>`）落 `_` 臂 | rust_ui.rs:1984-2001 default_expr match |
| F5 杂项 | E0599×1/E0071×3 | struct 字面量/方法 | 随 F1/F2 修复后重测再定档（疑似同一根源的次生） | 复测后归档 |

## 修复策略（T-01 输入，按族独立提交）

- F4 最小先修：`_ if ret_ty.starts_with("Option<")` → `"None"`（+ Option 内 Vec 等无需特判）。一族一提交，立即重测 notes 构建看剩余族漂移。
- F2：alias 与 struct 对同名 user type 仲裁——选定**实体 struct 直编**（derive Serialize/Deserialize 已备）为唯一形态，alias 通道对"已被 struct 覆盖的型名"跳过；评估 F1 是否因此自然消解（struct 直编有点访问合法性）。
- F1：若 F2 选 struct 直编后仍有 JSON-bucket 面（`list`/Unknown 降链），补 Value 访问器降链发射。
- F3：子组件 Msg enum 进编译集（对齐 vue 轨 parity 先例）。
- 每族修后跑 notes 构建门 + counter 回归；全绿后再铺 todo/calendar/database/chat/blog。

## 对照与边界确认

- counter（无 store 实体/子组件 msg/Option api）PASS ↔ notes 三族俱全失败：与族表自洽。
- 不动 app 仓源码；修复只落 auto-ui 生成器；auto-lang 只作 path 依赖消费（Category A 纪律）。

## 执行记录（T-01 进行中）

- **环境事故（2026-11-05）**：D: 盘 100% 满（681G）致 rustc "no space on device"。清理本会话 ui-052 组 target（~18G）后他方并发清理叠加，余量恢复（197G）。教训：清盘前先 du 定位、只清本会话产物。**大象在 main 检出：auto-ui/target 144G、auto-lang/target 75G——疑多为历史陈旧产物，但有在飞会话，未擅动，上报用户处置。**
- **F4 已修**：rust_ui.rs GET 桩 default_expr 补 `Option<…>→None` 臂。
- **F2 已修**：`compile_at_file` 增 `api_type_names` 参数；两处 app 编译面由 `parse_api_module(project_dir)` 取 back api.at 型名集合，front TypeDecl 的 Value alias 对同名跳过（shell-pack 面传空集保持原行为）。结构性发现：rust-ui front 生成物是**全 typed** 形态（`Vec<Note>`/`RepoResult.ok/err_code/doc.title` 等完整类型化 repository 协议），api struct 直编是唯一自洽形态——佐证 F2 仲裁方向正确。
- 待验证：F4+F2 重编 tool 后 notes 重构建量错误族漂移（F1 点访问应随 struct 直编大量消解；F3 子组件 Msg 待看残留）。

## F3 执行实录（三次迭代）

1. 文本替换注入首测：sidebar.at 报 "Expected identifier or *, got Str"——替换命中 use 行自身（导入项必须是标识符）。修：逐行替换跳过 use 行。
2. 二测：`k`（view-fn 片段引用调用点循环变量，PLAN-681 内联 idiom）UndefinedVariable；溯因中发现真根因——**pipeline.rs（vue 轨）在解析前调 `prepare_style_recipe_imports` 预注册 use-imported 样式配方（PLAN-635 parser 符号检查钩子），rust_ui.rs 的 compile_at_file 缺此步**，caption_text 族在解析期即炸；k 错误为语法错误恢复态的连带（vue 同链解析同文件通过可证）。
3. 正解实现：compile_at_file 解析前 prepare + 解析后 load_and_validate（与 pipeline 同链）；保留文本替换作为生成器 Str-only style 提取面的兜底。模块定位补 src/front 约定面 + 按文件名递归回退（stylekit 实际路径 vendor/stylekit/src/front/styles.at）。

## 前线态势（F3 验证后，28 errors）

F3 达成：E0425/E0433（NavTree/EditorPanel 组件缺失族）全消——sidebar/editor.at 解析通过、子组件发射。错误谱系转为纯 typed 方言细化（无架构未知数，逐族发射器修补）：

| 族 | 计数 | 表征 | 修复面 |
|---|---|---|---|
| E0608 | 14 | store 处理器 `.notes[idx].pinned` 投影为 `self.notes[idx]["pinned"].as_bool()`——typed 实体元素被 Value 化（赋值+读双路径；`value_bracket_chain`/store-handler 发射面） | api 型名感知的 struct 投影分支（字段型表：String/Vec/Option→.clone()，数值/bool→直读） |
| E0277 | 5 | `&String == String` 比较（typed 字段对字面量） | 比较发射面 typed String 侧补 .as_str() |
| E0606 | 4 | `&i32 as usize`（引用强转） | 索引发射面引用先行 deref `(*x) as usize` |
| E0599 | 2 | `.unwrap_or_default()` 落在 &str（Value 访问器链残留在已解型面） | 访问器链按端型截断 |
| E0308 | 3 | 杂项（含 1 处 delete_note 返回型） | 随上述修复后复测定档 |

里程碑：50 → 28；E0609/E0425/E0433/E0117/E0428/E0071 族全消。

## T-02 铺测谱（2026-11-05，当前 tool 复测）

| app | 错误数 | 主族 | 状态 |
|---|---|---|---|
| notes | 0 | — | ✅ exe 发布+冒烟 |
| todo | 0 | — | ✅ exe 构建绿 |
| counter | 0 | — | ✅ 回归绿 |
| calendar | 29 | handler-call-return（.FmtDate 表达式位自消息发送返回 ()）+E0599+E0507 | 设计完成待实现（__call_ 辅助 fn + 臂委托 + 调用点改写） |
| database | 60 | E0609×27（反向：typed 期望落 Value 面）+E0599×8+E0308×7+plain×16 | 待族析 |
| chat | 147 | E0609×80+E0425×50+E0308×11+E0277×5 | 待族析 |
| blog | 192 | E0119×48（冲突 impl）+E0609×33+E0034×24（歧义关联项）+E0599×20 | 待族析（疑 alias/struct 双定义另一通道） |

已修通用族（notes/todo 实证）：api prop 豁免 value_prop_names、条件串回译、循环变量 api 分支（含 Checkbox checked 专属臂）、事件实参 this. 回译。

## T-02 执行场（2026-11-05，本轮实机 sweep）

环境（组内 ui-052 树，生成器 = `D:/autostack/.wt/ui-052/auto-ui/target/debug/auto-ui-tool.exe`，
每次删 `rust-workspace/<member>-*` 后全量再生成）：

```
AUTO_UI_TOOL_BIN=<ui-052>/target/debug/auto-ui-tool.exe
AUTO_LANG_ROOT=<ui-052>/auto-lang  AUTO_UI_SOURCE_ROOT=<ui-052>/auto-ui
AUTO_WEB_SOURCE_ROOT=<ui-052>/auto-web  AUTO_WEB_ROOT=<ui-052>/auto-web
AUTO_UI_ROOT=<ui-052>/auto-ui
```

| 轮次 | notes | todo | calendar | database | chat | blog | counter |
|---|---|---|---|---|---|---|---|
| 进入 T-02 | 0 | 0 | 0 | 0 | 95 | 191 | 0 |
| 本轮后（2026-11-05 18:0x） | 0 | 0 | 0 | 0 | **0（exe 发布）** | **17** | 0 |

修复族（逐提交）：

1. try 体扫描缺席（chat 95→19）：`collect_assign_kinds`/`collect_declared_names`/
   `scan_stmts_no_union` 均不递归 `Stmt::Try`——登录/发送 handler 全在 try 内，
   `var m = []` + `m = json.to_value(..)` 的 Value 联合格、Value 循环变量
   （`for m in .messages`）整体漏判。补递归 + Value 联合格空数组初始式发
   `Value::Null`。
2. 无 `use back.api` 工程缺实体型（chat 8 E0425）：`generate_api_client` 仅在
   api imports 非空时发射 api 实体 struct，chat 全走 `Http.*` 无 imports——
   `Contact/Identity/Message/PendingMsg` 缺失。补 else 分支只发类型本体。
3. typed prop ↔ Value 桶边界（chat E0308/E0061/E0507 + blog）：无类型 store
   字段（Vec<Value>）传入 `[]Message` 形 prop——新增 `CHILD_PROP_RUST_TYPES`
   全型表 + `serde_json::from_value::<Vec<T>>(Value::Array(..)).unwrap_or_default()`
   （构造参数、持久子件同步、组件实参三路）。
4. 多级 Value 点链（`sr.message.conversation_id`）：整链括号化后末段访问器收口。
5. store 归属：逐文件注册（仅本文件 use 的 store；首个 canonical "store"，
   其余按别名），`store.<Msg>`/`<Store>.<Msg>` 按 STORE_VARIANTS 归属解析
   （blog `store.SetToken` → ArticleStore），`.store.<field>` 按字段归属跨
   store 解析（blog 三 store 页）。
6. nil 哨兵：`Expr::Nil` 此前无主臂（blog `x == nil` E0425/User!=Null）；
   AST 比较 typed 槽 → true/false、Value → is_null；条件串管线补
   `fix_condition_nil`（view 条件为字符串）。
7. typed 记录字段 move（blog `self.current.work_id` ×6）：store/态字段型为
   api 实体时按 API_STRUCT_FIELDS 补 `.clone()`。
8. typed 返回写槽：`Option<T>` api 返回 → typed 槽 `unwrap_or_default()`；
   Value 槽 → `serde_json::to_value(...)`（含 Store/Bina 两发射位）。
9. 无 handler 按钮的 `on_click(|_| ())` 强制 `View<()>`（settings E0308）——
   移除占位（`button_onclick: Option<M>` 天然允许缺省）。
10. 原始标识符：`ref` 形参 → `sanitize_rust_ident`（raw ident）；无载荷变体但
    handler 模式声明形参（blog `BodyChanged(v)`）→ 由 `last_input_text()` 补绑定。

剩余（blog 17 = 15 router + 1 compile_error(多路由) + 1 follow() 零参 stub）：

- **多路由 outlet/router.push 静态发射**（PLAN-039 D5 拒绝门）是唯一实质功能缺口：
  blog 8 路由 + 4 处路由参数（slug/username）+ 15 处 router.push。需
  App 路由态/参数表、`__Navigate` 消息、outlet match 渲染、link(to:) 点击、
  `router.param` 编译映射——下一轮 T-02 继续。
- `follow(username)` 由 api stub 生成零参（merged client 未覆盖该端点，
  中段路径参数 `/profiles/:username/follow` 解析面在 auto-lang parse_api_module，
  非本仓）——按 G3 边界需要在 auto-ui 侧补 stub 参数或由 054/auto-lang 消费。

## T-02 终态（2026-11-05 19:16，sweep-r8 干净态）

生成器 = ui-052 树 `target/debug/auto-ui-tool.exe`（提交 886a22de + 本轮收尾提交），
按族修完后**全量再生成**（删除 front member、保留 `*-back`）。

| app | exit | 错误 | 发布 exe（大小/时间） |
|---|---|---|---|
| notes | 0 | 0 | notes.exe 84MB 19:13:20 |
| todo | 0 | 0 | todo.exe 83MB 19:13:46 |
| calendar | 0 | 0 | calendar.exe 83MB 19:14:34 |
| database | 0 | 0 | database.exe 84MB 19:14:59 |
| chat | 0 | 0 | chat.exe 84MB 19:16:16 |
| blog | 0 | 0 | realworld.exe 85MB 19:15:55 |
| 002-counter | 0 | 0 | （回归对照） |

### blog 191→0 的 router 族（本轮实现，非禁用）

`routes{}` 多路由/带参路由此前走 Outlet 位 `compile_error!`（P039 D5 拒绝门）。
本轮在生成器侧做静态路由发射：

1. 路由表：`const __ROUTES: &[(&str,&str)]` + `__route_match`（按 `/` 分段，
   静态段相等 + `:param` 抽参 → `HashMap<String,String>`）。
2. App 路由态：`__route: String` / `__route_params`；消息 `__Navigate(String)`
   → `__apply_route`（设置当前路径/参数表、同步目标页参数与 store、重放页
   `Init`）。
3. outlet 渲染：`match __route_match(...)` → 目标页持久子件 clone（同步参数表 +
   canonical store）→ `view().map_msg(|m| AppMsg::<Page>(m))`。
4. `router.push(x)`：根 → `self.__apply_route(&x)`；页面件 → `__nav_request`
   由父 App 转发后消费（`self.<page>.__nav_request.take()`）。
5. `router.param("slug")` → `self.__route_params.get(...)`。
6. `link (to:)`：路由上下文发 `View::button(label).style(...).on_click(__Navigate)`；
   非路由上下文维持原容器渲染（零回归）。
7. 多 store 页：页面 canonical store 按文件 `use` 首导入映射（`WIDGET_CANONICAL_STORE`
   文本级预扫），父 App 用对应别名字段双向同步（author→draft_store、
   login→store(AuthStore)、home/profile/article_detail/editor→article_store）。
8. `follow(username)` E0061：merged/CRUD POST 桩签名此前只取 body 参数，丢弃
   path 参数（`/profiles/:username/follow`）——改为按契约全参发签名。

### AC-03 冷启动冒烟（notes）

```
exe=D:\...\auto-notes-45ee43a7\bin\debug\notes.exe
alive_after_8s=True            # 进程存活（窗口创建成功）
has_exited=True  exit_code=0   # CloseMainWindow 后正常退出
```

### AC-04 diff 范围

`git diff --name-only 8ed8a2e1..HEAD` 仅：
`crates/auto-ui-compiler/src/ui_gen/rust.rs`、`crates/auto-ui-tools/src/rust_ui.rs`、
`docs/plans/052-*.md`、`docs/plans/evidence/052/*`、053 计划文档（并行会话交接的 desktop-host 计划，非本计划实现面）。
auto-lang/auto-web/auto-down 三兄弟树 `dirty=0`。

### 环境噪声（非生成器缺陷，已排除）

`sweep` 删除 `*-back` 目录后首次 `auto build` 的 backend refresh 偶发 exit -1；
`-back` crate 手工 `cargo build` exit 0，保留 `*-back` 后 sweep 全绿。另一次
sweep 出现 3 app 瞬时 -1，定位为**上一轮残留 cargo 进程占 target 锁**（4 个
orphan cargo 存活）——清理后 sweep-r8 全绿。两例均与生成器 diff 无关。
