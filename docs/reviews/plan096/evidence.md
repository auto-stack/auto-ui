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

真实宿主成功启动 MCP，但窗口没有绘制。普通窗口、全屏、标准私有 rqhost 的结果一致。系统句柄对照发现标题为 `Auto - Desktop` 的真实窗口处于隐藏态。当前二进制的 MCP snapshot 和 VTree 均未取得首帧。恢复共享子组件默认值旧逻辑的对照二进制也复现隐藏窗口。在对照上调用非激活显示、改用 DX12 均未产生可操作界面。探针停止了本轮拥有的进程并验证旧 MCP 端口拒绝连接。

关键原始产物位于 `hosted-8/`、`hosted-prior-props/` 和 `hosted-dx12/`。Windows 宿主动作分支仍需用可正常渲染的 AutoOS 会话验证，AC-05 保持开放。

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

Windows Native VM 12 个产品用例通过，Windows Vue 11 个产品用例通过。模型目标构建为 debug/test，最后一轮准备 41.54 秒。耗时表按独立入口记录实测墙钟，包括产品启动、动作和清理。实际桌面宿主启动门单独开放。

内置 Command 与正式 CommandPalette 同页用例通过。现有 CommandGroup 在真实 Vue 页面保留旧搜索过滤，墙钟 1.992 秒，报告 SHA256 `32f5e479aa5b8a96bcf5e3d9df1d88b43d7f24c48da7bf56e79c73d3ef7199d7`。
