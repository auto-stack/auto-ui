# 多文件 Reload 就绪与状态 Schema 及 Effect 生命周期

> PLAN-042 交付合同（SD-01/SD-03）；PLAN-056 对账修订。逐实际入口的已支持/
> 保守跳过/需重启/具名缺口表；单文件状态 schema 迁移与失败保旧合同；
> owner/effect 边界；候选 bundle 原子发布合同（已由 PLAN-056 交付，见
> docs/specs/runtime/atomic-source-bundle-reload.md）。**本文档保留 042 研究
> 结论与 056 对账差额；现役行为以 056 合同为准**。证据：docs/plans/042
> 节 11 + docs/plans/056 §9 + crates/auto-ui-runtime/src/ui_tests/
> plan042_reload_readiness_tests.rs（13 例）+ plan056_atomic_reload_tests.rs
> （15 例，2026-11-05 全绿）。分类：feature ｜ 状态：active ｜ 来源计划：
> docs/plans/042-reload-schema-and-effect-readiness.md

## 1. 实际入口与支持矩阵

| 入口 | 位置（基线 22a925c7） | 支持状态 |
|---|---|---|
| 装配登记 `set_source_path` | ui/dynamic.rs；dynamic_build.rs | 已支持（登记根路径 + 内容快照基线，root+use 图） |
| 变更检测 `check_reload` | ui/dynamic.rs；ui/reload_bundle.rs | **内容级**（blake3）：同 mtime 改内容检出；缺源具名 Err 保旧（PLAN-056 取代 mtime 轮询语义） |
| 变更检测 `check_file_changed` | ui/dynamic.rs | **legacy 单文件 API**：mtime 轮询 + 多文件 guard 语义仅保留于此（泵不再使用；t03 探针已改写为新行为断言） |
| reload 执行 `try_reload` | ui/dynamic.rs；iced/renderer/host.rs（`__hot_reload` 500ms/2000ms 泵，`AUTOUI_HOT_RELOAD=0/1` 门控） | 候选 bundle 原子发布（root/import/store/child/handlers/schema/effects 整包）；失败保旧、合法下一编辑成功 |
| DevTools 回写 `write_source_range` | ui/dynamic.rs | 已支持；回写后下一拍内容级检出并原子提交 |
| action_config 热替换 | compiler action_config.rs；host.rs；MCP `action_config_reload` | 已支持（独立 DSL 源 mtime 泵，与组件 reload 无关；候选构建/提交**不**劫持它） |
| `hot_reload.rs` HotReloadComponent/UIWatcher | ui/hot_reload.rs | **存根，无能力**：parse_content 恒返回固定节点；无生产调用方。不得引用其声称 HMR |
| MCP 组件 reload 工具 | ui/mcp_server.rs | **缺口**：仅 action_config_reload；组件热重载无 MCP 直达（不随 056 新增公开 API） |
| Vue/Vite 轨 | — | **HMR 不在本包核验范围**：本仓无 Vue/Vite HMR 证据；Vue 浏览器活体实证已由 PLAN-057 交付（vue-app-store-live-isolation.md——冷生成双路径挂载/隔离/owner 退出，非 HMR） |
| Node `hot_reload` | — | 不属于本仓 UI runtime 能力 |

## 2. 检测语义（mtime，非内容）

- 比较规则 `current_modified > last_modified`。`set_source_path` 登记成功时
  基线=当前 mtime，此后首检与基线比较（无变化）；仅当登记时取不到元数据
  （`last_modified=None`）时首次检查才恒报变化。
- **同 mtime 修改漏检**（探针 `t02_same_mtime_content_change_missed`：
  `File::set_modified` 拨回基线后改内容 → Ok(None)）。后继如需内容级
  失效，须引入 hash/长度比对，且须保留 mtime 快路径。
- 缺源：`check_file_changed` 返回 Err；泵内 `if let Ok(Some(_))` 吞错，
  旧组件继续服役（探针 `t02_missing_source_errors_but_old_component_survives`）。
- 检出拍即推进 `last_modified`：**解析/候选失败该拍被跳过**，下一次
  mtime 前进的合法编辑正常恢复（探针 `t02_parse_failure_*`）。

## 3. 单文件状态 schema 迁移合同

`reload(new_widget)`（单文件形态，无 use/store/child）：

- 旧态快照 = 根状态字段（`__auto*` 内部槽不入快照，vm_bridge read_all_state）。
- 逐新定义字段：同名 → **保值**；新字段 → `initial` 默认值；旧字段缺席新
  定义 → 丢弃并在 `MigrationReport.dropped_names` 点名。
- **具名缺口（同名类型变化不判型）**：reload 传空 old_field_defs，
  `migrate_state` 按名乐观保值；实测 `Int(42)` 原样写入 `str` 字段且
  `write_state` 不判型（探针 `t02_same_name_type_change_*`）。后继须带
  旧 schema（编译期字段类型表或上次的 AuraWidget 快照）才能判型拒绝/转换。
- `MigrationReport.preserved/added/dropped` 只按新定义字段计；宿主字段
  不得出现在 dropped_names。

## 4. 候选失败保旧与恢复协议（已实证）

单文件泵链三层失败均为保旧语义（旧 bridge/视图/业务继续，用户数据不丢）：

1. **解析/提取失败**（坏候选）：该拍跳过，组件无感；下一次合法编辑成功。
2. **reload 内部失败**（新桥 VmBridge 构造 Err）：`reload` 返回 Err，
   旧 bridge 未被替换（探针 `t02_reload_bridge_failure_keeps_old_bridge`）。
3. **缺源**：Err + 旧组件继续；文件恢复后下一拍正常 reload。

恢复后行为：兼容字段保值、新字段默认（探针
`t02_real_entry_reload_preserves_and_defaults` / `t02_removed_field_dropped`）。

## 5. owner/effect 边界（多文件 guard 的存在理由）

多文件形态（use imports / child registry 非空）下整源 reload 路径**只重建
根 widget 的桥**（`VmBridge::new(new_widget)`，不带 imports/children），
强制执行的灾害边界（探针 t04 系列，绕过 guard 的受控实验）：

- **子实例数据灭失**：`child_instances` 随桥替换整体清空；用户输入的
  实例态不迁移；下一帧同身份实例从 model 默认重建。
- **迟到实例事件**：旧 identity 在新桥缺席 → 按 component-and-store-instances
  §2 合同丢弃 + 诊断，**不重定向**、不复活实例。
- **store 灭失**：store 字段不进候选 → 数据面丢失（实测 `field not found:
  total`）；store handler 缺席；**根 handler 体内引用 store 的候选编译失败，
  根 handler 一并灭失**（Plan 449/451 实机 "field not found + 派发失效"
  的单测级完整复现）。
- **effect 资源**：`tick_interval` 由新定义整体替换（探针
  `t04_tick_interval_replaced_by_reload`）；Tick handler 新语义即刻生效。
  订阅面经 renderer 每拍重建自然收敛。
- 结论：**不得通过删除 guard 冒多文件 HMR 能力**；guard 是当前唯一防止
  上述灭失的防护。

## 6. 后继完整实施包合同（SD-03；已由 PLAN-056 交付，逐项对账）

| 042 合同项 | 056 落地 |
|---|---|
| 候选 bundle 原子发布（root+imports+store+child+handlers+schema+effects 一次性编译校验；任一环节失败 → 具名拒绝、旧 bundle 完整保旧） | 交付：`try_reload` 独立候选 + typed 迁移 + 原子提交（atomic-source-bundle-reload §2/§4）；失败/冲突/失配全具名拒绝保旧 |
| 来源快照一致性（内容 hash 非 mtime；缺源/无效 import 具名拒绝；修复后下一次合法编辑成功） | 交付：blake3 内容快照 + 构建后复核（§1/§2） |
| schema 判型（同名类型变化默认拒绝；不得沿用乐观保值缺口） | 交付：`migrate_state_typed` 同名不同类型整候选拒绝（§3） |
| owner/effect 治理（实例/store/Tick/订阅按 041 合同保留或撤销；迟到回调不写新实例；失败保旧可继续操作） | 交付：实例迁移 + tombstone + 繁忙保旧重试 + 订阅表整体替换（§3/§4） |
| 资源条件（预算）：候选编译在独立 VM 模块试编译（不污染现役桥）；输入规模按 042 T-01 冻结口径 | 交付：候选独立 VM；快照图 ≤128 文件硬界 |
| 验收反例：t03 guard 探针在真实现之前保持绿；t04 强制 reload 灾害边界探针保持绿 | t03 改写为新行为断言（guard 退役）；t04 保持绿（不得回退部分替换的回归钉） |
| 明确未实现：多文件/含 store/含 child 工程热重载、内容级失效检测、MCP 组件 reload 工具、Vue/Vite 轨之外的 Node 热重载 | 前三项已交付；MCP 组件 reload API 与 Node 热重载保持未交付（057 交付 Vue 浏览器实证：docs/specs/runtime/vue-app-store-live-isolation.md，e2e 入口在该文 §5）——注意 Vue/Vite **HMR** 仍非本链能力：057 实证的是冷生成双路径 + 041 store 隔离/owner 退出活体语义，与 042 §1 "本仓无 Vue/Vite HMR 证据"不冲突，HMR 缺口继续明列 |

## 7. 具名阻断与移交

- **parser 病理分配（已关闭，PLAN-940 2026-11-06）**：截断的 widget 残句
  （`widget X { model {...} view {`）曾触发 ~51GB 分配、进程 OOM 中止
  （2026-11-04 探针实测）。归因实证在 **auto-ui 方言文法收集循环**（非
  auto-lang parser）：EOF 上 `advance` 不越过 EOF、收集臂可 `Ok` 返回且
  游标不动，只看闭合符的循环无限收集。PLAN-940 为 11 处循环加 EOF 受控
  拒绝守卫（方言侧反例族 `auto-ui-dialect/src/regression/parser_eof_progress.rs`；
  语言仓 Core 合同见 auto-lang `docs/specs/auto-parser-api/progress-and-rejection.md`），
  本文件依赖的"坏源保旧"腿已交付：`plan042_reload_readiness_tests.rs`
  `t02b_truncated_widget_keeps_old_and_next_legal_edit_wins`（截断候选
  保旧 count=11，修复文件后合法编辑 reload 成功）；PLAN-056 的
  `SOLO_BROKEN` 截断候选同样受控拒绝保旧（V-01/V-03）。owner：auto-ui
  dialect grammar。
- 实例 owner 语义以 component-and-store-instances.md 为正式合同，本文不
  重写（SD-02 仅记录真实差额——041 已交付，差额为 0；PLAN-056 的实例
  迁移规则见 component-and-store-instances §6）。
