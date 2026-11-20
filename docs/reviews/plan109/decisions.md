# PLAN-109 实施决定

实施基线为 `15fa7857`，包含 091 交付 `c4588a88`。AutoLang、AutoWeb、AutoDown 和 AutoMan 分别固定为 `94ac2d062`、`26d085f7`、`72b09700` 和 `4a30483a`。工作树为 `D:/autostack/.wt/ui-109/auto-ui`。

| 接缝 | 实测与处理 |
|---|---|
| 包导入 | 独立的 CollectionView 与 Thumbnail 经 Rust、Vue 生成。完整包移至第二绝对根后生成，取走包时严格生成具名失败。Windows VM 产品探针在隔离目录解析与运行，按 091 的现有测试规程把消费入口指向绝对包目录。 |
| 原生多层事件 | 修复前按重名 B，`Inner.changed` 到 `Middle.Forward`，App 仍显示 `waiting`。修复 `5c9df17c` 以实际 receiver 的父绑定继续处理同步重发。修复后同一控件经过三跳，App 显示 B。永久用例覆盖当前接收者、同型另一实例、未绑定、释放后迟到与直接回调。 |
| Thumbnail | 原生通过 `image.thumb` 票据触发真实解码与失败状态。浏览器使用授权 URI。子视图以资源 ID、revision、URI、状态和 fit 为 key，切换时重新建立图片和事件绑定。切换后已脱离 DOM 的旧图片保留自己的事件接收方。 |
| 实例与结果身份 | Windows Vue 生成器会在 props 变化时重跑 Init。首次初始化后保留 mount token，FileManager 在发布结果时推进 revision，并按文件路径重新解析当前序号。排序与切换模式不更换有效选择。 |
| 布局与键盘 | Grid 按 640 和 1024 逻辑像素在一、二、三列之间选择，方向键读取同一断点所用的窗口宽度。原生沿宿主 `vm.window_inner_width`，Vue 读取浏览器视口宽度。 |
| 四格平台覆盖 | 用户批准 Windows 原生和 Vue 为本轮必验，Linux 原生和 Vue 随统一 Linux 开发补跑，见 [DEBT-011](../../plans/debts.md#debt-011-collectionview-的-linux-平台验收)。 |

原生嵌套复现在 `D:/autostack/plan-artifacts/plan-109/t01/distinct/`。修复前 `current-result.txt` 的 SHA256 为 `FB4E26F128E262BCD4F7BEE86B699E150D826999479125DA435393AB0C532516`。首次修复后探针已读回 B，紧邻退出的 HTTP 探针收到 connection reset。后续独立组件门实测两次正常退出后的旧端口均拒绝同一 MCP 方法调用。
