# 多文件 Reload 就绪与状态 Schema 及 Effect 生命周期

> PLAN-042 交付合同（SD-01/SD-03）。逐实际入口的已支持/保守跳过/需重启/
> 具名缺口表；单文件状态 schema 迁移与失败保旧合同；owner/effect 边界；
> 后继完整实施包的候选 bundle 原子发布合同与反例。**本文件是研究核验
> 结论，不宣称完整 HMR 已实现**。证据：docs/plans/042 节 11 +
> crates/auto-ui-runtime/src/ui_tests/plan042_reload_readiness_tests.rs（12 例，
> 2026-11-04 全绿）。分类：feature ｜ 状态：active ｜ 来源计划：
> docs/plans/042-reload-schema-and-effect-readiness.md

## 1. 实际入口与支持矩阵

| 入口 | 位置（基线 7471b7c4） | 支持状态 |
|---|---|---|
| 装配登记 `set_source_path` | ui/dynamic.rs:2163；dynamic_build.rs:2111 | 已支持（单/多文件同样登记，仅记录根路径+基线 mtime） |
| 变更检测 `check_file_changed` | ui/dynamic.rs:2320 | 单文件：mtime 轮询可用；**内容不变、mtime 前进即报变化（误报，无害）**；**同 mtime 改内容漏检（负控实证）** |
| 变更检测（多文件形态） | ui/dynamic.rs:2327 guard | **保守跳过**：import_stmts 或 registry 非空 → 恒 Ok(None)；root/store/child 改动一律不 reload（不是"只 reload 根"） |
| reload 执行 | ui/dynamic.rs:2261；iced/renderer/host.rs:2822-2873（`__hot_reload` 500ms/2000ms 泵，`AUTOUI_HOT_RELOAD=0/1` 门控） | 单文件：保值迁移+视图替换可用；reload 失败静默（`let _ =`） |
| DevTools 回写 `write_source_range` | ui/dynamic.rs:2185 | 已支持；回写自刷 mtime，下一拍不重触发 |
| action_config 热替换 | compiler action_config.rs:680；host.rs:1886；MCP `action_config_reload` | 已支持（独立 DSL 源，与组件 reload 无关） |
| `hot_reload.rs` HotReloadComponent/UIWatcher | ui/hot_reload.rs | **存根，无能力**：parse_content 恒返回固定节点；无生产调用方。不得引用其声称 HMR |
| MCP 组件 reload 工具 | ui/mcp_server.rs | **缺口**：仅 action_config_reload；组件热重载无 MCP 直达 |
| Vue/Vite 轨 | — | **不在本文核验范围**：本仓无 Vue/Vite HMR 证据（examples 无 vue 形态、docs/web-integration.md 无 HMR 记载）；不得以其计 VM/iced 轨能力，是否存在属 web 栈自身事项 |
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

## 6. 后继完整实施包合同（SD-03；明确未实现）

后继 Plan（建议编号 048+，owner：UI runtime）须满足：

- **候选 bundle 原子发布**：candidate = root+imports+store+child+handlers
  +schema+effects 一次性编译校验；任一环节失败 → 具名拒绝、旧 bundle 完整
  保旧。禁止部分替换（旧 bridge/template/handler 不得被新候选半途覆盖）。
- **来源快照一致性**：root 与 import/store/child 绑定同一来源快照（内容
  hash，非 mtime）；缺源/无效 import 给具名拒绝，修复后下一次合法编辑成功。
- **schema 判型**：带旧字段 schema 判 保持/迁移/局部重建/拒绝；同名类型
  变化默认拒绝（不得沿用本文件 §3 的乐观保值缺口）。
- **owner/effect 治理**：实例/store/Tick/订阅按 041 合同保留或撤销；迟到
  回调不写新实例；失败保旧可继续操作。
- **资源条件（预算）**：候选编译在独立 VM 模块试编译（不污染现役桥）；
  输入规模按 042 T-01 冻结口径（单测级 <20 例/腿；真实 App 泵路径 GUI 腿
  另列）；不承诺无界性能压力。
- **验收反例（必须保持红）**：t03 guard 探针在真实现之前保持绿；t04 强制
  reload 灾害边界探针保持绿（作为"不得回退到部分替换"的回归钉）。
- **明确未实现（本包不交付）**：多文件/含 store/含 child 工程的热重载、
  内容级失效检测、MCP 组件 reload 工具、Vue/Vite 轨之外的 Node 热重载。

## 7. 具名阻断与移交

- **parser 病理分配**：截断的 widget 残句（`widget X { model {...} view {`）
  触发 ~51GB 分配、进程 OOM 中止（2026-11-04 探针实测）。owner：auto-lang
  parser；下一动作：语言仓开立 parser 健壮性 issue（输入最小复现见
  plan042_reload_readiness_tests.rs 注释）；不阻塞本包结论（探针已避开
  该类输入），但后继候选 bundle 的"坏源保旧"腿依赖 parser 不崩，须先行。
- 实例 owner 语义以 component-and-store-instances.md 为正式合同，本文不
  重写（SD-02 仅在其正式存在后记录真实差额——041 已交付，差额为 0，不
  另写）。
