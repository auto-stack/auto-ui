# PLAN-086 实施证据汇总（T-06）

日期：2026-11-14。工作树 `D:/autostack/.wt/ui-086/auto-ui`，分支 `codex/ui-086-pagination`。
原始探针工件（命令/日志/截图）在 `D:/autostack/plan-artifacts/plan-086/`（仓库外 scratch，
合并时按政策清理）。本文只留可复核的索引：命令、提交、门结果、边界。

## 提交链（本分支）

| commit | 内容 |
|---|---|
| `8e8eb85f` | T-01 decisions.md（API 冻结 + 探针结论） |
| `7eb39915` | T-02 packages/pagination（Init 派生首版） |
| `54712797` | T-02r/T-03 computed 派生 + eval.rs 四则臂 + 双端整除公式 + 静态具名错误 |
| `a8ec40ec` | T-04 双消费者迁移 + examples/pagination + vendor 闭包夹具 |
| `9700f1ee` | T-05 verify_pagination.py（native+vue）+ tools test + shadcn disabled 表达式适配 |
| （本次） | T-06 README/证据/SD 终版 + T-05 修正 |

依赖工作树组 `.wt/ui-086/{auto-lang,auto-web,auto-down,auto-man}`：全为 master 检出、
零源码改动；补丁版 `auto.exe`/`auto-ui-tool.exe` 由该组构建（组内同级布局使各仓
相对 path 依赖原样解析）。

## 门结果（§6 命令 → 结果）

| 命令 | 结果 |
|---|---|
| `cargo test -p auto-ui-tools --test pagination_component` | **2/2 pass**（双生成器 + vue 面 float-safe ceil 形态断言 + vendor 闭包 + strict 具名失败负例） |
| `cargo test -p auto-ui-tools --test file_dialog_component` | **3/3 pass**（官方包双生成器 + components 布局闭包 + vendor+dep 闭包，均含 PaginationControl 断言） |
| `cargo test -p auto-ui-runtime --test file_dialog --test file_dialog_model` | **8+10 pass**（迁移后组件级行为门 + 模型门） |
| `python tests/product/verify_pagination.py --auto-bin … --ui-tool-bin … --out-dir …`（默认窗） | **outcome: pass**（native 全门 ×2：四状态、受控回路实切片 oracle、容量 25→41 页、disabled、stale-reject、V-08 浅/深截图、NVM-05 退出+端口回收+重开） |
| 同上 `--window 420x560` | **outcome: pass**（窄窗腿，分页单行） |
| 同上 `--backend vue` | **outcome: pass**（vite+chromium 同矩阵 + 受控回路 + 截图） |
| `python tests/product/verify_file_dialog.py …` | **outcome: pass**（原产品门携迁移后包全量通过——1003 项 11 页流、逐页计数、尾项可选、tooltip 匹配） |
| `python examples/ui/026-database/tests/desktop_mcp.py`（AUTO_BIN=补丁版） | **25/25 pass ×2**（T6 重接线：新分页控件发现+状态页=显示页） |

lib 套件基线对照：worktree 52 失败 vs master 48 失败（桌面环境类噪声：窗口/悬停/
launch 类，10 新 6 复原、无 eval/computed 关联名）；统一回归归 review/merge 门主。

## V-08 视觉证据（r2）

`vp-native-13/`（900x700）：light-multi / dark-multi（整面主题 token 生效）/
light-single-hidden（单页整组不存在）。`vp-narrow-2/`（420x560）：分页单行保持。
`vp-vue-16/`：vue 轨 light 两态。截图由 autoui_screenshot（native）/playwright（vue）
产出并记 sha256 于 report.json。

## 已记录的边界与风险（不静默）

1. **Linux native 轨未运行**：本机 WSL Ubuntu-22.04 无 rust/node 工具链（WSLg 在位）。
   门脚本已带 X11 适配；runner 就绪即可原样执行。不记 Linux 通过。
2. **vue checkbox `text:` = 选中指示器 slot**（shadcn 惯例，全仓既有，FileDialog vue 同）：
   可见标签文本不在 DOM；产品门 vue 腿按声明顺序定位，组件本体不使用 checkbox。
3. **VM 轨缺依赖静默缺席**（build 轨具名失败已实证）：native 消费者需依赖 build 轨负例门。
4. **Init 内 emit 不派发**（native）：错误走可见具名文本，error 事件不设（API 偏差已记录）。
5. **prop 字面量负数**在 VM 轨静默失效：负例测试用状态绑定（真实消费形态）。
6. **keyboard**（Tab/Enter/Space 激活）：后台窗探针不触发；file-dialog 门同基础设施通过
   （模态焦点路径）；组件不注册全局快捷键。T-05 产品门未含键盘激活腿（与本门同条件的
   焦点管理归 file-dialog 模态路径），已如实记录。
7. 每次双列（native/vue）门跑完后残余进程/端口已清理；`auto run` 冷启动的
   auto-ui-tool 探针偶发失败（既知基础设施噪声，重跑即过）。

## Spec delta 终版（对 §5 表的落地核对）

- SD-01（add `docs/specs/ui/pagination.md`）：按 README/decisions 冻结面落地——
  props/事件/派生公式/错误语义/闭包三件套；AC-01…03 关闭。
- SD-02（add，交付门）：native Win + vue 双轨 + V-08 视觉 + vendor 闭包 + strict 负例；
  Linux 轨未运行已显式记录；AC-04…06 关闭（Linux 部分为显式边界）。
- SD-03（modify `docs/specs/ui/file-dialog.md`）：底栏分页改 PaginationControl、
  保留 pageSize=100/单页隐藏/第 n 页文案、闭包说明（双包 vendor）；AC-05 关闭。
- AC-07/08（review/merge）未到；本轮不写 canonical Specs、不写派生索引/ledger。
