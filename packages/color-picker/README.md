# color-picker（PLAN-088）

AutoUI 官方通用选色包：`ColorPicker`（内嵌受控选色器）与 `ColorDialog`
（组合 `ColorPicker` 的模态对话框）。sRGB 8 位 RGBA、HEX/RGB/HSV 编辑、
色板、当前/新颜色预览；实时预览与最终确认分离，组件不直接写业务文档。
native VM 与 vue 双轨同义。

## 快速消费

```at
use auto.json

widget Settings {
    msg { InkPreview(str), InkCommitted(str), InkCancelled, InkError(str) }
    model {
        var ink str = "#3974E6CC"
        var ink_rev int = 1
        var palette List = ["#ef4444", "#22c55e", "#3b82f6"]
    }
    use { package: color_picker from "<本包目录>" }
    view {
        ColorPicker (value: .ink, revision: .ink_rev, palette: .palette,
            onpreview: .InkPreview($event), oncommitted: .InkCommitted($event),
            oncancelled: .InkCancelled, onerror: .InkError($event)) {}
    }
    on {
        .InkCommitted(ev str) -> {
            // 受控回路：校验 baseRevision 后原子更新正式值
            var req = json.parse(ev)
            .ink = req.value
            .ink_rev = .ink_rev + 1
        }
    }
}
```

vue 轨消费需在 `pac.at` 声明 `dep color_picker { path: "<本包目录>" }`
（auto-man 物化按 `<dep.path>/pac.at` 读取）。可运行完整示例：
`examples/color-picker/`（native `auto run -r vm`；vue `auto run`）。

## props（r2/r3 合同）

| prop | 类型/默认 | 说明 |
|---|---|---|
| `value` | `str = "#000000FF"` | 受控正式值（canonical `#RRGGBBAA`；兼容 `#RGB`/`#RRGGBB`，输出固定 8 位） |
| `revision` | `int = 0` | 调用方修订；外部 value/revision 变化经 Tick（100ms）重基线草稿并失效旧手势 |
| `showAlpha` | `bool = true` | 为 false 时只隐藏 alpha 编辑，**不丢失**已传入 alpha |
| `palette` | `List = []` | 应用传入色板（规范化为 8 位后按序去重；空色板可正常选色） |
| `disabled` | `bool = false` | 禁用：取消活动手势并禁止后续提交 |
| `readOnly` | `bool = false` | 只读：同上，仅呈现 |
| `open` | `bool = false` | 仅 ColorDialog：由父组件控制 |
| `title` | `str = "选择颜色"` | 仅 ColorDialog |

## 事件（msg 引号变体，载荷 JSON 字符串）

| 事件 | 载荷 | 触发 |
|---|---|---|
| `"preview"(str)` | `{seq, baseRevision, value, source}` | 每次合法草稿变化（不写文档） |
| `"committed"(str)` | `{seq, baseRevision, value, source}` | Enter/手势收笔/色板命中/底栏确认；**同值零事件** |
| `"cancelled"` | 无 | Escape 撤销 HEX 草稿；Dialog 取消/关闭 |
| `"error"(str)` | `{code, message}` | 用户草稿错误（`invalid_color`/`invalid_channel`）；props 非法为 `invalid_config`（可见文本 + 不回落黑色假成功） |

`source` ∈ `hex` / `rgb` / `hsv` / `palette` / `surface`（画布手势）/
`confirm`（Dialog 底栏）。错误不发有效 `preview`/`committed`。

## 颜色语义（单一整数引擎 `color_model.at`）

- canonical 为规范化 8 位 `#RRGGBBAA`；输出大写 8 位 HEX。
- 通道：RGB/alpha 整数 0…255；HSV 为 H 0…360、S/V 0…100；360 等价 0。
- 输入兼容 `#RGB` / `#RGBA` / `#RRGGBB` / `#RRGGBBAA`（`#RGBA` 按位展开为 8 位），
  输出固定 8 位；非法文本保留草稿并具名错误，不发有效 preview/commit。
- 转 8 位按最近整数、恰好一半向上；`#3974E6CC` ⇒ RGBA(57,116,230,204)。
- HSV 在当前草稿变灰/黑时保留最近有效 H（`h_hold` 串字段，跨 handler 可靠）；
  打开灰/黑初始值（Init 或外部重基线）则 H=0。
- 无 HDR/广色域/CMYK/ICC、跨屏取色、渐变、全局色板存储。

## 通道输入（r3：内置 input + cp 引擎校验）

通道框为内置 `input`（HEX 与 R/G/B/A、H/S/V），不内嵌 `NumericField`
（depth-3 实例缺口见 `docs/reviews/plan088/decisions.md` §7）。合法输入即时
应用并 `preview`（source=rgb/hsv），Enter 提交一次；非纯数字或越界保留草稿
并具名 `invalid_channel`，不发有效 preview/commit。HEX 非法为
`invalid_color`。外部置值（重置/父组件推进）经 Tick 重基线草稿并失效旧手势。

## 交互与提交

- 色面/色相条/alpha 条为 canvas 手势，begin/update/commit/cancel；一次
  pointer 手势或 Enter 最多一条 `committed`，同值不发。
- 色面坐标：去边框局部 x→S 0..100，y 向下→V 100..0，越界夹取；色相条
  左 0 右 360；alpha 条左 0 右 255。手势与数值框共用同一量化。
- `disabled`/`readOnly` 取消活动手势并禁止后续提交。
- 主题只影响控件外观：同一色值在浅/深主题输出相同 RGBA。

## ColorDialog

- 打开以当前 `value` 建草稿；内部 Picker 的 `committed` 只更新本地草稿，
  底栏「确认」才向父提交一次（source=confirm，与外部同值零提交）。
- 「取消」/关闭/Escape 结束预览，不改正式值，不发 committed。
- 外部 value/revision 在编辑中变化时重基线草稿并失效旧手势。
- 沿既有 modal 输入隔离与关闭恢复；确认栏固定可达，窄窗正文竖向滚动。

## 平台边界

- native `input` 无 `onblur`：提交统一走 Enter / 手势收笔 / 色板命中；
  Escape 绑定在 HEX 输入上，不绑全局键。
- 初始焦点：Dialog 打开时由既有 modal 焦点事务聚焦内容区首个输入（本包视图
  首输入为 HEX），关闭后恢复打开前的焦点；包内不另发 `ui.focus`。
- canvas 8 位 HEX alpha 由 runtime 按**串长**判别（`#RRGGBBAA` 不被
  `Color::Hex(u32)` 丢 alpha）；行/条半字节与 rgb6 截取已在包内修正。
- 组件最小布局宽约 364px（色面/条列 192 + 间隙 + 通道列 ≥160）；示例页在窄窗
  纵向堆叠两个区段，640x560 native 与 420x560 vue 均可读（产品门在案）。

## 运行与验证

- 示例：`examples/color-picker/`（Inline ColorPicker + ColorDialog 双实例）。
- 产品门：`tests/product/verify_color_picker.py --auto-bin <cli> --ui-tool-bin
  <tool> --out-dir <isolated> --backend native|vue`；工具门：
  `cargo test -p auto-ui-tools --test color_picker_component`。报告记录
  提交 SHA 与被测 .at/门脚本的 sha256 前缀；native 覆盖 HEX 短形式/精确通道/
  同值零事件/非法错误、showAlpha 保 alpha、Paint 像素读回、窄窗浅深与重开。
- 证据与决策：`docs/reviews/plan088/decisions.md`；原始 gate 输出在
  `D:/autostack/plan-artifacts/plan-088/`。

## 支持矩阵（交付时实证口径）

| 轨 | Windows | Linux | macOS |
|---|---|---|---|
| native VM | ✅ Windows 产品门通过（HEX/RGB/HSV/alpha、手势/对话框、Paint 像素、窄窗浅深、退出重开） | not_run（本组无 Linux runner，不冒 pass） | 不宣称 |
| vue（浏览器） | ✅ Windows 产品门通过（拖拽/HEX/通道/浅深/窄窗截图；playwright chromium） | not_run | 不宣称 |
