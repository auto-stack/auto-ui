# PLAN-085 实施证据汇总（T-06）

日期：2026-11-15。工作树 `D:/autostack/.wt/ui-085/auto-ui`，分支
`codex/ui-085-tree-view`，基线 `c78208de`（master）。原始工件（命令输出、
截图、report.json）在 `D:/autostack/plan-artifacts/plan-085/`（仓库外 scratch，
merge 按政策清理）。本文只留可复核索引；实施决策见
[decisions.md](decisions.md)。

## 工具与依赖身份

| 项 | 值 |
|---|---|
| 本树交付 commit | `8f36b9e3`（T-05 双端全绿）→ 后续 docs 提交 |
| `auto.exe`（组内） | `D:/autostack/.wt/ui-085/auto-man/target/debug/auto.exe`，组内 detached `auto-man 54fe407`，相对 path 解析到本树 auto-ui |
| `auto-ui-tool.exe`（本树） | `cargo build -p auto-ui-tools --bin auto-ui-tool` |
| 依赖工作树组 | `.wt/ui-085/{auto-lang 86c7fd50b, auto-web 0adebc0, auto-down 5af11a3, auto-man 54fe407}` 全 detached。**本树（auto-ui）零依赖仓改动**，但 **`auto-man` 的 `Cargo.lock` 是 dirty 的**（`M Cargo.lock`，+11 行 `ace-foreign-contract` 包条目，来自其它计划的 SDK staging 工作，非本计划产生）。该差异不影响本计划取证：`auto.exe` 由该树的 `target/` 提供，path 依赖仍解析到本树 auto-ui；记录在此以免把"只读"误读为"全组 clean"。 |

## 阶段结果

| 任务 | 结果 | 证据锚点 |
|---|---|---|
| T-01 机制探针 | pass | P-0…P-8 见 decisions §9：环境控制门 `verify_pagination.py --backend native` 复跑绿；包内模块不自动导入、`class:` f-string 不插值循环字段、1000 节点完整渲染、MCP 行动作按 ID 命中 |
| T-02 正式包 | pass | `packages/tree-view/`（`854b9b5f`）；真实 native 窗口渲染色含图标/badge/加载中/重试/禁用删除线/focus ring；截图 `pkg-native-proof.png` sha256 `19918B0D…F4D03AA` |
| T-03 键盘归属 | pass | `815fec34` 修键归属不确定（`registry.all()` 遍历 HashMap 且含未渲染导入可抢页面键）；语义由 `plan085_tree_keyboard` **8/8** 钉住；真实窗口 `P1→Down→S1→Home→P1→Enter→selected=p1` |
| T-04 消费者迁移 | pass | 026 `b89829fc`（其自身套件 **25/25×3**，含树行断言与切表）；041 `fe411d45`（**44 passed / 9 failed**，与未迁移基线逐项相同） |
| T-05 产品门 | pass | `219f85d6`：**双端** pass，矩阵 `{a_tree_rows:5, b_tree_rows:2}`；native 双会话 + 浅/深截图 + exit 0 + 端口回收 + 重开；vue 真实 Chromium + 同矩阵 |
| T-06 文档与 Spec delta | 本文 + 包/示例 README + SD 提案 | 见 §Spec 增量提案 |

## 产品门实跑（T-05）

命令（两轨同形，`<out>` 每次新目录）：

```
python tests/product/verify_tree_view.py \
  --auto-bin D:/autostack/.wt/ui-085/auto-man/target/debug/auto.exe \
  --ui-tool-bin D:/autostack/.wt/ui-085/auto-ui/target/debug/auto-ui-tool.exe \
  --out-dir <out> --backend native        # 或 --backend vue
```

| 轨 | outcome | 矩阵 | 其它 |
|---|---|---|---|
| native | **pass** | `{a:5, b:2}` | 2 会话；窗口「树形演示」；浅/深截图（`1-light.png`/`1-dark.png`）；`exit_code=0`；`port_rebinds=true`；重开会话再选择成功（`2-reopen.png`） |
| vue | **pass** | `{a:5, b:2}` | 真实 Chromium；`1-light.png`；页面 `pageerror` 为空 |

门覆盖的行为断言：受控身份（选择载荷/回读、展开与选择分离）、chevron 展开不改
选择、retry 恢复且错误行消失、双实例互不串扰、空态出现且只影响 A、文档化退出
与重开。**不接受**"只编译"或"只有截图"作为通过。

## 本轮修掉的两个真缺陷

1. **空状态陈旧**（`b1c90bdd`）：`is_empty => tv_count_rows(.nodes,…) == 0` 在 VM
   轨上对 `.nodes` 变更保持陈旧——实测 `nodes` 2→0→2→0 时 `rows` 每帧正确
   重派生（2,0,2,0），空态分支一次不渲染；而"第一帧就空"的树正常。改为由
   `tv_derive_rows` 产哨兵行承载空态，删除 `is_empty`。回归
   `plan085_empty_state` 1/1，能区分修复前后。
2. **vue 腿三处选择器/捕获错误**（`219f85d6`）：生成 SFC 对行容器与标签行都打
   `data-auto-tag="row"`（裸选择器 5 行数成 10）；摘要正则锚定 native 引号形态
   而 vue `inner_text` 无引号；去掉引号后在 native 过度捕获（吃进 `" {`）造成假
   串扰。三处均已修，双端矩阵现一致。

## 回归（本树）

| 命令 | 结果 |
|---|---|
| `cargo test -p auto-ui-runtime --test plan085_empty_state` | 1/1 |
| `... --test plan085_tree_keyboard` | 8/8 |
| `... --test plan085_keydown_ownership` | 5/5 |
| `... --test plan085_routed_callbacks` | 2/2 |
| `... --test plan041_instance_ownership` | 4/4 |
| `... --test g16_imported_sibling_vm_tests` | 1/1 |
| `cargo test -p auto-ui-tools --test tree_view_component` | 2/2 |
| `... --test pagination_component` | 2/2 |
| `... --test file_dialog_component` | 3/3 |
| `python examples/ui/026-database/tests/desktop_mcp.py` | 25/25（连续 3 次） |
| `python examples/ui/041-auto-edit/tests/desktop_mcp.py` | 44 passed / 9 failed，与**未迁移基线逐项相同** |

## 具名遗留（显式 pending closeout 门，非本计划 work 义务）

| 项 | 状态 | 归属 |
|---|---|---|
| Linux/WSL native 腿 | not_run：WSL 图形栈 wgpu `Invalid surface`（与 PLAN-084 同一边界），不记 pass | merge/review closeout |
| 统一回归 | 本轮未跑（work 档位不跑；由 review/merge 执行） | review/merge |
| 041 的 9 项失败 | 先于本计划存在，迁移未改变其行为（menubar onclick、剪贴板粘贴、关标签/空态、`AUTO_OPEN_PATH`、T10×3） | 041 自身 |
| 路由页内树回调 | 真实窗口按下不写页面状态，库路径正常且无诊断；已隔离并由 `plan085_routed_callbacks` 守护；两个真实消费者与示例均为根 widget 形态，不受影响 | 后续计划（本计划不降级为鼠标专用树，也不搬动消费者形态） |
| Q-01（分页/路径文本视觉截断，DEBT-006） | 受影响视觉断言不以本计划关闭 | DEBT-006 |

## Spec 增量提案（SD，T-08 落 canonical）

**SD-01 → 新 `docs/specs/ui/tree-view.md`**：节点 schema 与全键书写、稳定
`id`、重复 ID 具名错误；受控 `expanded`/`selected`/`disabledIds`/
`loadingIds`/`errorIds` 与 `on_toggle`/`on_select`/`on_retry` 载荷；展开与选择
分离、chevron 与标签兄弟命中区；禁用不可达但可见；加载中 ≠ 失败；空态为正常
状态；键盘语义（↑↓/Home/End/→/←/Enter）、两端不回绕、禁用行跳过；MCP 可观察
面（行动作按稳定 ID、行状态在 class 中可见、旧 `vnode_` 不复用）；**包内模块
必须显式 `use`**；**行视觉状态用字面量分支**（`class:` f-string 不插值循环字段）；
**空态由派生哨兵行承载**（`is_empty` 类 computed 在 VM 轨陈旧）。

**SD-02 → `docs/specs/ui/tree-view.md`（验证面）**：双端验收（Linux/统一回归按各
计划 closeout 门记录，缺环境记 `not_run` 不记 pass）；MCP 稳定绑定（`vnode_` 绑定
当前渲染修订，数据变化后旧 id 必须被拒绝——native 实测 `stale_id_rejected: True`；
vue 按 DOM 位置寻址无修订身份，显式记不适用）；视觉证据；代表规模（1000 平铺 +
64 层嵌套，末行可达）与窄窗可读。**节点语义的显式可观察面**：`tv-tree` 根标记、
`tv-lvl-N` 行层级（1..4，超 4 为 `tv-lvl-deep`）、`tv-dis` 惰性行、`tv-sel` 选中行
——均在快照 `raw_class` 中可读；**本计划不声称 ARIA 合规树**（runtime 无 role/aria
通道），该缺口具名保留。
**SD-03 → `docs/specs/ui/examples-ownership.md`**：`packages/tree-view` 为
TreeView 单源；026/041 改用正式包并退役各自 `components/treeview.at`；示例保留
自身业务代码（026 的 `tree_util.filter_tree`/`collect_ids`/`toggle_id`、041 的
`editor_store` 与 `bps…toggle_id`）；语言 blueprint 仍是通用树工具 canonical
owner；018/027 的未实例化副本属清理项（不虚构消费）。

**附记（非 SD）**：018/027 无 `TreeView (` 调用位，仅在 `components/` 留有从未实例化的
副本；正式包的验收不以它们为消费者证据，清理归后续（记录于此避免误读）。