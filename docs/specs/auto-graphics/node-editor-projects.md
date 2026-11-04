---
title: AutoGraphics 纯声明混合维度 Node 编辑工程与项目化验收
status: candidate
plan: auto-ui/PLAN-040 r1
applies-to: node2d-lower --profile editor-projects（pilot experimental/autographics-node2d-pilot）；消费 experimental/autographics-auto-driver（editor Host/Storage；tests/node_editor_projects.rs）
assembly: 零新行为装配——behavior:null 静态场景（autographics.scene/6），无用户 VM、无角色、无运行相机/空 Tick
date: 2026-11-04
---

# editor-projects：G04 忠实纯声明双维编辑工程、基线/副本身份与项目化编辑证据

## 0. 范围与机制（T-02 冻结）

编辑工程入口 `--profile editor-projects` 是 039 `queries-3d` 静态双出口机制的
**身份孪生**：同一 dual dialect 解析、同一 `lower_project_queries3d_identity`
管线、同一 scene/6 产物规则与全部具名诊断；唯一差异是冻结身份串
（document_id 前缀 `plan040-editor-projects`、生成说明/再生控制注记/消费者
登记）。不为纯场景新增语法、渲染器或 profile 语义。支持域冻结为**纯声明**
：无 `model` state、无 `on`（stateful/Life 均无）⇒ `behavior:null`、
`user_source:none`、无 world.at；带 stateful/Life 的源在该入口具名拒绝
（`editor-projects accepts pure declarative projects only: behavior must
stay null (stateful/Life authoring belongs to the runtime profiles)`）。
双出口准入与旧档拒绝沿 039 逐字节不变：仅 root 允许 `view-2d`
（`view-2d is only allowed on the queries-3d root node (child nodes belong
to one dimension)`），默认 dialect 对双 view 的解析期拒绝字节不变。

## 1. 生成面（T-02/T-03 冻结）

- view-3d 叶：`player3d/mob3d/cylinder3d`（027 内置视觉/图元）与 `model3d`
  （GLB）；view-2d 标记：`shape2d`，kind 仅 `circle|quad`，属性仅
  `kind/position/scale/rgba/visible` 字面量（动态表达式具名拒绝：
  `marker numeric prop must be a literal number`；未知属性具名拒绝：
  `unknown marker prop '<p>' (kind/position/scale/rgba/visible)`——parent/
  yaw 等一概不在标记词汇内）。标记无 parent（顶层屏幕空间叶，z 恒 0）。
- 普通具名组件/实例（`use` + `node Def(params)` + `Def inst { … }`）按
  024/028 既有机制降低（实例叶行名 `Instance/elem`）；两同定义纯视觉实例
  属支持域（gates `instances` 夹具）。
- 行序：三维行（声明序）在前，root view-2d 标记行附于全部三维行后；
  二维数组绘制序 = 标记声明序。包络沿 039：行名唯一
  （`duplicate scene row name`）、≤128 行。
- 三维行 rgba 固定占位字节（0.9,0.3,0.2,1.0）；标记行携带作者 rgba。
  identity.json/closure//assets/ 沿 039 产物结构；behavior:null 时不写
  world.at，`user_source_sha256` 为 null。

## 2. G04 忠实夹具与显式 ID 映射（T-01/T-02 冻结）

永久夹具 `fixtures/editor-projects/{g04-editor-2d,g04-editor-3d}/`，冻结
产品 `generated-editor-projects/`（document_id `plan040-editor-projects-
main`，schema autographics.scene/6，behavior:null，无 groups/assets）。
原工程（ag-examples@501fca0 `examples/g04-editor`，schema/4）对象逐值保留
（kind/translation/scale/yaw/pitch/rgba/visible/parent）；生成行序 ≠ 原行
序（三维行先），映射显式双向：

| 工程 | 原对象（id:kind） | 生成行（id:symbol） | 映射 原→生成 |
|---|---|---|---|
| g04-2d | 1 circle, 2 quad, 3 player, 4 mob, 5 helper quad, 6 helper circle | 1 obj3_player, 2 obj4_mob, 3 obj1_circle, 4 obj2_quad, 5 helper_quad, 6 helper_circle | 1→3, 2→4, 3→1, 4→2, 5→5, 6→6 |
| g04-3d | 1 circle, 2 quad, 3 player, 4 mob, 5 helper cylinder | 1 obj3_player, 2 obj4_mob, 3 helper_cylinder, 4 obj1_circle, 5 obj2_quad | 1→4, 2→5, 3→1, 4→2, 5→3 |

二维等 alpha 序：每工程 2D-kind 行的相对顺序 = 原工程 2D-kind 对象的数组
相对顺序（2d：circle→quad→helper quad→helper circle；3d：circle→quad）。
展示名/作者 symbol/运行 ordinal 不混用：原对象名（"Object"/"helper …"）
不进入生成行名，消费映射只走上表。

## 3. 编辑消费与基线/副本身份（AC-03/04）

公开链：`Storage::load_project`（真实存储路径，schema/5..8 解码）→
`Host`（编辑器一窗双视口：2D 800×380、3D 800×400；下 400 像素行为 3D
区域）→ 实际 Auto 交互（pointer 手势、`interaction::axes/handle/AxisDrag`、
snap `view3d-snap`）→ 真实 GPU 合成（`rt.probe_pixel(s)`/`rt.capture`）。
保存/关开/SaveAs 沿既有 Storage 合同（epoch/revision 邮票；旧
revision/坏闭包/保存故障 → 保 last-good 或准确残留，修复后同入口重开绿）。
基线不可变：编辑动作不暗改作者源/生成基线字节；`saveas` 建独立副本，副本
编辑态经 save→close→reopen（新根）保留；副本不伪装基线 hash；再生成基线
是另一显式动作（`verify_plan040.py` 再生控制），不静默覆盖副本。Play 沿
显式 Run 的原 Builtin 路线（behavior:null 空 Tick），Stop 还原编辑相机与
接受内容。

## 4. 项目化编辑证据与原域（AC-02/03；D-02）

原 verify_g04 九 require 与 20/34 阶段保持为**旧正控**（gallery 验收，
真实 OS 输入）；本档不替换、不删除、不代跑。项目化证据走无抢焦点分层：
真实 Editor 消息（epoch/revision/view_revision/view3d_revision/play_token
邮票；错 token/旧 revision 具名拒绝）、实际 Auto 交互计算、真实 GPU 合成
读回。`tests/node_editor_projects.rs` 逐阶段映射 20/34 全表：每阶段绑定
"auto"（消息/Auto/GPU 证据）或 "os"（真实 OS 输入专属——焦点获取、物理
键/轮/按钮流、text-focus 隔离、跨按钮捕获），os 层 = 具名 pending
（D-02），不删项、不标 Ready；手工按原阶段绑定新工程/runner 可补真实交互
取证。20 轮 open/edit/save/reopen 资源 owner 结算：uploads 零增长、历史
零残留；导航帧不重传静态 mesh（暖上传 owner 保持）。原预算/包络不减：
128 场景行、grid 128 线/16KiB、4096 quad、snap/3px、viewport 尺寸沿旧档。

## 5. 支持拒绝域与非目标

超域具名拒绝：stateful/Life（上式）、子 Node 双 view、标记未知属性/动态
表达式/重复行名。非目标（沿 Plan §1）：通用 AST 反向源码回写、全局
HMR/动态拓扑/ECS、任意生产 UI 嵌 3D/多屏变体、游戏玩法/物理或新运行
API、跨端/生产 UI 融合；039 有限查询档不自动扩为所有 Node 的混合能力；
带行为的编辑工程不走本入口（运行档各 profile 的域）。真实 OS 键鼠自动化
维持暂停政策（m2-editor-and-release-convergence §4）。
