# PLAN-096 Windows 验收记录

合同为计划 r3。原始运行产物位于 `D:/autostack/plan-artifacts/plan-096/`。

## 模型、包和原生公共接缝

| 用例 | 结果 | 墙钟秒 |
|---|---|---:|
| `feedback_is_tied_to_request_data_and_mount` | pass | 0.312 |
| `first_invalid_configuration_notifies_once_and_can_recover` | pass | 0.100 |
| `invalid_records_report_missing_id_and_conflicting_group_labels` | pass | 0.158 |
| `local_keeps_selected_identity_and_ignores_disabled_empty_and_old_rows` | pass | 0.138 |
| `omitted_query_keeps_its_default_with_a_same_named_parent_field` | pass | 0.090 |
| `provided_requires_the_current_request_even_when_query_text_repeats` | pass | 0.105 |
| `relative_package_generates_after_relocation` | pass | 1.831 |
| `same_revision_command_change_invalidates_a_rendered_action` | pass | 0.104 |
| `two_same_title_mounts_keep_independent_request_identity` | pass | 0.106 |
| 聚焦的单行 Input 在 preedit 和取消后的 Enter/方向键 | pass | 2.153 |
| 已有模态焦点、Tab、关闭与背景动作保护 | pass | 0.712 / 0.096 |
| 已有子组件默认字段与 Enter 声明派发 | pass | 0.189 / 0.100 内 |

模型用例执行自已预构建的测试目标。移根用例通过正式 Vue 生成并对移走包目录的反例取得具名错误。Native VM 产品用例在第二绝对根使用 `auto run -r vm` 和实际窗口。

## Launcher 业务对照

与基线 `4dd9696d` 通过 Native VM 控件对同一注册表和最近使用数据运行对照，三组完整结果完全一致。

| 输入 | 基线与迁移后相同的结果 |
|---|---|
| 空查询 | 12 项。Notes 与 Calculator 依最近使用顺序置顶 |
| 查询 `to` | Todo、Calculator、Stopwatch、系统监视器、Auto Edit |
| tool 分类与 `to` | Calculator、Stopwatch |

原始对照 `launcher-ranking-parity.json` SHA256 `9befa344931d7f668c2677057697df054b686863f66ac24d8455bba189191765`。

## Windows AutoOS 宿主启动门

宿主以 `cdb4883` 的 `ui-desktop` 入口消费本实施树。使用同组固定依赖及 AutoUI 的两项 iced patch 完成 debug 构建。完整 OS 注册表声明的 017-chat 原生服务源码在隔离树缺席，因此使用只包含 028 Launcher 和 011 Calculator 的隔离注册表核验。

在此机上，首次使用默认渲染器、全屏、私有 rqhost 及恢复共享子组件默认值旧逻辑的对照构建均未绘制首帧。切换到现有 `ICED_BACKEND=tiny-skia` 后，正式 `ui-desktop` 二进制渲染了 `Auto - Desktop` 并发布实时 MCP 树。两轮直接 Windows 合成键盘探针未读回查询，最终用户路径通过已定位的真实窗口操作完成。

启动命令 `python hosted_operator_repair.py hosted-repair-window-capture tiny-skia` 退出 0。先逐文件核验隔离包闭包与实施树一致，再使用壳当前快照找到并按下召唤按钮。Launcher `[4:028-launcher]` 发布 `hosted = "1"`，此时尚无 Calculator 应用面。通过真实 Windows 窗口点击搜索框并输入 `cal`，MCP 嵌入面读回 `q = "cal"` 和 `palette_ids = ["011-calculator"]`。点击一次结果后，宿主日志恰有一行 `[session] launch_app(inproc) 011-calculator`，发布 Calculator `[5:011-calculator]`。同一拥有窗口的 Windows.Graphics.Capture 直接显示 Calculator 标题、模式标签、数字键和运算键。宿主正常退出 0，同一 MCP 方法确认旧端口拒绝连接。100.939 秒为人工操作、等待、窗口截图及清理的总墙钟，独立于 10 秒逐案自动化门。

`hosted-repair-window-capture/report.json` SHA256 `c0a260148a7607e373d371cefda62784be17ecc03cafd6e445e982138de4b653`。过滤后的 Launcher 树 SHA256 `566b2cdcd0d2cd99b56471d605ba0f0ef62ddb1e49194d4847c51504ca426d85`。Calculator 树 SHA256 `98b40b51d71939acf7d2ae301d14f5652332e8e1152b15f08c533877c3804734`，原始窗口截图 `calculator-window.jpg` SHA256 `cebc6a5b1cf3a87871d475e04c142c22f1b2e6ecb8cd7eb0a73d3f2d469811a3`，同像素 PNG 见报告。宿主二进制 SHA256 `07da3fd756170f8b9bd72131eb34bd00571a8b6edf861aad71a8317c5e9ca4e1`，Launcher 源 SHA256 `756a438b7b388e1c842b435c464cfe01c1792f8420c92a2867316e245f6be722`。报告含完整包源码摘要。旧 `hosted-resume-07` 用于诊断对照，包含旧包文件且壳截图不含应用窗，正式验收采用本轮窗口截图。

## Windows 产品执行

`auto run -r vm` 由实施树工具打开 Native VM 窗口。Vue 三个消费入口均经 `auto gen -p`、`vue-tsc --noEmit` 和开发 Vite 准备后，由 Chrome 中的 Playwright MCP 执行。产品报告含 CLI、工具、源码和包闭包摘要，MCP 请求与结果，应用读回，进程及端口清理。Native VM 使用实际 Windows UIA 的 ValuePattern 和 InvokePattern 执行系统树用例。

| 报告目录 | 覆盖 | 结果 | 墙钟秒 | report.json SHA256 |
|---|---|---|---:|---|
| `native-launcher-verified` | C-04 | pass | 5.707 | `1e80a86c3fa2af01ed5cb8daf106528d492c9b8f3103603a3fbc36310c5837a5` |
| `native-provided-pending-verified` | C-06/X-03 | pass | 5.038 | `79801d5b2cda014f49ccabc8fe14a4d8a1ded809f92ed7a1534e36c09907e0dc` |
| `native-provided-verified` | C-03/C-06 | pass | 5.159 | `9214259fc770cf2670fe79f518020be1c2f3dd74960317f6a8e9cb024b064bdb` |
| `native-provided-wrap-verified` | C-02 | pass | 4.892 | `152758543963ecc316a5db9d36b90764327cc0089e3ca024ab10e5854f4404a8` |
| `native-rejected-verified` | C-04/C-06 | pass | 7.069 | `d86323dab6d388c05647aa53683f09fb43fbc194bb735dbf8af5f759c816c3dd` |
| `native-reopen-app-verified` | X-02/X-06 | pass | 8.667 | `c54f0a95dc25b2f7c4b96555ec15233494bfa71f12ae34556371af14b7bf5ba4` |
| `native-reopen-dialog-verified` | C-01/C-02/C-05 | pass | 8.400 | `4bba0df77ac35215c85c09b1466758aa4c5a6b94ac6acd01b5001bd097a54358` |
| `native-scroll-verified` | X-03 | pass | 6.635 | `811660b5b29512d976a27296c75b969d7ef2718c6391cf5186b6de3a345c7a42` |
| `native-settings-verified` | C-01/C-02/C-04 | pass | 8.608 | `f80e66e1d1a2d1433bde73059e7df74891da76d08b506dee20c3ca9cc911ae34` |
| `native-system-input-verified` | X-07 | pass | 8.427 | `d0078101e3011d7e5e86ae4e28697f4b3ddef45473ff8d089ed91d2c758a7b0a` |
| `native-system-invoke-verified` | X-07 | pass | 7.142 | `c291aac2777cdf44c7c41827ce4fa67d2a4f2dc5d9c4a6c524cd8e9508f05c8c` |
| `native-visual-verified` | X-03 | pass | 8.142 | `f974f551932529fe32776225102e163df6dc370a7c108a70e9dfb45109377ba1` |
| `vue-ime-verified` | C-02/X-05 | pass | 3.546 | `ae51137bd02e3080745e78ec926f2409bf85228c12ca8e1d430381b6f8aac529` |
| `vue-launcher-verified` | C-04 | pass | 3.838 | `bc05f571513cbe096ece69dbfa47d8f55a17ca9bbcfe08a552563019d83c47d8` |
| `vue-missing-provider-verified` | X-06 缺浏览器提供方 | not_run | 0.051 | `0bbed29696f7ec3bd70515911d6fd25a5e763638d38e3e13e55212432782fad2` |
| `vue-provided-pending-verified` | C-06/X-03 | pass | 3.401 | `a75cffac27fc6dda2d4cf2fa8e7332864b90dc92144c4e5d8f888af34e66b792` |
| `vue-provided-verified` | C-03/C-06 | pass | 3.896 | `2616fb8630f8ee066a32646f0a74aea5ce7047626240691c1a0bcd1e87082065` |
| `vue-provided-wrap-verified` | C-02 | pass | 2.640 | `bc54a1e14cdec9c3d33772d85e47eec6a5d2ce75a265ef6fb1a2c95f2599d32b` |
| `vue-rejected-verified` | C-04/C-06 | pass | 3.207 | `333088443cfbe3ff49490f716a5057c276a1892965b6a54bcddd76832a163773` |
| `vue-reopen-app-verified` | X-02/X-06 | pass | 4.383 | `2fe626f612f0734c1d88180ee1ecb9cc61fa850fe1f96dbcd12c48c9a911f0e1` |
| `vue-reopen-dialog-verified` | C-01/C-02/C-05 | pass | 4.950 | `b3f0232044c06a114d5343c0ff1072184da0a57e775e49324d63eb50567800fb` |
| `vue-scroll-verified` | X-03 | pass | 3.151 | `683612d2ec147bfd857cf9255f7ba3c33bb2b6540d53c3951859546b7de2b9e4` |
| `vue-settings-verified` | C-01/C-02/C-04 | pass | 4.091 | `047b6ac3e095209161a6428f849cfdf7f9e2bc33c070a8fd343160409fe08328` |
| `vue-visual-verified` | X-03 | pass | 4.279 | `71fc2b9d3fe3dd63a6e6fabbe7a915a48d7c7fcab4538fa1e9949cfd375962dc` |

产品脚本的缺提供方反例退出 2。退出重开分别启动两个 App 进程，前一进程退出及端口释放后再运行当前控件。浅色选中行和深色 420×560 截图人工核对通过，长中文命令在两轨可读。Native VM 保留逻辑窗口尺寸，截图像素受系统 DPI 缩放。

Windows Native VM 12 个产品用例通过，Windows Vue 11 个产品用例通过。模型目标构建为 debug/test，最后一轮准备 41.54 秒。耗时表按独立入口记录实测墙钟，包括产品启动、动作和清理。Windows 桌面宿主单独通过 C-04。

内置 Command 与正式 CommandPalette 同页用例通过。现有 CommandGroup 在真实 Vue 页面保留旧搜索过滤，墙钟 1.992 秒，报告 SHA256 `32f5e479aa5b8a96bcf5e3d9df1d88b43d7f24c48da7bf56e79c73d3ef7199d7`。

## 首轮产品套件与覆盖

将 `examples/command-palette`、`examples/ui/028-launcher`、`tests/product/fixtures/command-palette-provided` 和声明包的源码复制到隔离根，保留清单和源码的相对路径。为各 Vue 项目生成前端、安装依赖、做类型检查，并通过共享 `prompt_dialog_preview.py` 准备 development bundle。首轮聚合入口的结果保留用于诊断，其中新增可执行包装器超出 10 秒，独立复审退回。当前 `verify_command_palette.py` 每次执行一案，`--project-root` 指定隔离根。对 `settings`、`launcher`、`provided` 分别使用 `--prepare` 记录各轨准备指纹。单案可用 `--app-dir` 指定项目目录，运行使用新的 `--out-dir`。

| 门 | 结果 | 墙钟秒 | 报告 SHA256 |
|---|---:|---:|---|
| `native-suite-r3` | 14/14 | 68.962 | `774c20481a265ef1ccbc38dbb40ac4518c0281e233bbb09fd1e43568bc1f7f69` |
| `vue-suite-r3` | 13/13 | 49.472 | `0739178703637170a84b7a551fe592f72f0a76029ec2301bce9103140307e4f6` |
| `vue-missing-provider-suite-r3` | 退出 2（预期 `not_run`） | 0.031 | `52282531a388a32d687463b050bd72e6b42b107da6d32794ab027493ecd93569` |
| `native-nested-r3`，共用嵌套模态用例 | pass | 4.961 | `e19b04fb398dd63896d4544ac4539efeb38d5b6c83515e1a60257ff5c43a7368` |
| `vue-nested-preview-r3`，共用嵌套模态用例 | pass | 4.022 | `603e6244b0385f6a72b692ebe9a8465782b637964372d6ba9665898f894a2a6c` |

套件在独立进程逐项执行，逐案上限 10 秒。全套墙钟合计包含所有启动、操作及清理。最长原生用例为整应用退出重开（8.417 秒），最长 Vue 用例为设置写回（5.553 秒）。同名命令正例通过搜索 `重置` 并选择两个结果，分别读回 General 与 Layout 设置回执，Native VM/Vue 耗时 4.875/3.620 秒。受控失败用例在真实查询后中断，分别在 3.839/2.075 秒留下最后树、释放本轮进程并验证同一入口连接拒绝。Vue 嵌套模态第一次缺已声明的预览收据，补跑既有 `prompt_dialog_preview.py` 后成功；两个平台核对上下层焦点范围及关闭后下层控件可操作。`visual/narrow-dark-cjk.png` 的两轨实图中，输入及长中文禁用行可读、取消按钮可用，本组件用例未复现 Q-01 字形缺失。

| 验收 | 证据 |
|---|---|
| AC-01 | 两轨正常包导入、移根生成及缺依赖具名错误，内置 Command 同页共存 |
| AC-02 | C-01/02/05 和 X-04/05，设置双轨、同名行、禁用空结果、方向键、原生 IME 输入桥、Vue composition 和共享嵌套模态 |
| AC-03 | C-03/06 与 X-04，提供方 A→B→A、同修订旧动作、双挂载、待确认防重和确认后外部修订保护 |
| AC-04 | X-02/05/06/07，两轨当前工具节点及读回、系统树 ValuePattern/InvokePattern、正常重开、缺提供方与动作失败清理 |
| AC-05 | C-04，真实 AutoOS 搜索后启动 Calculator 一次、028 排序对照及无宿主反馈，普通设置两轨归属验证 |
| AC-06 | X-01/03/06/07，移根、视觉、滚动、必要公共模态及旧消费者回归，README 与 SD-01/02 候选 |

## 独立复审退回后的复验

| 发现 | 修复及关闭证据 |
|---|---|
| R096-F01 | 产品入口逐案独立执行，保留此前 68.962 / 49.472 秒聚合失败与 `native-provided-3` 11.614 秒原始报告。当前 Native VM 16/16，逐案 4.185–8.512 秒，逐案耗时和 88.094 秒。Vue 15/15，逐案 2.654–5.580 秒，耗时和 65.955 秒。首次复跑 `launcher` 行为和清理成功，但启动 4.168 秒、操作 1.873 秒、清理 3.885 秒，计入报告后 10.535 秒。退出高负荷阶段后按完全相同入口重放为 5.904 秒，原超时报告保留。 |
| R096-F02 | 配置校验先于 provided 结果归属核验。原生一次组件派发只保存一条公开消息，首次查询与配置错误分开派发。两轨 `provided-config` 在重复 ID 尚无当前结果时显示错误，首次有且仅有一个错误通知，Enter 零调用，修正并提供当前结果后真实写回一次。Native VM 6.435 秒、Vue 4.862 秒。第一次原生修复复跑发现查询与 error 同帧造成初次查询丢失，11.122 秒退出失败，记录 `repair-native-provided-config/report.json`。 |
| R096-F03 | 过期 provided 查询元组或低修订结果不解除当前请求锁。两轨 `provided-stale-pending` 在真实点击后发布旧结果，Escape/Enter 都不提前关闭或重复调用，随后正确读回一次应用。Native VM 5.820 秒、Vue 4.331 秒。模型覆盖真实当前命令变更可使旧动作失效和 accepted 同帧保持锁；原 A→B→A 双轨现行用例均通过。 |
| R096-F04 | 启动前校验准备收据、工具指纹、当前消费源码与包闭包。Vue 校验已有 development preview 的源码与 dist 指纹。Native VM 拒绝会进入 split/Rust 构建的环境。缺收据、不匹配工具、缺浏览器 MCP、Rust 后端环境和替换单个 Vue 资产均在启动前退出 2，无 RPC 文件。分别耗时 0.068、0.255、0.052、0.272、0.223 秒。 |
| R096-F05 | 模型 `provided_reorder_preserves_selection_and_removal_selects_an_enabled_result` 选中 `second`，重排保留 ID，移除后选 `third` 且公开提交正确 ID，0.447 秒。两轨 `rejected.png` 和 `invalid-config.png` 实图均显示具名反馈，拒绝行为可恢复。真实宿主的完整闭包和 Calculator 窗口截图见上节。 |

当前模型用已准备的 debug/test 目标逐个 `--exact` 执行 12/12，0.117–2.730 秒，包含实际读取当前包源码的三个新增闭环。准备目标上一次增量 38.264 秒，另有测试线程栈探针，探针在较大测试栈证明加载成功，未纳入日常门。模型索引 `repair-sealed-model-results.json` SHA256 `a507a05105feeac264ff449bc2a4c3c66c11a44ab7d7ac0cf866218d50cb1b33`。两个正式反例的共享修复边界为配置通知一次、请求身份未被时序污染、修正或当前更新后仍可执行。

`repair-verified-product-index.json` SHA256 `58669d10ebb63ada65dd57e95c6a382af27d026b54af226912bb58ec02be2ec9`，记录 31 个独立报告的耗时、runner 及摘要，核对每份报告的实际工具与 `command_palette.at` 摘要一致。最后 `system-invoke` 改用应用内 Exit 控件，避免紧接 UIA Invoke 的裸 `WM_CLOSE` 与 AccessKit 过期窗口句柄竞态。原失败可见 `0x80070578`，应用状态读回已通过；正向重放 5.249 秒，系统输入及整应用重开仍使用原窗口关闭路径。其余用例的执行语义不受该退出分支变化影响。

磁盘临时耗尽的尝试保留 `repair-product-index.json`、`hosted-repair-old-control` 和相应日志。该轮多项未启动及浏览器退出失败，没有作为通过证据。已正常重放当前受影响产品门。为源码边界留可重放证据，原始 Windows 窗口截图和新旧报告均在单一 Plan scratch scope。
