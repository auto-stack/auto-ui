---
plan_id: PLAN-116
status: executing
feature_name: U05 Music Player 与 Video Player 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-24
plan_revision: 1
current_step: 3
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# PLAN-116 U05 Music Player 与 Video Player 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。从 [107](archived/107-autoui-u05-media-chart-and-paint-apps-av-vue.md) 的 U05 原矩阵承接本页所列行。 [原分母与交付映射](README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。本计划的 Widgets 交接取实际使用组件；独立 Widgets 页面由交付表对应计划负责。

| 原项目 | 原验收重点 |
|---|---|
| [020-music-player](../../examples/ui/020-music-player/pac.at) | 音频播放暂停、曲目切换与结束状态 |
| [030-video-player](../../examples/ui/030-video-player/pac.at) | 播放/暂停/seek、进度一致、关闭回收 |

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。复用 090 管理的受控媒体接口和原媒体实现。冻结共享控制、时间回灌和实例边界，音频和视频各有项目记录。

## 3. 技术与实现入口

目标入口为 两个项目的 viewport.at、controls.at、player_store.at。AV 走现有受检 AGE 和 ace_node_desktop 生产宿主。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-116/{auto-ui,auto-lang}，每仓单一 plan-116-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

按同一媒体引擎接口核 090，公共组件版本仅在使用时消费。已备播放接线可逐项目验收。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

2026-11-23 用户通过 `/goal 完成计划116` 授权按 `/auto-plan-goal` 推进全链交付：在 `D:/autostack/.wt/ui-116/{auto-ui,auto-lang}` 实施、独立复审、合同内有限修复与合并归档（目标、验收、范围与仓集不变）。截至 T-01，[PLAN-090](090-media-player-and-transport-controls.md) 仍为 `drafting 0/8`（`packages/media-player/` 尚未建立），本计划按 §2/§4 约定直接以原 020/030 的受控 `<video>` / `mpv` / `progress.onseek` 引擎接口验收，不阻塞于 090。

2026-11-24 **基线复测与 B-03 恢复裁定**：`PLAN-117`（图片应用）与 `PLAN-118`（Paint）已分别于 11-24 双仓落主并归档（`docs/plans/archived/117-*`、`archived/118-*`），§10 原裁定所依据的"与 PLAN-117 共享 `auto-lang` 上游写面"冲突因此消失；`README.md` 交付表当前指示为"先在 117 后的基线复测 020/030，再按残余缺口处理 B-03"。据此本会话把 `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`）由 `8a52d7364` 快进到 `bced4dd79`（post-117/118，含 1124 的 Dyn/JSON 与 117/118 的包面闭合），并在 `auto-ui` 侧提交 `e5ce230f` 完成两处接口适配（`HttpProvider::http_poll` → `HttpFailure`、`ComponentStmt::Try` 在扁平重放车道具名拒绝）。

复测结论：**原样 020/030 的 AV 包面仍未闭合**，但缺口形态已从"上游整体缺失"收敛为 5 个可最小复现的具名缺口（见 §10）。由于 117/118 已落主、共享写面空闲，且 §4 `/goal` 授权的实施树本就包含 `auto-lang`，B-03 由"等待上游"转为**本计划在自有 `plan-116-dev` 工作树内逐项闭合**；目标、验收标准、交付范围与仓集均不变。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 规范增量

本计划拆分本身只迁交付归属。实际实现若调整共享语义或组件合同，在 T-01 正式定位 owner、对应规范及 AC 后由实现计划提出增量。

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。以真实资产核 020 播放暂停、换曲和结束；核 030 播放暂停、seek、进度与关闭回收。每个项目按自己的媒体资产、端侧和结果独立列收据。 |
| V-04 A2Vue | 同样的原项目和输入，经已交 A2Vue 生成、实际构建和真实浏览器观察适用行为、错误与关闭。引用证据逐行核 SDK、入口、资源和版本。 |
| V-05 交付 | 对所交目标记录实际命令、生成/构建/运行终态、独立预期、日志摘要及落主提交。保原平台条件、失败、资源与恢复。 |

## 7. 验收标准

| ID | owner_stage | 通过条件 |
|---|---|---|
| AC-01 | work | 每个原项目/页面、原 oracle、适用端、实际前置和资源映射可复查。 |
| AC-02 | work | 所用接口及组件各有正反验证，真实 AV/Iced 的原行为、权限、恢复和关闭通过。 |
| AC-03 | work | A2Vue 的实际生成、构建、交互、失败和恢复按原矩阵通过。 |
| AC-04 | review | 收据与冻结版本对证，逐行核验缺失和临时代码，复审通过。 |
| AC-05 | merge | 可消费版本及两个目标的各自结果落主，各登记下游交接。 |

## 8. 执行步骤

当前 harness 如提供任务面板，开工时登记以下任务 ID。各目标按实际接口推进，阶段复核与最终复审分别覆盖对应结果。

- [x] T-01 固定原项目、依赖与验收矩阵
  - work，AC-01，核 §1 现有源和 §6 原运行入口，输出逐行的源、权限、预期和实际阻断。
  - 2026-11-23 完成：源冻结（020 13 文件 + 030 19 文件，SHA256 前 16 位见 §9）；工作树 `D:/autostack/.wt/ui-116/{auto-ui,auto-lang}` @ `plan-116-dev`（基线 `auto-ui` `40abfb1f` / `auto-lang` `8a52d7364`）+ `auto-web` `0cfc7c7` / `auto-down` `456c86a`（detached）；工具/环境冻结（`rustc 1.98.0 88d9e12ae`、`ace_node_desktop_window.exe` SHA256 `35958D9B…`、`libmpv-2.dll` SHA256 `673E6397…`、Playwright/Chrome/`E:\Music`/`E:\Video` 在位）。AV/A2Vue 首轮基线与首阻断（B-01..B-04）实测记录见 §9。
- [ ] T-02 接通本计划的具体运行接口
  - work，AC-02/03，按 §2–5 接实际缺口，使用已有 owner 的实现和定向正反用例。
  - 2026-11-23 部分完成（AutoUI 侧已闭环，AutoLang 共享运行期阻塞）：
    1. B-01 已解（`ace_source_prep.rs` `extract_quoted_style_literal` 剥除 `style X = "..." // comment` 尾注释）+ `ace_vue_script.rs`（`generate_sfcs_with_lane_product_stores` 传入 `same_file_module_fns` 使 `fmt_clock` 写入 `usePlayerStore.ts`）。
    2. B-02 已解（`ace_node_desktop/widgets/media.rs` + `widgets.rs` + `widgets/pointer.rs` + `ace_node_desktop.rs` 接通 `<video>` → `View::Video` + 5 路 `MediaEventHandler`/`onended` 上行 + `<progress onseek>` → `View::ProgressBar { on_seek }` + 条件分支与晚到关闭清理）。
    3. B-03 **阻断**：原样 020/030 多单元 + `PlayerStore` 在 `auto-lang` `age-package` / `WidgetSession` 缺少 `store` 包单元与跨单元 `.store.*`、`WidgetSession` `HttpProvider`（`Http.get_json("/api/media/scan")`）、`try/catch`、`Dyn`/`Object` JSON 迭代/字段、`var playlist = []`、`math.floor`/`dialog_open`，及视图 `.OnTime()`/`$0`/`use.web`/`view mini` 语法支持（与 `ui-117` 共享 `auto-lang` 上游写面）。
  - 2026-11-24 复测（post-117/118 基线 `auto-lang bced4dd79`，`auto-ui e5ce230f`）：`try/catch`、`Http`/JSON/Dyn、`math.floor`、`store` 声明本体、`widget` 包项、具名参数与 `var` 已被上游解除；`.store.*` 的**跨单元**可见性、`return` 早退、`use web component`、`$0` 四项仍缺，另有一项待定位（§10 R-01..R-05）。闭合按 R 编号推进，每项附最小复现与正反回归。
  - 2026-11-24 **R-01 已闭合**：`age-on-return`（裸形，两面同开，接 `age-on-stmt`/`age-on-widget-stmt`/`age-on-branch-stmt`）+ 早退按结构化控制流下沉（`auto-lang 05ee4ff8f`、`auto-ui 70a6a424`）。`020/player_store.at` 由 `expected fn, found store` 转为 OK；回归 `plan116_handler_return` 6/6，`ace-core` conformance 922 passed / 5 failed（与基线 916/5 的同一 5 项既有红）。余 R-02..R-05 见 §10。
  - 2026-11-24 **R-04a + R-07 已闭合**（`auto-lang 78b916591`）：跨单元 store 导入在最小 2 单元包上闭合（含"只写 `on`、无 `msg` 块"的 020 形态按参数注解补出消息）；AGE 面数值 `.to_string()` 与 `.str()` 同一显示物化。回归 `plan116_store_import` 3/3，conformance 925 passed / 5 failed。**R-04b**（无类型/动态 store 字段）新开并钉住具名拒绝。余 R-02/R-03/R-05/R-06 见 §10。
  - 2026-11-24 **R-04b 已闭合**（`auto-lang 174c6646b`）：无类型且初值 `[]` 的 store 字段投影为可变集合（`List`，元素型沿用 117 的 `List(Dyn)` 缺省），先定型后投影初值；非空无类型字面量仍只读（负控）。回归 `plan116_store_import` 5/5，conformance 927 passed / 5 failed；030 闭包前进到 R-03（`$0`），020 仍 R-05。余 R-02/R-03/R-05/R-06 见 §10。
  - 2026-11-24 **R-03 词法/文法半已闭合**（`auto-lang e528fcb53`）：`$0` 事件实参占位（`event-index-tok` + 两处实参位 alt）。回归 `plan116_view_event_arg` 4/4、`plan116_` 全 15/15，conformance 931 passed / 5 failed；030 闭包前进到 **R-02**（`use.web`）。**R-03b** 新开（宿主元素 `.Msg($0)` 被投影成 ChildEvent 而拒）并钉住。余 R-02/R-03b/R-05/R-06 见 §10。
  - 2026-11-24 **R-10/R-11 已闭合**（`auto-lang f59821dec`）：`age-view-if` 条件改走 `if-cond`（裸 pratt 会把 `if x == g {` 的 `{` 当结构字面量吞掉分支体）+ 包面回调作者实参臂（占位形仍先匹配）。回归 `plan116_view_if_cond` 3/3、`plan116_` 全 18/18，conformance 934 passed / 5 failed；030/playlist 去掉 `use.web` 行后整项解析通过。
  - 2026-11-24 **R-02 已闭合**（`auto-lang 979cd3427`）：`use.web component` 导入（`age-use-web-item`，仅 age-package）+ 宿主元素准入（元素侧无需改动）。回归 `plan116_web_component` 3/3，conformance 937 passed / 5 failed；030 闭包前进到 **R-12**（bracket 列表 model 字段的视图可观察性）。
  - 2026-11-24 **R-06 已闭合 / R-12 已锐化**（`auto-lang 563ee47de`）：视图回调的显式空参数表 `.Msg()`（`age-event-binding-args-empty`，零作者实参→裸消息引用=宿主供载荷）。回归 `plan116_` 全 24/24，conformance 940 passed / 5 failed。R-12 经最小复现钉在"`[]T` 拼写未接入视图列表观测通道"（与 store 内联无关；裸 `List` 可观察）。
- [ ] T-03 实跑 AV/Iced
  - work，AC-02，运行 V-03 真实窗口、操作和关闭，冻结可供 W1 复核的来源。
  - 2026-11-23 受控媒体宿主已实跑通过（`mpv_contract` 13/13、`mpv_channel` 10/10、`mpv_engine` 11/11、`plan116_av_media_widget` 2/2、`ace_node_desktop` 10/10 顺序 CPU 用例；`ace_node_desktop_window.exe --features ace-bridge,mpv-widget` 真窗播 `E:\Video\caelestia.mp4` 实测 `duration="49.429"` / `SeekMid=15` -> `OnTime="15"` / `TogglePause="true"` / `frames_rendered=10` / `errors=[]`）；**原样 020/030 多单元包入口因 B-03 阻塞无法在 `PackageStateEngine` 启动**。
- [ ] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
- [ ] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [x] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
  - 2026-11-23 完成：`plan116_a2vue_sites`（`p116_build_a2vue_sites` + `p116_a2vue_browser_e2e` 2/2 通过）：`020-music-player`（7 widget SFC + `PlayerStore` TS）与 `030-video-player`（4 widget SFC + `PlayerStore` TS + `FilePicker`）均冷构建 `vite build` 成功，并于真 Chrome + 真实 Rust `back_proxy`（`/apps/*/api/media/scan` 与 HTTP 206 `/api/media/stream/:id`）全项通过（020: 真实 `.wav` 起播/暂停/恢复/ProgressBar seek 1.8s/音量与静音/NextTrack/Shuffle-Single-Sequence 循环/短曲自然 `onended` 切歌/0 pageerror；030: 递归扫描 `nested`+根目录/1920x1080 H.264 解码 `duration=49.429s`/暂停/seek 24.9s/倍速 `1.25x`/音量静音/坏媒体 `errorCode=4` 恢复播放/本地 `FilePicker` `blob:` 起播/0 pageerror）。
- [ ] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [ ] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-23 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-01 冻结矩阵与首阻断 | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/{auto-ui,auto-lang}`（`plan-116-dev`）+ `auto-web` `0cfc7c7` / `auto-down` `456c86a`（detached） | base_commit: auto-ui `40abfb1f47732b4a65f592a3680fac9988ad6033` / auto-lang `8a52d7364e5b218e0a08b167038a5d71d54ee72f` | code_commit: -（工作树基线，T-01 仅观测） | task_ids: T-01 | evidence: `D:/autostack/plan-artifacts/plan-116/`（`av/t01-{020,030,020-pkg,030-pkg}.log`、`probes/t01-lane-probe.log`）

**源冻结（SHA256 前 16 位）。**
- `020-music-player`（13 tracked）：`pac.at` `C5ECE86EE7F76803`（752B）、`README.md` `BBDB2AF203E898E9`（2869B）、`src/back/api.at` `7B89C20DBFD50599`（419B）、`src/front/app.at` `D167CAC22F8FD9A1`（8467B）、`controls.at` `D04D2DD226F35221`（11553B）、`nav_sidebar.at` `CE88A5FB592FC229`（10744B）、`player_store.at` `ED6DBE0FE44A77C0`（14947B）、`queue_drawer.at` `3839FBBF5510F66E`（2983B）、`stage.at` `D91A89F720D80D07`（13969B）、`tracklist.at` `355BE0C3B2A1AE93`（13187B）、`viewport.at` `303175BAC4B23EC4`（2257B）、`tests/test_020_vm.py` `5E4371E5A256D976`（7160B）、`.gitignore` `C63523FE8FFE6B9B`。
- `030-video-player`（19 tracked）：`pac.at` `9760495B0464740A`（1385B）、`SPEC.md` `5D7D60F76A1032C6`（11740B）、`README.md` `B55006FFE9D7D675`（4259B）、`src/back/api.at` `D949A09CF2C7B897`（419B）、`src/front/app.at` `FE9E687F303374AF`（5228B）、`controls.at` `8B944DA9B1609FF3`（5782B）、`file_picker.vue` `700A33D685CF19BC`（1814B）、`player_store.at` `E5EB6F41F3353A0D`（18945B）、`playlist.at` `80D7A49BA1F9C539`（6632B）、`viewport.at` `38BD6F6901B93DFA`（5237B）、`tests/smoke.spec.ts` `FED5944F04368F9C`（14368B）、`tests/vm-smoke.mjs` `EBE4A842DAD44D0A`（11687B）及其余 7 测试/配置资产。

**工具与环境基线。** `rustc 1.98.0 (88d9e12ae 2026-08-18)`；AV 入口 `ace_node_desktop_window.exe`（Debug 构建 2m40s，SHA256 `35958D9BBC7CD234E68C0AB1ACB23DA01D7709367F0C82EE1C3872ACBFC2A259`）；原生解码库 `D:/autostack/tools/mpv/libmpv-2.dll`（SHA256 `673E6397920AB64A9C5B3A618F7F16D38854EFE72B58665F1F84E4E873B763A4`）；Playwright（`auto-os-config/node_modules/playwright`）、Chrome、`E:\Music`（真实 FLAC/MP3/WAV）、`E:\Video`（真实 `caelestia.mp4` 7.67MB + MKV）均在位。PLAN-090 当前 `drafting 0/8`，按 §2/§4 直接消费既有受控媒体接口。

**T-01 两端基线实测与首阻断。**
| 原输入 | AV/Iced（`--source` / `--package` / 单单元） | A2Vue lane（`plan116_t01_probe` 2/2 探针 1.61s） |
|---|---|---|
| `020-music-player` | `--source`: `source.age.structure: unexpected token in a bare element tag`（exit 4）；`--package`: `source.syntax: expected fn, found widget`（exit 4）；`player_store.at`: record literal 拒；`controls/nav_sidebar/queue_drawer/stage/tracklist`: `.store.*`（S1）；`viewport.at`: 装载 `frames=1` 但无 `video` 宿主投影 | 7 widget SFC + `PlayerStore` TS（11338B）**可生成**（`bodies=0`，handler 经旧 JS 域外体）；浏览器构建/运行待 T-06 |
| `030-video-player` | `--source`: `.store.*`（S1，`player_store` 探针失败未内联）；`--package`: `model has no field store`（exit 4）；`controls/viewport`: bare element tag（尾注释 + `$0`）；`playlist`: `source.use-rs.syntax: use.web is not a supported import` | `app/controls/playlist` 可生成 SFC；**`viewport.at` 解析失败**：`UnexpectedToken { expected: "Colon", found: ")", offset: 5302 }`（`ace_source_prep::local_styles` 未剥除 `style timecode = "..." // Phase B...` 尾注释，替换后注释吞掉元素右花括号） |

**具名阻断与写面归属（T-02 范围）。**
- **B-01（`ace_source_prep.rs`，`auto-ui`）**：`local_styles` 提取未剥除 `style X = "..." // comment` 尾注释，导致 `030/viewport.at` 在 A2Vue 与 AV 两端均在 `style: timecode }` 处破坏花括号配对；另有 `use.web` 未空白化（`030/playlist.at` 触发 `use-rs.syntax`）。
- **B-02（`ace_node_desktop/widgets.rs` + `ace_node_desktop.rs` + `Cargo.toml`，`auto-ui`）**：AV 桌面桥缺少 `"video"` 元素投影（未接 `View::Video` + `mpv-widget` / `VideoUplink` 上行事件泵与帧 tick）及 `"progress"` `onseek` 回调接线；`ace_node_desktop_window` 未启用 `mpv-widget`（或默认 `mpv`）。
- **B-03（跨单元 store + 多组件组合，`auto-ui` + `auto-lang`）**：020/030 的 `App` 经 `use <child>: <Child>` 组合子组件，且各组件共享 `PlayerStore`（含 `Http.get_json("/api/media/scan")`、`try/catch`、`for`、`??`、`fmt_clock`、`dialog_open` 等）；AV 单源/包面尚未把 `PlayerStore` 与多组件视图树打通到真窗运行。
- **B-04（真实后端 `/api/media/scan` 与 `/api/media/stream/:id` + 合成夹具，T-02/03/06）**：020/030 均消费媒体目录扫描与 HTTP 206 流，需在测试/宿主入口接通真实 `media_service`（含隔离合成 PCM WAV + WebM/MP4 夹具与损坏/不存在 URI 错误恢复对照），保持 `<=10s` 单例成本线。

**主检出预检。** `auto-ui` 主检出仅 `docs/plans/118-autoui-paint-av-vue.md` 与 `docs/plans/README.md` 有他会话计划簿记改动（非代码路径，本计划不触碰）。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 1 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/{auto-ui,auto-lang} | commit: -（基线 auto-ui 40abfb1f / auto-lang 8a52d7364） | next: work/T-02

2026-11-23 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: A2Vue 双端全绿 + ace_node_desktop <video>/<progress>/mpv 真窗实播通过；原样 020/030 全应用 AV 遇 auto-lang 上游 B-03 阻断 | outcome: blocked | worktree_path: `D:/autostack/.wt/ui-116/{auto-ui,auto-lang}` | base_commit: `auto-ui 40abfb1f` / `auto-lang 8a52d7364` | code_commit: `auto-ui plan-116-dev 3067ce58`（`auto-lang` 未改，零触碰） | task_ids: T-01, T-02(partial), T-03(partial), T-06 | evidence:
- `ace_source_prep` 单测 `style_with_trailing_comment_strips_comment`（及 7 项通过，1 项基线既有 `022-kanban` `real_u03_apps_load_through_desktop_bridge` 失败同基线）
- `plan116_t01_probe` 3/3 passed（`2.43s`）
- `plan116_a2vue_sites` 2/2 passed（`54.84s`，`AUTO_UI_ACE_WEB_E2E=1`：Chrome Playwright + `back_proxy` `NativeMediaApp` 206 流，020 11/11 checks + 030 10/10 checks 全部 `failures:0`）
- `plan116_av_media_widget` 2/2 passed（`0.34s`，`<video>` + 5 路 `MediaEventHandler` + `<progress onseek>` + `Cond` 分支 + 关闭晚到丢弃）
- `ace_node_desktop` 顺序 CPU 10/10 passed（`0.71s`，`-- --test-threads=1 --skip byte_len`，需 `ACE_NODE_DESKTOP_PREPARED_RUSTC`）
- `ace_node_desktop::widgets::tests` 9/9 passed（`0.08s`）
- `mpv_contract` 13/13（`3.39s`）、`mpv_channel` 10/10（`3.39s`）、`mpv_engine` 11/11（`1.89s`）（均配 `AUTO_MPV_LIB=D:/autostack/tools/mpv/libmpv-2.dll` + `AUTO_SPIKE_VIDEO=E:/Video/caelestia.mp4`）
- `ace_node_desktop_window.exe --features ace-bridge,mpv-widget` 真窗实测（`D:/autostack/plan-artifacts/plan-116/probes/av_window_evidence.json`）：`duration="49.429"`、`last_time="15"`、`paused="true"`、`frames_rendered=10`、`errors=[]`
- 既有无关基线对照记录：`plan105_u03_sfc_status` 2 passed / 1 failed（`p105_u03_apps_sfc_status` 对 `016-calendar` 诊断文本版本差异）

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 1 | from: work | to: blocked | status: blocked | outcome: blocked | worktree: D:/autostack/.wt/ui-116/{auto-ui,auto-lang} | commit: auto-ui 3067ce58 | next: user-decision (§10 B-03)

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 基线复测与 B-03 残余缺口定位 | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/{auto-ui,auto-lang}`（`plan-116-dev`） | base_commit: `auto-ui 3067ce58` / `auto-lang bced4dd79`（由 `8a52d7364` `--ff-only` 快进；`plan-116-dev` 领先 0 提交、树干净） | code_commit: `auto-ui e5ce230f` | task_ids: T-02, T-03 | evidence: `D:/autostack/plan-artifacts/plan-116/probes/t02-recheck-20261124/`

- **基线快进**：`auto-lang` `plan-116-dev` `8a52d7364` → `bced4dd79`（+118 提交，含 117 包面/媒体、118 包面闭合、1124 Dyn/JSON）。`git merge-base --is-ancestor` 通过，快进后树干净。
- **接口适配（`e5ce230f`，2 处编译断裂，均已解）**：① `ace_state_bridge.rs` `UiHttpProvider::http_poll` 改为 `Option<Result<String, HttpFailure>>`（117/1124 的 provider 合约；本宿主只承载已发起请求，失败统一 `HttpFailure::Request`）；② `ace_package_sfc.rs` 对新增 `ComponentStmt::Try` 在扁平重放车道具名拒绝（`source.age.sfc-handler-stmt`），不放宽面。
- **复测绿（`cargo check` 0 error）**：`plan116_av_media_widget` 2/2（0.36s）、`plan116_t01_probe` 3/3（1.53s）、`p116_build_a2vue_sites` 1 passed/1 ignored（20.13s，浏览器门默认 ignore）——快进未回归已交付证据。
- **原样 020/030 包面复测（`pkg-diag-post117.log`）**：020 `app` `expected fn, found widget`、`controls`（空参回调）、`nav_sidebar`/`queue_drawer`/`stage`/`tracklist`/`viewport` `model has no field store`、`player_store` `expected fn, found store`；030 `app` `model has no field store`、`controls`/`viewport` `source.lex: unknown character '$'`、`playlist` `use.web is not a supported import`、`player_store` **OK**、闭包 `on Init assigns undeclared model field playlist`。
- **缺口定位（最小复现，见 §10 R-01..R-05）**：新增诊断入口 `plan116_t02_parse_one_unit`（`PLAN116_UNIT_FILE`）。交叉件 `A-020prefix-030store.at` OK / `B-030prefix-020store.at` 拒 → 020`player_store` 的**项内**构造致整项回退；`E-model-only` OK、`G-on-only` 拒 → 定位在 `on` 块；按处理器二分 → 第 2 个处理器 `.SelectIndex`；`V3/V7/V8` 最小化 → **handler 体 `return`（裸/带值）不支持**（`V6` if 无 return OK、`V1/V2/V4/V5` 具名参数与 `var` OK）；`W1` → 包面 `widget` 项**可**解析（证明 020/app 的报错位置是项首回退、真因在项内）；`U1` → `use web component … from` 在 `component` 处拒。
- **未改**：`auto-lang` 未写入（只做 checkout 快进）；主检出保持干净（本计划只动自己的计划文件）。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 2 | from: blocked | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/{auto-ui,auto-lang} | commit: e5ce230f | next: work/T-02（R-01..R-05）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-01 闭合（handler 裸 `return` 早退） | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/{auto-ui,auto-lang}`（`plan-116-dev`） | base_commit: `auto-lang bced4dd79` / `auto-ui e5ce230f` | code_commit: `auto-lang 05ee4ff8f` + `auto-ui 70a6a424` | task_ids: T-02, T-03 | evidence: `D:/autostack/plan-artifacts/plan-116/probes/r01-return-20261124/`

- **语法面**：新增 `age-on-return`（**只收裸形**；带值 `return <expr>` 仍语法拒），接入 `age-on-stmt` / `age-on-widget-stmt` / `age-on-branch-stmt`，profiles = age-component + age-package。
- **语义**：`ComponentStmt::Return` 只带 span。`hook_stmts_lower` 不发明带值的 `Stmt::Return`（handler 无返回类型），而是把**本序列余量下沉到另一侧分支**——only-then 返回 → 余量成为 `els`；only-els 返回 → 余量接 then 尾；两侧都返回 → 余量不可达丢弃；不取反、不二次求值条件。循环体/`try`/`catch` 体内的 `return` 保持具名拒（fail-closed，不静默丢语句）。
- **消费者补齐**：`cst/project`（投影 `"age-on-return"`）、`frontdoor`（anchor span）、`legacy_record_lists`、`widget_selectors`×2、`age_package` 各 walker；`auto-ui` 扁平重放车道对 `Return` 具名拒（`source.age.sfc-handler-stmt`，与 try/catch 同款）。
- **回归**：新增 `plan116_handler_return` **6/6**（守卫臂正例、顶层正例、带值负控、循环体负控、try 体负控、store 单元整项解析）。`ace-core` conformance **922 passed / 5 failed**；基线 `bced4dd79` 同套 **916 passed / 5 failed**（同 5 项 `text_record_source::*` 既有红）→ **无新增红**，+6 即本轮新测试。`plan981_grammar_scope` 两面 grammar id 按既有"具名重基线"机制更新（`age-component 8507f3e0…→860c39dc…`、`age-package efb064f2…→b2a852c2…`），其余 13 邻居逐字节不变。
- **端到端**：`plan116_t01_package_diag_probe` 中 `020-music-player/player_store` 由 `expected fn, found store`（整项回退）→ **OK**；`PKG_CLOSURE 020` 仍在 `app` 项首回退（R-05 待定位）。
- **未采纳（记入 §10 基线观察）**：`syntax_grammar_data.at` 整份重生成。该文件在计划基线即与语法源不一致（`aac1046_grammar_data_identity_binding` 在未改动的 `bced4dd79` 上红；committed 344,941B vs 原始语法重生成 356,995B）。`ace-core` 由语法**源**编译（`cst/frontdoor.rs` `AUTO_SOURCE_GRAMMAR = include_str!(…)`），AV/auto-ui 面不依赖该数据；采纳重生成会静默重冻结全部 13 个 profile id 并引入未复核漂移。重生成产物留证（`data-with-return.at`）供 Auto 侧孪生 owner 后续一并对齐。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/{auto-ui,auto-lang} | commit: auto-lang 05ee4ff8f / auto-ui 70a6a424 | next: work/T-02（R-02）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-04a + R-07 闭合（跨单元 store 导入；数值 `to_string`） | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`） | base_commit: `auto-lang 05ee4ff8f` / `auto-ui 70a6a424` | code_commit: `auto-lang 78b916591`（`auto-ui` 本轮未改） | task_ids: T-02 | evidence: `plan-artifacts/plan-116/probes/r01-return-20261124/`、`plan116_store_import` 用例

- **最小复现先行**：新增 `plan116_store_import`（2 单元最小包：`store PlayerStore` 单元 + `use player_store: PlayerStore` 消费者单元），把"每个子组件读 `.store.*` / 调 `.store.<Msg>()`"压到可迭代的规模；本轮的每个语法/绑定缺口都在这里先出现、再修。
- **R-04a（`inline_store_decl`）**：只写 `on`、无 `msg` 块的 store（020 真语料形）——按 handler 的参数注解补出消息声明；参数无注解时**具名拒**（不猜 payload 类型）。跨单元导入链（`AgeStoreImports` → `imported_store` → 内联字段前缀/AgeStore 消息限定）在最小包上闭合成 正例。
- **R-07（`resolve.rs`）**：AGE 面数值 `.to_string()` 与既有 `.str()` 同一显示物化节点（`RExprKind::TextDisplay`）——两拼写一语义，不新增 lowering。020 `fmt_clock` / `item_count_label` 依赖此形。
- **回归**：`plan116_store_import` **3/3**（跨单元导入正例、缺 `use … : <Store>` 负控、`var items = []` 无类型 store 字段的**具名缺口**钉住）；`ace-core` conformance **925 passed / 5 failed**（同一 5 项既有红，+3 = 本轮新用例）。
- **R-04b 新开**：`run_component_store_item` 对无 `ty` 的 model 字段 `continue`（按"只读 view-data"处理），内联后 store 自己的 `.playlist = …` 写报 `undeclared model field`。030 真正**写**这个字段，故需要动态/无类型 store 字段语义——本轮只钉住现状，未放宽。
- **诊断口径修正（记入 §10）**：`PKG_UNIT` 行以 `store_imports: None` 解析，**必然**看不到跨单元 store；判断 R-04 只能看 `PKG_CLOSURE`。这解释了本轮 PKG_UNIT 行全无变化而最小包已闭合。
- **未改**：`auto-ui` 无改动；`syntax_grammar_data.at` 仍未采纳重生成（§10 基线观察不变）。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/auto-lang | commit: 78b916591 | next: work/T-02（R-04b/R-02/R-03/R-06）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-04b 闭合（无类型列表 store 字段 → 可变集合） | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`） | base_commit: `auto-lang 78b916591` | code_commit: `auto-lang 174c6646b`（`auto-ui` 本轮未改） | task_ids: T-02 | evidence: `plan116_store_import` 用例（5 例）

- **探路先证**：先加一条"显式 `List` 类型字段"的对照用例，证明包面接受 `var items List = []` 且可写——由此确定 R-04b 不必引入新类型词，只需为 `var x = []` 合成既有 `List` 拼写（元素型走 117 round-20 的 `List(Dyn)` 缺省）。
- **实现（`cst/project.rs`）**：无类型且初值文本恰为 `[]` 的 store 字段投影为可变集合字段（`ty="List"`、`mutable` 取作者 `var`）。关键顺序：**先定型、后投影初值**——无类型初值是 `age-view-data-lit` 原始文本节点，`project_expr_sync` 会报 `expected expression, found '['`；第一版正是踩了这个顺序，改用显式合成空 `ArrayLit` 才通。
- **不放宽**：无类型且非空字面量的 store 字段仍按只读 view-data 跳过，写它继续报 `undeclared model field`（负控用例钉住）。
- **回归**：`plan116_store_import` **5/5**；`ace-core` conformance **927 passed / 5 failed**（同一 5 项既有红，+2 = 本轮新用例）。
- **真实语料效果**：`PKG_CLOSURE 030` 由 `on Init assigns undeclared model field playlist` → **`source.lex: module app::controls: unknown character '$'`**（即 R-03），说明 store 导入 / 字段内联 / store 自身写这条链在真源上已通。`PKG_CLOSURE 020` 仍在 `app` 项首回退（R-05）。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/auto-lang | commit: 174c6646b | next: work/T-02（R-03）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-03 词法/文法半闭合（`$0` 事件实参占位）+ R-03b 新开 | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`） | base_commit: `auto-lang 174c6646b` | code_commit: `auto-lang e528fcb53`（`auto-ui` 本轮未改） | task_ids: T-02 | evidence: `plan116_view_event_arg` 用例（4 例）

- **先判语义再改文法**：确认 AGE 视图回调的实参本来就是**宿主通道**（`age-event-binding` 注释与投影均丢弃实参、载荷由宿主供），所以 `$0` 不需要新求值语义，只缺**词法准入**——这决定了改动只在 token 表 + 两处实参位，而不进 resolve/lowering。
- **改动**：新增 `event-index-tok` 字面 token（`$0`，两面）；`age-event-arg` 增 `alt`（组件面）；新增 `age-child-event-payload`（`$event` / `$0` 两臂）供 `age-child-event-binding`（包面）使用。两面 grammar id 具名重基线（`age-component 860c39dc…→d390e581…`、`age-package b2a852c2…→aa593326…`），其余 13 邻居未变。
- **回归**：`plan116_view_event_arg` **4/4**（两面 parse 正例、handler 体 `$0` 负控、`child-event-site` 缺口钉住）；`plan116_` 全部 **15/15**；`ace-core` conformance **931 passed / 5 failed**（同一 5 项既有红，+4 = 本轮新用例）。
- **真实语料效果**：030 的 `controls`/`viewport` 不再报 `unknown character '$'`；`PKG_CLOSURE 030` 由 R-03 前进到 **R-02（`use.web` in app::playlist）**。
- **R-03b 新开（语义半）**：`video`/`progress` 是受控宿主元素而非声明的子 widget，包面却把 `.Msg($0)` 投影为 `ChildEvent`，语义层因此报 `source.age.child-event-site`。宿主元素上的 `$0` 应等价于裸 `.Msg`。已用 `p116_host_element_event_arg_is_a_named_gap` 钉住当前具名拒绝。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/auto-lang | commit: e528fcb53 | next: work/T-02（R-02）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-10/R-11 闭合（视图 if 条件面 + 包面作者实参回调） | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`） | base_commit: `auto-lang e528fcb53` | code_commit: `auto-lang f59821dec`（`auto-ui` 本轮未改） | task_ids: T-02 | evidence: `plan-artifacts/plan-116/probes/r02-useweb-20261124/`（f1..k6 最小复现）、`plan116_view_if_cond` 用例

- **定位法**：R-02（`use.web`）开工前先确认它是不是唯一余量——把 `playlist.at` 的该行去掉后仍 `expected fn, found widget`，说明后面的缺口被它挡住了。于是用**删构造**法（e1..e4）+ 最小件（f1..k6）逐层二分，把 030/playlist 的余量逼到两条：①`if item.rel_dir == g {`（视图 if 条件的裸 ident 操作数）；②`onclick: .SelectIndex(i)`（包面作者实参回调）。
- **R-10（`age-view-if`）**：条件从裸 `pratt` 改走 `if-cond`（handler 面一直如此）——裸 pratt 把 ident 后的 `{` 当结构字面量吞掉整个分支体，于是整项在 `widget` 处回退。`k5`/`i3` 由拒转 OK。
- **R-11（`age-entry-value`）**：包面在 `$event`/`$0` 占位臂**之后**增通用 `age-event-binding` 臂——作者实参投影为 self 方法调用、实参保留（组件面既有语义）；占位形仍先匹配（回归里专门钉住这条优先级）。
- **回归**：`plan116_view_if_cond` **3/3**；`plan116_` 全 **18/18**；`ace-core` conformance **934 passed / 5 failed**（同一 5 项既有红，+3 = 本轮新用例）；两面 grammar id 再次具名重基线（`age-component d390e581…→cbb9b3bb…`、`age-package aa593326…→33362f8e…`）。
- **效果**：030/playlist（去掉仍不支持的 `use.web` 行）现在整项解析通过；`PKG_CLOSURE 030` 仍停在 **R-02**（该 import 语句本身），020 仍 R-05。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/auto-lang | commit: f59821dec | next: work/T-02（R-02）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-02 闭合（`use.web component` 导入 + 宿主元素准入）；R-12 新开 | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`） | base_commit: `auto-lang f59821dec` | code_commit: `auto-lang 979cd3427`（`auto-ui` 本轮未改） | task_ids: T-02 | evidence: `plan116_web_component` 用例（3 例）

- **先读语料自带的设计**：030 的注释把两端的差别写死了——Vue 端渲染隐藏的 `<input type="file">`，**VM 端该组件不渲染**（`size: 0` 占位、`onready` 从不触发、`local_pick_ready` 恒 false、按钮走诚实降级）。所以 AV 面不需要"渲染 web 组件"，只需要接纳 import 并把该名字的元素当宿主元素。这也避免了为 AV 造一个假组件。
- **改动**：新增 `age-use-web-item`（`use.web component <Name> from "<path>"`，仅 age-package，排在 `use.rs`/`use.c` 臂之前）；`web`/`component` 两个词不是通配——`use.web widget` 仍具名拒（负控）。
- **元素侧零改动**：先写了"元素是否被准入"的用例，结果最小 2 单元包（store + 带 `FilePicker { … }` 的消费者）**直接编译通过**——说明元素早已按宿主元素准入，本轮不需要动视图投影。
- **回归**：`plan116_web_component` **3/3**；`ace-core` conformance **937 passed / 5 failed**（同一 5 项既有红，+3 = 本轮新用例）。
- **grammar 影响面**：只有 age-package 漂移（`33362f8e…→57daa4a6…`）；age-component 与 13 个邻居**逐字节不变**——这验证了规则的 profile 门控是收窄的。
- **真实语料效果**：`PKG_CLOSURE 030` 越过 `use.web`，前进到 **R-12**（`package view cannot observe bracket list model field 'st_groups'`，即内联 store 的 `groups` 在视图里被 `for` 迭代）；020 仍 R-05。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/auto-lang | commit: 979cd3427 | next: work/T-02（R-12）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-06 闭合（显式空参数表视图回调）+ R-12 锐化并钉住 | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`） | base_commit: `auto-lang 979cd3427` | code_commit: `auto-lang 563ee47de`（`auto-ui` 本轮未改） | task_ids: T-02 | evidence: `plan-artifacts/plan-116/probes/{r12-list-view-20261124,r06-empty-args-20261124}/`、`plan116_store_list_view` / `plan116_view_event_arg` 用例

- **R-06（`age-event-binding-args-empty`）**：020 的写法是 `onseek: .SeekFraction()`——**显式空参数表**，与 030 的 `($0)` 同义（不写作者实参=宿主供载荷）。文法增空参臂（两面同开，镜像 118 的 handler 空参数形态）；投影侧把"零作者实参"落成**裸消息引用**而不是零实参方法调用——后者会与消息载荷 arity 相撞。020/controls 由此越过第 148 行。
- **R-12 锐化（未闭合，已钉住）**：先按"store 内联丢失字段信息"假设去查，被最小复现否掉——**widget 自己的 `[]str` 字段同样被拒**；而同一视图循环换成裸 `List` 即通过（对照用例）。所以缺口的形状是"**拼写** `[]T` 未接入视图循环的列表观测通道"，不是 store、也不是列表观测本身。拒绝点在 `FieldBinding::package_field` 的 `[` 前缀守卫；试给 `model_list_type` 增加 `[]T` 分支不改变该路径（视图循环的 iter 仍走 package_field），故不纳入本轮改动（已回退，不留半成品）。
- **回归**：`plan116_` 全 **24/24**（新增 R-06 正例 + R-12 钉住/对照两例）；`ace-core` conformance **940 passed / 5 failed**（同一 5 项既有红，+3 = 本轮新用例）；两面 grammar id 具名重基线（`age-component cbb9b3bb…→9c705b7d…`、`age-package 57daa4a6…→2c7c971e…`）。
- **真实语料**：`PKG_UNIT 020/controls` 越过 `)` 错误（下一处是逐单元预期的 `.store` 限制）；`PKG_CLOSURE 030` 仍 R-12、`020` 仍 R-05。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 9 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/auto-lang | commit: 563ee47de | next: work/T-02（R-12）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-12 根因定位（未闭合）+ R-05 定位（未闭合）；无源代码改动 | outcome: pass（诊断收敛） | worktree_path: `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`） | base_commit: `auto-lang 563ee47de` | code_commit: `auto-lang 42a52c2ab`（仅测试/证据；`auto-ui` 未改） | task_ids: T-02 | evidence: `plan-artifacts/plan-116/probes/{r12-list-view-20261124,r05-020app-20261124}/`、`plan116_app_init_gap` / `plan116_store_list_view` 用例

- **R-12 回栈实测定根因**：临时 panic + backtrace 证明拒绝来自**视图发射**通道 `ViewScope::with_binding`（`age_package.rs:2241` → `rename` → `package_field`），不是检查通道——所以"给检查通道加列表占位"必然无效（实测如此，已回退）。同时确认列表字段**按设计没有状态函数**（`instance_states` 只铸内建类型），故裸 `List` 只是没被 `[` 守卫拦下、会在函数解析期失败：两种拼写在发射期都缺列表访问器。收窄守卫的尝试也回退（只把错误换成 `unknown function __age_state_*`）。
- **R-05 定位**：`.Init` 的触发形是**裸名赋值 + 跨行字符串**（`pins = pins + "…⏎"`）。已排除字符串内容、`var` 声明、`.store` 绑定，并用插桩证明 `unbracketed expression cannot continue across a newline` 守卫**未命中**——真因仍在行敏感的赋值/表达式面。最小复现 + 字段赋值对照已落 `plan116_app_init_gap`。
- **不留半成品**：本轮两处尝试（检查通道占位、收窄 `[` 守卫）全部回退，提交里只有测试与证据。`ace-core` conformance **942 passed / 5 failed**（同一 5 项既有红，+2 = 本轮钉住用例）。
- **下一轮入口**：R-12 需给列表字段一条运行时观测通道（状态函数返回列表 / 视图作用域读 model 二选一）；R-05 用 `plan116_app_init_gap` 从"裸名赋值面为何对跨行 token 行敏感"切。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 10 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/auto-lang | commit: 42a52c2ab | next: work/T-02（R-12 通道 / R-05）

2026-11-24 stage: work | plan_id: PLAN-116 | plan_revision: r1 | checkpoint: T-02 R-03b 闭合（宿主元素事件实参=宿主回调） | outcome: pass | worktree_path: `D:/autostack/.wt/ui-116/auto-lang`（`plan-116-dev`） | base_commit: `auto-lang 42a52c2ab` | code_commit: `auto-lang 7d579aab4`（`auto-ui` 未改） | task_ids: T-02 | evidence: `plan116_view_event_arg` 用例（翻转项）、`plan-artifacts/plan-116/probes/r03b-20261124/`

- **分界线用标签大小写**：AGE 语料里声明的子 widget 一律 CamelCase（`FilePicker`/`Playlist`），受控宿主元素一律小写（`video`/`progress`）。投影据此把**小写标签**上的 `ChildEvent` 改投影为裸消息引用——即"载荷由宿主供给"，与裸 `.Msg` 同义；CamelCase 标签仍走子 widget 事件（既有语义不动）。
- **不加宽**：没有放宽 `child-event-site` 检查本身，只是把"宿主元素上的占位实参"在投影期归到正确的事件类；错误仍对真正的子 widget 误用保留。
- **回归**：`plan116_` 全 **26/26**（R-03b 由钉住翻转为正例）；`ace-core` conformance **942 passed / 5 failed**（同一 5 项既有红）。无文法改动 → 无 grammar id 重基线。
- **真实语料**：030 不再停在 `child-event-site`；其闭包现**只剩 R-12**（`package view cannot observe bracket list model field 'st_groups'`）。020 仍 R-05。
- **R-05 本轮无进展**（尝试在 Auto 侧解析器定位 pratt 行敏感的续行门，未落地即收手，未留半成品）。

stage: goal | plan_id: PLAN-116 | plan_revision: r1 | round: 11 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-116/auto-lang | commit: 7d579aab4 | next: work/T-02（R-12）

## 10. 待澄清事项

**B-03 已裁定（2026-11-24，本计划自有工作树内闭合）。** 原 A/B 选择的前提是"`PLAN-117` 正在 `auto-lang` 同一写面上持续改动"，该前提已随 117/118 双仓落主归档而消失；`README.md` 交付表现指示"在 117 后的基线复测，再按残余缺口处理 B-03"。`/goal` 授权的实施树本就含 `auto-lang`（见 §4），故无需再次拍板：**按残余缺口逐项闭合，目标/验收/范围/仓集不变**。若任一缺口经定位后需要改动已冻结的共享语义或超出本计划仓集，再按 §5 停下请示。

复测基线：`auto-lang bced4dd79`（`plan-116-dev`，原 `8a52d7364`）+ `auto-ui e5ce230f`（`plan-116-dev`）。诊断产物：`D:/autostack/plan-artifacts/plan-116/probes/t02-recheck-20261124/`（`pkg-diag-post117.log`、最小复现 `V3/V7/V8`、`U1`、`A/B` 交叉件）。诊断手段：`plan116_t02_parse_one_unit`（`PLAN116_UNIT_FILE=<path>`）。

| ID | 缺口（最小复现） | 归属 | 现状 |
|---|---|---|---|
| R-01 | handler 体 `return`（`V7-bare-return`、`V8-return-value` 均拒） | `ace-core` cst/grammar + `ComponentStmt` | **已闭合**（2026-11-24，`auto-lang 05ee4ff8f` / `auto-ui 70a6a424`）：`age-on-return` 规则 + 三处语句面 alt（两面同开）；早退按结构化控制流下沉（余量进另一侧分支），不为 return 发明带值 AST 节点；循环/try 体具名拒。回归 `plan116_handler_return` 6/6；`020/player_store.at` 由 `expected fn, found store` 转为 **OK**。 |
| R-02 | `use web component <Name> from "<path>"`（`U1-use-web` @`component`） | `ace-core` cst/grammar（包面 import） | **已闭合**（2026-11-24，`auto-lang 979cd3427`）：新增 `age-use-web-item`（仅 age-package，排在 `use.rs`/`use.c` 臂之前）；`web`/`component` 拼写非通配，错误声明词仍具名拒。元素侧无需改动——`FilePicker { … }` 本就按宿主元素准入；最小 2 单元包编译通过。仅 age-package grammar id 漂移（`age-component` 与 13 邻居逐字节不变）。 |
| R-03 | 视图表达式 `$0`（`source.lex: unknown character '$'`） | `ace-core` lexer/cst | **词法/文法半已闭合**（2026-11-24，`auto-lang e528fcb53`）：新增 `event-index-tok` 字面 token，接入 `age-event-arg`（组件面）与新的 `age-child-event-payload` alt（包面）；两面 grammar id 具名重基线。030 不再停在 `unknown character '$'`。**R-03b 新开**：见下。 |
| R-03b | 受控宿主元素上的 `.Msg($0)` 归类（`source.age.child-event-site: '$event' requires an event binding on a declared child widget`） | `age_package`/投影的宿主事件面 | **已闭合**（2026-11-24，`auto-lang 7d579aab4`）：投影按**标签大小写**分界（AGE 子 widget 为 CamelCase、宿主元素小写）——小写标签上的 `ChildEvent` 改投影为裸消息引用（=宿主供载荷，与裸 `.Msg` 同义）。无文法改动、无 grammar id 重基线。030/viewport 不再触发该检查。 |
| R-04a | 跨单元 store 导入（`use player_store: PlayerStore` + `.store.<field>`/`.store.<Msg>()`） | `packages`/`inline_store_decl` | **已闭合**（`auto-lang 78b916591`）：导入链在包面生效；**只写 `on`、无 `msg` 块**的 store（020 形态）按 handler 参数注解补出消息声明（无注解仍具名拒）。回归 `plan116_store_import` 2 正例 + 1 负控。 |
| R-04b | **无类型/动态 store 字段**（`var playlist = []`，030 写它） | `age_component` store 投影 | **已闭合**（2026-11-24，`auto-lang 174c6646b`）：无类型且初值恰为 `[]` 的 store 字段投影为**可变集合字段**（`ty = "List"`，元素型沿用 117 round-20 的 `List(Dyn)` 缺省）；定型判定先于初值投影（无类型初值是 `age-view-data-lit` 原始文本节点，`project_expr_sync` 不承载）。其余无类型形态仍按只读跳过（负控钉住）。回归 `plan116_store_import` 5/5；**真实效果**：030 闭包由 `undeclared model field playlist` 前进到 R-03。 |
| R-05 | 020 内部未定位构造（`020/app.at` 报 `expected fn, found widget`，但最小 `widget` 包项可解析 → 错误定位在项首，真因在项内） | 待定位 | **已定位（未闭合）**：真因是 `.Init` 里**裸名赋值 + 跨行字符串**（`pins = pins + "…⏎"`，闭合引号独占一行）。最小复现 `plan116_app_init_gap`：该形→拒绝；同一字符串写在 `.field = …` 位→通过。已排除：字符串内容、`var` 声明、`.store` 绑定、`unbracketed expression cannot continue across a newline` 守卫（插桩实测未命中）。 |
| R-12 | 包面视图**观察括号列表 model 字段**（`var groups []str = []` → `for g in .store.groups`） | `age_component` 视图观测 | **阻（根因已定位）**：最小复现证明与 store 内联**无关**（widget 自己的 `[]str` 同样被拒），与**拼写**有关（同一视图循环换裸 `List` 即通过）。**回栈实测**（临时 panic + backtrace）显示拒绝来自**视图发射**通道 `ViewScope::with_binding`（`age_package.rs:2241`→`rename`→`package_field`），**不是**检查通道；而列表字段按设计**没有状态函数**（`age_component.rs` 的 `instance_states` 只按内建类型铸），所以 `[]T` 与裸 `List` 在发射期都缺列表访问器——裸 `List` 只是没被守卫拦下、会在函数解析期才失败。闭合需给列表字段一条运行时观测通道（状态函数返回列表，或视图作用域改读 model），属跨层改动。已试两步均回退：给检查通道加列表占位（不影响发射通道）、收窄 `[` 守卫（只把错误换成"未知状态函数"）。`plan116_store_list_view` 两例钉住边界。 |
| R-06 | 视图**空参回调**（`020/controls.at` `expected expression, found ')'` @148） | `ace-core` cst/grammar（view 属性位） | **已闭合**（2026-11-24，`auto-lang 563ee47de`）：`age-event-binding` 增显式空参数表臂（`age-event-binding-args-empty`，两面同开，镜像 118 的 handler 空参数）；**零作者实参**投影为裸消息引用（宿主供载荷），不是零实参方法调用——否则与消息载荷 arity 相撞。020/controls 越过该行。 |
| R-10 | 视图 `if` 条件取**裸标识符**操作数（`if item.rel_dir == g {`） | `age-view-if` 条件面 | **已闭合**（2026-11-24，`auto-lang f59821dec`）：条件从裸 `pratt` 改走 `if-cond`（与 `age-on-if` 同一条件面）——此前 ident 后的 `{` 被当结构字面量吞掉整个分支体，整项回退。030/playlist 的分组过滤形。 |
| R-11 | 包面视图回调的**作者实参**（`onclick: .SelectIndex(i)`） | `age-entry-value`/`age-event-binding` | **已闭合**（`f59821dec`）：包面 `age-entry-value` 在占位臂之后增通用 `age-event-binding` 臂（`$event`/`$0` 仍先匹配），作者实参投影为 self 方法调用、实参保留（与组件面同语义）。 |
| R-07 | 数值 `.to_string()`（020 `fmt_clock`/`item_count_label`） | `ace-core` resolve | **已闭合**（`78b916591`）：AGE 面数值 `.to_string()` 与既有 `.str()` 同一显示物化节点（`RExprKind::TextDisplay`），不新增语义。**注意**：float 显示格式（`math.floor` 结果 → "3" 还是 "3.0"）须在 T-03 真窗按 020 时钟文案复核。 |

**诊断口径提醒**：`plan116_t01_package_diag_probe` 的 `PKG_UNIT` 行走 `parse_package_source_age(…, store_imports: None)`，**永远**看不到跨单元 store（因此 `.store.*` 在逐单元行里必然报 `model has no field store`）；跨单元导入只能看 `PKG_CLOSURE`。判断 R-04 是否推进须看闭包行，不看逐单元行。

**基线观察（非本计划引入，未修）**：`experimental/ace/aac/syntax/syntax_grammar_data.at` 在计划基线上**已与 `grammars/auto-source.at` 不一致**——`aac1046_grammar_data_identity_binding` 在未改动的 `auto-lang bced4dd79` 上即红（committed 344,941B vs 原始语法重生成 356,995B，约 12KB 漂移）。因 `ace-core` 由语法**源**编译（`cst/frontdoor.rs` `AUTO_SOURCE_GRAMMAR = include_str!(…)`），`auto-ui`/AV 面不依赖该数据文件，故本轮只改语法源与 Rust 侧，**不采纳**整份重生成（否则会静默重冻结全部 13 个 profile id 并引入未复核的上游漂移）。该漂移属 Auto 侧孪生（aac）的数据归属，需其 owner 重生成时一并纳入 `age-on-return`；证据 `plan-artifacts/plan-116/probes/r01-return-20261124/{control-data.posix.log,data-with-return.at}`。

**下一步**：R-12（列表字段运行时观测通道，现为 030 闭包唯一首阻）与 R-05（020 裸名赋值的行敏感面）。每项一个最小正例 + 负控；两者闭合后重跑 `plan116_t01_package_diag_probe` 与 `plan116_t01_probe`，再进 T-03 真窗实跑原样 020/030。
