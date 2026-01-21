# 统一样式系统 - 快速开始指南

## 简介

AutoUI 统一样式系统提供了一种跨后端（GPUI、Iced）的统一样式语法，使用 Tailwind CSS 风格的类名。

## 基本用法

### 1. 解析样式字符串

```rust
use auto_ui::style::Style;

// 解析 Tailwind 风格的样式类
let style = Style::parse("p-4 gap-2 bg-white flex items-center").unwrap();

// 或者使用 From trait
let style: Style = "p-4 bg-white flex".into();
```

### 2. 应用到组件

#### GPUI 后端

```rust
use auto_ui::style::{Style, gpui_adapter::GpuiStyle};

let style = Style::parse("p-4 bg-white flex").unwrap();
let gpui_style = GpuiStyle::from_style(&style);

// 使用 gpui_style 参数应用样式...
if let Some(padding) = gpui_style.padding {
    // 应用内边距
}
```

#### Iced 后端

```rust
use auto_ui::style::{Style, iced_adapter::IcedStyle};

let style = Style::parse("p-4 bg-white rounded").unwrap();
let iced_style = IcedStyle::from_style(&style);

// 使用 iced_style 参数应用样式...
if let Some(padding) = iced_style.padding {
    // 应用内边距
}
```

## 支持的样式类（L1 核心）

### 间距
- `p-{0-12}` - 内边距（例如：`p-4`, `p-6`）
- `gap-{0-12}` - Flex 项目间距

### 颜色
- `bg-{color}` - 背景颜色（例如：`bg-white`, `bg-blue-500`）
- `text-{color}` - 文本颜色（例如：`text-white`, `text-slate-500`）

支持的颜色：
- 灰度：`slate`, `gray`, `zinc`, `neutral`
- 彩色：`red`, `blue`, `green`, `yellow`
- 基础：`white`, `black`

### 布局
- `flex` - Flex 容器
- `flex-row` - 横向布局
- `flex-col` - 纵向布局
- `items-center` - 垂直居中
- `justify-center` - 水平居中
- `justify-between` - 两端对齐

### 尺寸
- `w-full` - 全宽
- `w-{0-12}` - 固定宽度
- `h-full` - 全高
- `h-{0-12}` - 固定高度

### 圆角
- `rounded` - 圆角

## 示例

### Counter 组件

```rust
let style = Style::parse("p-5 bg-blue-500 text-white rounded flex items-center gap-2").unwrap();
```

### 按钮样式

```rust
let style = Style::parse("p-2 bg-white text-slate-500 rounded w-full").unwrap();
```

### 布局容器

```rust
let style = Style::parse("flex flex-col items-center justify-center gap-4 h-full").unwrap();
```

## 运行验证示例

```bash
cargo run --package auto-ui --example style_demo
```

## 运行测试

```bash
cargo test --package auto-ui --lib style
```

## 架构

```
Tailwind 类名
       ↓
   StyleParser
       ↓
  StyleClass (IR)
       ↓
   ┌────┴────┐
   ↓         ↓
GPUI     Iced
Adapter  Adapter
   ↓         ↓
GPUI     Iced
Styles   Styles
```

## 在 Auto 语言中使用

```auto
col {
    button {
        onclick: Msg.Click
        label: "Click Me"
        style: "px-4 py-2 bg-blue-500 text-white rounded"
    }
    style: "p-5 bg-gray-100 flex items-center"
}
```

## 下一步

- 查看 [style-system-mvp-report.md](style-system-mvp-report.md) 了解 MVP 实现详情
- 查看 [style-capability-matrix.md](style-capability-matrix.md) 了解特性规划
- 查看 [004-unified-styling-system.md](../plans/004-unified-styling-system.md) 了解完整实施计划
