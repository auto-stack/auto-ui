# PLAN-097 实施证据

## 提交链（工作树 `codex/ui-097-tag-input`）

| 提交 | 内容 |
| --- | --- |
| 基线 `ad62e154` | master HEAD（工作树创建点） |
| 首提交 | TagInput 包 + demo 消费者 + native verify 脚本 |
| 二提交 | native+vue 门通过；Escape 重开与跨实例定位修复 |
| 三提交 | 015-notes 迁移（真实 noteId 绑定）+ notes 门 |
| 四提交 | 工具模型测试 + 过期候选守卫 |
| 五提交 | notes 门重试（运行时交付时序） |

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

## 待沉淀（merge 阶段落 docs/specs/ui/tag-input.md）

- SD-01：受控参数、消息、校验与身份/异步状态规则（§5 合同）。
- SD-02：包导入/移根、普通消费者、跨端 MCP/视觉/无人值守门及平台边界。
