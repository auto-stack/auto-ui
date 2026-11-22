# runtime: ace-sqlite-host-bridge — AV 桌面宿主真实 SQLite 桥（PLAN-113）

> Status: **已交付**（PLAN-113 r1，2026-11-22；AV 腿 V-03 全项验收，final review pass）。
> 本文件是 PLAN-113 frontmatter 声明的规范落点（`new_spec_components`）。
> 只写已验行为与接口；样本展示语义（026 原样源内存样本）不属于本文件。

## 1. 定位与合同

026-database 原样源的 RunSql/CommitTx/RollbackTx 是**内存样本语义**（字符串匹配 +
状态更新，T-01 冻结）。`SqliteHostBridge`（`crates/auto-ui-runtime/src/ui/ace_sqlite_bridge.rs`）
在**宿主侧**把同一批调度消息镜像为对真实临时 SQLite 库的读写，使「样本展示」与
「真实 SQL 结果」两层证据互相独立（V-03「各有独立证据」）。

- **opt-in**：仅测试入口显式安装（`ace_node_desktop_window --sqlite-db <path>` 或
  `NodeDesktopBridge::install_sqlite_bridge`）；默认 `None`，其他 app 零行为变化。
- **挂接点**：`NodeDesktopBridge::dispatch_event` 在引擎派发**之前**按消息名通知桥
  （读 `last_states` 标量槽）。app 面惰性 handler（引用惰性列表的 handler 被
  fail-closed 跳过、具名拒 `no on … handler is declared`）**不阻塞**真实层执行。
- **库实现**：复用 auto-lang `a2r-std::sqlite`（rusqlite 0.30 bundled，Plan 415-B1
  受检面）；per-op 诊断经 additive `a2r_std::sqlite::clear_error()`（op 前清、
  op 后读 `last_error`——a2r-std 错误粘性 by design 不变）。

## 2. 镜像规则（026 合同）

| 消息 | 真实操作 |
|---|---|
| `RunSql` | 对真实库执行 `.sqlInput`（SELECT/WITH/PRAGMA/EXPLAIN 走 query，其余走 exec）；真实结果集行数/错误诊断/耗时入 evidence |
| `SaveNewRow` | `.activeTable`+表单标量真实 INSERT；首个 staging 写 `BEGIN`（合同限 customers 新增表单） |
| `DeleteRow` | 跳过（inert 循环载荷按钮，沿 ceb9045f 先例） |
| `CommitTx` | `COMMIT`（无事务时 no-op ok） |
| `RollbackTx` | `ROLLBACK`（无事务时 no-op ok） |
| `ResetDb` | 回滚未决事务 + 三表 DELETE + 重种子 |

窗口 close → 桥随 `NodeDesktopBridge` 销毁 DROP 连接；**连接释放证据** = 进程
退出后临时库文件可删除/重开（Windows 文件锁强约束）。

## 3. 种子 fixture 与「相同数据」

`ace_sqlite_bridge_seed`：三表（customers 15 / products 12 / orders 10 行）逐行
复制自冻结 026 源模型字面量（T-01 钉版 app.at SHA256 前 16 `D37E703351DEDFEB`；
列值保持源显示形态如 `$18.00`，id 整型）；DDL 与 app `.schemaDdl` 同款形状。
「相同数据」由钉版 fixture 成立，不做运行时源解析；fixture 变更必须随
对源逐行校验脚本（review 对回原源）。

## 4. evidence 面与独立核对

`NodeDesktopBridge::evidence()` 增 `sqlite_bridge` 节：`{db_path, in_tx, events[]}`
（seq/msg/detail——detail 含 sql/ok/rows|affected/error/duration_ms）。

**独立连接核对（V-03）**：验收方用**独立连接**（同受检 a2r-std 库新开
`SqliteDb`）开同一文件——commit 后新增行可见且内容精确匹配、rollback 的行不
存在、未涉事务的表种子不被污染。独立连接同时是判别 oracle：桥若写内存库或
commit 空转，独立连接核对必红。

## 5. 永久用例

`crates/auto-ui-runtime/tests/plan113_026_sqlite_bridge.rs`（1 例，实测 ≤5.2s
<10s 成本线）：全流脚本（preset 查询/错误 preset/表单键入（modal 输入目标
14..17）/Save/Commit/第二 Save/Rollback）→ evidence 六事件断言 → 独立连接
count=16/已提交行精确匹配/回滚行缺席/products/orders 未污染 → 目录可删
（连接释放）。排窗档 `AUTO_UI_ACE_NODE_E2E=1`；前置
`cargo build --example ace_node_desktop_window --features ace-bridge`（准备道）。

## 6. 边界（显名）

- A2Vue/浏览器腿**不**配真实库（无 rusqlite）；真实库核对属 AV 腿（V-03），
  A2Vue 腿按 V-04 保生成/构建/真实浏览器矩阵（内存语义）。
- `ResetDb` 的 seed 事件会再入 evidence（计数断言需按场景口径）。
- 桥消息名映射是 026 demo 合同；通用 service 声明面（pac.at service 块）不在
  本交付，后续宿主服务计划另行立项。
