---
plan_id: PLAN-107
status: executing
feature_name: AutoUI U05 媒体、图表与绘制 AV 与 A2Vue 交付
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-21
plan_revision: 1
current_step: 1
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-107 AutoUI U05 媒体、图表与绘制 AV 与 A2Vue 交付

## 0. 变更摘要

2026-11-19 按用户裁定建立独立交付计划。承接语言 980 B-AV、1071 冻结清单中 U05 的原示例和关联 Widgets，验收 ACE AV 桌面和 A2Vue 浏览器。由本计划负责 AutoUI 和所需 AutoLang 改动，并补齐这些应用实际用到的缺失组件。A2R、AC 由后续独立计划接同源验收。本次为规划，进度 0/8。

## 1. 目标

以下 7 个原项目及关联 Widgets 各有两端可复跑结果。复用原 `pac.at`、作者源、依赖及独立预期。交付可从干净目录构建和运行的项目、固定工具与 SDK 版本、每端操作记录及独立复审结论。

| AutoUI 原例 | 两端项目级检查重点 |
|---|---|
| [019-video-app](../../examples/ui/019-video-app/pac.at) | 真实媒体载入、切换、播放与坏源恢复 |
| [020-music-player](../../examples/ui/020-music-player/pac.at) | 音频播放暂停、曲目切换与结束状态 |
| [024-charts](../../examples/ui/024-charts/pac.at) | 可见图形、数据更新、指针/图例交互及空数据 |
| [029-photo-gallery](../../examples/ui/029-photo-gallery/pac.at) | 图片浏览与查看、组件嵌入、缺图后恢复 |
| [030-video-player](../../examples/ui/030-video-player/pac.at) | 播放/暂停/seek、进度一致、关闭回收 |
| [031-image-viewer](../../examples/ui/031-image-viewer/pac.at) | 图像载入、缩放/平移、可见区域和坏图像恢复 |
| [031-paint](../../examples/ui/031-paint/pac.at) | 实际指针绘制、像素或区域变化、原编辑操作和卸载 |

关联 Widgets 使用 [AutoOS 规范来源](../../../auto-os/widgets-gallery/src/front/) 的 pages 与实际 components 闭包。area-chart、bar-chart、donut-chart、line-chart、flow-diagram、carousel、code-editor。T-01 把 1071 标识逐项对应到实际页、组件与原测试，给复用页记录版本和各组消费结果。

## 2. 架构与跨仓职责

- **AutoUI 主责。** 负责示例、组件契约、Iced 宿主消费、Vue 生成产物及真实交互。复用已有 media provider、相关图表和画布模块，核 089/090 等现有组件合同。媒体用固定可验证的实际资产，GPU/画布用固定视口及可重放输入验关键区域和状态。真实播放、实际 seek 和画布像素/输出各留可复查证据，资源回收采用实际宿主和浏览器观测。
- **AutoLang 为同一计划的实施仓。** 核资源内容身份、媒体状态和事件载荷、时序/取消、指针数据与画布相关跨端类型。补实际缺失的 AutoLang 检查、AV 宿主调用或 A2TS 运行时映射，保留媒体结束、失败和资源释放语义。通过同一原源与资产对比两端可观测状态。 相关改动在组内 AutoLang 树完成，沿原模块 owner 审查与合入。
- **真实组件。** 对本组使用的每个缺失组件或缺失行为，优先补已有组件，再补所需新组件。交付适用属性、事件、绑定、生命周期和无障碍行为，两端实现、永久回归及原项目调用都计入本计划的完成条件。已有其他计划在写同一组件时，先对账实际界面与写面，固定单一实施负责人，当前计划持有本消费者验证及关闭条件。

## 3. 技术与实现入口

AV 沿 `crates/auto-ui-runtime/src/ui/ace_node_desktop.rs` 的公开宿主路径、受检 AGE Schema 和 typed 事件。A2Vue 沿 `crates/auto-ui-runtime/src/ui/ace_vue_script.rs`、`ace_ts_provider.rs`、`ace_package_sfc.rs` 和正式 `auto-ui-tools` 入口，验生成的 Vue/TS、运行时与资源。AutoLang 优先复用已有 AGE/ACE、AV 和 A2TS 管线。Windows AV 及所选浏览器配置各有实测记录，其他原适用平台按冻结的运行条件记账。

## 4. 来源与前置

- [980 原 W1 和 ④-A 设计](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 和 [1071 示例/Widgets 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)。已有运行收据按实际输入、范围和版本复用。
- [AV 宿主及 W1 收据](../../../auto-lang/docs/specs/ace-migration/m2-autoui.md)、[1072 Vue 生产计划](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)、[976 TS/JS 交付](../../../auto-lang/docs/plans/archive/976-ace-m2-typescript.md)、[978 构建交付](../../../auto-lang/docs/plans/archive/978-ace-m2-build.md)。
- U02–U06 按用例所需接口推进。遇到共享的 ④-A、事件、集合、外部服务接面或组件缺口，记录具体接口、实现负责人、交付版本与消费测试，已具备前置的样例继续验收。1072 的 982 联合 CLI 验收可消费本计划的 A2Vue 证据。
- 2026-11-20 用户以 `/goal 完成计划107` 授权 r1 原范围的实施、独立复审、限定修复和落地归档清理。

## 5. 实施与交付边界

先冻结 U05 矩阵，逐行绑定项目原源、Widget 页和组件、预期、失败反例、权限、资源、版本和 AV/A2Vue 状态。从原项目的第一个失败处建立可复跑入口，修所用能力或组件，回同一项目完成两端验收。公共语义走原 owner 的统一接口，源级修复覆盖 AV 和 A2TS 的实际消费者。

实施使用 `D:/autostack/.wt/ui-107/{auto-ui,auto-lang}` 的关联工作树。用到其他仓版本时明确固定提交，按组内路径或既有显式覆盖解析。合同、进度和归档在 AutoUI 主检出管理，实施和构建在工作树。跨仓落地顺序以实际生产者和消费者验证为准。

### T-01 原样回放矩阵

应用入口均为 `examples/ui/<行名>/pac.at`、`src/front/app.at` 及其原始模块。画廊入口为 AutoOS `widgets-gallery/pac.at`、`src/front/app.at` 与 `src/front/pages/<行名>.at`，组件从原 `components/package.at` 解析。下表基于 §9 的 Git 来源。两端保留原有适用性，CodeEditor 的 vi 模式和系统 IME 按原 native 条件检查。首轮顺序为 002 对照、031-paint、024-charts、其余应用、画廊；同一 Iced 工具串行排窗。

| 原样输入 | 实际闭包及独立预期来源 | AV 与 A2Vue 必验状态及负控 |
|---|---|---|
| `031-paint` | `paint_color.at`、ColorPicker、NumericFields、Canvas；`SPEC.md`、`tests/desktop_mcp.py` | 真实指针落笔，像素 42 按选色变化，Undo/Redo 回放，取消选色保正式色，存储往返后卸载 |
| `024-charts` | `components/{line_chart,bar_chart,area_chart,donut_chart,chart_geom}.at`；`SPEC.md`、`tests/desktop_mcp.py`、`tests/test_024_vue.mjs` | 四类图有实际几何，更新数据后几何变化，指针命中、图例操作及空数据 |
| `019-video-app` | `pages/{home,watch}.at`、`video_store.at`、`settings.at`、`back/{api,db}.at`；`tests/{acceptance.atd,smoke.spec.ts,vm-smoke.mjs}` | 卡片进入 `/watch/:id`，真实解码与时间推进，切换相关视频，坏 URL 后恢复；仅向已选用例开放媒体请求 |
| `020-music-player` | `player_store.at`、`viewport.at`、`controls.at`、队列/导航与 `back/api.at`；`README.md`、`tests/test_020_vm.py` | 固定两首 1.5 秒 WAV，播放/暂停、换曲、到尾策略与设备释放；扫描根仅指向样本目录 |
| `029-photo-gallery` | `back/api.at`、ImageViewer、AssetBrowser、TreeView、Pagination、CollectionView；`SPEC.md` 的 scan/thumb/full 与 UI 行为 | 缩略图与原图身份、相册和查看器切换，坏图失效后可打开同目录好图；`AUTO_PHOTO_ROOT` 指向固定图片目录，撤根或未授权路径不能被读取 |
| `030-video-player` | `player_store.at`、`viewport.at`、`controls.at`、`playlist.at`、`file_picker.vue` 与 `back/api.at`；`tests/{smoke.spec.ts,vm-smoke.mjs}` | 真实帧与时间、暂停、seek 至 50%（时长 ±5%）、坏媒体后恢复及关闭；保留原用例媒体目录与真实 Chrome 编码支持条件 |
| `031-image-viewer` | `back/{api,directory_index,image_service,types}.at`、ImageViewer；`tests/{vm_matrix.py,vue-actions.json,fixtures/manifest.json}` | JPEG/PNG/WebP、Alpha/EXIF、缩放/平移和可见区，坏图具名失败后恢复，关闭会话不残留图像请求 |
| `area-chart` | `components/area_chart.at`；原页 `area-chart-overlay`、图表 `plan484` 与 `plan499` 回归族 | 叠加区填充和顶边几何、数据与指针回放、空数据 |
| `bar-chart` | `components/bar_chart.at`；原页 `bar-chart-basic`、`bar-chart-grouped` 与 `plan484`/`plan499` | 单组/分组柱几何，命中与更新，空数据 |
| `donut-chart` | `components/donut_chart.at`；原页 `donut-chart-basic` 与 `plan484`/`plan499` | 各段弧、指针/图例状态，空数据 |
| `line-chart` | `components/line_chart.at`；原页 `line-chart-basic`、`line-chart-multi` 与 `plan484`/`plan499` | 单/多列路径、指针提示、更新与空数据 |
| `flow-diagram` | `components/flow_diagram.at`；原页 `flow-diagram-lr`、`flow-diagram-basic` 与 `plan502_diagram_tests.rs` | 左右、上下两方向独立布局；节点命中，高亮/提示及指针离开复原 |
| `carousel` | 外壳 `official` → `components/carousel.at` 五个同族组件；原页 `carousel-{basic,sizes,spacing,orientation,options}`，`component_registry_test.rs` | 指针与键盘改变滑页；横/竖、多个可见项、边界与循环可观察；关闭后无遗留事件 |
| `code-editor` | 原页 `code-editor-{basic,auto}`、本仓 `ui/code_editor` 和 Vue CodeMirror；`plan413_code_editor_natives.rs`、原页 Props 表 | 编辑/撤销、光标、搜索、换行、内容回写与关闭，native vi/IME 按原条件 |

单写面从实际代码分工。PLAN-105 继续负责通用 Store、组件面源准备及其 collections 接线；PLAN-114 负责正被其原树页消费的 AGE 包闭包、可变类型列表、计算值和事件参数；PLAN-123 负责在途桌面项目入口和路由源发现。PLAN-107 持有这些提交的 U05 消费复验、媒体/绘制/图表/编辑器实际使用的受检能力与该组原项目端到端证据。ImageViewer 沿 PLAN-089 已交包，PLAN-090 的通用 MediaPlayer/TransportControls 为独立草拟抽取；本组沿现有受控 `video` 契约验证原播放器。Gallery 源与闭包固定为 AutoOS 只读输入。

## 6. 测试设计

| 项 | 独立证据 |
|---|---|
| V-01 来源与覆盖 | 原项目、Widget 页、具体组件与两端矩阵对账。逐例固定原入口、工具/SDK/Schema、期望值和资源。 |
| V-02 语言与宿主 | 受影响的 AutoLang 测试组、实际 AV/A2TS 消费用例、有效用法与相邻坏源、诊断 span 和错误恢复。回归测试保留对共享语义的判别力。 |
| V-03 组件 | 新补的组件及行为具名入表。两端测试属性/事件、键盘或指针、可访问性、关闭或卸载及所用项目正反例。 |
| V-04 AV 原项目 | 真实 Iced 窗口运行全部本组项目及相关 Widgets，核实际输入、状态、输出、生命周期与资源。按触面复用 `ace_node_desktop` 等入口并记录窗口准备。 |
| V-05 A2Vue 原项目 | 从干净目录经公开工具生成并构建 Vue/TS 和运行时，在真实浏览器中跑相同原源与独立行为断言，核真实 ACE 生成与构建派发、关闭和错误恢复。 |
| V-06 交付复跑 | 冻结两个仓的提交与各端依赖，复跑对应矩阵，核发布产物、可执行命令和子进程退出。原适用行均有真实结果。 |

## 7. 验收标准

| 编号 | 完成条件 |
|---|---|
| AC-01 | 示例、Widgets、资源和各端原预期完整追溯，版本固定。 |
| AC-02 | 本组所需的 AutoLang 改动已交，对应组件缺口在实际应用中关闭。V-02/V-03 正反两面通过。 |
| AC-03 | V-04/V-05 中本组每条必验项均有各自的通过证据，生成、运行、错误恢复及资源终态可复查。 |
| AC-04 | 独立复审通过，按对应仓流程合入修复并发布 V-06 收据。向 W1 汇总提供 AV 格，向 1072 提供 A2Vue 格，归档和清理本计划工作树。 |

## 8. 执行步骤

- [x] **T-01｜work** 冻结原项目、Widgets 与两端覆盖矩阵，确认首批运行顺序及共享接口和组件的唯一写面。见 §5 矩阵及 §9 基线与首轮回放。
- [ ] **T-02｜work** 补本组所需 AutoLang 源码、AV/A2TS 语义与宿主接口，交正反例及来源。
- [ ] **T-03｜work** 完成缺失组件及行为、双端接线和实际消费者回归。
- [ ] **T-04｜work** 从真实入口逐例验 AV/Iced、权限、错误恢复、关闭与资源。
- [ ] **T-05｜work** 从真实入口逐例验 A2Vue 生成、构建、浏览器行为、关闭与错误恢复。
- [ ] **T-06｜work** 冻结各仓提交，完成受影响回归及两端可复跑交付，补齐矩阵和规范增量。
- [ ] **T-07｜review** 同树独立终审 AC-01..04，核缺失行、错误路径、共享归属和收据，修复发现后复验。
- [ ] **T-08｜merge** 按 owner 顺序正式落地，记录 AutoUI/AutoLang 来源，通知证据消费方的交接在计划记录中登记。归档并按仓规清理。

## 9. 复审记录

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

## 10. 下游交接

本计划按 U05 独立完成。AV 收据与 1071 已交范围及其他 U 组结果汇合后供 W1 签发、1068/T-04 和 985/B 接收。A2Vue 收据供 1072 与产品 CLI 消费。A2R/AC 后续计划沿同一源码、Widgets 和独立预期补验。跨仓 golden 基线维护按已有 PARITY 路线归档。

- **B-107-01｜共享生产基线。** 019/020/030 的原 Store、页面模块与前端绑定依赖 PLAN-105 的共有实现；029/031/031-paint 的 `$event`、图表和原画廊的类型列表与包闭包依赖 PLAN-114 的受检实现；原桌面树形挂载需要 PLAN-123 的生产入口与路由源发现。对应在途提交分别为 AutoLang `b987e2c23`、`7d6eabe4c` 与 AutoUI `ffbb488c`/`9d5ea5b3`，它们未形成 107 所需的稳定发布基面。恢复条件是相应 owner 提供可消费并且包含原形态正反例的固定提交，随后在 107 两树接入并重放 §5 矩阵。030 的第一个条件读取 `.store.has_current`，需要共享受检状态与真实控件树共同实现。
- **B-107-02｜原 CodeEditor 源归属承接。** 同源双端均在 `QueryChanged` 缺少消息声明被拒；补声明的材料对照由受检编译转报缺 `code_editor_cursor_line`。请确认允许 107 增加 AutoOS 原画廊页的最小迁移，或由 AutoOS 负责人提供固定页提交并由 107 消费。之后由 107 接入编辑器光标的 AV/A2TS 能力并核编辑、搜索与卸载。材料对照保持独立，固定原页仍用 §9 的 AutoOS 提交。
