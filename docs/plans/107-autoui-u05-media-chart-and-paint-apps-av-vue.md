---
plan_id: PLAN-107
status: execution_done
feature_name: AutoUI 024 Charts 与图表 Widgets AV 与 A2Vue 交付
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-21
plan_revision: 2
current_step: 6
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: [docs/specs/runtime/ace-package-widgets.md]
new_spec_components: []
touched_goals: [auto-lang/GOAL-002]
---

# PLAN-107 AutoUI 024 Charts 与图表 Widgets AV 与 A2Vue 交付

## 0. 变更摘要

2026-11-21 r2。恢复用户已批准的 U04–U06 拆分口径。107 持有 024 Charts 和五个图表 Widgets 原页；U05 其余行按下列归属交付。现有实施树、T-01 已完成状态、原样回放和不通过证据继续沿用，整组原验收由路线索引逐行对账。

2026-11-19 按用户裁定建立独立交付计划。承接语言 980 B-AV、1071 冻结清单中 U05 的原示例和关联 Widgets，验收 ACE AV 桌面和 A2Vue 浏览器。由本计划负责 AutoUI 和所需 AutoLang 改动，并补齐这些应用实际用到的缺失组件。A2R、AC 由后续独立计划接同源验收。本次为规划，进度 0/8。

## 1. 目标

交付原 024 Charts 和五个独立图表 Widget 页的两端可复跑结果。复用原 `pac.at`、作者源、依赖及独立预期。交付可从干净目录构建和运行的项目、固定工具与 SDK 版本、每端操作记录及独立复审结论。

| AutoUI 原例 | 两端项目级检查重点 |
|---|---|
| [024-charts](../../examples/ui/024-charts/pac.at) | 可见图形、数据更新、指针/图例交互及空数据 |

关联 Widgets 使用 [AutoOS 规范来源](../../../auto-os/widgets-gallery/src/front/) 的 pages 与实际 components 闭包。area-chart、bar-chart、donut-chart、line-chart、flow-diagram。T-01 把 1071 标识逐项对应到实际页、组件与原测试，给复用页记录版本和各组消费结果。

其余 U05 应用由 [115](115-autoui-video-app-av-vue.md)、[116](116-autoui-media-players-av-vue.md)、[117](117-autoui-image-apps-av-vue.md)、[118](118-autoui-paint-av-vue.md) 承接。Carousel 归 117，CodeEditor 原页归 [120](120-autoui-auto-edit-av-vue.md) 并仍计 U05。原 T-01 首轮对照及已录证据见 §9，接手计划按原路径、预期及证据身份复用。

## 2. 架构与跨仓职责

- **AutoUI 主责。** 负责示例、组件契约、Iced 宿主消费、Vue 生成产物及真实交互。复用图表组件，验真实几何、数据更新、指针/图例交互、空数据、模式切换和关闭。资源回收采用实际宿主和浏览器观测。
- **AutoLang 为同一计划的实施仓。** 核图表输入记录、几何计算、时序、指针数据与事件载荷的跨端类型。补实际缺失的 AutoLang 检查、AV 宿主调用及 A2TS 运行时映射，验证卸载和资源释放。通过同一原源与资产对比两端可观测状态。 相关改动在组内 AutoLang 树完成，沿原模块 owner 审查与合入。
- **AutoOS 承担已批准的最小源修。** 修复原 Area/Bar 的空数据处理与原 FlowDiagram 的节点高亮、提示锚点。冻结版保留复跑证据，修正版在同一画廊外壳和原页验证 AV 与 A2Vue。
- **真实组件。** 对本组使用的每个缺失组件或缺失行为，优先补已有组件，再补所需新组件。交付适用属性、事件、绑定、生命周期和无障碍行为，两端实现、永久回归及原项目调用都计入本计划的完成条件。已有其他计划在写同一组件时，先对账实际界面与写面，固定单一实施负责人，当前计划持有本消费者验证及关闭条件。

## 3. 技术与实现入口

AV 沿 `crates/auto-ui-runtime/src/ui/ace_node_desktop.rs` 的公开宿主路径、受检 AGE Schema 和 typed 事件。A2Vue 沿 `crates/auto-ui-runtime/src/ui/ace_vue_script.rs`、`ace_ts_provider.rs`、`ace_package_sfc.rs` 和正式 `auto-ui-tools` 入口，验生成的 Vue/TS、运行时与资源。AutoLang 优先复用已有 AGE/ACE、AV 和 A2TS 管线。Windows AV 及所选浏览器配置各有实测记录，其他原适用平台按冻结的运行条件记账。

## 4. 来源与前置

- [980 原 W1 和 ④-A 设计](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 和 [1071 示例/Widgets 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)。已有运行收据按实际输入、范围和版本复用。
- [AV 宿主及 W1 收据](../../../auto-lang/docs/specs/ace-migration/m2-autoui.md)、[1072 Vue 生产计划](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)、[976 TS/JS 交付](../../../auto-lang/docs/plans/archive/976-ace-m2-typescript.md)、[978 构建交付](../../../auto-lang/docs/plans/archive/978-ace-m2-build.md)。
- U02–U06 按用例所需接口推进。遇到共享的 ④-A、事件、集合、外部服务接面或组件缺口，记录具体接口、实现负责人、交付版本与消费测试，已具备前置的样例继续验收。1072 的 982 联合 CLI 验收可消费本计划的 A2Vue 证据。
- 2026-11-20 用户以 `/goal 完成计划107` 授权 r1 原范围的实施、独立复审、限定修复和落地归档清理。
- 2026-11-21 用户确认当前 goal 按 r2 收口，并授权最小 AutoOS 源修。原版和修正版各有可核对输入，最终验收用修正版。

## 5. 实施与交付边界

沿已冻结的 024 与五个图表页矩阵，逐行绑定项目原源、Widget 页和组件、预期、失败反例、权限、资源、版本和 AV/A2Vue 状态。从原项目的第一个失败处建立可复跑入口，修所用能力或组件，回同一项目完成两端验收。公共语义走原 owner 的统一接口，源级修复覆盖 AV 和 A2TS 的实际消费者。

实施使用 `D:/autostack/.wt/ui-107/{auto-ui,auto-lang}` 的关联工作树和 `D:/autostack/.wt/os-107/auto-os` 的源修工作树。用到其他仓版本时明确固定提交，按组内路径或既有显式覆盖解析。合同、进度和归档在 AutoUI 主检出管理，实施和构建在工作树。跨仓落地顺序以实际生产者和消费者验证为准。

### T-01 原样回放矩阵

应用入口均为 `examples/ui/<行名>/pac.at`、`src/front/app.at` 及其原始模块。画廊入口为 AutoOS `widgets-gallery/pac.at`、`src/front/app.at` 与 `src/front/pages/<行名>.at`，组件从原 `components/package.at` 解析。下表基于 §9 的 Git 来源。两端保留原有适用性。按 002 对照、024、四图原页及 FlowDiagram 推进；真实 Iced 窗口串行排程。

| 原样输入 | 实际闭包及独立预期来源 | AV 与 A2Vue 必验状态及负控 |
|---|---|---|
| `024-charts` | `components/{line_chart,bar_chart,area_chart,donut_chart,chart_geom}.at`；`SPEC.md`、`tests/desktop_mcp.py`、`tests/test_024_vue.mjs` | 四类图有实际几何，更新数据后几何变化，指针命中、图例操作及空数据 |
| `area-chart` | `components/area_chart.at`；原页 `area-chart-overlay`、图表 `plan484` 与 `plan499` 回归族 | 叠加区填充和顶边几何、数据与指针回放、空数据 |
| `bar-chart` | `components/bar_chart.at`；原页 `bar-chart-basic`、`bar-chart-grouped` 与 `plan484`/`plan499` | 单组/分组柱几何，命中与更新，空数据 |
| `donut-chart` | `components/donut_chart.at`；原页 `donut-chart-basic` 与 `plan484`/`plan499` | 各段弧、指针/图例状态，空数据 |
| `line-chart` | `components/line_chart.at`；原页 `line-chart-basic`、`line-chart-multi` 与 `plan484`/`plan499` | 单/多列路径、指针提示、更新与空数据 |
| `flow-diagram` | `components/flow_diagram.at`；原页 `flow-diagram-lr`、`flow-diagram-basic` 与 `plan502_diagram_tests.rs` | 左右、上下两方向独立布局；节点命中，高亮/提示及指针离开复原 |

单写面从实际代码分工。105 的 AGE Store/列表语义使用有效落主版本。114 持有源包、组件属性、列表及事件的共享接点；123 持有其在途的生产桌面入口及路由源发现。107 持有图表原源、消费回归及图表所需的共享能力增量。无路由的 024 与独立原页推进时，路由接缝按正式可消费提交接入；画廊整包在 114 树/包接缝和 107 图表接缝具备后复验。

### 规范增量

| 操作与目标 | 现有合同与本次增量 | 对应验收 |
|---|---|---|
| 修改 AutoUI `docs/specs/runtime/ace-package-widgets.md` | 既有 AV/A2Vue 投影增加真实 SVG 绘图、`coords: WxH` 与绝对元素位置缩放、受检子组件信号、指针与图例、关闭及空输入的约定。图表从作者 View 的模型与绑定读取几何和事件。 | AC-02、AC-03 |
| 修改 AutoLang `docs/specs/ace-migration/m2-autoui.md` §17 | 既有 AGE 包接缝增加静态调用点的 List 和名义记录类型、旧稀疏形状兼容边界、受检参数和文本所有权、一次模型物化及原有资源上限。 | AC-02、AC-03 |

两处均更新现有组件规范。AutoOS 由画廊源与原项目 README 维护调用入口，AutoUI 规范描述跨宿主行为。下游目标是 AutoLang GOAL-002 的 1071 与 1072，两端证据见 §9。

## 6. 测试设计

| 项 | 独立证据 |
|---|---|
| V-01 来源与覆盖 | 原项目、Widget 页、具体组件与两端矩阵对账。逐例固定原入口、工具/SDK/Schema、期望值和资源。 |
| V-02 语言与宿主 | 受影响的 AutoLang 测试组、实际 AV/A2TS 消费用例、有效用法与相邻坏源、诊断 span 和错误恢复。回归测试保留对共享语义的判别力。 |
| V-03 组件 | 新补的组件及行为具名入表。两端测试属性/事件、键盘或指针、可访问性、关闭或卸载及所用项目正反例。 |
| V-04 AV 原项目 | 真实 Iced 窗口运行全部本组项目及相关 Widgets，核实际输入、状态、输出、生命周期与资源。按触面复用 `ace_node_desktop` 等入口并记录窗口准备。 |
| V-05 A2Vue 原项目 | 从干净目录经公开工具生成并构建 Vue/TS 和运行时，在真实浏览器中跑相同原源与独立行为断言，核真实 ACE 生成与构建派发、关闭和错误恢复。 |
| V-06 交付复跑 | 冻结 AutoUI、AutoLang、AutoOS 的提交与各端依赖，复跑对应矩阵，核发布产物、可执行命令和子进程退出。原适用行均有真实结果。 |

## 7. 验收标准

| 编号 | 完成条件 |
|---|---|
| AC-01 | 示例、Widgets、资源和各端原预期完整追溯，版本固定。 |
| AC-02 | 本组所需的 AutoLang 改动已交，对应组件缺口在实际应用中关闭。V-02/V-03 正反两面通过。 |
| AC-03 | V-04/V-05 中本组每条必验项均有各自的通过证据，生成、运行、错误恢复及资源终态可复查。 |
| AC-04 | 独立复审通过，按对应仓流程合入修复并发布 V-06 收据。向 W1 汇总提供 AV 格，向 1072 提供 A2Vue 格，归档和清理本计划工作树。 |

## 8. 执行步骤

- [x] **T-01｜work** 冻结原项目、Widgets 与两端覆盖矩阵，确认首批运行顺序及共享接口和组件的唯一写面。见 §5 矩阵及 §9 基线与首轮回放。
- [x] **T-02｜work** 补本组所需 AutoLang 源码、AV/A2TS 语义与宿主接口，交正反例及来源。
- [x] **T-03｜work** 完成缺失组件及行为、双端接线和实际消费者回归。
- [x] **T-04｜work** 从真实入口逐例验 AV/Iced、权限、错误恢复、关闭与资源。
- [x] **T-05｜work** 从真实入口逐例验 A2Vue 生成、构建、浏览器行为、关闭与错误恢复。
- [x] **T-06｜work** 冻结各仓提交，完成受影响回归及两端可复跑交付，补齐矩阵和规范增量。
- [ ] **T-07｜review** 同树独立终审 AC-01..04，核缺失行、错误路径、共享归属和收据，修复发现后复验。
- [ ] **T-08｜merge** 按 owner 顺序正式落地，记录 AutoUI/AutoLang 来源，通知证据消费方的交接在计划记录中登记。归档并按仓规清理。

## 9. 复审记录

2026-11-21 stage: new | PLAN-107 r2 | outcome: pass（恢复原拆分、保留在途证据） | next: work/T-02。T-01 仍计完成；T-02..T-08 及 AC-01..04 按 §1、§5 的当前范围执行。下表保留 r1 已冻结并移交的行。

| 原样输入 | 实际闭包及独立预期来源 | AV 与 A2Vue 必验状态及负控 |
|---|---|---|
| `031-paint` | `paint_color.at`、ColorPicker、NumericFields、Canvas；`SPEC.md`、`tests/desktop_mcp.py` | 真实指针落笔，像素 42 按选色变化，Undo/Redo 回放，取消选色保正式色，存储往返后卸载 |
| `019-video-app` | `pages/{home,watch}.at`、`video_store.at`、`settings.at`、`back/{api,db}.at`；`tests/{acceptance.atd,smoke.spec.ts,vm-smoke.mjs}` | 卡片进入 `/watch/:id`，真实解码与时间推进，切换相关视频，坏 URL 后恢复；仅向已选用例开放媒体请求 |
| `020-music-player` | `player_store.at`、`viewport.at`、`controls.at`、队列/导航与 `back/api.at`；`README.md`、`tests/test_020_vm.py` | 固定两首 1.5 秒 WAV，播放/暂停、换曲、到尾策略与设备释放；扫描根仅指向样本目录 |
| `029-photo-gallery` | `back/api.at`、ImageViewer、AssetBrowser、TreeView、Pagination、CollectionView；`SPEC.md` 的 scan/thumb/full 与 UI 行为 | 缩略图与原图身份、相册和查看器切换，坏图失效后可打开同目录好图；`AUTO_PHOTO_ROOT` 指向固定图片目录，撤根或未授权路径不能被读取 |
| `030-video-player` | `player_store.at`、`viewport.at`、`controls.at`、`playlist.at`、`file_picker.vue` 与 `back/api.at`；`tests/{smoke.spec.ts,vm-smoke.mjs}` | 真实帧与时间、暂停、seek 至 50%（时长 ±5%）、坏媒体后恢复及关闭；保留原用例媒体目录与真实 Chrome 编码支持条件 |
| `031-image-viewer` | `back/{api,directory_index,image_service,types}.at`、ImageViewer；`tests/{vm_matrix.py,vue-actions.json,fixtures/manifest.json}` | JPEG/PNG/WebP、Alpha/EXIF、缩放/平移和可见区，坏图具名失败后恢复，关闭会话不残留图像请求 |
| `carousel` | 外壳 `official` → `components/carousel.at` 五个同族组件；原页 `carousel-{basic,sizes,spacing,orientation,options}`，`component_registry_test.rs` | 指针与键盘改变滑页；横/竖、多个可见项、边界与循环可观察；关闭后无遗留事件 |
| `code-editor` | 原页 `code-editor-{basic,auto}`、本仓 `ui/code_editor` 和 Vue CodeMirror；`plan413_code_editor_natives.rs`、原页 Props 表 | 编辑/撤销、光标、搜索、换行、内容回写与关闭，native vi/IME 按原条件 |

2026-11-19 stage: new，drafting 0/8。U05 的原样示例、Widgets、AutoLang 支持和组件补齐均处于待实施状态。

2026-11-20 stage: work | plan_id: PLAN-107 | plan_revision: r1 | T-01 基线 | 授权链：`/goal 完成计划107`，实施、独立复审、限定修复、合入归档清理。主检出 AutoUI 干净，AutoLang 既有在制改动限文档与计划。独立工作树 `D:/autostack/.wt/ui-107/{auto-ui,auto-lang}`，两仓 `plan-107-dev`，基面分别 `514c27113e1aaebd683a72aaf5d6196099869214`、`c941db401fffe08abae1e1f198238adec46d4f5c`。只读 Cargo 兄弟树：AutoDown `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`，AutoWeb `26d085f7e54ea20c6997edf2dc23eef0cb57faf6`，AutoMan `4a30483a590d81d5724cb453a0fd1fbc98d9aaf2`。只读 AutoOS 源 `efcfced51b58a1174f0b4fd3c0288c589495f70e`，`widgets-gallery` Git 子树 `d78f0ef4a67c69cab402f2a2d3d2a815162fa7f2`。`cargo tree --locked --offline -p auto-ui-tools --features ace-vue --depth 1` 退出 0，实际依赖落到组内检出。

**输入与工具。** 材料根 `D:/autostack/plan-artifacts/plan-107/` 已登记，Git 文件树按固定提交导出到 `clean-source/`。`input-sha256.txt` 包含 306 项，SHA256 `48fc70b5e6e1ee04deee55e1ed3a37fac6a0e716046c3af10e5981db1f66dd7d`。020 使用独立合成的 16 kHz 单声道 PCM 440/660 Hz 音轨，SHA256 分别为 `6597c395421e3c01f4297e5c9c9c56c0afe769c300c4a075f9447557a76d0c09`、`019505cf956e1143925ecd4fe4ed42b06f2c6b9a9df842c01c7a4f1269e2fadf`。030 复制原测试指定的 `caelestia.mp4`，SHA256 `f619e2a111080267cd00cca7135947ad0dd4d978fe38381d46b75ba6d2b80153`。图片沿 031 原测试素材和损坏对照。019 固定原视频目录 API 及代码座标，源解码待两端准入后回放。Windows x64，Rust MSVC 1.98.0 `88d9e12ae`，ACE host 0.1.0，Node v25.2.1、pnpm 11.6.0、Chrome 157.0.8103.80。Auto CLI `0.1.0+plan-890-verified-r7-t19-116-gc879b3e`，SHA256 `eddb2cce50fd8afe08cd26959c2fe52e090eb833b5c8b9a3f27538b28d947fcf`，锁文件 SHA256 `2411b8a65fb03cdd33898b8d97a05e4a6e0b7d27ca941548fdaaac781137341be`。

**T-02/T-04/T-05 首个拒绝点（已执行）。** 基线用 `cargo build --locked --offline -j 4 -p auto-ui-tools -p auto-ui-runtime --features auto-ui-tools/ace-vue,auto-ui-runtime/ace-bridge --bin auto-ui-tool --example ace_node_desktop_window`。全程 dev/debug 无调试符号，显式开启增量。初次 sccache 拒绝增量，当前进程移除 wrapper 后 214.241 s、exit 0。AV 基线 exe SHA256 `8da4513cda1d72a2e10a91e0b5fc839a6ad634bfdea2b1f15e85183cb5a32feb`，Vue 工具 `2aec578c891e31da78dc0bac9c8949942cd95033d613bf1c35a5a36c941693b4`。用 `first-admission.py` 运行原应用 `--source` 与 `--package`、原画廊页最小包，以及 `AUTO_UI_ACE_VUE=1 AUTO_UI_TOOL_BIN=<冻结工具> auto build --render vue --gen-only`。`admission.json` SHA256 `945ba7a20c45cf7b6039e143b480f93e045213e301daa0052f008dd5a716c6c8`。每行均停在实际验收前。

| 输入 | AV 原源包入口，exit 4 | 严格 A2Vue 实际生成，exit 1 |
|---|---|---|
| 031-paint、029-photo-gallery、031-image-viewer | `source.lex` 拒 `$event` | `ace.vue-lane-unsupported`，各自业务 handler 缺席 |
| 024-charts | 包面 `source.syntax`，`expected fn, found widget`；组件面模型声明槽拒绝 | `.SetLine`、`.Tick` 等 handler 缺席 |
| 019-video-app | 包面 `source.syntax`，`expected fn, found widget`；组件面 `use` 拒绝 | 入口 `app.at` 同样在包面拒绝 |
| 020-music-player | 包面 `source.syntax`，`expected fn, found widget`；组件面 `use` 拒绝 | `.NextTrack/.PlayPause/.PrevTrack/.ToggleDarkMode` 缺席 |
| 030-video-player | `ace.desktop.view-cond-unsupported`，首个条件是 `.store.has_current`；当前按实例只过滤文本叶，清除门会暴露未走真实树的控件 | `.ToggleDarkMode/.TogglePlaylist` 缺席，`playlist.at` 另有解析跳过 |
| 图表四页、FlowDiagram、Carousel | 六个逐页包输入均为 `source.syntax`，`expected fn, found widget` | 画廊原外壳在包面拒绝；直连 `generate_package_sfc` 六页分别重现同一首诊断 |
| CodeEditor | `source.age.msg-unknown`，`QueryChanged` 无声明 | 直连包 SFC 复现该诊断。材料副本仅补消息后，两端均转为 `source.module-symbol`，缺 `code_editor_cursor_line`，去掉重复空光标处理器对该诊断无影响 |

**对照、范围及时间。** Provider 准备 0.769 s。AV 22 次，共 3.610 s，原应用和画廊包 21 次 exit 4；首个标量对照虽 exit 0 但 0 帧。按 `--hold-ms 8000 --script 3:click:0:2,7:close` 复跑，4 帧、1 次点击、会话关闭、零错误，JSON SHA256 `01b742db86f65e06eda1d8bf6ea238a2d4ff787e814307e8204fe915ae7f3ee9`。严格 Vue 生成 9 次共 30.394 s，002 exit 0，其余 8 次 exit 1。SFC 直连对照以同树已编库链接，首次缺 SDK import-lib 路径，补路径后构建 6.470 s；执行 SFC 10 次加 AV 探针 2 次，002 生成 SFC，通过对照以外 11 次 exit 4。`direct-package.json` SHA256 `08e7836c57606651c0d3c7f9fd4e2079da03e411fe548d571419bf8e9275c088`。应用和画廊的媒体、画布、浏览器行为以及资源终态仍待验收。发现/清理未单独计时。

**窗口验证切片。** AutoUI `81771ad12f8146c55ad514c672c1df41ea6b996a`。修复前在 1 s 截止、目标 1,000,000 帧下只观察到 5 帧却 exit 0。原样反例现在 exit 6 并写出部分证据；输出路径无效 exit 8。首次修复仍按成功派发计数，已有 U01 反例因此 exit 6；现按实际取自会话队列的事件计数，合法拒绝的 `abc` 也计入已处理输入，保留原诊断与四次成功派发。`pageclick` 指向无可用页面复跑 19/2 帧、0 派发、仍 exit 6，证明空轮询或帧充足不能替代事件。`window-fix-v2-execution.json` SHA256 `43af4cffce869a91f5d2c62f20f327405029c7476045c9ef73aac5b48cd88fc4`。未汇成 T-04 通过。

| 窗口测试（`ace_node_desktop` 实际单进程 `--exact`） | 实测墙钟 | 结果 |
|---|---:|---|
| 新增 `p107_window_deadline_fails_with_partial_evidence` | 1.16 s | 1/1，≤10 s，检查 exit 6、具名截止、Counter 部分 JSON 与帧短缺 |
| `p980_node_desktop_window_e2e_counter_full_script` | 1.68 s | 1/1，原计数与关闭重开脚本 |
| `p980_u01_window_input_e2e` | 1.19 s | 1/1，拒绝 `abc`、四次派发、两个实例状态 |

最终增量工具准备 32.828 s、测试编译 9.338 s；直接执行三个准备好的二进制过滤，每次 11 项未选，发现/清理未单独计时。额外窗口实验为脚本未到达 5.310 s（首次短排窗 0 帧，未用于该断言）及证据写盘失败 1.129 s。统一影响面回归留待相关生产接面齐备。

**A2Vue 干净构建切片。** AutoUI `b409acd977ac363c252398a07c9d9141711e4d15`。002 初版项目生成成功，但 `pnpm build` 在 `vue-tsc` 以 exit 2 拒缺 `a2x_rt.d.mts`、`reka-ui` 和 `class-variance-authority`。根件已经使用原生 `<button>`，临时旧 SFC 的预扫描却把 shadcn 按钮复制到了产物。现采用最终 SFC 语料收集组件，A2Vue 工具全量、增量根件及增量组件出盘共用 A2TS 的官方类型声明。开关键关闭的旧根件继续使用 shadcn 按钮，仍无 ACE 运行时文件。

| 测试范围 | 结果 | 准备 | 执行 |
|---|---|---:|---:|
| `plan980_ace_vue_tool_wiring`，`--exact p980_ace_vue_tool_wiring_key_gate_and_named_rejects`，dev/test 不在测试体编译依赖 | 1/1，补断言：旧件真实需要 shadcn、ACE 原生按钮无旧件、冷及增量声明逐字等于 A2TS 来源 | `auto-ui-tool` 19.247 s；测试编译 6.211 s | 2.847 s（≤10 s），发现/清理未另计 |
| 从新导出的 002 原样目录，`auto build --render vue --gen-only` → `pnpm install --offline` → `pnpm build` → Playwright Chrome | 4/4 exit 0，`vue-tsc`/Vite 通过，状态 `0 → 1 → 2 → 1 → 0`，0 页面异常且浏览器页关闭 | 生成 1.559 s；安装 10.389 s；构建 4.630 s | 浏览器 2.267 s，含关闭；单独的集成准备，无新增构建型日常用例 |

`vue-replay-status.json` SHA256 `3e15eeacd4a8ec36c2f692cd0773961d564bd0f99f9b69818049db5d4533c967`，浏览器输出 `vue-replay-browser.log` SHA256 `6265960ad5ec97fa0dfa4703a3a746719a8cb7b48d43212fbcbdffe0dbabf5be`。此对照验证工具链，U05 原项目仍保持首轮拒绝待实现状态。

**Schema/资源补核。** 同树 `ace_core` 运行探针所得 AgeComponent `3abe2d4a53c607fee4ba474de2ee1cf3d8fc6e79d7f8b04648a632b0d9a30be7`，AgePackage `42b6eedec484000eeff74717d8159ac514b9905eda18ea95dab9e87c63f672e5`。Chrome 直接解码素材，2.894 s、exit 0：`caelestia.mp4` 1920×1080、时长 49.429 s，实际时间推进，暂停后 seek 到 24.7145 s；两首 WAV 各 1.5 s，首曲到尾、换源恢复到 0；PNG 320×240；页面关闭。`media-preflight.log` SHA256 `978f3d2f410157985c87d96dcb8ea718dd1a631c9bd744f08e9bff328b2963c2`。素材预检独立于各端应用。Windows libmpv 本地候选 `D:/autostack/tools/mpv/libmpv-2.dll`，SHA256 `673e6397920ab64a9c5b3a618f7f16d38854efe72b58665f1f84e4e873b763a4`，原应用尚未装载。019 原源的三个 HTTP HEAD 各 8 s 超时，另一公开存储入口返回 403。下载/原 URL 播放待可达媒体与准入同时具备。

`stage: work | plan_id: PLAN-107 | plan_revision: r1 | outcome: blocked | worktree_path: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | branch: plan-107-dev | base_commit: ui 514c2711, lang c941db401 | code_commit: ui b409acd977ac363c252398a07c9d9141711e4d15, lang c941db401 | task_ids: T-01 pass, T-02..T-06 open | evidence: page and app admissions, window and Vue tool repairs, real controls above | blockers: B-107-01 and B-107-02 in §10 | next_responsible_role: shared producers and source decision, then work`

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui b409acd9, lang c941db401 | next: user`

**原生媒体换源切片（T-03 进行中）。** AutoUI `ac5f6a011ff70c256a84c8a6c8484bb81933db8d`，AutoLang `c941db401`。固定 §9 的 MP4 与 libmpv client API 2.5，实测 `MediaContract` 在旧坏源完成后、事件出队前切回原好源。修复前同一次 poll 先为好源上报 `MediaError`，随后又报告真实 `LoadedMetadata(49.429)`，用例 exit 101、0.128 s。`media-switch-red.log` SHA256 `5bbc52238427741a6089ef3a3874e6e807d8f722f643ba499e260c446e1b1fc1`。

原生适配器现在从 `loadfile` 成功返回的 node map 读取播放条目 ID，对 `END_FILE` 中的实际条目 ID 过滤迟到的失败与结束事件。对当前播放列表重定向加入解码器返回的子条目范围，保留有效子项错误。当前坏源读取 `mpv_event_end_file.error`，实测返回 `-13`（`loading failed`）；缺事件 ID 或命令返回能力的旧库沿原行为。见 [mpv 事件说明](https://mpv.io/manual/master/#list-of-events) 与固定 SDK `include/mpv/client.h`。

| 实际测试（`--exact --nocapture --test-threads=1`） | 数量和结果 | 墙钟执行 |
|---|---|---:|
| 新增 `mpv_contract::replacement_source_does_not_inherit_previous_decoder_error`，旧坏源出队后有效源仍有时长且无错误 | 1/1，0 跳过，≤10 s | 0.26 s |
| 新增 `mpv_contract::redirected_playlist_keeps_active_child_errors`，有效列表子项先载入，切至缺失子项必须获得实际错误 | 1/1，0 跳过，≤10 s | 0.12 s |
| `mpv_contract` 既有起播暂停、真实 seek、元数据只发一次、EOF、坏源 | 5/5，0 跳过，逐次 14 项未选 | 1.24 / 0.09 / 3.11 / 0.34 / 0.09 s |
| `mpv_engine` 渲染 context 建拆重建、软件帧实际写入缓冲 | 2/2，0 跳过，逐次 10 项未选 | 0.15 / 0.39 s |

测试准备 `cargo test --locked --offline -j 2 -p auto-ui-runtime --features ace-bridge,autodown,mpv-native --test mpv_contract --test mpv_engine --no-run` 初次 90.726 s，修复后增量 20.872 s。反例编译 13.480 s。`native-media-switch.json` SHA256 `a076a2978fa87eaeF34bc9e1d353663f3d02f0a921f07f6d4c6a0f9a1961cd3c`，最后一组进程总耗时约 6.076 s，发现与清理未单计。实际 seek 从 49.43 s 素材对 19.77 s 的请求落在 19.77 s。此次验证覆盖原生内核与契约，AV/Iced 的原应用窗口、A2Vue 原项目和整条资源生命周期仍待同源验收。

共享基线复核。PLAN-105 新提交 `9540fd1db` 已接通同文件内的 Store，`48498d60a` 补 `List<T>` 名义类型；该分支的 `age_package.rs` 仍具名拒绝跨单元 Store。019 的 `video_store.at`、020/030 的 `player_store.at` 各与入口处于不同模块，当前仍需要跨单元支持。PLAN-114/123 的有效发布门与 CodeEditor 原页的负责方决策仍待齐备。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui ac5f6a01, lang c941db401 | evidence: real libmpv counterexample closed and 9 focused cases passed | next: work`

**原样图表与 Carousel 源码接入切片（T-02 进行中）。** AutoLang `d414aec0eb8a9603c3308869416b896f9b518581`，AutoUI `ac5f6a01`。在 `c941db401` 用两个实际 AGE 源入口分别回放固定原页和组件，发现画廊 AreaChart 等图表组件把中文块注释报成非 ASCII 词法错误。Carousel 页拆取 20 个未改动的直属子块，仅 `carousel-options` 在两个入口拒绝；材料副本只改 `loop` 为普通标识符时整页准入。`carousel-original-children.json` SHA256 `bee24801e58669042451b59f454db0a0de35bebedf86a4454ad311b7250433c7`。PLAN-105/114 在制分支未修改这两个语法接面，各自已有共有路线保持原分工。后续汇合需要从最终语法重算两项身份。

AGE Component/Package 使用现有 block-comment 扫描器保留非嵌套 `/* … */` 注释与原 span，未闭合注释在两个实际入口具名为 `source.lex.block-comment` 并定位至文件末尾。`loop` 只新增为视图属性键，保留原值和普通属性分类。冻结身份更新为 AgeComponent `d320a674ac1359ce52c85af23082c90323c42bea5241c7296746e97043d2125e`、AgePackage `5b71fa58c4e1abddd4373a17a87f180637da106ce6275c5aa4df428d17cc93d4`，相邻 profile 的旧身份断言均通过。

| 测试与输入 | 实际结果 | 墙钟执行 |
|---|---|---:|
| 新增 `plan979_component_profile::p107_age_views_preserve_labels_across_block_comments` | 1/1，保留两个文本叶、引号内 `/*…*/` 与原字节位置，注释内的假文本和 `$event` 无语义 | 0.149 s |
| 新增 `plan979_component_profile::p107_age_views_reject_unterminated_block_comments` | 1/1，内层与尾随未闭合注释均按完整 span 拒绝 | 0.106 s |
| 新增 `plan979_component_profile::p107_carousel_loop_is_a_view_property` | 1/1，两个入口都投影 Carousel 原 `loop: "true"` 属性及值 | 0.067 s |
| `plan979_component_profile::p979_component`、`plan981_grammar_scope`、`plan981_age_package`、`source_frontend::text_stage_rejections` | 24/24、4/4、3/3、1/1，0 跳过。最后一组含相邻语言 profile 的旧块注释负例 | 0.113 / 0.392 / 0.088 / 0.054 s |
| 固定 024 与画廊四类图表作者组件，及 Carousel 页与组件，分别回放实际 AGE 前门 | 图表均转过注释词法拒绝，下一拒绝点在声明/类型语法。Carousel 原页两个入口均准入，组件的 `text \| Slide` 仍在包面被拒 | 0.170 s |
| 原样 Carousel 页，`ace_node_desktop_window --package` 与直连 `generate_package_sfc` | 单页 Iced 2 帧/exit 0，生成 SFC/exit 0。加入固定 `official::carousel` 后两端 exit 4，均为 `source.lex` 拒 `\|`。只在材料副本改三句为带引号文本，包面进入下一层语法失败 | Iced 3.048 / 0.189 s，SFC 0.305 / 0.144 s |

测试准备 `cargo test --locked --offline -j 4 -p ace-core --test conformance --no-run` 冷 32.307 s，最终增量 1.853 s。首次未闭合注释在组件面被结构诊断覆盖，已优先保留该词法证据。AutoUI 工具和真窗入口同树重建 29.743 s，源与 SFC 探针分别链接 0.407 s、4.745 s，发现与清理未单计。`age-view-final-runs.json` SHA256 `d7b209dff78022f7bce0becbef548732c18bc80108041cf09f827fc27b6a8bd6`，`source-diagnostics-final.log` SHA256 `1517f70b76592c0340fab2d44cdb9bbc497b9296ac54615dc908812a7d0c39c`，`age-view-window-runs.json` SHA256 `ccde39338952e054812f44f66ae490f83204c3691b691b4ed1a8d6892990d444`，`age-view-package-sfc-runs.json` SHA256 `b30e4b6dd6dce8da374081c4bb946d813cef30ea62335de88b7944a523990908`。Carousel 单页结果供定位，完整组件运行、交互、访问性和卸载待 §5 同源验收。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui ac5f6a01, lang d414aec0e | evidence: 35 focused cases pass and original Carousel page reaches both host endpoints in isolation | next: work`

**Carousel 行内文本（T-02 进行中）。** AutoLang `9f4a65ad3783d2b38537837b8105ac2e3c4eb0b7`。两个 AGE 前端通过正式词法与 CST 投影收录 `text | …`。语法只在视图文本叶启用，保留原字节位置；CRLF 和同行闭合花括号仍结束文本，`||` 保持逻辑运算符。新 matcher 经过语言包二进制往返。两项身份为 AgeComponent `0e7f902d5e21f68cded1a6626252793ebb55d95aac2b669379975cc7ca03c83a`、AgePackage `58d31db5dd4a4bdb239c84b869bfeb85b435ff6953ec88bdd1c99af18b92fad9`，相邻 profile 的冻结断言通过。

| 实际测试或原源回放 | 结果 | 墙钟执行 |
|---|---|---:|
| `p107_age_inline_text_preserves_labels_and_view_boundaries` | 1/1，两个前端逐一检查标签、span、同行条件块和两项非法位置 | 0.084 s |
| Component、grammar scope、AGE package、相邻 text 过滤 | 28/28（含上述新增项）、4/4、12/12、1/1 | 0.146 / 0.398 / 0.146 / 0.069 s |
| Runtime codec 与 lexer 过滤 | 1/1、6/6 | 0.012 / 0.014 s |
| 固定官方组件中的 `CarouselDemo` 原字节片段，附三个 `Slide` | 两条 AGE 前端均准入 | 0.061 s |
| 完整官方组件文件与原文档页经 Iced `--package` 及直连 `generate_package_sfc` | 两端均已越过 `|` 词法错误，仍在根件 `Carousel` 的声明结构以 exit 4 拒绝 | 0.252 / 0.139 s |

编译准备分别为 conformance 6.72 s、最终增量 3.82 s，runtime 测试最终增量 1.75 s，同树 AutoUI 工具及窗口 47.11 s，两探针链接 0.532 / 5.872 s。发现和清理未单独计时。测试材料 `inline-text-runs.json` SHA256 `f656dc3880b2d4a16813156bc0050a3c75db2825d1dd7b20459ac6c734b2f84a`，原样回放 `inline-text-original-replay.json` SHA256 `2f4da0f123dc316bac293df06da79b2a92eb57f4b337fd87c3069abb34619717`。逐件材料对照 `carousel-individual-closure.json` SHA256 `bcfc5cbc2fa7394131a078e5d20f8f14b4e5f22d902ab7b2e85e7a481d3014c1`，另定位多行消息声明、计算样式及引号属性键。两端整页行为仍待 §5 同源验收。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui ac5f6a01, lang 9f4a65ad3 | evidence: original CarouselDemo parses in both AGE profiles and full closure advances to declaration diagnostics | next: work`

**Carousel 引号属性与 A2Vue 无障碍接面（T-02/T-03 进行中）。** AutoLang `defb955107bc60ce7ff19da59f15e0f6c9f1862b`，AutoUI `27ae0d33a50a6b6d9ab3315d827110d8109e87c6`。两个 AGE 前端对引号里的普通属性名投影同一受检键，具名拒绝含空白、事件前缀或转义的键。A2Vue 包模板输出 `role` 和 `aria-*`，保留字面值、响应式表达式与 HTML 转义。新身份为 AgeComponent `32ecce68ef1a4345d8079fc8cdd4556a3c813452506e2a00d0fdd979bf3c7c38`、AgePackage `7398b1d210df247c779db16f1992e03e92baaca8987e6a0ab898ba89c9f6ea94`。

| 实际测试或材料 | 结果 | 墙钟执行 |
|---|---|---:|
| Component、grammar scope、AGE package 过滤 | 29/29、4/4、12/12，包含新增双前门属性正反例 | 0.15 / 0.37 / 0.14 s |
| A2Vue 包模板测试二进制 | 新增测试 1/1，锚定角色、描述、转义和绑定；非浏览器用例合计 10/10，两个既有浏览器测试留给其独立环境 | 0.195 / 0.149 s |
| 材料对照保留官方引号键，使用逗号消息与字面样式 | 原先 `source.syntax`；现在受检生成带 `role="region"` 和 `aria-roledescription="carousel"` 的 SFC | 0.077 / 0.067 s |
| Chrome 对照载入实际生成 SFC | 从无障碍树取得具名区域和按钮，空格触发 `aria-pressed` 从 `false` 到 `true`，卸载后区域消失，零页面异常并关闭 | 0.914 s |

AutoLang 准备 5.65 s，最后重链 1.83 s。AutoUI 测试准备 44.10 s、探针链接 4.24 s、生成浏览器对照 0.198 s、离线安装 9.49 s。复用 002 的 TypeScript 配置初建 1.25 s，`vue-tsc` 报包 SFC 的 JS 默认语言缺类型声明；材料项目通过 `allowJs` 消费原生成 SFC，保留 TypeScript 的严格检查，构建 2.04 s。此设置仅在材料项目。发现和清理未单独计时。材料 `quoted-attr-lang-runs.json` SHA256 `2b0c2fb7c8ff5b8f4e0ab17e310fd063e639553f2cda157ddd09fe8265673db7`，`quoted-attr-sfc-regression.log` SHA256 `c0833ccf961f82098ace0c14f2866c7eedfea32db941c52df9dbE69AA59B70D7`，`quoted-attr-browser-run.log` SHA256 `0a0bc953be09c0b5010db3d8c6bc0c0fe567784b06e1671518ef5ae4431ce40c`。官方整包与 Iced 无障碍及交互验收待完成。PLAN-114 已在分支 `466af4251` 接入受检组件属性，其中 `6559fb2b1` 覆盖包面格式化文本，本计划继续沿该 owner 的固定发布面消费共享计算值和消息解析。PLAN-105 的 `31013d31` 记录 013 实际挂载及 016 的模块 helper/循环作用域缺口，U05 的跨单元 Store 门仍在。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 27ae0d33, lang defb95510 | evidence: projected quoted ARIA and observed generated SFC keyboard binding and unmount in Chrome | next: work`

**Carousel 的 A2Vue 实际组件（T-03/T-05 进行中）。** AutoUI `6e8f8d89641e25fc3c3b5da8c8391eb8a81d0973`，AutoLang `defb95510`。基线以同一原页生成标签 `<carousel>`，没有组件实现。受检包模板现在复用仓内 Embla 的 Carousel、Content、Item、Previous、Next，保留 orientation、align、loop、样式与 `span/td` 的字面文本。Vue 脚手架从最终 SFC 收集 Carousel 及其 Button 依赖，初次离线安装包含 Embla、VueUse、Reka 和 CVA。验证使用独立单路由应用，复制原画廊页逐字不改，原页 SHA256 `f43f17c29a83924abc3061494e873f54140982ba828367a4a33f820cae891bf1`。

| 实际命令或操作 | 结果 | 墙钟执行 |
|---|---|---:|
| 严格入口 `AUTO_UI_ACE_VUE=1 auto build --render vue --gen-only` | exit 0，生成主件及实际路由页，物化 10 个组件文件 | 1.784 s |
| `pnpm install --offline` → `pnpm build` | 安装 exit 0。原生成配置下 `vue-tsc` exit 2，缺 JS SFC 模块类型。对照配置复用 PLAN-123 `ad522af8` 的 `allowJs` 后 `vue-tsc`/Vite 均 exit 0 | 6.886 / 1.920 / 4.728 s |
| Chrome 真实指针、键盘、拖拽和卸载 | 五组可访问区域挂载，Basic 从首项前进、回退，末项禁用下一页；拖拽后滑轨位移超过宽度 75%。Sizes 比例约 1/3，Spacing 为 1/2。Orientation 的 `ArrowDown` 仅移动纵轴，Options 首项可回绕。零页面异常，卸载成功，页面关闭 | 3.248 s |
| `plan980_package_sfc` 及 `vue::tests::package_json_component_groups_conditional` | 11/11、1/1，覆盖 Carousel 属性、标签、内容和 Button 传递依赖，另两个既有浏览器用例未选 | 0.159 / 0.250 s |

工具最终增量准备 13.88 s，两个测试编译 16.32 / 10.58 s，发现/清理未单独计时。首次 `align` 将引号做 HTML 实体导致 `vue-tsc` 语法检查失败，已改用经过校验的 Embla 对齐选项值。后续 build 对照只加 PLAN-123 已有的 `allowJs` 配置。生成、原配置构建、配置对照构建、浏览器日志 SHA256 依次为 `46a23bb16087452c39817b4fca627cc6930cdb0589e9f2de4c52503b8e95c755`、`45be3aa5deb3bc341b3688e78402205559548c49c8b987046c3958b593fc09c8`、`347baf9db0423a697f6479595785f5c8392e651788d1745d401fe38eac1be973`、`496077fa9740e1db075b7e3ea9ce9cf78b79ceee1787a03f48d2fb9d1174a0d2`（`carousel-route-final-{generate,unadapted,build}.log`、`carousel-route-browser-final.log`）。整画廊与官方源闭包、Iced Carousel 仍待 §5 验收。CodeEditor 原调用可复用现行 Iced 编辑器注册表与 Vue `editorBridge`，AGE 包面需要受检绑定和真实挂载；原页消息与归属门继续按 B-107-02 追踪。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 6e8f8d89, lang defb95510 | evidence: checked original Carousel page in a route replay with five Embla previews and real browser interactions | next: work`

**Carousel 的 Iced 实际组件（T-03/T-04 进行中）。** AutoUI `f5ea801f0fab4d9d0d2a4786a3a85627accad714`，AutoLang `defb95510`。沿现有 AGE 桌面控件树和窗口消息实现独立滑页状态、横竖布局、响应式 `basis` 可见数量、方向键、边界、循环、鼠标拖拽和焦点。AccessKit 区域名称、可见滑页顺序与按钮可用性从同一状态导出。页面仍为固定 AutoOS 原页，原文 SHA256 与 round 6 相同。

| 执行 | 结果 | 墙钟执行 |
|---|---|---:|
| `auto_ui_runtime` 的 `ui::ace_node_desktop::widgets` | 5/5，新增覆盖循环可见顺序、无障碍焦点和点击、纵轴、拖拽消息及 700/900 px 断点 | 0.119 s |
| 生产运行时示例，`ace_node_desktop_window --package` 原页及会话关闭脚本 | exit 0，12 帧，零错误会话关闭，JSON SHA256 `3333d0f0c607e9e2b3a25c6a9497e6d6c389ec02834daf6f86c421fb2b25ebf7` | 1.946 s |
| 原页真窗加材料驱动，调用挂载控件的实际 keyboard、MouseArea 和 AccessKit 消息入口 | exit 0，8 帧，五组可见项分别为 1/3/2/2/3，Basic 键盘到第 2 项、拖拽回调到第 3 项并禁用下一页；竖向到第 2/3 项，循环回绕至第 5/1/2 项。关闭后无可访问树，捕获的晚到 Carousel 回调拒绝 1 次，零错误 | 2.141 s |

测试增量准备 19.25 s。普通 debug 示例的 Windows 主线程默认栈在原页实际渲染时报 `STATUS_STACK_OVERFLOW`，材料命令用 `cargo rustc --locked --offline -p auto-ui-runtime --features ace-bridge --example ace_node_desktop_window -- -C link-arg=/STACK:67108864`，沿 `auto-ui-tool` 已有 64 MiB 主栈口径，最终准备 11.62 s。材料驱动链接 4.42 s，发现/清理未单计。回放 JSON `iced-carousel-replay-final.json` SHA256 `63dc2bd6620f585381f28a97fa759ad0334508686a4dd661de26662a156a55ca`，日志 `iced-carousel-replay-final.log` SHA256 `722e469e43502ab9f0079cc1ec0cc4d67763c56d875e4c694fbb0b79410de310`。Computer Use 找到测试窗口，激活失败，重新观察确认 Windows 锁屏。现场物理指针及键盘输入待解锁复测，官方画廊组件闭包继续沿共享基面恢复。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui f5ea801f, lang defb95510 | evidence: original Carousel page mounts and dispatches five preview controls through the live Iced widget and accessibility message paths | next: work`

**图表与 FlowDiagram 的 SVG 渲染接面（T-03 进行中）。** AutoUI `ea71a751c7eecd2ee82fe2cc3ca32724d866aa3f`，AutoLang `defb95510`。基线包 SFC 输出空的 `<path></path><line></line>`。现从已受检视图保留 SVG 命名空间、几何、样式、标签和绑定。Iced 走现有 `svgdoc:` 渲染通路，A2Vue 输出原生 SVG 元素。另修正 A2Vue 包初值发射，常量 fragment 没有语句时沿已验证的标量表达式取值，闭合原 `gridPath` 生成 `ref()` 导致浏览器路径长度为 0 的反例。

| 执行 | 结果 | 墙钟执行 |
|---|---|---:|
| `ui::ace_node_desktop::widgets` 过滤 | 6/6，新增用真实受检包 AV handler 将线段终点 4→18，经 Iced 的 SVG 文档与 resvg 栅格化，像素 (12, 4) 从透明变为 RGBA (255, 0, 0, 255) | 0.181 s |
| 包模板非浏览器用例与已交 VM SVG 回归 | 12/12、1/1。含动态属性、受检字符串初值、标签转义和不支持属性的具名拒绝；两个既有浏览器用例未选 | 0.172 / 0.124 s |
| `AUTO_UI_ACE_VUE=1 auto build --render vue --gen-only` → `pnpm build` → Chrome | 材料路由复用原 LineChart `gridPath` 和 PLAN-123 `allowJs`，全链 exit 0。SVG 命名空间正确、四段原网格总长 2040，点击后线长 50→510，标签完整，卸载且零页面异常 | 1.871 / 2.122 / 0.994 s |

测试最终增量准备 14.57 s，工具最终增量准备 10.88 s，首次离线安装 9.90 s，发现/清理未单计。浏览器失败日志 `svg-geometry-browser.log` SHA256 `63199c9e8f74bf30158ebd0cc25d48746555f8eb32c3c47e6e7f1136e8cd87d4`，最终日志 `svg-geometry-browser-final.log` SHA256 `e3d5b26a627b4964b9b4693b22372b9db54b4dc0b98db9338b821cd20f972104`，最终定向测试日志 `svg-geometry-test-{native,sfc}-final.log` SHA256 `54a45537d77c46e883cbd67ec80ea26864a934895d80250e954408828e04c113`、`382fe900170242a6d0b88ce0a122a4f6f60aec3bcb339f340942aa8907ce7905`。实测覆盖几何消费和数据更新。原组件的列表循环、条件图元、图例及指针仍待共享包闭包与事件接线，桌面遇到未求值的 SVG 控制流具名显示诊断。PLAN-114 已推进至 `e65a1d5a9` 的受检遍历与文本谓词，固定发布面依赖仍沿 B-107-01 跟进。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui ea71a751, lang defb95510 | evidence: AV state drives SVG pixels and production-generated Vue renders and updates the canonical grid fragment in Chrome | next: work`

**SVG 条件图元与 AV 事件（T-02/T-03 进行中）。** AutoLang `a0fb51825de1a4ab0edf32ad6214158cea2a283f`、AutoUI `86aec6c21ab81d986c7c88106f8a95b56284170a`。AV 的平铺观察入口仍具名拒绝条件视图。新增布局输入入口提供受检状态和含两支的文本槽，Iced 按实例的已提交状态选择 SVG 分支，并跳过隐藏分支的文本槽。实例文本分组沿 PLAN-114 `6559fb2b1` 的源序分块方式，解决两实例多叶交叉取值。源按钮沿原 AV 会话队列派发。A2Vue 模板支持短路逻辑表达式。材料页复用 FlowDiagram 的 `.hoverDg == 999` 普通态/高亮态结构。

| 执行 | 结果 | 墙钟执行 |
|---|---|---:|
| 普通 AV 条件观察及包观察用例 | 5/5、9/9。两实例各得到正确文本槽，原观察口保持具名拒绝 | 0.087 / 0.088 s |
| Iced 控件、A2Vue 包模板及四条桌面桥回归 | 6/6、12/12、4/4。AV handler 切换后复验栅格像素、分支标签和后续文本；桌面桥覆盖两实例、重开、类型输入与 provider 撤权 | 0.218 / 0.182 / 0.127–0.367 s |
| Iced 真窗材料驱动，调用实际挂载按钮的消息 | exit 0，9 帧、零错误。两实例依次为普通/普通、高亮/普通、普通/普通、普通/高亮。每个 SVG 只有一个 rect，SVG 和预览尾标签对齐。关闭后访问树消失，晚到按钮和脚本事件均丢弃 | 1.863 s |
| 非 SVG 顶层条件对照 | exit 4，锚定 `ace.desktop.view-cond-unsupported` | 0.127 s |
| A2Vue 真实生成、`pnpm build`、Chrome | 独立材料路由复用 PLAN-123 `allowJs`。普通→高亮→普通三帧分别只有一个实际 SVG rect，描边变化正确，保留标签并成功卸载，零页面异常 | 1.101 / 2.606 / 1.007 s |

AutoLang 冷准备 102.36 s、最终增量 4.04 s，AutoUI 测试最终增量 17.60 s，工具准备 52.44 s，Iced 材料驱动最终链接 4.29 s，Vue 首次离线安装 10.12 s。撤权测试需要其自身 provider 注册模式，移除回放用的 `ACE_NODE_DESKTOP_PREPARED_RUSTC` 后原测试通过。发现/清理未单计。`svg-conditions-window-final.json` SHA256 `968dc9fb89c009579f2d66d59e79d9ff1b4091ac4d787d752cb47b9c25c4c5a2`，回放日志 SHA256 `25b07257c5c9d6cf4fc87e567db7d53bb5bea8808aed702b8af23625d4b690f3`，`svg-conditions-browser-final.log` SHA256 `34127100cb344e5cc669b72b32882419e47cdabb0a155da3d9e033af1f4b726e`。原页完整列表闭包、列表循环以及原指针载荷仍沿 B-107-01 跟进，材料回放使用按钮事件触发状态。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 9 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 86aec6c2, lang a0fb51825 | evidence: installed Iced buttons switch checked SVG per instance, with matching browser geometry and an unchanged flat-observation guard | next: work`

**Iced 指针载荷和包派发（T-02/T-03 进行中）。** AutoLang `bd449b3919d6b2929cb1459a3bea5b5cf6fd5390`、AutoUI `b4e52146df3e3c1506af6bd65087dc49941719bd`。`float/f32` 消息沿同一受检事件槽验证，已有 int、bool、str 保持原行为。包桥将宿主值按声明位置抬升为 AGE 场景实参，按入口组件与实例定向，只在检查和 AV 观察成功后追加重放记录。Iced 的 `mouse-area` 使用已有指针坐标换算和限频入口，按视图声明绑定 `PointerMove` 的两项逻辑坐标及离开事件，消息保留会话与实例身份。

| 执行 | 结果 | 墙钟执行 |
|---|---|---:|
| AGE Core 和 AV 的 `plan981_event_b` 过滤 | 9/9、8/8。域外载荷对照仍具名拒绝 | 0.151 / 0.296 s |
| 包桥、桌面控件、Iced 指针投影与撤权用例 | 3/3、6/6、6/6、1/1。含错型、少参数、非有限值无状态效果，两个入口实例交换小数坐标后取得独立结果 | 0.752 / 0.209 / 0.074 / 0.318 s |
| 材料页 Iced 真窗回放，触发挂载的实际 `on_move` 与 `on_exit` 回调 | exit 0，10 帧，零错误。两实例绑定同一声明幅面 120×44，三次小数移动和一次离开后终回普通态，两个晚到事件丢弃 | 2.068 s |

AutoLang 准备 41.49 s，AutoUI 测试最终准备 45.70 s，真窗驱动链接 4.34 s。发现/清理未单计。`svg-pointer-window.json` SHA256 `daceb8d0e59ffffd9e9a8eb8f6e3de46942e0bfe30fd3e2033775ed9f4d89ecc`，`svg-pointer-window.log` SHA256 `3e80654b0432d654f502bebc5c7cdf1930ca49f49b5ae89009abf97b76134344`。材料页沿原图表的 `onmousemove: .PointerMove`、双 float 参数与离开恢复形式，运行时仍走现有场景重放，每次高频事件的长期成本待整体验收。原页物理输入、捕获列表元素参数、A2Vue 带载荷事件与完整依赖闭包继续跟进。PLAN-114 共享分支已推进至 `872441620` 的受检子组件输入绑定。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 10 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui b4e52146, lang bd449b391 | evidence: mounted Iced pointer callbacks send ordered F32 payloads to the correct checked package instance and drop stale callbacks | next: work`

**A2Vue 指针载荷和离开复位（T-02/T-03 进行中）。** AutoLang `20573cdf047d1e7f5e102c0719fa4e95a7cefe11`、AutoUI `c5a159923fbf741b6c361e0b3d7fdc84e52d5fa4`。AGE 包描述登记受检载荷处理器的绑定符号。A2Vue 复用检查函数的完整受检语句和两个位置参数，把当前实例的响应式标量字段传入处理器，执行完成后提交状态。`mouse-area` 将指针坐标按声明幅面换算为 f32，以 33 ms 和半像素采样门发送。Chrome 的悬停子图元被换掉后实测未向容器发送原生 `mouseleave`，容器使用文档级鼠标移动核对离开，组件卸载时清理该监听。

| 执行 | 结果 | 墙钟执行 |
|---|---|---:|
| `plan980_package_sfc` 定向回归 | 15/15。含两参数与多字段 `let`/`if`、错误幅面和错误载荷来源的诊断 | 0.150 s |
| `auto build --render vue --gen-only` 加前端构建 | 严格 ACE 生成通过。生成目录中对照 PLAN-123 启用 `allowJs` 后，`vue-tsc` 和 Vite 构建通过 | 2.647 / 2.085 s |
| Chrome 157 中移动真实鼠标输入 | 600×120 CSS 幅面换算到 120×44。测点的 CSS `x > y` 而逻辑 `x < y` 时回普通态。高亮、离开复位、同点重进和卸载通过。无页面异常 | 1.143 s |

AutoUI 测试准备 13.42 s，工具准备 17.97 s，首次离线安装 9.7 s，发现与清理未单计。浏览器材料和第 10 轮 Iced 材料页逐字节相同，SHA256 `9ec95820222124938e907c41e999cdb74e4c850bf3c4854f86eb7df1f6cbcd32`。`svg-pointer-vue-final-browser.log`、首次离开失败对照 `svg-pointer-vue-browser-debug.log`、`svg-pointer-vue-final-regression.log` 的 SHA256 分别为 `6198e88ecd21383f29ce9c65d713aec651822f04a9c700a87c15a696a424b004`、`fa83fc33f7ef0ad3463f906a6f506f48f65b2336b0588bbd399a83b90de7664e`、`c72c52d60e1367bfb9fa1d728d6ac9313c57c6ae018b3062fcf11413158bea35`。此切片支持复制型标量及浏览器坐标绑定，处理器需要额外脚本 helper、拥有型字段或其它载荷来源时给出具名诊断。PLAN-114 的 `e784c0e1f` 已有受检组件视图组装，生产宿主接线继续追踪 B-107-01。后续原图表需要受检表达式载荷、运行时 helper 和集合视图接入后的双端重放。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 11 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui c5a15992, lang 20573cdf0 | evidence: Chrome mouse input updates checked typed model state, resets after SVG child replacement and stops on unmount | next: work`

**正式 A2TS 运行时与原图表闭包对账（T-02/T-03 进行中）。** AutoUI `a9c4dec1a0c6b83e7ece5bd4ecf38a586b3a517c`，AutoLang 维持 `20573cdf047d1e7f5e102c0719fa4e95a7cefe11`。包 SFC 从受检语句收集 A2TS 正式运行时导出，存在引用时由原 `auto-ui-tool` 同步写出 `a2x_rt.mjs` 和 `a2x_rt.d.mts`。同名整数 helper 仅保留一份定义。增量生成能在 SFC 未变化时恢复缺失的配套运行时。

| 执行 | 结果 |
|---|---|
| `plan980_package_sfc`、`plan124_routes` | 16/16、2/2，分别用时 0.215 / 0.657 s，后者使用已准备的 ui980 provider。 |
| 新目录 `auto build --render vue --gen-only`，随后移出生成的运行时及声明再执行 | 首次发布两文件，后续命中 SFC 缓存并恢复两文件。运行时内容 SHA256 `bd9adfde09fe0740391f4a20f1b8fba0d752605eb5212d6901c4cf07b739baef`。 |
| Vue 构建和 Chrome 157 鼠标输入 | 材料加入 `x + 150.0`、`y + 160.0`，同一测点的原逻辑坐标比较与带偏移比较相反，真实 SVG 颜色与状态按受检算术改变。八个观测状态、离开及卸载通过。构建仍在生成目录对照启用 `allowJs`。构建 2.592 s，浏览器 1.476 s。 |
| 固定原页及组件重跑 | 107 当前编译器在四张原图表页和 FlowDiagram 页都报 `expected fn, found widget`。现有 PLAN-114 构建能解析四张原图表页，接着在组件 `style` 内的块注释处报 `source.unsupported`。仅以等字节空白替换该注释的对照仍报组件 `expected fn, found widget`。原 FlowDiagram 和 024 入口在两构建中仍报该错误；024 的 `chart_geom.at` 函数签名另报不接受 `float`。固定原输入保持原样。 |
| `git merge-tree --write-tree --name-only --messages HEAD e784c0e1f` | exit 1，七个内容冲突，涉及文法、前门、包绑定、lowering 和 AV 观察。仅校验合入，计划分支没有合并在途代码。 |

Rust 测试增量准备 4.73 s，工具准备 20.01 s，前端首次离线安装 10.470 s。`vue-runtime-helper-browser.log`、`original-age-114-donut-chart.log`、`original-age-114-donut-chart-comment-control.log`、`plan114-lang-composition.log` 的 SHA256 依次为 `073bfaec4553377a5b7916e52adeb67056899e78b915ab7a8f148f10eee717e6`、`73aa8328331d0301ae788d3affedc017416d979218ad0e5ee80e19f794d7c2cf`、`44f03ceef4c68f5dd9dc78dc4dbd3848b76c0a84faead9f88217ffc8f9912ac0`、`50fd89192e6c5c6660e96efb9b51bc2d2a99f3f64a499ce2c73fca869bdf71bf`。第一轮路由测试缺少 provider 环境变量，使用已准备的 provider 后两例通过。原应用和画廊的 AV/A2Vue 证据仍按 §5 收口。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 12 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui a9c4dec1, lang 20573cdf0 | evidence: official Vue ABI restored after cached generation, verified f32 pointer arithmetic in Chrome, and identified remaining admission and shared integration gaps | next: user`

**024 原图表函数签名与尾值（T-02 进行中）。** AutoLang `efe49ea3858e8fa73fad5141dbd45b012aa9847d`，AutoUI `a9c4dec1a0c6b83e7ece5bd4ecf38a586b3a517c`。AGE 包面将 `float` 和 `f32` 归到同一内建类型，用在函数参数、返回值和局部注解。声明了返回值的函数把最外层末尾表达式交给既有返回值检查。错型尾值、混宽算术和中间位置的表达式语句继续具名拒绝。AgePackage 文法 ID 更新为 `b17f6885b7a7ed5b3b3cd8ed9370589a97add95520b74e13c0da5a71bc143910`，冻结邻接面全部通过。

| 执行 | 结果 | 墙钟执行 |
|---|---|---:|
| ACE Core、AV 的 `plan981` 定向回归 | 248/248、57/57。AV 的两个实例分别用 11 和 9.5 调用同一跨模块 helper，观察值分出 `wide` 和 `narrow` | 0.647 / 0.371 s |
| 同一几何材料函数通过 A2TS 受检片段与正式脚本运行时，在 Node 25.2.1 执行 JS、TS | 两个目标都读取数值 5.5、4.75 及文本 `wide`、`narrow` | 未单计 |
| 固定原 `024-charts/src/front/components/chart_geom.at` | AGE 语法投影通过。包绑定首错是字节 390..394 的 `math` 缺少注册，`math.cos` / `math.sin` 的受检数值接口仍缺。原文件 SHA256 `78a02b598d169d0caa278a32aa913be1800ab76a02084a9c389f3ea64aa58863` | 未单计 |

最终 Rust 增量准备 9.92 s。`chart-helper-exact-original.log`、`chart-helper-core-final.log`、`chart-helper-av-final.log`、`chart-helper-script-js-final.log`、`chart-helper-script-ts-final.log` 的 SHA256 依次为 `d976505324c628828021cf766fee9e4b00b111dfc9d75e9f82a5554ed6f16e39`、`f4dabf6e188ae3cbcb7ccf9759e82a374c610c349971f0a796d4d20ed4f1c870`、`a8a032f5dd70ad7bc6a6e8ae2f3f896ab444b69e95bd17d694716231542e1baf`、`e32a33dd7ec4dc8df3802fd1a4c2338d93230d7ca4d0d7f2db793d715c228a31`、`b5d180d1d2b541f45e45be4631c1e989c413c54d13d0d31b25c12afab9f76e94`。下一切片核对原图表的 `math.cos`、`math.sin`、`math.atan2` 与已交数值库的签名及精度约定，再走原包绑定和两个正式宿主。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 13 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui a9c4dec1, lang efe49ea38 | evidence: unchanged chart geometry module parses and reaches the missing checked math binding, with AV and A2TS tail result evidence | next: work`

**024 原几何模块运行和 Vue 传递函数闭包（T-02/T-03 进行中）。** AutoLang `e9d77f0a274c9791c6878d680d97b64e0b701b79`，AutoUI `8e236a3f123e5acec28f19a5146ad62f9227dd14`。AGE 包将图表的隐式 `math` 绑定到固定的纯 Auto `std::age_math` 源，显式导入、同包模块和局部名字保留其作用域。新模块提供 f32 的 `sin`、`cos`、`atan2`、`sqrt`，复用装配库的级数和二进制归一化方法，按图表角度和有限坐标作精度验证。A2TS 的 f32 除法保留 f32 舍入。Vue 消费 PLAN-114 `af933b9ca`、`f7957fb6` 的完整函数发射切片，以受检符号追传递函数并去重，运行时随正式生成工具发布。

| 执行 | 结果 |
|---|---|
| 固定原 `024-charts` 的 `chart_geom.at` | SHA256 `78a02b598d169d0caa278a32aa913be1800ab76a02084a9c389f3ea64aa58863`。原模块受检包绑定通过。用原 `dc`、`ds` 和同一模块解析链下的 `atan2`、`sqrt` 跑 39 个测点，AV 与 A2TS 数值位型全部相同，与独立数值基准的最大绝对误差 `1.75e-7`。 |
| ACE Core、AV、A2TS 片段和 AutoUI 回归 | `plan981` 248/248 与 58/58，`ts_ui_fragment` 4/4，`plan980_package_sfc` 17/17，`plan124_routes` 用已准备 provider 为 2/2。含同包 `math` 遮蔽、递归 helper 去重和运行时导入。 |
| 严格 ACE CLI 生成、Vue 构建和 Chrome 真实鼠标 | 材料页引用逐字节相同的 `chart_geom.at`，通过极角和半径搭配 `dc`/`ds` 计算高亮阈值。八个状态、离开复位及卸载通过，无页面错误。生成目录对照启用 `allowJs`。传统目录扫描对无 widget 的 helper 报警，实际 SFC 引用和产物执行均通过。 |
| 重跑完整 `024-charts` 原包 | 原几何模块通过，整包仍首报入口 `app.at` 字节 1217..1223 的 `expected fn, found widget`。四个原图表组件仍有同族解析拒绝，整包运行继续沿 B-107-01。 |

AutoLang 增量准备 9.85 s，AutoUI 首次增量准备 31.10 s，严格工具准备 30.05 s。严格生成、前端构建、Chrome 真跑分别为 1.422 / 3.657 / 1.664 s。`chart-math-exact-av.log`、`chart-math-exact-a2ts.log`、`chart-math-vue-browser.log`、`chart-math-full-original-admission.log` 的 SHA256 分别为 `2be0365aab666eabe93e8473f215de2b84385b90fb1edd070bc4c8924d6c8931`、`41ff187d78f1fab159edc85d18f2a43b8a0df9b54ecab3987d8e9d6811ed616c`、`aae94f40dec6ac065d32ad494bc4d355715ccb5d8a00767ec1328eab2674ee7a`、`d2328943389ccaef4ba3fbebad4901a3773db61f08f781f49e151e9a6e66f5ae`。PLAN-114 已推进到 AutoLang `4d1883bb1` 和 AutoUI `b2d9e05a` 的受检展开视图及桌面绑定，仍需在共同基线复验原 024 入口和组件，按 §5 重跑完整项目。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 14 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 8e236a3f, lang e9d77f0a2 | evidence: unchanged chart_geom runs in verified AV and A2TS with matching numeric bits, and its Vue pointer consumer runs in Chrome | next: work`

**024 入口的组件包声明和标签绑定（T-02/T-03 进行中）。** AutoLang `3e4cfa6e275749ca4a2b23706e0febc141d1596e`，AutoUI `596482e1a0c5ce1673b9b42cc62374b71b482bda`。消费 PLAN-114 `f0284bc09` 至 `b84a0a39c` 的包声明、列表投影和受检组件引用。`use { package: official from "components" }` 保留目录、包名和作者位置。`line-chart` 通过既有组件注册规则的折叠形在声明包作用域内绑定到 `official::line_chart` 的 `LineChart`。缺包与别名冲突具名拒绝。当前 AV 布局观察与 A2Vue SFC 在遇到已绑定的子组件时具名拒绝挂载，待完整的受检子树执行接入。

| 执行 | 结果 |
|---|---|
| ACE Core、AV 与 AutoUI 定向回归 | `plan981` Core 248/248，AV 58/58，`plan1072_component_style` 20/20，`plan980_package_sfc` 18/18，`plan124_routes` 用已准备 provider 2/2。覆盖别名、包作用域、缺包和冲突。 |
| 原 `024-charts` 的入口块单独投影 | 实际包声明、`model`、`design`、完整 `view` 均通过。原 `fields` 和 `labels` 的字符串列表保留源码顺序。 |
| 保留入口其余内容的定向对照 | 从材料副本只移除 `.Tick` 和 `.Reset` 后通过投影。将 `.Tick` 单独换成空体并保留其余入口时，以 `source.age.msg-unknown` 定位到 `Tick`。原 `.Tick` 和 `.Reset` 各自含尚未受检的可变局部、列表遍历与写回。 |
| 原 `app.at` 与五个组件模块全量重跑 | 原入口 SHA256 `b89e1aad4741b1a75bd1eb0cbce19d582d798ed0ae8569c0b45f91466867b020`。无改动入口仍报 `expected fn, found widget`，四张组件本体也仍有解析拒绝，`chart_geom.at` 继续通过。 |

最终语言准备 25.01 s，AutoUI 测试准备 60 s。新 AgePackage 和 AgeComponent 文法 ID 分别为 `7ab01bf6f3795fe688fd4ceae3f20f624f52fdaf73e84695b5f4ce1941dfdc9a` 与 `620bfa50490a1bd81197c09bdbbdac18f4106a68c3854ce85a3ea44d8323236f`，相邻冻结身份通过。`chart-entry-block-final.log`、`chart-entry-full-original-final.log`、`chart-entry-block-final/without-Tick-and-Reset.log`、`chart-entry-block-final/unit-Tick-without-Reset.log` 的 SHA256 依次为 `59f436d8beca1c136677ef53dcb74e8d898b50554cd1089daed5c0f919985143`、`d2328943389ccaef4ba3fbebad4901a3773db61f08f781f49e151e9a6e66f5ae`、`72691546177703d05df62a4645cf8d6fd33f8592b49b9d09616dec22aa3de499`、`a37ca4b2d6e992a658eb9997e731c99ceed6a3be4c8b4705e0657db9d8ff90b5`。现有自动计时入口仍需对接原 `interval` 和 `.Tick`，之后结合受检的列表更新和组件刷新验证 400 ms 流式行为。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 15 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 596482e1, lang 3e4cfa6e2 | evidence: original chart view and package declarations project, checked kebab chart aliases bind, and exact source controls isolate the remaining Tick and Reset blockers | next: work`

**024 的处理器可变局部与受检分支（T-02/T-03 进行中）。** AutoLang `0c10b6a5f276766b45382148a470e4895fe4a572`，AutoUI `707b9e5d318e35ac68fb97b11025a3d54e7854c3`。AGE 包处理器支持 `var` 和裸名局部写入，按声明序和分支作用域绑定；显式 `.field` 与未被局部遮蔽的旧式裸名模型写入保持原语义。`let` 和消息参数保持不可变。Vue 对含局部或分支的无载荷处理器发射完整受检函数，并继续复用 A2TS 运行时运算。当前结构化发射覆盖标量模型；遇到所有权状态仍具名拒绝。

| 执行 | 结果 |
|---|---|
| ACE Core、AV 与 AutoUI 回归 | `plan981` Core 248/248，AV 59/59，`plan1072_component_style` 20/20，`plan980_package_sfc` 18/18，`plan124_routes` 用已准备 provider 2/2。原 Tick 算式、两层同名局部、模型与局部遮蔽，以及 `let` 写入拒绝均有检查。 |
| AV/Iced 正式桌面窗口 | 材料页保留同一 Tick/Reset 体，用固定文本叶启用包状态观察。两实例的派发记录为实例 0 的 `187 → 224`、实例 1 的 `187`、实例 0 复位后 `0 → 187`。渲染 8 帧，关窗后丢弃 1 次迟到事件，错误为空，耗时 1.81 s。桌面直接条件文本入口仍具名拒绝。 |
| 严格 ACE CLI 生成、Vue 构建及 Chrome | 材料页通过真实按钮得到 `Ready → 187 → 224 → Ready → 187`，卸载后按钮离开 DOM，错误为空，浏览器核验耗时 1.16 s。按既有材料验收，在生成目录启用 `allowJs`。 |
| 原 024 入口与五个原组件模块重跑 | 原 `.Tick` 的 `var nd`、`var lbl` 单独保留时，均通过语法进入 `source.age.msg-unknown`。保留列表迭代或记录构造的对照仍报语法错；完整原包的首个诊断仍为入口字节 1217..1223 的 `expected fn, found widget`。布局、列表写回与 400 ms 自动 Tick 尚需在原包联验。 |

最终语言、CLI 和桌面示例的增量准备分别为 3.64 / 38.00 / 5.23 s。本轮仅更新 AgePackage 文法 ID 为 `da9a638f8bf25128274dcf4af0e71c7d5e66f406d0c16316d7d94f4406dc3eff`。`chart-entry-block-mutable.log`、`chart-mutable-original-final.log`、`chart-mutable-native-final.json`、`chart-mutable-vue-browser-final.log` 的 SHA256 依次为 `e2a931c6ca9fc88d7bcd6a63644bfe71a2cea6e68172acada86c8f58556671a1`、`d2328943389ccaef4ba3fbebad4901a3773db61f08f781f49e151e9a6e66f5ae`、`237cc917d0ce21dc7217d44d5785d19440fb93c389e703e6ea2da6c858d75f87`、`9f7012db57cf4133bd71cc0765417c7c7f09d420515901777d8bbbaf2d72fe1d`。PLAN-114 后续列表提交的预检在文法、前门、投影、绑定和 HIR 适配重叠，待在共同受检基线上逐项接入后重测原 `.Tick` 和 `.Reset`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 16 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 707b9e5d, lang 0c10b6a5f | evidence: checked mutable local scope and structured unit handlers run in Iced and Chrome with matching tick and reset values; full original chart package still rejects list and record forms | next: work`

**024 的受检记录列表、滑窗对照与桌面文本更新（T-02/T-03 进行中）。** AutoLang `ccb98e942492d43cc19b8c151c020822e4927151`，AutoUI `213368da193d1ff7a2074f05402e77f8e429bf1d`。接入 PLAN-114 `35142ba50`、`6559fb2b1`、`e9bd89553`、`af933b9ca`、`652c6ba7f`、`1cee3518e`、`7d6eabe4c`、`b3316dc86` 的参数默认值、格式化文本、类型列表、嵌套记录、辅助函数与 AV 标量观察。合并保留了普通 Auto 条件数组及错误溯源。AGE 的分离表达式语句通过受检单元类型约束，值函数的根尾值维持既有规则。桌面包桥从同次派发观察提交标量状态和变化的文本叶，并按实例保留重开时的显示值。

| 执行 | 结果 |
|---|---|
| 编译与定向回归 | Core `plan981` 248/248、`plan1072_component_style` 29/29，AV `plan981` 63/63、`aggregates_t01_probe` 33/33，AutoUI `plan980_package_sfc` 18/18、`plan124_routes` 2/2、包窗口闭包 4/4。两张 AGE 文法身份更新为 AgeComponent `2d612124f0c0cdd84e06260f055a3e8db64f480828addcb646775c2e930a3c2b`、AgePackage `a9c41104708789945f3d9de104da48668d0c43b46de975f6a528bfb781b1b31d`，冻结邻接身份通过。 |
| 取原 024 的两份月度列表和流式算式 | 材料页的 `seed`、`monthly` 逐字复制原记录列表，补 `List<Month>` 类型，以受检辅助函数逐条复制与计算，复位逐条读取六条 seed 记录。AV 与专用 A2TS 集合发射在两个实例、7 个状态检查点与独立预期一致。长度 `6→7→8→8→8`，首月 `Jan→Feb→Mar`，暂停后 Tick 保持状态，复位恢复六条记录且另一实例保持 `t1`。直接把 `.seed` 交给按值辅助函数的反例在 MIR 报 `mir.use-after-move`，表明实际 `for` 仍需借用并逐条重建。 |
| Iced 真窗及重开 | 基线提交了流式标量，文本只显示初值。修复后 11 次真实派发和 11 次文本提交逐项对账，12 帧，两个会话各丢弃 1 次迟到事件。重开保留实例 0 的原六条和实例 1 的七条及 `t1`，无错误，耗时 5.55 s。 |
| 严格 ACE Vue 工具和固定原包 | 材料路由生成具名错误页 `source.age.sfc-handler: Tick: checked handlers require copyable int, bool or f32 bindings`，说明 A2TS 的受检集合执行尚未接入 A2Vue 响应式状态。原 `app.at` 与五模块重跑首报仍为入口字节 1217..1223，单独保留原 `out.push` 记录式的对照前进到 `source.age.msg-unknown`。原 `for`、四个组件的 `List<str>` 参数头、Donut/Line 的原 `timer` 块，各自单独缩减时仍在语法前门被拒绝。 |

最终语言测试、AutoUI 测试和真窗二进制增量构建为 22.52 / 58.14 / 15.16 s。`chart-collection-material-av.log`、`chart-collection-material-a2ts-final.log`、`chart-collection-native-final.json`、`chart-collection-component-probes.log`、`ace-route-error-0.vue` 的 SHA256 依次为 `ae5bfb4b952b63c741a5b18ba08ae423f793cfc37312cfef78b581be3e14af02`、`60efe2a6d1487746d3d33856a5646ba30067e44f31019c909c0f2e96563e6b68`、`4d99518c261dbded326628a51d50a503fcfc06e59f370c79d43b12d70ac4539b`、`34748cf565404d4a17dc6432c3b3fc811e0dbabf4cdaa0d70c85d26838ffcb7a`、`e3c5be847b0ddca2df002e428b060b22733748562661fa8c7f309e9ab1e49583`。继续按原 024 的处理器迭代、未标注记录列表、组件列表参数、响应式集合与计时接面逐项联验。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 17 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 213368da, lang ccb98e942 | evidence: checked record list material preserves original chart data and sliding-window values across AV/A2TS, Iced publishes the complete text sequence and preserves each instance on reopen, raw chart and A2Vue blockers isolated | next: work`

**024 原处理器的循环与列表追加（T-02/T-03 进行中）。** AutoLang `5fc2d7648bd349cfe5b63b0a97421798cd7ac250`，AutoUI `57f995fd3ae89e1cbe216288d7039b280dd10c8a`。AGE 包处理器的 `for` 沿既有受检集合循环执行，绑定按词法作用域，`push` 走共享类型与所有权检查。将借用的记录项追加到另一列表时创建新记录，源列表保持可读，遍历期间结构性修改源列表仍具名拒绝。A2TS 集合语义视图接受受检记录复制，复用现有记录复制发射与模板。未被局部绑定遮蔽的 `router.push` 保留已有宿主路由处理。

| 执行 | 结果 |
|---|---|
| 固定原 024 入口与五个组件模块重跑 | 原 `.Tick`、`.Reset` 完整语法均通过投影。原入口首报推进为 `source.age.msg-unknown`，覆盖 `.Tick` 字节 4176..4914。材料副本只声明 `Tick` 后，首次具名拒绝推进到原 `.monthly = out`，`monthly` 未进入受检模型字段。类型记录材料移除 `out` 注解时首报 `source.array-lit-empty`，移除追加项的名义记录类型时首报 `source.move-required`。四个组件本体仍有独立语法拒绝。 |
| 直接从处理器跑原 024 的滑窗和复位 | 材料沿用原月份数据、原循环、`out.push(d)`、原流式算式和两条 `.monthly = out`，为 `seed`、`monthly` 与 `out` 补类型，并为追加的新记录补构造类型。两个实例的 7 个检查点在 AV 与 A2TS 集合生成均通过。seed 始终 6 条，Tick 3 与 4 分别滑出 Jan 与 Feb，暂停和复位结果与独立预期一致。 |
| Iced 真窗 | 11 次派发、11 次文本提交和 12 帧通过，重开仍保留两个实例不同的文本，各会话丢弃 1 次迟到事件，无错误；本次耗时 15.31 s。现行包宿主按场景重放，400 ms 计时及正式源的运行性能仍待验收。 |
| 定向回归 | Core `plan981` 248/248、`plan1072_component_style` 29/29，AV `plan981` 64/64、`aggregates_t01_probe` 33/33，A2X Core 2/2、`ts_ui_fragment` 4/4，AutoUI `plan980_package_sfc` 18/18、`plan124_routes` 2/2、窗口闭包 4/4。AgePackage 文法身份为 `76930bede5ca4fce8e2a8f4ebfc2206ccbb6b9c23553e9195397a46794627a35`，冻结邻接面通过。Vue 平铺发射对循环和效果要求完整受检处理器，正式 A2Vue 集合状态接线仍待完成。 |

最终语言、AutoUI 测试和真窗准备为 28.01 / 97 / 29.21 s。`chart-for-controls/original.log`、`chart-for-controls/declared-tick.log`、`chart-for-material-av-final.log`、`chart-for-material-a2ts-final.log`、`chart-for-native-final.json` 的 SHA256 依次为 `b3df75cbfb4c3d35cc6cf2b5b4ac42e7db2aa827748bf42390e3b27bba6af748`、`b943ed7e1160f3b57f861d59e2e1e4476928e01c4c02d0ceb50b873f0de6e8dd`、`9192f126c2e8b6d4db2eb0b9a4d2bf5120d48c62ed30b3faa5045325003f5adf`、`ac5f3ae8b98f63cb2af4a3c9f67c3d47062a8832502cb8d20f2228250e30b331`、`4d99518c261dbded326628a51d50a503fcfc06e59f370c79d43b12d70ac4539b`。PLAN-114 的 Vue 宿主和 A2TS 挂载级运行时已前进至 AutoUI `f0ac2e40` 与 AutoLang `0a525158f`，继续核对可消费提交和原 024 的定型与计时接面。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 18 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 57f995fd, lang 5fc2d7648 | evidence: original chart handlers now project, checked handler loops preserve borrowed seed rows across AV/A2TS and desktop reopening, raw contextual types and clock binding remain open | next: work`

**024 原入口的记录列表推导与 Tick 声明（T-02/T-03 进行中）。** AutoLang `b863cb6428eeb543791e57259c00be38846ae2b7`，AutoUI 基线 `57f995fd3ae89e1cbe216288d7039b280dd10c8a`。AGE 包从 handler 直接迭代的同构标量记录列表推导模型类型，同名同类型的字段按稳定顺序归一。`var out = []` 从整值写回的列表字段取得类型，`push({ … })` 的字段由既有名义记录检查校验。旧 widget 的零载荷 `.Tick` 与正整数 `interval` 形成同一受检描述里的消息和定时条目。`chart-inferred-metadata.log` 从原入口投影读回 `seed` 与 `monthly` 的同一记录类型、空载荷 `Tick` 和 `400 ms` 定时条目。宿主的自动派发仍待接线。

| 执行 | 结果 |
|---|---|
| 未改动的 024 入口与原包闭包 | 固定 `app.at` 本身通过投影。带五模块闭包的首报为 `official::area_chart` 原组件头的 `source.syntax`，字节 `1458..1464`。只提供原入口时，具名检查要求 `official` 组件包链接。四个原组件各自的完整准入进一步待验。 |
| 源码对账、AV 和 A2TS | 材料中的 `seed`、`monthly`、`.Tick`、`.Reset`、`interval` 与原入口对应片段逐字核对一致。独立视图用于观察，两个实例各 7 个检查点通过。滑窗达到 8 条后依次滑出 Jan 和 Feb，暂停后的 Tick 保持数据，复位恢复原六条。把原追加项的 `tablet` 改为 `"invalid"` 时具名报告 `i32` 与 `str` 不匹配，原位置字节 `2281..2298`。 |
| 真窗与定向回归 | Iced 材料由手动派发脚本驱动，11 次派发、11 次文本更新、12 帧，两个会话各丢弃 1 次迟到事件，重开保留两个实例的独立文本，无错误，耗时 21.18 s。Core `plan981` 248/248 与 `plan1072_component_style` 29/29，AV `plan981` 64/64 与 `aggregates_t01_probe` 33/33，AutoUI `plan980_package_sfc` 18/18、`plan124_routes` 2/2、包窗口闭包 4/4。零间隔与错型记录反例通过。 |

AutoLang 集合发射与定向测试准备为 16.13 / 25.61 s，AutoUI 测试与真窗准备为 100 / 17.36 s。`chart-inferred-controls/original.log`、`chart-inferred-controls/bad-record.log`、`chart-inferred-source-parity.log`、`chart-inferred-material-av-final.log`、`chart-inferred-material-a2ts-final.log`、`chart-inferred-native-final.json` 的 SHA256 依次为 `dfe23da42dae3afef704779ea48096db28183d697b60422048131777c8461e17`、`a32a485c9d64a5ffa0f4113c5e70eca1dd00253f4721574313aba22d6739d718`、`ed91dce3decf7f1e238ae64e567e5388b83b20bdf65c5ccfad577050183189df`、`9192f126c2e8b6d4db2eb0b9a4d2bf5120d48c62ed30b3faa5045325003f5adf`、`ac5f3ae8b98f63cb2af4a3c9f67c3d47062a8832502cb8d20f2228250e30b331`、`04124519fe2eafd6a11a43f28f2c8d8d830cdb86f3dfce36d709c3fb473f0cab`。下一批继续接入原组件列表参数、动态布局与正式两宿主的计时及集合执行。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 19 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 57f995fd, lang b863cb642 | evidence: unchanged 024 entry projects with inferred record lists and a 400 ms Tick declaration, original handlers replay in AV/A2TS/Iced material, and original closure now stops at AreaChart | next: work`

**024 四个图表的参数、索引循环和定时守卫（T-02/T-03 进行中）。** AutoLang `cd91223cd8ef23d80e1ec84a9642224530265d5c`，AutoUI `ee142965bd66f08459abd8ba3883db8463cd0bda`。AGE 包投影保留 `List<str>` 元素类型、默认值顺序与空列表，以及 BarChart 原 `type` 参数和 `.type` 引用。处理器 `while` 走共享受检循环，索引读写走现有集合操作。显式 `timer { Msg (every_ms: n, when: …) }` 在两个 AGE 文法投影守卫，在包面由共享检查确认布尔类型。Vue 采用完整受检处理器接面识别 `while`。

| 执行 | 结果 |
|---|---|
| 四个原组件及闭包 | 四份原参数头、模型块和处理器块分别通过语法投影，Line 和 Donut 的原定时块也通过。整包首报仍为 `official::area_chart` 字节 1458..1464 的组件声明语法拒绝。分别隔离出四份原 `style { … }` 以及条件类列表，两种形状各自重现语法拒绝。替换视图、移除局部样式块的材料中，Line 通过投影，Area、Bar、Donut 首报原 `HoverOut` 缺少消息声明。仅在材料补该声明后，三者同样通过投影。 |
| AV、A2TS 和桌面受检索引材料 | 原处理器的 `while`、整数索引读写及 `every_ms: 33, when:` 形态经正式包入口执行。AV 与 A2TS 的三组双实例文本为 `0/0 → 0/31 → 31/31`。在真实 Iced 窗口手动触发两次，重开保留实例各自状态，8 帧，两会话各丢弃 1 次迟到事件，无错误，耗时 3.12 s。错型列表默认值、非布尔守卫和错型索引赋值均具名拒绝。 |
| 回归与身份 | Core `plan981` 248/248、`plan1072_component_style` 30/30，AV `plan981` 65/65、`aggregates_t01_probe` 33/33，AutoUI `plan980_package_sfc` 18/18、`plan124_routes` 2/2、包窗口闭包 4/4。AgeComponent `c070f84f2364c3ae46c7a4f626eeba090f6bc69a2f6fe089b182ebb0111d674e`，AgePackage `368e8a0c253e661f0c38ace5c0bc005e34e65132390791f01faa4e7cae31def0`，相邻冻结面通过。 |

语言定向测试和 A2TS 准备为 19.16 / 12.02 s，AutoUI 定向测试和真窗准备为 88 / 44.55 s。`chart-params-original-final.log`、`chart-params-final-probes.log`、`chart-params-style-boundary.log`、`chart-loop-material-av-final.log`、`chart-loop-material-a2ts-final.log`、`chart-loop-native-final.json` 的 SHA256 依次为 `c6251b3201a7aaf2053bb788fe27f588e53e8383c0e6e54c52159c71f2a002f1`、`e3ef8e6fd1df7a13c03d6c874453974cacd9a8e145b1ffd1b1d04ab1ac3a81c3`、`d2b413089c9ec3a5fa15e0aa0eb6e6a800cac43b4cf27ce952bda9795f4a0435`、`b538b55039a156dd629a87fbc027fe9c906553d3af0cb470f8d82107ad4517ce`、`596e722113b0c3b86b7b5031c14e010836f846672fd97d2618e989caec60d1a6`、`3fddd2e0a7fe7b901fe888de1bb7aea3879fd6e7c5a846dcf67dce4c45ec7e27`。接下来核对原组件 CSS、条件类列表、消息约定和受检属性挂载，之后用正式宿主验计时和图形行为。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 20 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui ee142965, lang cd91223cd | evidence: original chart parameter, handler and guarded timer fragments project, indexed loop material agrees in AV/A2TS/Iced, and exact style, conditional class and message blockers are isolated | next: work`

### 第 21 轮｜图表局部样式和条件类列表

AGE Package 在组件成员位置保留 `style { … }` 的原 CSS 和范围。条件类列表只在 `style` 与 `class` 属性接纳，列表项为文本字面量，条件经共享验证器核为 `bool`。普通列表仍走原表达式投影。AGE Component 及周边语法指纹保持冻结。

A2Vue 把原 CSS 发射为 Vue `<style scoped>`，在普通元素和 SVG 模板中绑定条件类。受检 Vue 项目开启 `allowJs` 以编译生成的 JavaScript SFC。Iced 的包控件树按实例状态选择类分支并将选中类交给样式适配层，容器子树的整体透明度仍待真窗核验。

四份原始 CSS 独立投影通过，完整 024 原闭包已跨过此前第一个样式语法阻断。Area、Bar、Donut 在 `HoverOut` 未声明处报 `source.age.msg-unknown`。仅在对照材料声明 `HoverOut` 后，三者都推进到 Line 原文件同样报出的导入 `caption_text` 的 `source.age.style-call`。四份原文件保持原始字节。对照验证了把条件列表用于 `value` 或把整数用于条件会被拒绝。

从独立目录用 `auto build --render vue --gen-only` 生成路由材料，复用相同依赖的固定锁文件离线安装并通过 `vue-tsc && vite build`。Chrome 157 实测 SVG 路径有 scoped 属性且计算样式包含 `stroke-width, fill-opacity`，点击图例按钮使计算透明度从 1 到 0.4 再恢复为 1，随后卸载，浏览器错误为零。Core 定向 248 + 30、AV 65 + 33、UI SFC 19、路由 2、桌面闭包 4 通过。

`round21-chart-admission-summary.log`、原闭包投影 `round21-chart-admission/original-closure.log`、`round21-vue-browser.log`、`round21-vue-build-final.log` 的 SHA256 依次为 `788b13415b99de54fa09937f0fb892415090f3baa3cde1d249160631a7f423f5`、`011b1acb91b64ff0c7fb4d3517945ea92feae3f1509a663511148ac633da1d7a`、`f99433c66597a9b0b6f03d8f08c9da6ba991f676e4909197757878e449da8cd6`、`a32ef23164194de99bfa13b7e54286d5574e9db6f8b9db2bf5486e1cb8f7f610`。下一步核对旧 widget 的零载荷 handler 补变体约定和 stylekit 公共样式绑定，再对未修改的 024 闭包运行双端。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 21 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui cdc1fd94, lang bf6fdd8af | evidence: original chart scoped CSS passes and original closure advances to message/style imports, route builds and Chrome verifies scoped transitions and reactive class | next: work`

### 第 22 轮｜旧图表 widget 零载荷消息

受检包在旧 `widget` 兼容入口补齐由零载荷 `on` handler 引入的消息，显式签名优先。无声明的定时 `Tick` 继续由正 `interval` 规则接入。载荷 handler 与规范 `node` 仍按显式声明校验。对照通过 AV 派发给第二个实例，两个实例的数值和文本分别为 100 与 40。Core 定向 249 + 30、AV 66 + 33 通过。

四份未修改的 024 图表均推进到导入 `caption_text` 的 `source.age.style-call`。把真实 `stylekit::styles` 一同列入闭包后，入口仍在 Area 的原引用处报该诊断。真实 stylekit 独立投影还在 `pill` 参数默认值的 bytes 726..738 报 `source.age.style-defaults`，后续需要同时解决公共字面量样式绑定和依赖样式模块中未用配方的处理。

`round22-chart-admission-summary.log`、`round22-chart-admission/original-with-stylekit.log`、`round22-chart-admission/stylekit-original.log` 的 SHA256 依次为 `41c1555e885d56c4a3a7c46b3c45241bbd3d002dc17a15ad8467738c3676b53f`、`65568b058d9b16992275c5ead1959dc09744110dcd5f5f9e83cf3c40247c7295`、`4b7a33f3ca4f16343afa4959e53f4c812689d1dbf247f3b4bf655a76cc27f955`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 22 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui cdc1fd94, lang 67947b9cf | evidence: AV replays the legacy unit handler on one instance, and all original chart components advance to the public style binding boundary | next: work`

### 第 23 轮｜真实 stylekit 公共样式绑定

包绑定器根据解析后的公共符号和可见性折叠跨模块的字面量 style。Vue 与桌面包入口消费同一份已绑定的组件树。真实 stylekit 的未被引用的公共参数化配方保留其具名诊断，依赖方实际引用时仍报告配方原位置。入口包原有的默认值与插值诊断继续通过回归测试。

真实 `stylekit::styles` 在闭包里提供 `caption_text`，SFC 对照产生原类串 `text-xs text-muted-foreground`。四份原图表连同真实 stylekit 和 `chart_geom` 均推进到共享类型验证。各自首个诊断位于 Area 的 `.legend`（14072）、Bar 的 `.legend`（12803）、Donut 的 `.data`（11905）、Line 的 `.legend`（18014），均为未绑定的 `self`。完整 024 同时暴露 `.data`、`.fields` 等构造参数、迭代局部值和借用问题。未修改原组件和依赖源码。

Core 249 + 30 + 11、AV 66 + 33、SFC 20、路由 2、桌面闭包 4 通过。`round23-chart-admission-summary.log`、原闭包 `round23-chart-admission/original-with-stylekit.log`、Area 原组件闭包 `round23-chart-admission/area_chart-with-dependencies.log` 的 SHA256 依次为 `bb8267fa364c56ca860640f8b9841fdfe6bb9c1c998ff520ba5436d8895c5c6d`、`036e5d9b4f48fe4cc7cd9e9682f739f7398ca217aba3c9d4f087ded3a7a86984`、`1bbde347488de6abace95347288469083c3312939ca09bccd0862c3087f0677a`。下一步按原组件和调用者核对参数类型与作用域，包括必填的裸 `List` 数据及带默认值的只读属性。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 23 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui b2c795c4, lang 21f406eb9 | evidence: real stylekit literals bind into the checked tree for both lanes, original charts advance to widget parameter and view-loop verification | next: work`

### 第 24 轮｜视图的只读参数作用域

包视图的值检查与事件参数检查从已声明签名读取只读参数，覆盖图表用到的 `bool`、`str`、`int` 及带元素类型的 `List`。这些参数进入共享验证器的检查函数。测试验证了合法的图例条件、列表长度和整数消息载荷，以及将文本当布尔条件、将布尔当整数载荷时保留原引用位置的类型诊断。构造参数的运行时传递和 handler 内的 props 读取仍待接入。

带完整真实依赖的 Area、Bar 和 Line 已跨过原先的 `.legend` 属性失败，首个诊断依次落在视图循环内的 `b`、`seg`、`p`。Donut 的首个诊断则在 handler 读取必填的裸 `List` 参数 `.data`。Core 250 + 30 + 11、AV 66 + 33 通过。完整原闭包证据 `round24-chart-admission/original-with-stylekit.log` 的 SHA256 为 `792deaa11ec9138eb63bb5b10400887cba2e2fbe351be8a2edd32f5e9ed9910b`。

PLAN-114 的固定在途提交已有受检子组件输入、双端挂载、原路由视图和条件 props 的实现及测试。AutoLang `872441620`、`e784c0e1f`、`fa1daa2dc` 和 AutoUI `b2d9e05a`、`f9254500`、`9a916217` 需要对照 PLAN-107 当前代码检查可消费边界。下一步同时核对现有运行时输入接线和循环局部值的共享检查路径，并用原图表重放。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 24 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui b2c795c4, lang f7630b7dd | evidence: declared chart view properties type-check and original components advance to loop locals and required List handler input | next: work`

### 第 25 轮｜视图循环中的普通属性检查

普通视图属性、元素标签和条件样式引用循环变量时，复用事件载荷的受检循环降层，按作用域和元素类型检查循环值、下标及其派生表达式。相同的类型规则接受整数循环值与下标，并拒绝把整数元素与文本比较。其余值沿原检查路径。

真实依赖齐备的 Area、Bar 和 Line 已跨过循环局部名未绑定。Area 的 `b["i"]` 和 Line 的 `p["x"]` 现收到元素为 `Str` 的索引类型诊断，Bar 的 `seg.g` 收到同一元素类型的记录投影诊断。三者的原列表均从空的裸 `List` 开始，由 handler 构造记录并写回。Donut 仍先在 handler 的必填 `.data` 处失败。原始源码保持输入哈希。

Core 251 + 30 + 11、AV 66 + 33、SFC 20、路由 2、桌面闭包 4 通过。完整原输入 `round25-chart-admission/original-with-stylekit.log` 的 SHA256 为 `78a3c629ba9421b90425aaeb99e969304b271abf47f12558bb6f50a8a9c9c5a4`。合并树对照发现 PLAN-114 的运行时提交与当前工作在 AutoLang 13 个、AutoUI 6 个文件重叠。下一轮需要保留已核对的图表路径，并以真实构造、记录元素形状和输入传递为判据逐项兼容。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 25 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui b2c795c4, lang 18d325284 | evidence: ordinary view expressions use checked loop locals, exact charts advance to authored record element and input typing | next: work`

### 第 26 轮｜从 handler 构造推断旧图表的记录列表

旧 widget 的空模型 `List` 如果从 handler 中接受局部列表，且该局部列表中追加的记录字段及类型一致，则投影为共享的名义记录列表。模型和 builder 使用同一类型，追加和赋值仍由共享验证器检查。派发对照在两实例中得到列表长度 0 与 1，混合追加形状被拒。

逐字节原组件的 Area `bands`、Bar `segs0..3` 与 `bands`、Line `pts0..3` 均得到正确的记录元素类型，同形元素共享名义类型。Area、Bar 和 Line 的首错推进到文本键索引，验证器明确看到接收者为 record。Donut 仍先在 handler 的必填 `.data` 处失败。原闭包证据 `round26-chart-admission/original-with-stylekit.log` 的 SHA256 为 `fb93aa0d87cbc69f4bf72a6354cffab31131b1dc0cbedb8e210ecce4168a9a02`。

Core 251 + 30 + 11、AV 67 + 33、SFC 20、路由 2、桌面闭包 4 通过。下一步支持已知记录的静态键读取及拥有型字段借用，再联结构造调用者提供的裸 `List` 数据。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 26 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui b2c795c4, lang 2aecf2157 | evidence: exact native chart builders infer shared typed records and advance to record bracket reads, AV confirms instance-local appends | next: work`

### 第 27 轮｜图表记录文本键读取

AGE 包中，静态文本键读取已知记录局部名时复用点号字段的类型和借用规则。该规则覆盖列表循环绑定，原字符串键的 span 保留给未知字段诊断。AV 用同一材料检查 `band["i"]` 的整数值和 `band["s"].len()` 的字符串字段，并验证错误键拒绝及两实例列表长度 0 与 1。

完整的四份原图表首次同时推进到 handler 参数读取。Area、Donut 和 Line 在 `.data` 处，Bar 在 `.type` 处报 `self` 未绑定。原闭包证据 `round27-chart-admission/original-with-stylekit.log` 的 SHA256 为 `90c24d32704c0192202be39316fd66accdc77995ff3db501ae1ec16386424ccf`。Core 251 + 30 + 11、AV 67 + 33、SFC 20、路由 2、桌面闭包 4 通过。

PLAN-114 的 `872441620` 已有处理器参数降层和 AV 调用的共享路径。单提交合并树对照有五个重叠文件。该实现覆盖显式集合类型或有默认值的裸文本列表，024 必填的裸 `.data` 仍需要从调用点绑定数据元素类型。下一轮验证共享接线能否作为该绑定的基面。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 27 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui b2c795c4, lang 8988a6ee8 | evidence: checked record bracket reads advance all four exact chart closures to handler widget inputs | next: work`

### 第 28 轮｜共享组件输入与实时子图例

两计划工作树分别接入 PLAN-114 的 AutoLang `fa1daa2dc` 和 AutoUI `f9254500`，另纳入 `0a525158f` 的浏览器组件运行时作用域。组件包发现与绑定共用标签匹配规则。AV 使用受检组件帧求值，A2Vue 由产品 CLI 生成挂载路由，已接入子组件的条件类名及字符串所有权修复。

同一份父级切换材料通过 AV 组件帧断言。生产 CLI 生成 Vue，`vue-tsc` 和 Vite 构建通过。Chrome `157.0.8103.80` 回放图例计算透明度 `1 → 0.4 → 1`，组件卸载后无页面异常。`round28-vue-browser-owned.log` SHA256 为 `b700495144037a4b89accd76f9126e76414dfe7e6b81aaf6c81cacafcddb08fc`。修复前浏览器超时日志 SHA256 为 `c7e04bed9b12270084900e61b5727f416341c4164bf278f098427d817ca69cb3`，调试见 `Str` 释放后读取。Core 251 + 30 + 11、AV 69，组件包 8、路由 2、SFC 20、桌面闭包 4 和运行时独立挂载 1 通过，`p107_` AV 定向复验 7 通过。

原始 `024-charts` 闭包首报官方 LineChart 的必填 `data: List` 缺元素类型，Bar 也已推进到 `data`。原包证据 `round28-chart-admission/original-with-stylekit.log` SHA256 为 `d0e8675a4d55f6cd8e7d1553af9b48353a7a8eac53b48c1013582cec27d757da`。同入口对照的显式 `List<ChartRow>` 通过，裸 `List` 复现，后续从实际调用范围绑定数据形状。图例回放证据适用于本轮验收材料。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 28 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 855c387e, lang f2f524ef9 | evidence: checked child aliases, live class updates, AV frame and production Vue browser replay | next: work`

### 第 29 轮｜必填图表列表采用调用端记录类型

绑定器从已解析组件调用的 `model` 字段取得必填裸 `List` 的元素类型。跨模块的生成形参和求值函数使用同一名义记录类型，类型查找复用声明和导入绑定。子模块从保留的 CST 补投影，源码及语法标识保持原样。所有调用表达式仍经过共享受检编译。

AV 组件帧回放读到子图例 `7 → 13`，Vue 产品 CLI 路由通过 `vue-tsc` 和 Vite，Chrome `157.0.8103.80` 实测 `7 → 13 → 7` 并卸载，无页面异常。浏览器日志 `round29-vue-browser.log` SHA256 为 `d27a3e28ae2e52668e73e2ae4cde0a3b9613a74384b578fbc2992f74b1975b7d`。导入记录类型通过，空列表无来源及同一参数收到不同记录形状均具名拒绝。Core 完整 conformance 866、AV PLAN-981 69、组件包 9、路由 2 和 SFC 20 通过。

四个原图表闭包均越过 `data` 类型错误。`round29-chart-admission/original-with-stylekit.log` SHA256 为 `fc4d933d142864f76e2a644e6cf7740b6bbf7a6c84cf6283b9ad239de5b2e4c0`。首项新错误是原页 `checkbox checked` 布尔值被当成字符串插值，另有处理器循环参数读取、动态记录键和内部列表等共享诊断。原 `024-charts` 的整体挂载及页矩阵仍待验证。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 29 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 9624d1bd, lang 9ba35fe8b | evidence: caller record list, AV frame, production Vue browser replay, exact 024 closure reaches downstream checks | next: work`

### 第 30 轮｜图表复选框与处理器参数作用域

AGE 处理器的内部赋值、`for`、`while` 和效果表达式沿用同一个受检组件参数作用域。视图 `checked` 用布尔条件求值，非布尔输入具名拒绝。AV 组件帧把同一受检值和当前消息映射到 Iced `View::Checkbox`，Vue 输出原生复选框。

Iced 视图回放通过子组件 `Init` 循环求和 7，确认复选框由已选变为未选。Vue 生产 CLI、`vue-tsc` 和 Vite 通过，Chrome `157.0.8103.80` 实测 `true → false → true`，初始化求和值保持 7，卸载后无异常。`round30-vue-browser.log` SHA256 为 `4ebcb1492a1de523b41623747ff57661540811fd9bf83b6a1633820464de91c3`。Core PLAN-981 251、AV PLAN-981 69、组件包 10、路由 2 和 SFC 20 通过。上一轮启动的扩展 AV 全量运行持有旧测试可执行文件，已结束该进程并重链本轮定向测试。

原 `024-charts` 的布尔属性和参数作用域诊断已清除，首项推进到 AreaChart 的动态记录键读取。原闭包 `round30-chart-admission/original-with-stylekit.log` SHA256 为 `6cec7bff34db0235659972bbbd9680c70783d76ae43a5d0a017cdb78028fe967`。在固定 AutoOS 原 CodeEditor 页重跑，消息错误已通过旧式 widget 的单位消息接线消除，当前首报缺 `code_editor_cursor_line`。`round30-code-editor-original.log` SHA256 为 `296f8f07bb07565f0a7498b4b6a32c6def85d8a47e246f6c168046d8da35f87a`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 30 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui bfa24e2c, lang 9bae6413d | evidence: checked native and browser checkbox, child input loop, exact chart and CodeEditor probes | next: work`

### 第 31 轮｜图表临时数值列表推导

复用 AGE 的 handler 列表推导，空列表构建器在追加项一致时取得标量类型。后续索引和算术可使用先前推导的列表元素类型，提交给 model 时沿用共享检查。AV 定向用例检查两个实例分别为 `0/0` 与 `1/1`，异构追加具名拒绝。Core 完整 conformance 866、AV PLAN-981 70 通过。

原 `024-charts` 的空列表类型诊断由 18 处降至 7 处，首项仍在动态记录键。原闭包日志 SHA256 为 `d4fc69664dc669a475ff6d05531198e53265779e0a0dbc2c4b838ebd1e4149be`。Vue 路由材料的 `components/` 独立扫描拒 `ChartLegend.Build`，日志 SHA256 为 `73a2e731350d8c115fc2404f8043999534a379dd3cc4ea6f199e183061273fc5`。从显式依赖目录生成的同材料保留闭包，路由报告 A2TS 集合种子缺 `F32`，诊断页 SHA256 为 `d1fdbdc1d62116d0d0180dce879fdd605c95e4f05744f6416455fe3dbc78c57e`。本轮浏览器验收仍待补齐。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 31 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui bfa24e2c, lang bc07e75e9 | evidence: AV numeric list builders and exact chart diagnostics, named Vue seed and standalone scan rejections | next: work`

### 第 32 轮｜A2TS 集合种子保留 f32

A2TS 集合种子为 `F32` 字段和列表元素保存 `f32` 类型，复用按值复制及现有单精度算术。集合种子往返和 Node 运行时回归通过，A2TS 单元 20、运行时与 provider 4 通过。

使用第 31 轮显式依赖目录材料生成生产 Vue 路由，`vue-tsc` 和 Vite 通过。Chrome `157.0.8103.80` 点击后读到 `0/0 → 1/1`，卸载后无异常。`round32-vue-browser-final.log` SHA256 为 `ffbafe1d67cf3fbee76beee2a7a944fb00bb5a774545c7307a309a766becd4f3`。原材料的 `components/` 扫描仍需接入已生成的受检路由闭包状态，原 `024-charts` 动态记录键及整体挂载仍待验收。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 32 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui bfa24e2c, lang 0f16585f7 | evidence: f32 collection seed and exact numeric action in the production Vue browser material | next: work`

### 第 33 轮｜Vue 项目构建复用受检路由组件

组件发现保留模块的规范文件路径。路由编译记录模块内全部 widget 已由受检运行时发射的文件，全量和增量构建据此复用路由产物。增量路径沿用已有缓存产物清理，独立组件的缺席负控仍报告具名错误。

第 31 轮标准 `components/` 目录材料已通过生产 `auto build --render vue --gen-only`、`vue-tsc` 和 Vite，页面内为 `<AcePackageView />`。Chrome `157.0.8103.80` 实测 `0/0 → 1/1`，卸载后无异常。`round33-vue-browser.log` SHA256 为 `ffbafe1d67cf3fbee76beee2a7a944fb00bb5a774545c7307a309a766becd4f3`。组件包 10、路由 2、SFC 20、全量及增量工具综合回归 1 通过。

固定 AutoOS 画廊外壳仍在 `widget App` 附近报 `source.syntax`。下一阶段接入 owner 的连字符路由模块和受检 outlet，重放实际画廊入口。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 33 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui afd90665, lang 0f16585f7 | evidence: checked production standard component directory, cold/incremental regressions and Chrome replay | next: work`

### 第 34 轮｜受检画廊外壳、抽屉与路由挂载

合入 AutoLang `cc393e7a3` 与 AutoUI `9a916217`，并接入 `668abc15d` 的受检单位回调。保留连字符模块、外壳 outlet 属性和现有复选框及图表闭包。语法身份实测更新 `AgeComponent` 与 `AgePackage`，邻接裁剪冻结测试通过。

Iced 路由回归验证嵌套 outlet、页面切换、关闭后的消息拒绝及回到页面重新初始化。生产 Vue 材料通过 `vue-tsc` 和 Vite，Chrome `157.0.8103.80` 实测关闭抽屉、进入 `/charts-page`、执行数值动作 `0/0 → 1/1` 并卸载。`round34-vue-browser-verified.log` SHA256 为 `dd0340b8431c44aea7bfca9f6a3fa7da65088230f53526de7d00b3e0506b9afe`。Core PLAN-981 251、AV PLAN-981 70、组件包 12、路由 4、SFC 20、Vue 路由与工具综合各 1 通过。Windows 并行构建遇分页文件错误，串行重跑通过。

固定 AutoOS 原外壳及 SettingsPanel 已从语法错误推进到 `SetAccent(name)` 缺少消息载荷声明。原闭包日志 SHA256 为 `223f24b7ef704d17053b9ba16d7655d01f05e7cc2973029613e9667189a8cb2c`，仅给材料副本增加 `SetAccent(str)` 后闭包通过。继续用受检的旧式直接赋值推导重放不改写的原闭包。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 34 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 50e010f1, lang 889033418 | evidence: scoped gallery navigation, drawer, typed callback and actual shell admission advances | next: work`

### 第 35 轮｜原画廊 Carousel 的生产 Vue 回放

旧式 widget 的直接 model 赋值可确定隐式消息的标量载荷。原 AutoOS 外壳及 SettingsPanel 的受检闭包通过，原字节闭包日志 SHA256 为 `473845513e120e23c6027121926177882ebf21e70e24aa1e7dd8d3231df98f2e`。AV 两实例实测只有收到 `SetAccent("ocean")` 的实例更新，载荷被不同类型字段约束时具名拒绝。Core PLAN-981 251、AV PLAN-981 71 通过。

生产 CLI 在隔离输入中保留固定 AutoOS `pac.at`、外壳、Carousel 页及 SettingsPanel 和 Carousel 组件源，生成 70 条路由。固定 AutoDown `45b642dad` 的源码重建后通过产物新鲜度校验。`vue-tsc` 和 Vite 构建通过，Chrome `157.0.8103.80` 经实际侧栏进入 `/carousel`，验证五组区域、横向点击和键盘、拖拽、宽度比例 `1/3` 与 `1/2`、纵向位移、循环及卸载，页面异常为 0。`round35-gallery-ready-browser.log` SHA256 为 `48d35a1034e92d355e4d6684eacd333cc509d5e474d4e525c3ea3b71bb4b5ca9`。镜像包含本轮测试使用的画廊页和组件，其他路由仍待回放。

Iced 原外壳最初报 `ace.desktop.view-cond-unsupported`，定位原源码字节 6371，后续需要由已受检的路由帧承接该条件。原 Carousel 的 Iced 路由入口、指针回放及会话终态继续验收。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 35 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 50e010f1, lang 5950a6222 | evidence: original gallery shell and settings compile, original Carousel page production Vue action replay | next: work`

### 第 36 轮｜原画廊条件外壳进入 Iced 窗口

桌面路由在加载后检查是否具备受检帧。受检帧直接承接条件与链接，旧平铺路由仍按原边界检查。路由回归覆盖在模型条件内呈现的链接，组件包 12 和路由 4 通过。

固定 AutoOS 外壳实际打开 Iced 窗口，记录 2 帧并脚本关闭，外壳会话错误为 0。默认调试窗口在主线程栈溢出，单独用 `/STACK:67108864` 重新链接窗口驱动后通过。`round36-gallery-large-stack.json` SHA256 为 `90bc8d7c456b3cc4f98085066cb1188b8e2b79f394de19586ffcd97df04dffff`。默认 `/` 页 `index.at` 仍为具名源码诊断，后续从实际外壳进入原 Carousel 页并回放控件动作。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 36 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 3dceaa13, lang 5950a6222 | evidence: native checked conditional route frame and real original shell mount/close | next: work`

### 第 37 轮｜原 Carousel 从画廊进入并完成双端控件回放

原 AutoOS 外壳与原 Carousel 页挂在独立窗口。脚本读取 Iced 实际按钮并触发其当前受检路由消息，`/carousel` 成功开出页面会话 2。7 帧中验证五组区域，初始可见项数 `[1,3,2,2,3]`，横向键盘和真实拖拽推进到第 3 项并禁用下一项，无障碍点击与纵向焦点得到 `[2,3]`，循环回退得到 `[5,1,2]`。关闭后外壳和页面会话均关闭，页面拒绝一次迟到的轨道回调。

Iced `round37-original-gallery-carousel-window.json` 与同名 `.log` SHA256 分别为 `0e4643c8e0fafe4c8e3de3f78c15ae4a5ef982e69d0ca03f1e80108e86f86248` 和 `6d38958e175b6f42cabf6f2742ed288117e9655dc7e89efcb6897f872f7bdd35`。沿用第 36 轮窗口驱动栈设置。默认 `/` 页仍报告 `index.at` 源码诊断，Iced 记录 `sticky` 与 `fixed` 的既有平台降级。与第 35 轮 Chrome 证据共同覆盖 Carousel 原页的两端交互。

重跑四个原画廊图表页及其组件，首项均为裸 `data: List` 尚未取用页面的静态记录数据。LineChart 的两个实际调用分别使用 `{m,v}` 与 `{m,desktop,mobile}`，后续需逐调用核对元素形状和组件实例。FlowDiagram 与 CodeEditor 仍有原页准入诊断。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 37 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 3dceaa13, lang 5950a6222 | evidence: original Carousel route and real native controls plus production browser replay | next: work`

### 第 38 轮｜原画廊静态数据绑定图表列表输入

组件包绑定器识别作为必填裸 `List` 传入的静态 model 字面列表，并沿用受检记录推导。仅重投影实际输入涉及的字段和组件，保留原源码和语法标识。AV 组件包回归移除测试专用的 handler 循环后仍实测切换 `7 → 13`，异形输入保留具名拒绝。Core PLAN-981 251、AV PLAN-981 71、组件包 12、路由 4、SFC 20 均通过。

按固定原字节重跑 Area、Bar、Donut、Line 四个画廊页。四条闭包均越过必填列表诊断，首项推进到 `source.index-base` 动态记录键。`round38-gallery-{area,bar,donut,line}-with-component.log` 的 SHA256 依次为 `ea23404333c5290ae925bc2c04235a5350dcd5388fd2c23b2011e2f3529f27ca`、`2091fa702b1ddc3d40d1f2ccd9888af8755e25b3ef698d9257034befb129928f`、`3b92e5b5e9bab704190dcdd538f65893c4e0513630ab745e5d04aa2a1aa61f95`、`a2c858e9669ef25903b67d03eb2c952c893be7f96335fd70813b03e68c66bc9c`。原 `024-charts` 命中与第 31 轮相同的诊断日志哈希 `d4fc69664dc669a475ff6d05531198e53265779e0a0dbc2c4b838ebd1e4149be`。两端的原图表窗口回放仍待准入。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 38 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui a674e157, lang ba467d71c | evidence: static record model inputs, checked Iced frame and all four original gallery chart closures reach record indexing | next: work`

### 第 39 轮｜原 FlowDiagram 页解析边记录的 `to` 键

包面在静态 model 记录键位置接收路由保留字 `to`。AV 两实例回归按 `edge["to"]` 统计原形态的目标节点，仅收到消息的实例由 `0` 变到 `1`。Core 完整 conformance 867、AV PLAN-981 72 通过。AgePackage 语法标识更新为 `364ebfe7c4e0a9bc0ffc7de6ffca626327109b516156750dd03ddaebcc690c9d`，其余冻结语法面通过逐字节对照。

原 `pages/flow-diagram.at` 已按固定字节解析出组件。闭包下一项诊断在真实 `components/flow_diagram.at` 的 widget 入口，`round39-original-flow-closure.log` SHA256 为 `3011e704716d796a7a8c33193ff6de1250bf10a5b084cf2e71ec917339484bdb`。原页布局与交互仍待端到端回放。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 39 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui a674e157, lang 300f50f5b | evidence: exact FlowDiagram page parses and AV executes graph edge destination reads | next: work`

### 第 40 轮｜按原拼写绑定受检子组件事件

接入 PLAN-114 的 AutoLang `53f36cc3a` 和 AutoUI `1bed1330`，保留本组的包别名匹配、列表绑定及条件类名。两种组件标签形式均验证命名事件。原样 `$event` 转发要求子组件声明出站事件且父消息载荷类型一致。缺少可见子组件、未知事件或目标、重复绑定与异型载荷均具名拒绝。AgePackage 语法标识为 `288491af2724439e9445ff2d901cd26327719cca1be82f850bf47a1e3132b3e4`，邻接冻结通过。

Iced 受检帧点击子组件后，父状态读取到 `docs/计划`。Vue 产品 CLI 冷生成路由，经固定依赖的 `vue-tsc` 和 Vite 构建，Chrome `157.0.8103.80` 点击子组件后显示载荷 `detail/图像`，卸载后无页面异常。首次 Chrome 回放的文本定位器超时，调试确认实际文本已更新，按可见文本行复验通过。最终浏览器日志 SHA256 为 `03ebd5bb4942ae1e5279e11f5d3fcb8e844ede6cc458c36e1a26cde0c183329d`。Core PLAN-981 251、AV PLAN-981 72、组件包 13、路由 4、SFC 20 通过。

原 `029-photo-gallery`、`031-image-viewer` 和 `031-paint` 入口从 `$event` 词法错误推进到 widget 语法诊断，探针日志 SHA256 依次为 `4510ab10d2eeaaa508545a029c366ec24efacfdc1dd99942ef2eab3b80228b55`、`2145d3bcad8507622bd1668e5b6d921b171a0eb96d14bfed851796fc8ade277b`、`c46f8cc39b41932f803f10e51eef40742137dbc718c88cac35da45e3b33ec0f1`。原 030 控制条和图像视口另有 `$0` 等宿主参数，原组件处理器的出站调用仍待接入。两端通过项适用于本轮定向材料。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 40 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 46b78613, lang ee32dd1b4 | evidence: checked named event forwarding in Iced and production Vue, three original app entries advance past event lexing | next: work`

### 第 41 轮｜处理器保留并校验原形参类型

接入 PLAN-114 `bb0aaf2d0` 的形参注解部分，复用本组已有的作用域和循环降层。注解在原 span 按载荷槽核对。AV 两实例测试以 `str, i32` 接收声明的 `str, int`，定向实例由 `/1` 更新为 `图像/2`，异型注解在 `bool` 原 span 具名拒绝。Core PLAN-981 251、AV PLAN-981 73 通过，AgePackage 语法标识为 `78d6c6bb79eab93920e5241124acf42d9efe9fcb6e5c51b852139b62302de36f`。

从原 `031-image-viewer` 提取的 `.ApplyView(ev str)` 片段越过语法门，缩减材料在未保留消息声明处具名拒绝。原项目整体仍有后续语法诊断。隔离探针同时定位了原 FlowDiagram 组件的 `stack.pop()` 和局部 `take`，以及默认画廊页的连字符 `to:` 标签及带属性文本体。探针 `round41-shape-probes.log` SHA256 为 `97b9ba48d6fcb8a0c79381a3b95e6e712cb8844cc6cc66aa1e53653a2b9a85d0`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 41 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 46b78613, lang c10dc8ed0 | evidence: media handler type annotations checked by AV and original source forms isolated | next: work`

### 第 42 轮｜原 FlowDiagram 组件进入数据形状检查

包面支持布局代码的局部 `take` 和丢弃结果的 `stack.pop()`。后者沿用受检列表读写，空栈保持为空。仅在 handler 中用到列表时也启用同一集合验证器。Core PLAN-981 251、AV PLAN-981 74 通过，AgePackage 语法标识为 `74629b0eda27ec57619d61ef26c55bc5c57630b23b6b57745935e36c033af9c8`。

AV 定向实例栈计算结果为 `0 → 8`。生产 Vue CLI 冷生成两个路由，经固定依赖的 `vue-tsc` 和 Vite 构建。Chrome `157.0.8103.80` 核栈结果同为 `0 → 8`，另核带显式类型的父处理器收到 `detail/图像`。两个路由分别卸载且无页面异常。默认生成路由采用 hash history，以实际路由地址复验通过。浏览器日志 SHA256 为 `9f21e81118280e37f9617408867f595d9dd5d4872c62488291c78687e62c3907`。

固定原 FlowDiagram 页和实际组件各解析出一个 widget。原闭包推进到 `nodes: List` 的异形记录绑定。原节点有缺省 `shape` 的行，边记录的 `line`、`label`、`head`、`tail` 也有缺省值语义。原闭包日志 SHA256 为 `61300d5cabbd938f66d7c96db42d7137f1a75a1aafbcd7dcc8c336430aa556b8`，真实双布局及悬停仍待回放。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 42 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 46b78613, lang 4a808bbac | evidence: original FlowDiagram component parses, checked stack effects replay in AV and Chrome | next: work`

### 第 43 轮｜受检浮点插值更新两端 SVG 几何

AGE 包面浮点插值经受检 `TextDisplay` 进入 AV 和 A2TS。AV 使用 f32 十进制表示，JS 选择能还原同一 f32 的最短十进制候选并展开为定点文本。差分比较覆盖 1,008,491 个有限值，含二进制指数边界和次正规数，输出差异为 0。差分日志 SHA256 为 `04ee2ad069c755416b84e453c93090f11786c97ca9629098d248a4c3b29c4d9e`。AV MIR 验证操作数和结果槽类型。A2C、A2R 与 native 编译器对该新增浮点格式给出具名拒绝。

首次 Chrome 生成验证定位到受检子组件渲染器缺 SVG `path`。接入后，Iced 和 Vue 从同一个受检树消费共享的 SVG 标签与属性规则。Iced 测试解析实际 `svgdoc`，点击后 `d` 从 `M 0 0 L 1.2 4` 更新到 `M 0 0 L -0 4`，并检查标签中的 XML 转义。新目录生产 Vue CLI 冷生成，经 `vue-tsc` 与 Vite 构建，Chrome `157.0.8103.80` 观测到相同的 DOM `d` 更新，卸载无异常。浏览器最终日志 SHA256 为 `f98ac9c3446a3c56839072bdce1a981fa8476533735017df7f14e15060f30962`。Core 完整 conformance 867、AV PLAN-981 75、A2TS Node 4、组件包 14、路由 4、SFC 20 通过。

原 `024-charts` 闭包的 f32 插值诊断由 58 降至 0，四个原画廊图表页的同类诊断也均清除。原应用日志 SHA256 为 `47a55142b80fc5e055bea1c27219652a6a865817e7a094ad0ece6fa2649a47ff`。动态记录键仍是原图表闭包首项诊断，真实原图表交互仍待验收。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 43 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 52f003a3, lang d34746351 | evidence: exact chart f32 diagnostics cleared, numeric differential and real SVG state updates in Iced and production Chrome | next: work`

### 第 44 轮｜按调用端声明的选择键读取图表记录

必填裸 `List` 参数绑定后，从全部可见调用点收集已知标量键和键列表。受检处理器按作者的列表顺序选择对应记录字段。实际组件参数和循环仍参与运行时求值。参数值不明、列表可被修改或逃逸、局部别名被重写时保留原诊断，未知字段由共享类型检查拒绝。

Iced 受检组件帧验证组合结果 837、参数列表的顺序，以及未知字段和别名重写负例。生产 Vue CLI 冷生成，经 `vue-tsc` 和 Vite 构建。Chrome `157.0.8103.80` 实测替换父数据后结果为 `837 → 525 → 837`，卸载完成且页面异常为 0。日志 `round44-chart-keys-browser.log` SHA256 为 `30720b702ca38ec353805d0e5f9f0f01656c6047c046c1b50c6ea93397157dfd`。Core PLAN-981 251、AV PLAN-981 75、组件包 15、路由 4、SFC 20 通过。

原 `024-charts` 和原 Area 闭包首项推进到混合宽度的数值运算。原 Donut 推进到局部裸列表推导。原 Bar 的两个实例使用不同记录形状，首报缺 `cost`，原 Line 对应缺 `desktop`。五个源入口保持原字节，原 `024-charts` 闭包日志 SHA256 为 `4efb1dfbbe98fa9a1fc5ba07fd17a0cbd78712fcaa012428c578ffe3d602e572`。实测的两端选择键回放使用本轮定向材料，四个原页的挂载和交互仍待验收。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 44 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui ce24d8e0, lang eaaee4973 | evidence: call-site chart key domains, named mutation controls, Iced frame and production Chrome selection replay | next: work`

### 第 45 轮｜图表坐标在共享验证链中完成数值转换

原始 widget 布局的整数偏移量在浮点目的地及混合算术、比较中转为受检 f32。只有实际生成转换的标量包才切换到已有数值能力的集合验证器。AV 两实例结果为 `x=0`、`x=6.5`，普通 node 和模块函数仍具名拒绝混合宽度。Core PLAN-981 251、AV PLAN-981 76 通过。

生产 CLI 的独立页面在旧 SFC 表达式边界给出 `source.age.sfc-expr`。随后按原图表的父子闭包组织材料并声明实际依赖包身份，冷生成经 `vue-tsc` 和 Vite 构建。Chrome `157.0.8103.80` 实测 `x=0 → x=6.5 → x=0`，卸载无异常。最终日志 SHA256 为 `4ada3aec0613a7859979e944ed8545a63a6ee15fc97bab01fd3086ebca45c2bc`。

原 `024-charts` 混合宽度诊断由 18 降到 0，原 Area 由 6 降到 0。两处首项均推进到原样文本列表的借用值读取。原应用闭包日志 SHA256 为 `04c326122033ec8941e7da2e62d9b381865125c2472b5db6134cc814b38d6fb4`，原 Area 为 `6bc69f70621c8c9e95d1c97b07d78f934d50a1ba660c785b2ddea45ffbb78768`。原页挂载仍待验收。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 45 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui ce24d8e0, lang 06fa116ad | evidence: legacy chart numeric casts, AV execution and production Vue replay, exact mixed-width diagnostics cleared | next: work`

### 第 46 轮｜从实际记录列推导图表临时列表

组件从调用端取得名义列表类型后，重投影阶段按真实字段声明推导 handler 中追加的值。记录字段选择须满足分支类型一致，并由共享绑定及验证链复核。受检 AV 帧按作者的字段顺序输出 `13/7.5`，同材料生成受检 Vue 组件。Core PLAN-981 251、AV PLAN-981 76、组件包 16、路由 4、SFC 20 通过。

默认运行时构建发现合并成分的 ACE 包发现模块缺少 feature 门。恢复相邻 ACE 模块使用的 `ace-bridge` 条件后，默认 `cargo check` 与启用 ACE 的组件测试均通过。

原 `024-charts`、原 Area 与原 Donut 的空列表诊断分别由 7、2、1 降到 0。闭包日志 SHA256 依次为 `de19450bfa67076979910bc985d465666f2c3860f39e8ff325aa76cbb1e2431b`、`87aee3c422c439618bbacdc910ea1df52172fdb7fa486f6823ddae4080c0c8e5`、`4f75a1ccfc5cd25e5c9143a1aecc5a8649b59735399c1004ed482e4b02491276`。定向 Vue 材料已生成，浏览器和原始页面交互仍待复验。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 46 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui e31b1e04, lang 5324a3368 | evidence: typed builders for caller records, same AV frame and generated Vue, exact empty-list diagnostics cleared | next: work`

### 第 47 轮｜旧图表文本读取按值保存

旧 widget 的局部绑定和赋值从列表视图复制文本。推导所得文本列表复用旧文本追加的受检复制语义，已拥有的插值结果可以转移。原始列表保持可读。AV 两实例分别显示 `none` 和 `after/x=after/before`，普通 node 中保存整个借用元素的负例仍具名拒绝。Core PLAN-981 251、AV PLAN-981 77 通过。

同材料作为受检子组件经生产 CLI 冷生成，`vue-tsc` 与 Vite 通过。Chrome `157.0.8103.80` 两次触发后都读取到 `after/x=after/before`，卸载无异常。浏览器日志 SHA256 为 `5f339c4a6d707913fcbe67fdbe51176530f12a02f0b3a9309452e6602c7d36bb`。

原 `024-charts`、原 Area、原 Donut 的 `source.view-escape` 和 `source.move-required` 分别由 `88/11`、`27/3`、`6/2` 降到 `0/0`。原 024 先报 `to_int`，原 Area 先报 `memo` 的列表属性表达式，原 Donut 推进到生成位置的 UTF-8 边界校验。三个闭包日志 SHA256 依次为 `c2f66bdfb4d93dadc05b38b16a54dc27a94066274e5c7a0fe154419f056e5784`、`7bbff2722966f5bcfb464f1a98712e97caf905e08c4fad5ecd3fd5ab66c14f2a`、`a0130df84b668d57a71c3b12bbfe02df4b684c46518fb5f22ecc850d5d8c52ee`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 47 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui e31b1e04, lang a0df95710 | evidence: checked text snapshots in AV and production Chrome, exact chart ownership diagnostics cleared | next: work`

### 第 48 轮｜原 DonutChart 闭包编译与原外壳挂载诊断

生成的列表签名采用组件自身参数的源码位置，继续使用调用端记录的名义身份。多字节注释负控由越过 UTF-8 边界的内部错误推进到编译通过，受检组件帧仍输出 `13/7.5`。Core PLAN-981 251、AV PLAN-981 77 和组件包定向回归通过。原 Donut 页和组件编译为 2 个受检组件，原闭包日志 SHA256 为 `95419ec7262dc3347944eac967c6aabffa4e7352a25fd79d887db5c88f9c6cff`。

保留原画廊 manifest、外壳、索引、Donut、Carousel 和声明依赖的源字节。隔离工程复用固定 AutoDown 源和产物，校验同一本地依赖的新路径后沿用此前锁定的依赖版本。生产 CLI 冷生成 `donut-chart.vue`，`vue-tsc` 和 Vite 构建通过。Chrome 由实际侧栏进入 `/donut-chart`，初始化遇到 `Str` 重复释放，未显示圆环。未压缩对照定位到 `Init` 的字段文本 `lv` 与局部 `name` 共用同一所有权。浏览器失败日志 SHA256 为 `991dc4c39c3e90f997e77cc4d201a0f87b605c5f4667a7b24a63077c9ef031b9`。

Iced 真窗经原侧栏进入相同页。AV 的视图闭包在同一 `lv` 上报 `mir.use-after-move`，尚未建立页面实例。窗口记录 SHA256 为 `cf8ee1107396254492e21c9ceef2585ad23a8fec0a1a37138e17ca0f26699eb7`。标签快照、图形、指针、图例和卸载继续沿原输入复验。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 48 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 93b037e0, lang 5aa02dc94 | evidence: original Donut compiles, cold production route and real Iced navigation reveal matching ownership error | next: work`

### 第 49 轮｜原 024 图表闭包准入，原 Donut 在 Iced 显示几何

旧 widget 的 `float.to_int()` 复用受检转换，保持原 VM 的截断、范围饱和和 NaN 转零。旧 handler 的可变 `str` 局部持有文本，读取其他文本局部时复制，显式转移仍沿用验证。AV 核对截断、上下限和 NaN，标签在源值被重新赋值后仍保留两个独立值。Core PLAN-981 251、AV PLAN-981 78 通过。原 `024-charts` 入口及原四张图编译为 5 个受检组件，闭包日志 SHA256 为 `1eacebc84800e1b8a0f3351d1026b3e259ce582116523805e1c212c4afda15be`。

原 Donut 在真 Iced 窗口经侧栏打开，4 个 SVG 扇区可读，点击原 Desktop 图例后剩 3 个。关闭后外壳和页面会话都已关闭，页面计入 1 次关闭后回调丢弃。窗口记录 SHA256 为 `9157ae4c7344fdfdfa8cf113e302ac0a0ca13bf36072a3eb608e62c45aaa46ec`。生产 Vue 重跑已越过重复释放，首次绘制推进到 `PointerMove` 缺宿主浮点坐标，日志 SHA256 为 `9548b7c70c6b382311a782d14ae997c6838091f2ac8f65e4ea2c78e49b63e408`。

原 024 的真实 Iced 入口缺 `stylekit.styles` 项目依赖。生产 CLI 的非路由入口仍选用单文件旧产品检查而拒绝原处理器。两次入口日志 SHA256 分别为 `0c1bbc1547436e6c1f682f5feb35355666930f033f53e531ec98fb54db673d8e`、`73514cad08ed28a2ea064872cc93c36a5230dbe91b4130c8a2bcd9e65b0c451e`。后续接通同一受检闭包和真实项目依赖。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 49 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 93b037e0, lang e172e837c | evidence: exact 024 closure verifies, original Donut AV geometry, legend click and teardown, named production entry gaps | next: work`

### 第 50 轮｜原 Donut 在两端执行指针、图例与卸载

受检组件的鼠标移动将宿主坐标换算为作者声明的逻辑幅面，按两个有限 f32 交给当前页面、实例与视图代次的原消息。浏览器沿用已有坐标采样和限频。Iced 测试显示 `1.5/2.25`，拒绝旧视图的离开回调，再从当前视图离开。组件包 17、路由 4、SFC 20 通过。SFC 原断言同步检查抽取后的采样返回值和转发调用。

原画廊 Iced 真窗实测 Desktop 扇区命中、高亮几何和提示文本，移出后复原，图例开关使扇区数 `4 → 3 → 4`。页面共收到 4 次动作，两个会话均关闭，页面拒绝 1 次关闭后指针回调。Chrome `157.0.8103.80` 经生产 CLI 冷生成和构建，同一页实测命中、移入内孔后的清除、图例悬停和开关，进入 Carousel 后重进原页，扇区数为 `4 → 3 → 4 → 4`，卸载无异常。几何起点以 f32 容差核对，原弧 y 为 `50.000008`。Iced JSON 和 Chrome 日志 SHA256 分别为 `0a41ab5ad486220674e2bc78decab75e5c6c14a8dadc540015e76ea84025b6cf`、`84b4c74b9bef660d482fce04a6b3eeb0ddf2029b2112822a3709492a520bac0e`。

原页的实际数据通过上述动作，空数据负控仍待执行。原画廊首页仍保留 `index.at` 的独立语法诊断。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 50 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 42908603, lang e172e837c | evidence: original Donut chart Iced and production Chrome pointer, legend, route lifecycle | next: work`

### 第 51 轮｜项目声明依赖接到两端，Donut 空数据通过

无路由的组件包应用接到受检包入口。原 `024-charts` 的样式模块按项目 manifest 和已用导入解析，两端均越过缺 `stylekit.styles` 或旧产品处理器缺席的入口诊断。Vue 冷生成推进到 A2TS 的受检函数闭包，失败日志 SHA256 为 `745a285b6cb2f70c9c6fd53d7e185c95402c5406f7ef90d344e1ad7ae756f35b`。Iced 进入原 LineChart，所有权验证报告其状态函数需要约 61 MB，超过既有 16 MiB 上限。窗口日志 SHA256 为 `6844217570723b2733a041eb6d878f2a43a634b48447ef7615a831afa80ad906`。定向依赖材料已修正成与原项目一致的多行 manifest 声明，继续复验。

空数据对照沿实际产品入口挂载固定原 `donut_chart.at`，由单独调用页提供类型一致的空记录列表。Chrome 与 Iced 都观察到 1 个底环、0 个扇区，逻辑指针输入后保持该状态，没有图例。Chrome 卸载无异常，Iced 关闭后拒绝 1 次迟到指针回调。Chrome 日志和 Iced JSON SHA256 分别为 `2e0f4048fd56fdf79f78b78c1a3d6f3a13e1241047bdf7f07f1808372743777d`、`6b45d97398c0cc58279293afe82d89d645a47fe2f292f13a7d5e271b6e664e02`。与第 49、50 轮固定原页回放合看，覆盖 Donut 的数据、指针、图例、空数据及会话关闭。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 51 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 5528a2bd, lang e172e837c | evidence: original Donut empty-input control on two hosts, exact 024 linked entry reaches named downstream checks | next: work`

### 第 52 轮｜原 024 图表在生产 Chrome 完成四类切换

受检 Vue 出盘从组件初始化、视图、事件和属性取值的实际函数开始，沿调用、闭包与回调引用遍历完整依赖。共享检查仍覆盖全包。旧包样式字面量在检查生成函数之前通过原导入绑定进入函数体，同时保留局部遮蔽规则。AutoLang Core 完整 conformance 867、AV PLAN-981 78、AutoUI 组件包 18、路由 4、SFC 20 通过。

固定原 `024-charts` 项目从新目录经生产 CLI 冷生成，锁定依赖后通过 `vue-tsc` 和 Vite。Chrome `157.0.8103.80` 通过原按钮依次切换 Line、Bar、Area、Donut 和 Line，绘图区路径数为 `4 → 19 → 7 → 4 → 4`。分组柱以 SVG 路径绘制，Desktop 对应 6 根柱。折线图 Desktop 图例开关使路径数 `4 → 3 → 4`。窗口选为 8，播放、暂停、重置按钮可操作，等待 1.3 秒后的几何未变化，流式更新尚未通过。页面请求失败及 JS 异常均为 0，卸载后根节点清空。独立回放日志 SHA256 为 `b78220cd76f5ab96d35322f69d752e51552c8134a44b7572ade87c85ce3a77aa`。

Iced 仍在第 51 轮的所有权分析内存门。原折线图初始化有 977 槽、123 个借用令牌槽和 214 块，视图函数有 2,590 槽、374 个令牌槽和 139 块，其令牌均无返回借用令牌的调用。用实际来源压缩分析表后继续复跑同一原入口。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 52 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 09ed6d02, lang 51d8325d8 | evidence: original 024 production Chrome renders all four charts, legend and reset, stream unchanged, native memory analysis measured | next: work`

### 第 53 轮｜原 024 Iced 越过所有权分析内存门

AV 只为携带令牌来源的位置和实际执行字段转移的根分配分析表。单根来源内联保存。静态来源预分析覆盖直接借用、来源别名、带借用参数的调用及分支汇合，为可能增长到多根的集合计入完整容量。活借用和保留型 Place 的汇合及逃逸规则保持。内存上限仍为 16 MiB。AV Core 库测试 71、conformance 51 通过，日志 SHA256 为 `3a21d2e7b0cff26eb624deb2870bdefb523ec9a76e4c0aa46f80874178c5ef75`。

同一原 `024-charts` Iced 入口越过 LineChart 初始化与视图的内存门。挂窗下一项检查定位到原 `ptsOut` 列表在相邻 `k == 0/1/2/3` 分支中的转移。窗口诊断为 `mir.use-after-move`，日志 SHA256 为 `4e4396b95e702f47df90611e05400910de2aa623dfe97fe77fcc05279c53eee3`。本轮未获得原 024 的 Iced 几何。组件包 18、路由 4、SFC 20 串行通过。先前直接运行组件包时缺 provider 环境，再以并发运行触及共享 provider 状态，按准备参数和单线程重跑通过。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 53 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 09ed6d02, lang 60aaede97 | evidence: original Iced chart passes frozen verifier memory gate and reaches authored branch ownership diagnostic | next: work`

### 第 54 轮｜原 AreaChart 在生产 Chrome 回放

将旧 widget 中连续的局部标量等值分支在证明互斥时降为共享分支链。要求各比较常量不同、可精确表示，分支体只从简单值写入 model，因而不会更改被比较的局部。AV 核对原形态的列表转移两条路径，分别显示 `0/2`、`2/0`，比较常量重叠的负例仍拒绝。旧 `memo` 只接受文字依赖提示并透传受检子树，未知提示具名拒绝。AV PLAN-981 79 通过，日志 SHA256 为 `46363ee16f65e3a9b2b9614fe06a427a5b8278f9fd8c8672e333ae4d3fdf9f93`。

原画廊 manifest、外壳、AreaChart 页与组件逐字核对。生产 CLI 冷生成路由，经固定依赖的 `vue-tsc` 与 Vite 构建。Chrome `157.0.8103.80` 实测两层填充及对应描边、首个命中带显示 `desktop 186`、指针离开后清除、图例开关和离页重进。绘图区路径数 `5 → 3 → 5 → 5`，卸载无异常。源散列清单和浏览器日志 SHA256 分别为 `61f0e942f1a6cb80eedf98827867d90a75bd2c74ed045034cb775f552f74047d`、`3cd2e9a4e1e2efc7d42e015ed54e1a2556acd774a9238fbb18de2df825656dfb`。空数据和数据变更仍待验。

Iced 原 `024-charts` 的首报从 `ptsOut` 推进到只读属性列表 `labels` 的重复转移，日志 SHA256 为 `b19549c0a3f842c29982bf2c541703b1a7247b40fa74f5bc6417b73df437cbb6`。原 AreaChart 的 Iced 路由进入 `/area-chart`，首次几何断言未找到 SVG，正在提取实际视图诊断。初次窗口日志 SHA256 为 `7b0da4b0e646d72a0bd700f3faacb5615c78dbabd0fffedfa2b2f5f68d1359a3`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 54 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 09ed6d02, lang efb0b4287 | evidence: original AreaChart geometry, hover, legend and route lifecycle in Chrome; AV branches verified; named 024 list input diagnosis | next: work`

### 第 55 轮｜两端原 AreaChart 正例与 Chrome 原图表滑窗

旧组件绑定列表属性作局部值时，通过共享受检列表操作逐项建立快照。定向 AV 测试修改第一份列表后，第一份、第二份与原属性长度为 `3/2/2`。显式计时器和原 widget `interval`/ `.Tick` 的启停条件进入受检视图，宿主按挂载实例派发并在停止或卸载时清除。Vue 按属性实际值变化在既有模型上重跑原 Init。Core 完整 conformance 867、AV PLAN-981 80、AutoUI 生产工具构建通过。

固定原 `024-charts` 冷生成、构建后，Chrome `157.0.8103.80` 实测初始折线 6 点，选择 8 点滑窗后出现 `t3`，折线达到 8 点。暂停后再等 500 ms 几何保持一致，重置恢复原 6 点几何。重开播放后卸载，页面根清空，页面异常和请求失败均为 0。日志 SHA256 为 `36b5801c9f08690310bd0745c6bb8a55b188b7722fea9fb9334379e22c195341`。Iced 越过 `labels` 读属性门，原 BarChart 的并列分支构造记录时报告 `rcol` 所有权诊断，日志 SHA256 为 `c243d1d62ba927ccbd2629e20a4eee4185c0d4872c71b15a66fe4fa9b6e96e3e`。原 024 的 Iced 流式回放仍待验。

原画廊 Iced 真窗经原侧栏进入 AreaChart，显示两层填充与对应顶边，用挂载视图回调命中首带显示 `desktop 186`，离开后清除，再用原图例使路径数 `5 → 3 → 5`。关闭后迟到回调拒绝 1 次。窗口 JSON SHA256 为 `ec67e206d3623afb4849be01d8ddd1e5907d5af4257729786726066edea7ba9c`。同源 Chrome 冷复验路径数 `5 → 3 → 5 → 5`，指针与卸载通过，日志 SHA256 为 `3cd2e9a4e1e2efc7d42e015ed54e1a2556acd774a9238fbb18de2df825656dfb`。空数据和更换数据仍待验。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 55 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui bc4e854a, lang f8e1cf511 | evidence: exact AreaChart on both hosts; original 024 Chrome streaming, pause and reset; named BarChart ownership gate on native | next: work`

### 第 56 轮｜原 BarChart 分支推进及 AreaChart 空数据实测

旧 widget 的互斥分支证明覆盖向其他局部列表追加由简单值构成的记录。选择键在分支内保持不变，未能证明互斥的记录转移仍交给所有权检查。AV PLAN-981 80 通过。原 `024-charts` 在 Iced 越过 BarChart 的 `rcol` 所有权诊断，根件标量投影 `theme_name` 推进到 `VerifySteps`，尚未挂窗。窗口诊断日志 SHA256 为 `9dae9d8d510b0087d7a322e52c7a4cf53ac719a1aa80ec09b1d448d6adba9aec`。继续核对标量投影的实际调用闭包。

独立调用页保留原 `area_chart.at` 的 SHA256 `44bcd6700457eed38cfd47be6f8bbb63e7615ed20ec4b3c8296e38eb7f08643a`，初始为空，通过原组件属性提供三条记录再清除。首次对照的列表赋值未类型化，在两端均具名拒绝；改用显式类型的列表局部后成功进入原组件。Iced 对空输入在原 `slot` 除数上报告 `DivByZero`，旧 VM 的浮点除法同样对零拒绝。Chrome 在空态和清除态生成两条无起点的填充路径，可见长度均为 0，浏览器记录 4 条 SVG 路径错误。填入后两条可见路径长度约为 1286.85、1154.45，卸载完成。Iced 诊断和 Chrome 观测日志 SHA256 分别为 `65021a6f7e9b9ecf67d226aa3d370cfdcc08c4d5428f25dab8fcbf5011c4db52`、`260e53e974d90458126dd6718277ec92160b2f26410e1f6beb6ace78a9097287`。空数据在两端表现未统一。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 56 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui bc4e854a, lang ffdee7676 | evidence: original BarChart ownership gate cleared, native snapshot verifier work gate and empty Area behavior recorded | next: work`

### 第 57 轮｜原 024 Iced 真窗与 Chrome 指针取整诊断

受检组件包的标量观察沿视图已有的 MIR 调用闭包切片执行。原 Iced 入口越过全包 `VerifySteps`，在真窗显示 Line、Bar、Area，路径为 `4 → 19 → 7`。120 秒观察窗口截止时已有 4 帧，关闭脚本未完成，Donut 的最后一条路径记录被截断。本轮保留原应用的部分证据，窗口日志和部分 JSON SHA256 分别为 `8df94ea0e76bfd00ebdcf90e621286e77de7567d5a6d6d565239283c35130759`、`3a6456e25e27b82d71d86d7ec693966651c64865fbb70f29efaa65aabe25d6f9`。继续减少标量观察的重复降层。

Chrome 指针已实际命中原折线图覆盖层。原 `.PointerMove` 把 `0.6463046669960022` 的浮点值转换为整数，脚本目标先检查整数而抛错。A2TS 当前在整数范围检查前截断，并把负零规范为整数零。真实 Node 测试覆盖 `-3.75 → -3`、`-0.125 → 0` 和原 widget 的饱和上限，A2TS 5 项通过。初次浏览器失败日志和 Node 测试日志 SHA256 分别为 `a1fdecd51be6ea23bf45fa7737804579dec1507cbdc2b644961ef3852b063c0c`、`799901d8780ca09681ef9efb20fddd0e7ebf1ddb3bf36cc3c77965e5f5b5714b`。生产 CLI 冷重建及原浏览器指针复验待执行。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 57 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui bc4e854a, lang e428ecc05 (cast change), AV closure projection under replay | evidence: three original Iced chart shapes, deadline receipt, checked JavaScript float cast regression | next: work`

### 第 58 轮｜原 024 四图真窗切换及 Chrome 原折线命中

受检包的标量投影每帧复用一次降层，再按目标函数切片。发现新组件、还需运行 Init 的临时帧不读取标量状态。AV PLAN-981 80、A2TS 真实 Node 5、AutoUI 生产工具构建通过。

原 `024-charts` Iced 真窗依次观察 Line、Bar、Area、Donut，路径数 `4 → 19 → 7 → 4`。原按钮派发 `SetBar`、`SetArea`、`SetDonut`，发布 6 帧并执行关闭脚本，错误数组为空。延长观测期限至 240 秒，AV 内存和验证步数上限均维持冻结值。窗口 JSON SHA256 为 `1249e0b1d7981250386b1eb33968f450017ef6d0be8393360ee57fe3a156c0fb`。Iced 滑窗、指针及空数据仍待逐项核对。

固定原项目生产 CLI 冷生成、固定依赖构建。Chrome `157.0.8103.80` 实际命中原折线覆盖层并显示 `Desktop 186`，离开后提示清除。原图例与四图切换继续通过，滑窗点数 `6 → 8 → 8`，重置及播放中卸载通过，无请求失败和页面异常。第一次额外淡入断言超时，随后的观测记录到 `opacity-0 → opacity-25` 后提示消失，已读取的计算样式不透明度均为 1，未计入实际淡入通过。浏览器日志 SHA256 为 `372a85e1ef49201709135c8c327dd58fe07b5980484fd49d6f5bbd4d66f9b4ca`。

原画廊七页再冷生成。Carousel、DonutChart、AreaChart 生成实际页面。LineChart 与 BarChart 分别在多调用点的列表记录形状检查报告缺 `desktop/mobile`、`cost/sales`；FlowDiagram 仍拒未确定的 `nodes: List`；CodeEditor 仍缺 `code_editor_cursor_line`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 58 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui df6a1b59, lang 8e3e91c6f | evidence: original 024 four-chart and close in Iced, original Chrome line pointer + streaming, exact remaining gallery diagnostics | next: work`

### r1 移交前的历史下游记录

- **B-107-01｜共享生产基线。** 019/020/030 的原 Store、页面模块与前端绑定依赖 PLAN-105 的共有实现；029/031/031-paint 的 `$event`、图表和原画廊的类型列表与包闭包依赖 PLAN-114 的受检实现；原桌面树形挂载需要 PLAN-123 的生产入口与路由源发现。对应在途提交分别为 AutoLang `b987e2c23`、`7d6eabe4c` 与 AutoUI `ffbb488c`/`9d5ea5b3`，它们未形成 107 所需的稳定发布基面。恢复条件是相应 owner 提供可消费并且包含原形态正反例的固定提交，随后在 107 两树接入并重放 §5 矩阵。030 的第一个条件读取 `.store.has_current`，需要共享受检状态与真实控件树共同实现。
- **B-107-02｜原 CodeEditor 受检宿主能力。** 第 30 轮重跑固定原页，`QueryChanged` 已通过旧式单位消息的受检接线，当前首报缺 `code_editor_cursor_line`。继续按 §9 固定 AutoOS 源，由 107 接入编辑器光标的 AV/A2TS 能力并核编辑、搜索与卸载。

### 第 59 轮｜受检同组件消息调用

接入 PLAN-114 `633849248`，合并时保留本组循环、列表读写及旧图表分支。参数先在调用方求值，再进入目标 handler 的独立局部作用域，沿原对象模型提交。已声明载荷核对、未知目标和调用循环均由共享验证具名拒绝。AgePackage 语法标识为 `3adef60bc30e0092525585df6be020361209b815d3a2f04f8aa64c27794660b8`，相邻冻结通过，Core PLAN-981 251、AV PLAN-981 81 通过。

定向媒体形状在 AV 的第二实例显示 `image:图像/图像`，第一实例仍为初值。生产 CLI 冷生成、固定依赖构建后，Chrome 两次点击均显示 `image-details 图像/图像`，卸载无页面异常。首次定位器把直接文本节点和按钮文字当成一个节点，改按应用可见文本结尾断言后通过。AV/Core 和 Chrome 日志 SHA256 分别为 `df015c2bf365f91bfc5ee689016f3a658c70082cd46316869d917f9ea380019a`、`a5fa3a89334d2dc20092c107f80465922f01d26e329270d03276a32ce828b108`。

原 029、031 图片查看、031 绘制及所声明依赖在隔离目录核对 73 个文件。三份原应用整体仍在 AGE 语法及生产入口检查失败。源清单 SHA256 为 `9e8ac89ff7a4c8fff5e87b19df470badf8cd7118c9911da8896d266a64d23fee`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 59 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 173993a7, lang 4fb2a2714 | evidence: same-widget typed messages in AV and production Chrome; exact media inputs still rejected at project admission | next: work`

### 第 60 轮｜原 024 Iced 计时数据及原媒体语法定位

原 `024-charts` 真窗按实际计时器完成 `Play → Tick → Pause → Reset`。Desktop 折线点数 `6 → 7`，暂停 500 ms 保持几何，重置逐字恢复原 6 点 SVG。观察使用实际墙钟等待计时器，早先按固定界面拍数关闭的脚本只记录了 Play。完成回放发布 889 帧，错误 0，会话关闭。窗口 JSON SHA256 为 `abc68721bf4f951c4703ca3c4133ca8f2f49407034e0043d03ad5f440a5ce0f3`。Chrome 复验从提示层下方命中，计时拟合状态到达 `opacity-100`，计算样式仍为 1，未计入视觉淡入。

对固定原媒体入口逐成员和处理器缩减探针。原 031 图片查看的 `ApplyView` 已越过语法门，仍定位到 `SelectImage` 分支中的 `request_view(...)` 及 `SettleTick` 的 `for j < names_list.len()`。原 029 另涉及 `try/catch` 和未类型化的宿主 JSON；原 031 绘制另有 model 声明及 `.substr(...).to(int)`。探针日志 SHA256 为 `2b87b9a72f5783374f6e4d2b3c14915a668e9dc1c903b1637e5fa190809a7ab1`。

`stage: goal | plan_id: PLAN-107 | plan_revision: r1 | round: 60 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 173993a7, lang 4fb2a2714 | evidence: exact 024 Iced stream, pause, reset and close; media first syntax shapes isolated | next: work`

### 第 61–63 轮｜图表调用点隔离及原 024 Iced 指针

首次命中原折线图的 Iced 指针回调，模型字段的重复重放达到 AV `VerifySteps` 上限，状态提交 0 次。现由每次 widget view 调用执行一次受检模型重放并读取同一份记录。仍使用冻结的内存和验证步数。第二次真窗回放实际显示 Jan 提示及十字准线，离开后清除，图例路径 `4 → 3 → 4`。从挂载的 Iced View 取得回调并送入真实窗口消息队列。会话关闭，304 帧、4 次消息轮询及成功提交、1 次关闭后的输入丢弃、0 错误。窗口 JSON SHA256 为 `8ab2a72757035a1c859d665a0268af6a34e0ae029a5d2a53d2d49654b8c82d05`。

AutoLang `ef7faf610ebad390a9e687ca1fbc6d70c3938b7a` 将静态 widget 实例绑定到各自调用点的原记录类型和键集合。AV 回归验证两个记录签名的初值和独立状态，另一个实例的键不能用于当前实例。原画廊镜像核对 18 个输入文件，生产工具冷生成 Bar、Line 的实际受检页面。带 CodeEditor 诊断页的整包从固定依赖构建时，诊断中的原生函数名被误识别为可执行调用，`vue-tsc` 因而报告缺编辑器桥。Bar 和 Line 的 Chrome 行为在本轮尚未验收。FlowDiagram 的稀疏匿名记录仍待处理。

`stage: goal | plan_id: PLAN-107 | plan_revision: r2 | round: 63 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui 173993a7, lang 9d17e7b4f | evidence: Core PLAN-981 251 and AV PLAN-981 82 pass, original 024 Iced pointer and legend and late event rejection, real Bar/Line generation with gallery build diagnostic | next: work`

### 第 64–65 轮｜Bar、Line 原页双端正向回放

AutoUI `a002e7fc385648f8dddf416b3da06788eca31a54` 让生成器跳过诊断组件中的文本依赖。专项回归通过。18 个画廊原输入文件逐一核对哈希，正式工具冷生成并以锁定依赖通过 `vue-tsc`、Vite 和 Chrome 回放。Bar 的单组/分组为 5/8 根柱，分组指针显示 `sales 155`，非目标 6 根柱降透明，离开恢复，图例 `8 → 4 → 8`。Line 单/多组为 1/2 条折线、每线 5/6 点，多组指针显示 `desktop 186` 和双准线，离开恢复，图例 `2 → 1 → 2`。两页的目标图操作不改变另一张图的几何。路由离开、重开和卸载通过，浏览器异常 0。浏览器日志 SHA256 为 `454b9fde7fdf69c0f7a4577eaa7cb11c4b6d980afab5827ba3334c229d0d0249`。

Iced 从同一原外壳经两个侧栏链接进入对应路由。实测图形、提示、离开、图例计数变化与另一实例不变，200 帧，目标页各 4 次事件提交。关闭后拒绝 1 次迟到指针输入，桥错误 0。首轮观察器读取了外壳的事件日志，错过子页面已成功的 Hover；修改观察器读子页面事件日志后，两条路由完成回放。初始非验收路由 `/` 仍有具名语法诊断，两个目标路由均正常进入。窗口 JSON SHA256 为 `4fab15bb63c2327d259323aff456d089703691b34f90db33296e077340b57345`。

原 LineChart 组件 SHA256 `930e812e3cf52b435f2cec6a3cd5a43830ca2f29c6454b37e99bd9f0e2c47740`。额外的类型化数据夹具在真实 Chrome 执行空 → 三条记录 → 清空 → 再填，实际可绘制折线数 `0 → 2 → 0 → 2`，第二次填入几何逐字相同，卸载和错误计数通过。Chrome 日志 SHA256 为 `ad0d022ceafbb41af934b2bbdc3d6a8960ea8dadcbe1c9fe4e6655024b720c33`。Iced 数据变化控制和 Bar 空数据待复核。

`stage: goal | plan_id: PLAN-107 | plan_revision: r2 | round: 65 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang} | commit: ui a002e7fc, lang 9d17e7b4f plus uncommitted sparse FlowDiagram projection | evidence: original Bar/Line pages in Chrome and Iced, independent geometry and events, LineChart Chrome empty data and refill control | next: work`

### 第 66–70 轮｜空数据根因与 FlowDiagram 冻结版

原 LineChart 空、填入三条、清空、再填在 Iced 发布 304 帧，折线数 `0 → 2 → 0 → 2`，几何与浏览器逐字相同，关闭、错误 0。窗口 JSON SHA256 为 `70e6283db586b509028b06c261540ed933ebad9f5f75377cc0c412ffedca157a`。AutoOS 冻结版 Bar 在空列表的首帧 Iced 报 `DivByZero`，Chrome 如常完成 `0 → 6 → 0 → 6`。两个日志 SHA256 为 `551f31eb8a6243b7a3cab966925f0531ae68d67ca082a014f36a62b1cb845722`、`308a377bd5baa6086b7fefc73b593f6cced65594e08a9290246695239a551912`。

AutoUI `b58c7ebbd8613035c3f9eba210cade5d0927e27b` 在 024 自带组件防护零分母，Area 的空状态不生成错误闭合路径。带两种实际组件的类型化夹具，生产 Chrome 与 Iced 的可绘制路径同为 `(0,0) → (2,6) → (0,0) → (2,6)`，二次填充相同，均关闭且无异常。Chrome、Iced 收据 SHA256 分别为 `8b848d4e2009919795a3bb24dd905ed4576681440ff0bd300b1f9c6d67f60b56`、`9f968b21144d914f2b023a9a3da093efaa0c03530b0faf264174fa4c55e95618`。用原 024 入口复验 Chrome 四图、点击、指针、滑窗和卸载，异常 0。日志 SHA256 为 `3b55e70995388157210ad36d57655766bf9f7378457788b5fa01ae425e4dd663`。实际淡入仍未确认。

AutoLang `c9232953427843e0362b0e917e7ca8749ac9e001` 检查稀疏的旧式匿名行，保留具名记录的未知键诊断，并将被记录构造消费的行文本复制为独立值。AV 83、Core 251 项通过。冻结版 FlowDiagram 在 Iced 两卡显示 `672×200` 和 `336×416`，各有五节点、五边、五箭头。悬停第一卡显示 `id: start`，实际高亮 0、降透明 5，第二卡不变，离开复原且关闭后拒绝迟到输入。窗口 JSON SHA256 为 `dae99fdc9b1b8a49527759a8b8dbc986750f468a927c7c478df3bf0c5eb6ce0a`。Chrome 冷构建识别出该原页，执行 Init 在记录字段与局部变量清理阶段重复释放同一 `Str`，未挂载。带符号的浏览器日志 SHA256 为 `ccc6a6e483d86cc97390c91665ac16c2950fe95e2610a577e4918bda86ba6d55`。直接记录字段赋给旧式文本局部时复制，定向 A2TS 真 Node 回归已通过，整页待复验。

`stage: goal | plan_id: PLAN-107 | plan_revision: r2 | round: 70 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang}; D:/autostack/.wt/os-107/auto-os | commit: ui b58c7ebb, lang c92329534 | evidence: original line empty data, exact 024 empty-data repairs and Chrome project, original FlowDiagram Iced layout and hover diagnostic, A2TS record text lifetime failure isolated | next: authorized AutoOS source repairs and final replay`

### 第 71–72 轮｜经授权修复原图表、双端回放

AutoOS 源修 680a6ae344f89cbf4dcf5efd25a118889fe611c7。Area 和 Bar 跳过零分母，Area 空输入不生成错误闭合路径。FlowDiagram 从实时命中的节点 ID 派生高亮，在声明的图例坐标内放置不截取鼠标的提示。AutoUI 083bb61e 将 Vue 命中框和绝对定位偏移映射到同一逻辑尺寸。AutoLang 95d9d69e9 复制从记录字段读取并独立持有的文本局部。保留冻结源、源文件哈希及其失败原始记录。

| 原入口或负控 | AV/Iced | A2Vue/Chrome |
|---|---|---|
| 024 原项目 | 真实窗口 834 帧，播放 6→7、暂停与重置，折线指针、图例 4→3→4，四图切换 4→19→7→4→4，关闭后旧回调拒绝，错误 0。round72-024-project-iced.json SHA256 5ef291401ff5a3aa6d49267d68a2430cb931f93e2f2ad5bbb0b2ba8231d45633 | 生产构建，鼠标、播放 6→8、暂停及重置，并卸载。SHA256 3b55e70995388157210ad36d57655766bf9f7378457788b5fa01ae425e4dd663 |
| Area、Bar 空数据恢复 | 用修正版组件，空→填→清空→再填，面积及柱几何分别 (0,0)→(2,6)→(0,0)→(2,6)，301 帧，成功提交 3 次，关闭，错误 0。round71-gallery-data-controls-iced.json SHA256 b72acbae5eb06fc3fabac564dda0e45a5e6b32096845468ec1218c03939d076c | 同样两个组件，四个阶段的几何与 Iced 一致，卸载，错误 0。round71-gallery-data-controls-browser.log SHA256 8b848d4e2009919795a3bb24dd905ed4576681440ff0bd300b1f9c6d67f60b56 |
| Area、Bar、Donut、Line 原页 | 原画廊正向与各空数据测值见 §9 第 50–72 轮。Iced 使用实际挂载 View 的回调，并经窗口类型化消息队列送入 | 修正版画廊原页通过实际鼠标、数据、图例、重开及卸载。round72-gallery-area-browser.log SHA256 3cd2e9a4e1e2efc7d42e015ed54e1a2556acd774a9238fbb18de2df825656dfb，Donut 84b4c74b9bef660d482fce04a6b3eeb0ddf2029b2112822a3709492a520bac0e，Bar/Line 454b9fde7fdf69c0f7a4577eaa7cb11c4b6d980afab5827ba3334c229d0d0249 |
| FlowDiagram 原页 | LR、TD 各 5 节点、5 边、5 箭头。命中第一卡后 1 高亮、4 降透明、第二卡独立，离开复原。304 帧，迟到事件丢弃 1，错误 0。round71-fixed-flow-iced.json SHA256 638a8dc985b71f7e0b3654140f456f7f67ce5195e4a2b7186a9365ea9f8858c8 | 两种布局的坐标与 PLAN-502 独立期望一致，实际鼠标分别触发 1 高亮、4 降透明，提示锚点误差 ≤1.5 CSS px，离开、重开、卸载。round71-gallery-fixed-browser-final.log SHA256 6508bf451cb73366c992e9db8e1f1b9a54b922aff860e325d9e7ba65e42ab822 |

同一套修复后运行时重播冻结版 FlowDiagram，两个布局命中均为 0 高亮、5 降透明。原始日志 round71-gallery-frozen-browser-final.log SHA256 f798a689a6f84eeacffa0cc965bd3aa5fa58a5a5a9b0e86e1e94fb30c11e7a3c。范围内永久 024 空数据用例 p107_024_area_and_bar_accept_an_empty_input 直走 AV，用时 8.11 秒。原项目窗口及数据回填在集成回放验。

AutoOS 主检出子模块预检退出 1，日志列出未初始化的既有应用入口，27 个清单项，缺失 pac.at 0。AutoOS 源修没有 gitlink 改动，从固定原源及依赖生成的图表项目通过真实回放。审阅前须合入各自已落主的共享语义，再冻结最终版本验收。

`stage: goal | plan_id: PLAN-107 | plan_revision: r2 | round: 72 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang}; D:/autostack/.wt/os-107/auto-os | commit: ui 083bb61e, lang 95d9d69e9, os 680a6ae3 | evidence: six two-host entries, empty/refill exact components, case 8.11 s, AutoOS fixed and original comparison | next: integrated gate`

### 第 73–84 轮｜合流回归及交付范围

以 AutoLang 主基线 `5e5121dac6d6034a7eeb28cf4ed8d71686d21cd9` 和 AutoUI 主基线 `80cc1874ce411f019537cb16af6545ca3b852fbb` 整合。工作分支先核对合流后的 Git 树，再在各自基线创建同内容的线性提交。中间提交保留为本地 `codex/plan-107-r1-ui-handoff` 与 `codex/plan-107-r1-language-handoff`，便于后继计划读取。合入候选移除了 MPV、Carousel 及配套内联文本专用改动。原图表闭包所需的共享 SVG、包绑定、路由、所有权和事件改动仍由本计划验收。AutoLang r1 媒体工作保留在原 stash。

合流回归修复了四处行为。AGE 原列表构造保留内联及多行记录，语言仍检查局部可写性和消息实际类型。普通集合的 12,000 处保留引用仍以 `VerifyMemory` 拒绝。包视图的括号列表边界改成定位作者字段的具名诊断。AutoUI 保留来源准备入口，统一交受检 SFC 发射。独立对照发现核心文本记录组的五个用例在主基线及合流版均报相同 `verify.double-move`，AV Core 空记录布局断言也在主基线同样失败。全 AV 探测进入旧用例 `av_owned_r1_new_process_restore_executes` 的内嵌构建，已结束该轮并改用预构建的定向组，保留原失败日志。

AutoLang 定向 AV 131/131（1.23 秒），含 105 列表、981 包、普通集合资源负控和指令表。A2TS 两组各 5/5（0.51、0.57 秒）。较早合流 AutoUI 的 desktop 12/12、provider 27/27（0.95、4.52 秒）。U03 的 013 诊断已补精确命名，准备重跑 AutoUI。采用当前工作树 `e2b9fcf8`、`6265e5757`、`680a6ae3` 的生产工具重放六条入口，固定输入及依赖均已复制、逐文件比对 SHA256。

拟沉淀 Canonical Specs：AutoUI `docs/specs/runtime/ace-package-widgets.md` 增加 SVG、逻辑坐标与空数据行为。AutoLang `docs/specs/ace-migration/m2-autoui.md` 增加调用点 List/名义记录、受检事件、文本所有权及一次模型物化合同。AutoOS 源项目 README 提供所有者与计划索引入口。

`stage: goal | plan_id: PLAN-107 | plan_revision: r2 | round: 84 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang}; D:/autostack/.wt/os-107/auto-os | commit: ui e2b9fcf8, lang 6265e5757, os 680a6ae3 | evidence: 131 focused AV passes, main text-record/layout controls, typed package diagnostics, r2 scope isolated, production replay inputs ready | next: run frozen tools and independent review`

### 第 85–88 轮｜完整画廊复跑和边界回归

以 AutoOS 冻结提交和修正版各复制完整画廊。两份源清单均为 91 文件，只在获批的 AreaChart、BarChart、FlowDiagram 三个组件有差异。生产生成覆盖 70 个路由、90 个输出文件。首次完整包构建在原 Sidebar 的静态路由目的地报 `vue-tsc` 表达式错误，AutoUI `d4f1ed03` 发射静态 `to` 属性并固定两个路由断言。重新冷生成后 `vue-tsc`、Vite 均退出 0。Chrome 在完全相同的修正版画廊闭包实测五个请求页，Bar/Line、Area、Donut、Flow 的原日志重现第 71–72 轮哈希。冻结版同样完成生成和构建，原 Flow 路由给出 `source.type-mismatch: expected i32, found f32`，保留日志和此前冻结运行时两张图均 0 高亮的对照。

最终整合候选前一版本的 024 生产 Chrome 真实鼠标、流 6→8、暂停、重置、卸载，错误及异常请求均为 0。SHA256 `748863a75b068eca449fac728b31c5d567435ab0ada162b9017ba1607c52dd9c`。在完整原外壳中，Iced 024 661 帧、Area 65、Bar/Line 270、Donut 65、Flow 268。所有五份窗口证据的宿主错误数为 0。逐份 SHA256 为 `a5106f742cf8b6a61f8d7f4356731c6e7c7b515b533beddf5463d35a11682f8b`、`51fbe7051e6f5f065e959090da697b9fd0d0a96b33eb420c237e889104a66434`、`4d22493795f7b7eb45278ced8f35869d4821166f4d8aca3c53aebe3311c8de11`、`642050a5bc86672e143999647aea1ffdf7cc9379175a6e468f2aac785e237280`、`7abefa40708c37b8f018259bb8aa47e2f0f961b58ae6709a15e7ddf6df039d33`。

追加回归识别出专化组件名透出、已重赋值的 selector 局部仍被静态推定、同进程准备过的 Iced provider 再注册，以及已移交内联文本的测试语料残留。AutoLang `50e63ec9c` 将作者标签和调用点内部身份分开，并让已赋值的裸名进入共享 lexical 检查。AutoUI `1a8bdc0f` 两端按相同作者标签核对 view 记录，使用调用点 ID 找正确实例，复用已准备的 Iced provider。024 空数据测试在整合工具的两次尝试用时 25.313 和 12.215 秒。以一个真实 series 触发两个原组件的空数据路径，仍执行两次 Init、渲染并断言无有色数据路径。独立计时探针 6.261 秒检查加 0.962 秒执行，正式用例待复验。完整双端两组数据回填证据保留。

最新版 AutoLang `50e63ec9c` 的定向 AV 131/131（1.322 秒），A2TS 5/5 和 5/5（0.635、0.638 秒），AV Core conformance 51/51（0.246 秒），Core 886 通过及 5 个已经对照的基线失败（16.865 秒），AV Core unit 70 通过及 1 个已经对照的基线失败（0.241 秒）。目前仍在验证 AutoUI `81b063dd`，随后进入 T-07。

`stage: goal | plan_id: PLAN-107 | plan_revision: r2 | round: 88 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang}; D:/autostack/.wt/os-107/auto-os | commit: ui 81b063dd, lang 50e63ec9c, os 680a6ae3 | evidence: full 91-source gallery, production Chrome repaired and frozen diagnostic, native 6-entry replay on prior candidate, final language affected groups | next: verify bounded repairs and new-case timings`

### 第 89–91 轮｜实施交接与双端复跑

按用户确认的 r2 与最小 AutoOS 源修收口。T-01 到 T-06 的候选均已提交，三个工作树均无未提交改动。

| 仓 | 工作树 / 分支 | 候选提交 | 差异基线 |
|---|---|---|---|
| AutoUI | `D:/autostack/.wt/ui-107/auto-ui` / `plan-107-dev` | `9b130370f4be23181ae2d2b79d4f51885b1cd914` | `80cc1874ce411f019537cb16af6545ca3b852fbb` |
| AutoLang | `D:/autostack/.wt/ui-107/auto-lang` / `plan-107-dev` | `50e63ec9cef4468afb3823a8960933f7af83bd39` | `5e5121dac6d6034a7eeb28cf4ed8d71686d21cd9` |
| AutoOS | `D:/autostack/.wt/os-107/auto-os` / `plan-107-dev` | `680a6ae344f89cbf4dcf5efd25a118889fe611c7` | `efcfced51b58a1174f0b4fd3c0288c589495f70e` |

编译期兄弟仓为 AutoDown `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`、AutoWeb `26d085f7e54ea20c6997edf2dc23eef0cb57faf6`、AutoMan `4a30483a590d81d5724cb453a0fd1fbc98d9aaf2`。双端生产构建使用 AutoUI `1a8bdc0f` 的工具和宿主，`git diff --name-only 1a8bdc0f 9b130370` 只有 `crates/auto-ui-runtime/tests/plan114_widget_packages.rs`。调试版 `auto-ui-tool` 的 SHA256 为 `343295c35bad54f15a59f510a8abb320fcf8f5d027f563120d9c2e9aa1a88ffe`。绑定 Cargo 工件见 `round89-ui-artifacts-for-replay.jsonl`。使用 rustc 1.98.0 (88d9e12ae 2026-08-18)、auto 0.1.0+plan-890-verified-r7-t19-116-gc879b3e、pnpm 11.6.0 与 Chrome 157.0.8103.80。

A2Vue 从隔离的原项目根执行 `auto build --render vue --gen-only`，设置 `AUTO_UI_ACE_VUE=1`、`AUTO_UI_TOOL_BIN` 指向已固定工具、`RUST_MIN_STACK=67108864`。在 `gen/front/vue` 用 `pnpm install --offline` 和 `pnpm run build` 运行正式 `vue-tsc`、Vite。锁文件优先冻结，画廊按实际本地依赖路径重映射并核实只删移交组件的依赖，存量依赖版本均相同。最终画廊锁 SHA256 `1dc15503c6694d73e00d4f2f5a1263e173747ec6b449cd6401077e9a5924ed3a`。画廊两份输入各 91 个作者源，SHA256 比对仅三处获批组件有差异；产物覆盖 70 路由、90 生成文件。冻结版也通过生成和构建，原 FlowDiagram 在最终工具下给出 `source.type-mismatch: expected i32, found f32`，较早运行时的实际命中对照仍为高亮 0、降透明 5。最终合格判定使用修正版。

原 Iced 测试以 `pac.at` 的 `src/front/app.at` 为源，从 Cargo 工件记录选择原宿主 rlib，显式使用准备好的 `ACE_NODE_DESKTOP_ROOT` 和与 `rustc --version` 一致的 `ACE_NODE_DESKTOP_PREPARED_RUSTC`。每条串行打开实际窗口，`--instances 1 --hold-ms 420000 --evidence <json>`，把挂载 View 的类型化回调提交到窗口消息队列。Chrome 指针由真实浏览器鼠标执行。下表各窗口关闭完成、最后一个目标会话的晚到回调各丢弃 1，顶层 Iced `errors` 与 Chrome 异常均为 0。Iced 原画廊默认 `/` 路由仍返回此前记录的 `app::index` 语法诊断，随后五条图表目标路由均成功创建会话，各自的路由错误为 null。

| 原入口 | AV/Iced 最终回放 | A2Vue/Chrome 最终回放 |
|---|---|---|
| 024 | 770 帧，12 次提交。路径 4→19→7→4→4、Jan Desktop 186 与准线、图例 4→3→4；播放 6→7 后暂停、重置。 | 四图切换、指针命中/离开，播放 6→8、暂停重置，卸载完成，异常请求 0。 |
| Area | 65 帧，4 次提交。提示跟随命中区，路径 5→3→5。 | 两组面积填充及轮廓，指针、图例路径 5→3→5、重开和卸载。 |
| Bar | 与 Line 共 275 帧，Bar 提交 4 次。单组 5 柱、分组 8 柱，命中与复原。 | 单组/分组 5/8，命中 sales 155 且 6 根非目标柱降透明，离开复原，图例 8→4→8。 |
| Donut | 65 帧，8 次提交。四段弧、层序、指针与图例状态。 | 弧 4→3→4，提示层级与鼠标离开复原，卸载。 |
| Line | 与 Bar 共 275 帧，Line 提交 4 次。单/多列与相互独立的状态。 | 单/多列 1/2 条路径，分别 5/6 点，命中 desktop 186 与双准线，图例 2→1→2，重开卸载。 |
| FlowDiagram | 283 帧，2 次提交。两布局各 5 节点、5 边、5 箭头，目标高亮 1、降透明 4，另一布局独立。 | 两布局命中坐标与 PLAN-502 原预期一致，各自高亮 1、降透明 4，提示误差 ≤1.5 CSS px，离开、重开卸载。 |

第 89 轮原证据在 `D:/autostack/plan-artifacts/plan-107/`。

| 入口 | Iced JSON 后缀及 SHA256（前缀 `round89-iced-`） | Chrome 日志及 SHA256 |
|---|---|---|
| 024 | `024.json` `048b0ea469d551d6ec46f3a9f41a9efaa5d04b901ee1092988307e2408bed9da` | `round89-chart-round59-original-charts-pointer-clearance.log` `dec09b749a5878c8d0770edac2fc4228f23848802efde65f1f533b0734d42739` |
| Area | `area.json` `51fbe7051e6f5f065e959090da697b9fd0d0a96b33eb420c237e889104a66434` | `round89-gallery-round54-original-area-browser.log` `3cd2e9a4e1e2efc7d42e015ed54e1a2556acd774a9238fbb18de2df825656dfb` |
| Bar 与 Line | `bar-line.json` `acb2134aa52bef2bed70a6eb1f6cd15ceaca3e97daa65cc1525b32263be60bc3` | `round89-gallery-round64-gallery-charts-browser.log` `454b9fde7fdf69c0f7a4577eaa7cb11c4b6d980afab5827ba3334c229d0d0249` |
| Donut | `donut.json` `642050a5bc86672e143999647aea1ffdf7cc9379175a6e468f2aac785e237280` | `round89-gallery-round50-original-donut-browser.log` `84b4c74b9bef660d482fce04a6b3eeb0ddf2029b2112822a3709492a520bac0e` |
| FlowDiagram | `flow.json` `560dd7c4ebe002c683857fb4bbf485e4e74fa3c4a5f8e3ff2cae6b20766b22a6` | `round89-gallery-round71-gallery-fixed-browser-final.log` `6508bf451cb73366c992e9db8e1f1b9a54b922aff860e325d9e7ba65e42ab822` |

第 91 轮在同版工具和 Iced 构件重跑原组件负控。Area 与 Bar 用各自原组件输入空→填→清空→再填，两端均见 `(0,0)→(2,6)→(0,0)→(2,6)`；Iced 278 帧、提交 3 次，SHA256 `61bb5f679f6bdba6b2b453b6e5e56ea2bdaa719aa7f81a74aea8704c42e86a19`，Chrome SHA256 `8b848d4e2009919795a3bb24dd905ed4576681440ff0bd300b1f9c6d67f60b56`。Line 路径 `0→2→0→2`，Iced 264 帧、提交 3 次，SHA256 `e9cc7ec1e353ac89aa0cad55456cb64cab130661cbc3def4fa761f7b6657a028`，Chrome SHA256 `ad0d022ceafbb41af934b2bbdc3d6a8960ea8dadcbe1c9fe4e6655024b720c33`。Chrome 空 Donut 为 1 个中性环、0 数据弧、0 图例按钮，可收指针并卸载，SHA256 `2e0f4048fd56fdf79f78b78c1a3d6f3a13e1241047bdf7f07f1808372743777d`，同期原组件 Iced 空输入对照见第 52 轮。所有本轮负控的子进程退出 0，宿主和浏览器错误 0。

AutoUI 先用 `cargo test --locked --offline -j 1 -p auto-ui-tools --features ace-vue -p auto-ui-runtime --lib --test <受影响二进制> --no-run` 显式准备 debug/test，再串行直跑相应测试二进制。结果为 desktop 12/12（0.897 秒）、TS provider 27/27（4.028）、013 Store 1/1（0.532）、U03 SFC 3/3（9.319）、U03 sites 1/1（2.736，既有忽略 1）、路由 4/4（1.251）、关闭 4/4（0.677）、包 SFC 19/19（0.693）、Vue 路由 1/1（1.272）、生产工具入口 1/1（3.785）、包消费 20/20（13.689，单用例单独计时）。合计集成 93 通过、忽略 1。原类型负控要求在 bool 插值处给出 `source.fstr-arg`，保留原失败触发条件。原空输入组合用例的 25.313、12.215、10.431 秒记录保留，最终各自完整执行原 Area 和 Bar 的 Init 和渲染断言，分别 5.297 和 5.482 秒。新增 AutoUI 37 项全通过，最长 5.482443 秒。逐项 `round91-timing-auto-ui.json`。

AutoLang 在最终提交上显式准备 debug/test 后串行直跑。定向 AV 131/131（1.322 秒），A2TS Node 5/5、5/5（0.635、0.638），AV Core conformance 51/51（0.246）；Core 886 通过另 5 项 `text_record_source::` 失败（16.865），AV Core unit 70 通过另 `layout_rejects_non_record_and_empty` 失败 1 项（0.241）。已用主基线分别复现同 span 的 `verify.double-move` 和相同空记录尺寸结果，主对照 `round83-lang-main-text-record.log`、`round84-main-layout.log`，候选 `round88-lang-gate.log`。新增 53 项全通过，最长 0.269504 秒。逐项 `round88-timing-auto-lang.json`。完整固定组门由最终合入负责，已用 `cargo t ace-source ace-av ace-a2x ace-syntax --select-only` 确认选组，配置 SHA256 `69165441f188d5dde0a17a1291a0a5e2b0748f778104a7e2b027632a76611fbf`。

AutoOS 预检 `pwsh -NoProfile -File scripts/check-app-submodules.ps1 -OsRoot D:/autostack/auto-os` 退出 1，既有未初始化 gitlink，27 个清单项且缺失 pac.at 0；候选独立检出也显示组外应用兄弟路径缺失。日志 `round89-autoos-{main,candidate}-submodules.log`。基线到候选 Git 差异仅三个 `.at` 与项目 README，四个均为普通 100644 文件。画廊复跑由以上固定项目和依赖完成。

规范候选 SHA256 分别为 AutoUI `75d8689f28b910382717119949b8aa8b9910908a5a2d818670da7cc3e2cefb87`、AutoLang `7a7b60dcd32587bd1882fa35640668f045e65fbafd45f6586122fa24f55b60a2`、AutoOS 项目 README `a8d4859c3a23447d6fe1005624e1e9f95ddfbdfd2344123a716b1a783b1fd168`。验证配置、各端证据及源置于本轮同一基线，T-07 待终审，T-08 的回归、落主、归档和清理由终审结果触发。

`stage: work | plan_id: PLAN-107 | plan_revision: r2 | round: 91 | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-107/{auto-ui,auto-lang}; D:/autostack/.wt/os-107/auto-os | commit: ui 9b130370, lang 50e63ec9c, os 680a6ae3 | evidence: six original entries, actual two-host replay, frozen comparison, negative controls, new case timings and canonical Spec candidate hashes | next: T-07 final review`

### 第 92 轮｜同版 Donut Iced 空数据复跑

从第 89 轮准备好的 Iced 构件重链空输入验证器，使用第 91 轮浏览器对照的同一 Donut 作者组件和空列表调用页，设置 `PLAN107_EMPTY_DONUT=1`，参数 `--instances 1 --hold-ms 60000 --min-frames 42 --script '28:close'`。第 5、13 tick 均见 1 个中性环、0 数据弧，期间由实际挂载的命中区回调提交指针。窗口关闭后拒绝同一回调的再次提交，42 帧、目标提交 1、迟到丢弃 1、错误 0、退出 0。证据 `round92-donut-empty-iced.json` SHA256 `537751ed9591e1036a892da5c7d5b61264f2183f9028b15b556234c154c0ec96`。首条运行为正向验证后漏传关闭脚本，回调仍在活跃会话，失败原记录 `round92-donut-empty-iced-missing-close.log`。仅复跑参数有调整。

交接校验重新核对三个候选 HEAD 与差异基线，工作树均干净，`git diff --check` 通过。样例、画廊 Iced JSON 和 Chrome 日志的 SHA256、规范候选 SHA256、37 与 53 条逐项计时均与交接一致。六个计划工作树经 `wt-guard.ps1` 检查均无 reparse point。独立审阅上下文待用户确认，T-07 保持待办。

`stage: goal | plan_id: PLAN-107 | plan_revision: r2 | round: 92 | from: work | to: review | status: execution_done | outcome: waiting | commit: ui 9b130370, lang 50e63ec9c, os 680a6ae3 | evidence: latest Donut empty on both hosts, artifact and source hashes rechecked | next: confirm final review context`

### 第 93–94 轮｜补核缩放、故障恢复与键盘

复用冻结的正式画廊构建及 Chrome 157.0.8103.80。修正版 FlowDiagram 在 1550、880、430 CSS px 宽的窗口分别显示为 576、526、364 CSS px 宽。用真实鼠标操作，两种布局各自高亮 1、降透明 4，另一布局保持原始几何，水平提示误差小于 1.5 CSS px，离开复原、路由重开、卸载，三轮错误 0、退出 0。宽屏走原侧栏。源外壳在窄屏隐藏桌面侧栏，手机抽屉没有菜单项，探针改用原 hash 路由，并把纵向节点滚到可见区中央。首次 760、640 的侧栏探针未进入目标路由，首次 760 的直达探针在实测纵向命中前调整了滚动位置。修正后 760、640 也通过，未改产品源。日志 `round93-flow-1550.log` SHA256 `e9641f6696d42734a51711dcd29582618c83ec1aa47d4ddad6bb9dd289276817`、`round93-flow-880.log` `8700ce11b6eb7d21026514d1a54f6aa992e03a8693e8851693c0905d1a7e2e2a`、`round93-flow-direct-centered-430.log` `f047895d588ddd4d36c215e34eb0867f509498074c3761c7b901048fabe81150`。

从冻结版外壳连续两次打开原 FlowDiagram，均给出 `ace.route.page` 的 `source.type-mismatch: expected i32, found f32`；各次从原侧栏进入 Area 后，错误提示消失，图表正常挂载。冻结版及修正版两轮 Area 都通过键盘 Enter 隐藏首组、Space 恢复，路径数 5→3→5，有正长度的填充路径 2→1→2。最终卸载根容器为空、浏览器异常 0、退出 0。冻结版日志 `round94-frozen-flow-route-recovery.log` SHA256 `d76c282cc0add6b2dd18f276fcc4d9fbb8d4727e4bc3f69d3673a2202480c658`，修正版 `round94-fixed-area-keyboard.log` SHA256 `f29a37f1c8a6fea0017cc8758d3035d86ec2c2af0fa490557cf86de2a953d0a4`。早期探针记录分别保留多个 SVG 的选择器歧义和把五条总路径计成五条填充的断言，最终用原图表 viewBox 和填充/总路径各自计数。

Iced 第 89 轮四份画廊记录分别保留默认 `/` 的 ``source.syntax: expected `fn`, found `widget` ``。后续通过原外壳路径成功创建五条图表会话，最终路由错误为 null，会话关闭、顶层错误 0。回放矩阵记录了默认首页与五个目标图表页各自的状态。T-07 的独立审核仍待确认。

## 10. 下游交接

107 按原 024 与五个图表页给 W1 和 1072/产品 CLI 提交各端证据。U05 剩余行由各 owner 按路线索引汇入，保留本计划此前探针及用例映射。

- 024 和五个原页的最终双端回放、空数据和恢复对照已在 §9 第 89–91 轮绑定提交，供 W1 和 1072 消费。FlowDiagram 双布局与冻结版的失败证据一并保留。
- 向 123 提供原全画廊需要的图表 handler/组件及证据绑定的版本。涉及 123 在途的桌面入口时，接已审路由/入口切片与 114 实际包、列表、事件接口，再跑 107 的整合行。
- CodeEditor 的光标宿主缺口及原用例收据移交 120。媒体、图片和画布的旧观测按 §9 各归属接续。
