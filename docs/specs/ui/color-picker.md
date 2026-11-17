# 颜色选择器（color-picker）

> PLAN-088 新增合同。状态：拟议（随 Plan 交付沉淀，merge 落主）。
> 分类：ui ｜ 来源计划：docs/plans/088-color-picker-and-dialog.md

## 1. 合同

官方包 `packages/color-picker/` 提供 `ColorPicker`（内嵌受控选色器）与
`ColorDialog`（组合 `ColorPicker` 的模态对话框）。sRGB 8 位 RGBA；HEX/RGB/HSV
编辑、色板、当前/新颜色预览；实时预览与最终确认分离。组件不直接写业务文档，
正式颜色与撤销历史由调用方持有。native VM 与 vue 双轨同义，MCP 能以数值/
文本精确输入而不依赖色面拖动。

- **颜色模型**：canonical `#RRGGBBAA`（大写 8 位）；输入兼容 `#RGB`/`#RGBA`/
  `#RRGGBB`/`#RRGGBBAA`（`#RGBA` 按位展开），非法文本保留草稿并具名错误。
  RGB/alpha 整数 0…255；H 0…360（360 规范化 0）、S/V 0…100；转 8 位按最近
  整数、恰好一半向上。`#3974E6CC` ⇒ RGBA(57,116,230,204)。HSV 在当前草稿
  变灰/黑时保留最近有效 H；打开灰/黑初始值则 H=0。
- **透明度**：默认支持 alpha；`showAlpha=false` 只隐藏编辑，不丢失已传入
  alpha；不提供隐式“强制不透明”。透明预览用棋盘底，不用黑底伪装。
- **色面/条**：去边框局部 x→S 0..100（整数 floor；端点 100）、y 向下→V
  100..0（端点 100/0）、越界夹取；色相条左 0 右 360、alpha 条左 0 右 255。
  canvas 手势 begin/update/commit/cancel；一次 pointer 手势或 Enter 最多一条
  committed，同值零事件；手势内的色相在按下时固定并沿用到收笔。手势与数值框
  共用同一量化。
- **通道输入**：内置 `input` + 引擎夹取校验（不内嵌 NumericField）；合法输入
  即时应用并 preview（source=rgb/hsv），Enter 提交一次；非纯数字或越界保留
  草稿并具名 `invalid_channel`，不发有效 preview/commit；HEX 非法为
  `invalid_color`。外部 value/revision 变化经 Tick（100ms）重基线草稿、失效
  旧手势并可见化非法配置（`invalid_config`）。
- **Dialog**：`open`/正式 value 由调用方持有，Dialog 只持草稿；打开以当前受控
  值建草稿；内部 Picker 的 committed 只更新本地草稿，底栏确认向父提交一次
  （source=confirm，同值零提交）；取消/关闭/Escape 结束预览、不改正式值、
  不发 committed，且每次打开会话只应消费一次取消通知（平台在按钮取消时会
  额外触发一次 dialog onclose，消费方按会话去重，示例与 README 记录）。
  打开时由既有 modal 焦点事务聚焦内容区首个输入（即 HEX）；关闭恢复既有焦点。
- **调用方数据**：色板与最近颜色由调用方持有，组件不写全局偏好；palette 规范
  化后按序去重；空色板可正常选色。同一视图两个实例互不影响，旧手势/旧绑定
  不作用于当前实例。
- **主题**：只影响控件外观，不改变内容色；浅深主题下同一色值输出相同 RGBA。

非目标：HDR/广色域/CMYK/ICC 色彩管理、跨屏取色、渐变、全局色板存储、重写
主题系统或图像处理、调用系统颜色弹窗。

## 2. 验收锚点

| ID | 锚点 |
|---|---|
| V-01 | 真实包与颜色输入：`#3974E6CC`⇒57/116/230/204；短 `#F00`⇒`#FF0000FF`；非法 HEX/通道具名拒绝且保留草稿、零提交；同值零事件 |
| V-02 | 手势/Dialog：一次手势一次提交；preview 不写正式值；取消零提交；确认一次；外部重置撤销旧草稿 |
| V-03 | 两实例隔离、业务拒绝保留正式值、modal 背景不可操作、旧 canvas/旧绑定不生效 |
| V-04 | Paint 普通选色：确认值=落格像素值；保存/读回保留；取消不改文档与历史；旧类串样本可读 |
| V-05 | Win native + Win vue 真实窗口/浏览器：色面/数值两入口同值、浅深主题、窄窗（native 640×560、vue 420×560）、正常退出重开；Linux 轨 not_run（无 runner，不冒 pass） |
| V-06 | `showAlpha=false` 隐藏编辑但保留 alpha；H360≡H0；灰草稿保留最近有效 H；色面端点 S/V=100 |

门：`python tests/product/verify_color_picker.py --auto-bin <cli> --ui-tool-bin
<tool> --out-dir <isolated> --backend native|vue`；工具门：
`cargo test -p auto-ui-tools --test color_picker_component`。报告记录 revision、
实现提交、输入 sha256 前缀；原始证据在 `D:/autostack/plan-artifacts/plan-088/`
（merge closeout 折叠并删除）。

## 3. 平台边界与依赖

- native `input` 无 `onblur`：提交统一走 Enter / 手势收笔 / 色板命中；Escape
  绑定在 HEX 输入上（撤销未提交草稿），不绑全局键。
- native 模态打开期间截图工具报窗口零尺寸（平台限制）；对话框状态以结构断言
  与 vue 截图取证。
- 初始焦点依赖既有 modal 焦点事务（内容首个 input = HEX），包内不另发
  `ui.focus`。
- 组件最小布局宽约 364px（色面/条列 192 + 通道列 ≥160）；更窄视口由调用方
  页面负责横向滚动或纵向堆叠（示例页窄窗纵向堆叠）。
- 依赖：自足引擎 `color_model.at`（模块级纯 fn，双轨内联）；canvas 场景契约
  （PLAN-563/661）；不依赖 NumericField 实例。PLAN-087 仅作语义基准。
- 登记债：VM 实例作用域整型字段写可见性（当前草稿设计不再依赖整型跨 handler
  状态）；Package 级取消通知可能双发（消费方按会话去重）。
