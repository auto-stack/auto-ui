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
