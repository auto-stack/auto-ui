# ACE 迁移可行性：AGE 作者链与接口边界

Status: candidate（auto-ui PLAN-067 T-01..T-06 只读实测与裁决；本稿为
`docs/specs/auto-graphics/ace-migration-feasibility.md` 的 SD-01 候选文本，
merge 阶段按收据落位。只描述实际执行并核对过的能力与边界；不认领迁移完成、
ACE M1/M2 签章、AGE M1-ready、Schema/UI 融合或全量替换。）

## 0. 范围与裁决

PLAN-067 是独立研究合同：追踪 AGE 旧引擎依赖，先证「正常作者源 → 统一源码
前门 → 受检语义 → AV → 受检 host → 真实 GPU → 改源/错误恢复」，输出
go/bounded-go/no-go 与可执行后继接口。本稿是该研究的 canonical 知识沉淀。

裁决（2026-11-07）：**no-go（作者 profile 整链）**。依据是正常作者源在交付
的 ACE 统一前门**第一层（Text）**被具体拒绝，且同一前门上的相邻合法标量模块
正控通过、四条具名负控按预期拒绝。bounded-go 不成立（没有任何作者域支路可达）；
语义/AV/host/GPU/编辑/恢复支路因此 not-run。legacy 侧参考语义已与独立 oracle
对齐，但那不是 ACE 运行结果。

两候选裁决：A（Schema/统一前门直接领域投影）不可执行——目标语言构造整体缺失
（见 §2.2）；B（中立领域描述/受检 adapter）同样 no-go——它必须先自行解析
`node/view/msg/on/Life` 或复制一份领域语义核心，并把资源/回调/宿主语义编码进
标量模块，违反本合同「不调用旧 Parser/AST emitter、不正则改写源码、不另造
解释器」的禁令。裁决后停止替代机制探索。

## 1. 读取基线（冻结身份）

| 对象 | 身份 |
|---|---|
| auto-ui | `4bb54e36`（Plan 唯一树 `D:/autostack/.wt/ui-067/auto-ui`，分支 `codex/plan-067-dev`） |
| auto-lang / ACE | `436ac91f`；`experimental/ace` 子树 0 项修改 |
| ag-examples | `5384f7b`（clean）；`engine-lock.json`：engine commit `8b12632d`，`sdk_pin 7b987ca8`，`sdk_files 681`，17 个本地包，runner `autographics-authoring` sha256 `498ce814…` |
| 冻结 SDK 副本 | `sdk/frozen` 与生成的 `.sdk-source` 各 681 文件、逐字节全等；两钉不一致（`research/source-lock.json`=`932ad487` vs `baseline.json.txt`/engine-lock=`7b987ca8`）仍为 UNKNOWN，归 959 K-02 → 981/SOURCE → 987 |
| 旧链参考 | driver `Session`（`experimental/autographics-auto-driver/src/lib.rs:34,57,59,85,86,90`：AutoVM/AutoTask/旧 Parser/`create_vm_from_source`）；pilot `src/load.rs:8,11,141,147-150`（旧 Parser + `Node2D*Dialect`）；AGP 渲染 `experimental/autographics-prototype`（wgpu 27.0.1） |

## 2. ACE 已验证支持域（只述实测）

### 2.1 源码 profile 与能力

`auto-lang/experimental/ace/grammars/auto-source.at` 的 profiles(version:1) 共 13 个：
`core`、`scalar`、`text`、`record`、`text-record`、`entry-unit`、`host-console`、
`collections`、`closures`、`scalar-instances`、`scalar-modules`、`packages-modules`、`scalar-ct`。
公开能力常量：`ac.source.modules.scalar/v1`、`ac.source.instances.scalar/v1`、
`ac.source.cte.scalar/v1`。

有限模块域（`core::source::modules::parse_module_source` / `bind_modules`）：
仅 i32/bool 顶层自由函数；`use` 只接 `<module>.<name>` 形式；无包/目录发现、
通配/别名/重导出、循环模块、公共类型/资源或任意泛型；重复模块/函数/导入、
未知模块/符号、跨模块私有访问与循环在绑定前具名拒绝。模块绑定**不**铸
Checked 凭证：签名/类型/arity 由下游唯一 resolve/typecheck 核验。

### 2.2 明确不在当前交付域

`node`、`model`、`msg`、`view`、`on`、`Life`、`view-2d`、`view-3d`、`shape2d`、
`cylinder3d` 等 AGE 作者构造不在任何 profile；无公开 f32 位置 lane（profile
词法把浮点字面量归 `Raw`，报 “not part of this source profile”）；无图形资源
句柄/代次/释放、无同步回调、无 UI host/文本槽/事件派发面。

### 2.3 正控与负控（同一前门、同一构建）

| 控制 | 输入 | 结果 |
|---|---|---|
| 正控 | `std`: `pub fn answer() int { return 42 }`；`entry`: `use std.answer` + `pub fn main() int { return answer() }` | accepted（`bind_modules` Ok，units≥2，items≥1） |
| 负控 | `use nope.answer` | rejected `source.module-missing` |
| 负控 | `use a.f` + `use b.f` | rejected `source.module-duplicate-import` |
| 负控 | 跨模块访问私有 `hidden` | rejected `source.module-private` |
| 负控 | a↔b 循环 import | rejected `source.module-cycle` |

观察：`answer(1)`（错参数）在绑定期被接受；类型/arity 属下游，不把绑定期接受
当作类型已验。

## 3. 正常作者源实测拒绝

| 输入 | 字节 | 入口 | stage | code | span | span 文本 | message |
|---|---|---|---|---|---|---|---|
| `main.at` | 1990 | modules + scalar | text | `source.unsupported` | 486..489 | `use` | `` `use` is not part of this source profile `` |
| `counter.at` | 1025 | modules + scalar | text | `source.syntax` | 319..323 | `node` | `` expected `fn`, found `node` `` |
| legacy `world.at` | 15689 | modules + scalar | text | `source.unsupported` | 228..231 | `var` | `` `var` is not part of this source profile `` |

解读：阻断在首个 Text 门，早于绑定/类型/HIR/AV/host；`use counter: Counter`
的作者层导入形式与 `node` 声明都不在当前文法；连 legacy 已生成的真实业务源
（`var` 顶层状态、数组、文本缓冲）也在域外。三个输入在模块能力与标量 profile
两条入口上同判，排除单入口选择问题。

## 4. 最低样本与独立 oracle（可复现）

样本语义：普通 `use` root（`Main`）+ 两个同定义 `Counter` 实例（`a`/`b`），
各自 `model`/`msg`/`view`/`on`、`Life.Start`/`Life.Tick(dt)`；root 的 `.Step`
同时驱动 `view-2d reg2`（quad `x=.reg2x`）与 `view-3d reg3`（cylinder
`translation x=.reg3x`）；组件局部 `.Bump` 只影响被点实例；过程几何与作者文本，
无第三方素材、无 Rust 状态、无手写 HIR。

冻结场景（`scenario.json`）：5 × `Life.Tick`、2 × `.Step`、1 × `Region2`、
1 × `Region3`、`a` 一次局部 `Bump`、`b` 零次。独立 oracle（只读作者源文本重导
常量）：

| 量 | 期望 |
|---|---|
| root.steps | 5 |
| root.reg2x / reg3x | 14 / 11 |
| a.count / a.bumps | 6 / 1 |
| b.count / b.bumps | 20 / 0 |
| 改源 `b.step: 2→3` 后 b.count | 25（其余不变） |

已执行对照（legacy 交付前门 `load_project_mixed` + `lower_project_node_mixed_regions`，
来源 `4bb54e36`）：样本被接受，产出 `schema=autographics.node-ui-mixed-regions/2`、
`world_sha256=1599d225623e9874d8fd1731888fd77c3a9a0fb5e1e91293f060d0c8df4218b9`、
15,689 B；生成源 `app_start`/`app_update` 的 per-instance fan-out、
`counter_lifetick` 加 `step`、`.Step` 位移 6/4、区域点击 2/3 与 oracle 常量逐项
相同（`counter_start=[0,10]`、`counter_step=[1,2]`）；改源后 `counter_step=[1,3]`。
这些是**参考语义**证据，不是 ACE 运行结果。

## 5. 未证边界与后继接口（不另立重复合同）

| # | 未证缺口（具体符号/类型/接口） | 接收 owner（既有合同任务） | 关闭 oracle |
|---|---|---|---|
| G1 | 作者层 `use <name>: <Def>` 组件导入 | 981 `B-SOURCE-I/V/R`；979 `T-08` 组合相容表 | 该语法被接受且 span/诊断保真；981 永久 fixture |
| G2 | `node` 声明与 `model`/`msg`/`view`/`on` | 981 `B-SOURCE-I/V/R`；979 `T-02` 主能力、`T-08` 领域规则 | 同文件经统一前门产受检 HIR；反例具名 |
| G3 | `Life.Start`/`Life.Tick(dt)` 调度与 per-instance 推进 | 981 `B-SESSION-I/V/R`；968 `T-02/T-03`；967 `T-03` | N 步 N 次且实例隔离；tick/回调/停止可核 |
| G4 | `view-2d`/`view-3d` 区域、`shape2d`/`cylinder3d` 呈现 | 981 `B-SOURCE-*`；980 `B-DIALECT-I/V/R` | 区域点击→状态→像素 ROI 一致 |
| G5 | 公开 f32 位置/布局 lane（含舍入/NaN/Inf/-0 判据） | 961 `T-02/T-03`（完整数值）；979 `T-02/T-08` | f32 独立舍入 oracle；不得强制 f64 cast |
| G6 | 图形资源句柄/代次/释放/同步回调 | 939 `T-03..T-08`；967 `T-08` | 代次拒收、释放幂等、回调错误通道 |
| G7 | UI 文本槽/事件派发/宿主接面 | 980 `B-NATIVE-I/V/R`；967 `T-02/T-03`；968 | 同 PID 更新、事件隔离、宿主装配 |
| G8 | 生成物 `var`/数组/文本缓冲源码形态 | 979 `T-02/T-03`；981 `B-SOURCE-*` | 该形态被接受或官方替代 lowering 等价 |
| G9 | 签名/arity 在绑定期不核（下游 typecheck 才核） | 979 `T-02/T-03`（唯一 resolve/typecheck） | 坏签名在发布前具名拒绝 |

最终认证/发行/退役门归 985/986/987；本表不重复其职责，也不把研究结论当迁移
完成。959 保持唯一产品分母；本稿只引用其 F-11/K-02 行，不维护第二套总表。

### 5.1 REP 分账（研究不签章）

| REP | 内容 | 状态 | owner |
|---|---|---|---|
| REP-01 | 旧执行器替换/接管 | pending | 981 `B-SESSION-*` + 967/968 |
| REP-06 | 宿主完整性/嵌入面 | pending | 980 `B-NATIVE-*` + 967 |
| REP-08 | 安装/发行图 | pending | 981 `B-CONTENT-*` + 986 |
| REP-10 | vendored 旧 SDK 副本清理（`.sdk-source` + `sdk/frozen`，各 681 文件） | pending | 981/SOURCE 定钉 → 987 |
| REP-11 | 测试迁移义务 | pending | 985 + 981 `B-CONTENT-V/R` |

未归属 unknown：无。UK-06（两钉 `932ad487` vs `7b987ca8` 并存且不一致）已具名
移交 981/SOURCE 定操作钉、987 清；不属未归属。

## 6. 未执行（not-run）与不可达因果

| 门 | 状态 | 因果 |
|---|---|---|
| ACE 受检语义/HIR/AV | not-run | 首个 Text 门拒绝，未产 AST/事实 |
| ACE 受检 host / UI 事件 | not-run | 同上 |
| 真实 GPU / 2D/3D 像素 ROI | not-run | 同上；不主张 GPU 失败或成功 |
| 源编辑/增量/同 PID | not-run | 无 ACE 目标；legacy 仅有产品级改源 |
| 冷恢复/取消/Stop/20 轮/Run 代次/资源收支 | not-run | 无 ACE 运行会话；REP 不由研究签章 |

## 7. 复现入口

Plan 材料 scope（merge 前）：`D:/autostack/plan-artifacts/plan-067/auto-ui/`。
`python probe/run.py --out <dir> --controls` 依次执行 oracle、legacy 前门、
改源变体与 ACE 前门（含正/负控）；输出 `report.json`（`result ∈ pass|partial|fail`），
退出码 0=无失败阶段。ACE 前门探针以库方式依赖 `ace-core`（只读，独立 target）。

本 Spec 附录 A/B 保存冻结样本与 oracle 期望，使上述诊断 span 与期望在材料
scope 删除后仍可重建。

## 附录 A — 冻结样本（逐字，2026-11-07）

`main.at`（1990B，sha256 `7a14519f2fafd472110d8857625fa70b5a3711fd25884cadadc30e037eabc144`）：

```auto
// PLAN-067 T-02 frozen minimum-sample root.
// Ordinary `use` root: one UI action (.Step) drives one view-2d region and one
// view-3d region; two same-definition Counter components are instantiated with
// different constants. Life.Start/Tick on both root and components; local
// component events stay per-instance (2 instances : 1 affected).
// Procedural geometry only (quad / cylinder3d) + author text; no third-party
// asset, no hand-written HIR, no Rust gameplay substitution.
use counter: Counter

node Main {
    model {
        var steps = 0
        var reg2x = 0
        var reg3x = 0
    }
    msg {
        Step
        Region2
        Region3
    }
    view {
        column root {
            spacing: 8
            padding: 12
            text step_text { text: .steps }
            button step {
                text: "步进"
                onclick: .Step
            }
            Counter a {
                start: 0
                step: 1
            }
            Counter b {
                start: 10
                step: 2
            }
            row regions {
                view-2d reg2 {
                    width: 160
                    height: 90
                    onclick: .Region2
                    shape2d mover { kind: quad, x: .reg2x, y: 20, w: 24, h: 24, color: (200, 40, 40) }
                }
                view-3d reg3 {
                    width: 160
                    height: 90
                    onclick: .Region3
                    cylinder3d spin { translation: (.reg3x, 0, 0), color: (90, 200, 90) }
                }
            }
        }
    }
    on {
        Life.Start -> {
            .steps = 0
        }
        Life.Tick(dt) -> {
            .steps = .steps + 1
        }
        .Step -> {
            .reg2x = .reg2x + 6
            .reg3x = .reg3x + 4
        }
        .Region2 -> {
            .reg2x = .reg2x + 2
        }
        .Region3 -> {
            .reg3x = .reg3x + 3
        }
    }
}
fn main() {}
```

`counter.at`（1025B，sha256 `8734bef74b65616222dc5ba9e40d90b891ab3a1c1f59a6363ea773325fdaaa44`）：

```auto
// PLAN-067 T-02 frozen minimum-sample component.
// Ordinary `use` component with the same definition instantiated twice:
// model/msg/on + Life.Start/Life.Tick, per-instance state, local event Bump.
// No Rust state and no hand-written HIR: every observable below is business
// state declared in this author source.
node Counter(start: int, step: int) {
    model {
        var count = .start
        var bumps = 0
        var label = "计数"
    }
    msg {
        Bump
    }
    view {
        row card {
            padding: 4
            text name { text: .label }
            text value { text: .count }
            text hit_count { text: .bumps }
            button bump {
                text: "加一"
                onclick: .Bump
            }
        }
    }
    on {
        Life.Start -> {
            .count = .start
        }
        Life.Tick(dt) -> {
            .count = .count + .step
        }
        .Bump -> {
            .count = .count + .step
            .bumps = .bumps + 1
        }
    }
}
```

## 附录 B — 独立 oracle 规则（期望表见 §4）

oracle 只读 `main.at`/`counter.at` 文本，提取实例 `start`/`step`、root 的
`Life.Tick` 增量、`.Step`/`.Region2`/`.Region3` 位移、组件 `Life.Tick` 与
`.Bump` 的加数，再按场景做加法求值：

```
root.steps = T × tick_add
root.reg2x = presses.Step × d(Step→reg2x) + presses.Region2 × d(Region2→reg2x)
root.reg3x = presses.Step × d(Step→reg3x) + presses.Region3 × d(Region3→reg3x)
instance.count = start + T × per_tick + bumps × per_bump
instance.bumps = bumps × 1
```

改源变体把 `b.step` 从 2 改为 3，期望仅 `b.count` 变为 25；oracle 不读取产品
产物，因此匹配是证据而非复述。
