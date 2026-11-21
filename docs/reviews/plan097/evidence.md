# PLAN-097 实施证据

## 提交链（工作树 `codex/ui-097-tag-input`）

| 提交 | 内容 |
| --- | --- |
| 基线 `ad62e154` | master HEAD（工作树创建点） |
| `f48363ea` | TagInput 包 + demo 消费者 + native verify 脚本 |
| `029b54a0` | native+vue 门通过；Escape 重开与跨实例定位修复 |
| `28d9d253` | 015-notes 迁移（真实 noteId 绑定）+ notes 门 |
| `05e6a878` | 工具模型测试 + 过期候选守卫 |
| `8bc668b0` | notes 门重试（运行时交付时序） |
| `83467ad8` | 包/demo README、decisions/evidence 记录 |
| `1d45063f` | F-01..F-07 修复（backspace oracle、IME 守卫、X-07 UIA、text 语义、清理检查） |
| `771100ea` | 修复轮证据 |
| `30805dd3` | F-08 修复（candidates 有界快照，case ≤10s） |
| `734519ce` | F-09 修复（notes 有界轮询）+ F-10 修复（清理并入 result/exit） |

## 门矩阵（Windows）

| 门 | 入口 | 结果 |
| --- | --- | --- |
| Win native demo | `verify_tag_input.py --backend native --case demo` | pass ×3（逐 case：candidates-escape-enter / add-accept / dedup / remove / limit / reject / config-error） |
| Win vue demo | `verify_tag_input.py --backend vue --case demo` | pass ×3（同矩阵经 Playwright MCP 0.0.86 + Chrome） |
| Win native notes | `verify_tag_input.py --backend native --case notes` | pass（添加→chip→切换读回→正文不变） |
| tools 模型门 | `cargo test -p auto-ui-tools --test tag_input_component` | 6/6 pass（0.98–1.38s/轮） |
| Linux native / Linux vue | 缺 VMware runner | not_run（AC-06/04 对应平台保持未关闭） |

report.json 均在 `D:/autostack/plan-artifacts/plan-097/probe-*` 与
`vue-stab-*`/`notes-gate-*` 独立目录（含 tools/source 摘要、case→AC、cleanup/
reopen、last-tree/last-dom 与截图）。

## 视觉

- demo 各实例浅/深主题渲染（默认暗色 indigo，`切换明暗` 控制）；420×560 窗内
  三实例可滚动可操作；中文标签/错误文案真实渲染（截图随门产物 `native-core.png`
  等）。Q-01 文字字形：当前截图无缺字；公共渲染器问题未在本组件复现。
- 参考图区域（右下 Reading/Work 标签、de 候选、2/5 计数、重复错误态）与实现
  布局一致；图不冻结为像素金标准，以真实 GUI 可读可操作为准。

## 修复轮（2026-11-21，复审 needs_fix F-01..F-07 后）

| 门 | 入口 | 结果 |
| --- | --- | --- |
| Win native demo（含 X-07 + 中文/逗号/大小写） | erify_tag_input.py --backend native --case demo | pass ×3 |
| Win vue demo（含 IME 守卫 + backspace） | --backend vue --case demo | pass ×3 |
| Win native notes | --backend native --case notes | pass |
| tools 模型门 | cargo test -p auto-ui-tools --test tag_input_component | 10/10 pass |

- X-07：真实 Windows UIA 系统树（system-tree.json：role/name/value/focus/
  bounds），ValuePattern.SetValue + InvokePattern.Invoke 真实系统输入落地 chip，
  经普通 MCP 快照读回确认。
- IME：Vue 生成器 @keyup.enter 的 isComposing||keyCode===229 守卫经浏览器
  composition 事件序列验证（composing Enter 不提交、结束后提交）。
- F-01 backspace：模型层（BackspaceDelete 单次移除末位标签）+ Vue
  keydown.backspace 浏览器派发；native MCP 键盘对 input 键的绑定派发不可靠
  （运行时行为，decisions.md 记录），测试层选择见 §6 分层。
- 清理检查：port_free（进程退出后端口可重绑）与 reopen 为真实检查；连续门
  运行即重开证据。

## 修复轮二（2026-11-21，复审 needs_fix F-08 后）

- F-08：`candidates-escape-enter` 原 15.08–15.12s 超 10s 门；改有界快照模式
  （type→sleep→单次快照→单次重打），本轮独立复测 7.470s、随后 7.203s，关闭。

## 修复轮三（2026-11-21，复审 needs_fix F-09/F-10 后，HEAD `734519ce`）

| 门 | 入口 | 结果（fresh out-dir `plan-artifacts/plan-097/fix-f09-f10/`） |
| --- | --- | --- |
| Win native notes | `timing_wrap.py --backend native --case notes` | pass；case 墙时 **1.391s**（修复前 8.829–12.210s） |
| Win native demo | `timing_wrap.py --backend native --case demo` | 9/9 pass；逐 case 0.234–8.328s 全 ≤10s |
| Win vue demo | `timing_wrap.py --backend vue --case demo` | 9/9 pass；逐 case 0.531–6.234s 全 ≤10s |
| tools 模型门 | `cargo test -p auto-ui-tools --test tag_input_component` | 10/10 pass（1.04s） |

- F-09：notes case 固定 sleep（0.8/2.5/1.5/1.5）换有界轮询；chip 出现/空输入
  拒绝双目标快速判定，切笔记以「chip 随 active 笔记消失再重读」验证后端读回。
- F-10：`port_free` 有界重试（20×0.5s，TIME_WAIT 残留）并供 `reopen_probe`
  复用；`main()` 将清理/重开失败并入 result 与退出码（§6 契约：退出码 0 仅当
  全部必需 case 通过且清理重开成功）。
- 本轮全部清理 `port_free=true`、native `reopen.ok=true`。

## 待沉淀（merge 阶段落 docs/specs/ui/tag-input.md）

- SD-01：受控参数、消息、校验与身份/异步状态规则（§5 合同）。
- SD-02：包导入/移根、普通消费者、跨端 MCP/视觉/无人值守门及平台边界。
