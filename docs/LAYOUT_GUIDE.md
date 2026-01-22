# AutoUI 布局指南

本文档详细说明 AutoUI 的布局系统，包括支持的属性、嵌套规则和最佳实践。

## 目录

- [布局组件](#布局组件)
- [布局属性](#布局属性)
- [嵌套规则](#嵌套规则)
- [自动包装](#自动包装)
- [最佳实践](#最佳实践)

## 布局组件

### col / column (垂直布局)

垂直排列子组件，从上到下。

```auto
col(spacing: 10, padding: 20) {
    text("Item 1")
    text("Item 2")
    text("Item 3")
}
```

**属性**:
- `spacing: <number>` - 子元素之间的间距（像素）
- `padding: <number>` - 容器内边距（像素）
- `style: "<css>"` - Tailwind CSS 样式字符串

### row (水平布局)

水平排列子组件，从左到右。

```auto
row(spacing: 15, padding: 10) {
    text("Left")
    text("Center")
    text("Right")
}
```

**属性**:
- `spacing: <number>` - 子元素之间的间距（像素）
- `padding: <number>` - 容器内边距（像素）
- `style: "<css>"` - Tailwind CSS 样式字符串

### center (居中布局)

将子组件在水平和垂直方向上居中。

```auto
center {
    text("Centered Content")
}
```

### container (容器)

包装单个子组件，提供额外的布局控制。

```auto
container(padding: 20, center_x: true, center_y: true) {
    text("Centered Text")
}
```

**属性**:
- `padding: <number>` - 内边距
- `width: <number>` - 固定宽度
- `height: <number>` - 固定高度
- `center_x: <boolean>` - 水平居中
- `center_y: <boolean>` - 垂直居中
- `style: "<css>"` - 样式字符串

### scrollable (滚动容器)

为内容提供滚动功能。

```auto
scrollable(width: 300, height: 200) {
    col {
        text("Long content...")
        text("Line 2")
        text("Line 3")
    }
}
```

**属性**:
- `width: <number>` - 容器宽度
- `height: <number>` - 容器高度
- `style: "<css>"` - 样式字符串

## 布局属性

### spacing (间距)

控制子元素之间的距离。

```auto
col(spacing: 20) {
    text("Item 1")
    text("Item 2")  // 上面有 20px 间距
    text("Item 3")  // 上面有 20px 间距
}
```

### padding (内边距)

控制容器内部内容与边框的距离。

```auto
col(padding: 30) {
    text("Content with 30px padding on all sides")
}
```

### center_x / center_y (居中)

仅在 `container` 中可用。

```auto
container(center_x: true, center_y: true) {
    text("Perfectly centered")
}
```

## 嵌套规则

### 多级嵌套

可以任意深度嵌套布局组件。

```auto
col(spacing: 10) {
    text("Level 1")

    row(spacing: 5) {
        text("Level 2 - Item 1")
        col(spacing: 5) {
            text("Level 3 - Item 1")
            text("Level 3 - Item 2")
        }
        text("Level 2 - Item 3")
    }
}
```

### 混合布局类型

在一个布局中混合使用不同的布局类型。

```auto
col(spacing: 20) {
    text("Header")

    row(spacing: 10) {
        button("Left")
        button("Center")
        button("Right")
    }

    center {
        text("Middle Content")
    }

    container(padding: 10) {
        text("Footer")
    }
}
```

## 自动包装

### 多个顶级节点

当 `view()` 方法中有多个顶级节点时，它们会自动被包装在一个 `col` 中。

```auto
type MyComponent {
    fn view() {
        // 这3个顶级节点会被自动包装在 col 中
        text("Title")
        text("Subtitle")
        button("Click")
    }
}
```

生成的 Rust 代码：

```rust
fn view(&self) -> View<Self::Msg> {
    View::col().spacing(0).padding(0)
        .child(View::text(&"Title"))
        .child(View::text(&"Subtitle"))
        .child(View::button("Click", 0))
        .build()
}
```

**注意**：单个顶级节点不会被包装。

## 样式系统

AutoUI 支持 Tailwind CSS 风格的样式类。

### 使用 style 属性

```auto
col(style: "p-4 gap-4 bg-white rounded-lg") {
    text("Styled Container")
}
```

### 常用样式类

**间距**:
- `p-{0-12}` - padding
- `px-{0-12}` - padding X (左右)
- `py-{0-12}` - padding Y (上下)
- `m-{0-12}` - margin
- `gap-{0-12}` - gap

**尺寸**:
- `w-full` - width: 100%
- `w-{1-96}` - 固定宽度
- `h-full` - height: 100%
- `h-{1-96}` - 固定高度

**布局**:
- `flex` - flex 容器
- `flex-1` - flex: 1 (填充可用空间)
- `flex-row` - 水平 flex 方向
- `flex-col` - 垂直 flex 方向
- `items-center` - 垂直居中对齐
- `items-start` - 垂直顶部对齐
- `items-end` - 垂直底部对齐
- `justify-center` - 水平居中对齐
- `justify-between` - 两端对齐
- `justify-start` - 左对齐
- `justify-end` - 右对齐

**颜色**:
- `bg-{color}` - 背景颜色
- `text-{color}` - 文字颜色

**边框**:
- `rounded` - 圆角
- `rounded-sm/md/lg/xl/2xl/3xl/full` - 不同大小的圆角
- `border` - 边框
- `border-0` - 无边框
- `border-{color}` - 边框颜色

**文字**:
- `text-xs/sm/base/lg/xl/2xl/3xl` - 字体大小
- `font-light/normal/medium/bold` - 字体粗细

## 完整示例

### 登录表单

```auto
type LoginForm {
    username str = ""
    password str = ""

    fn view() {
        col(spacing: 20, padding: 40) {
            text("Login")

            col(spacing: 10) {
                text("Username:")
                input("Enter username")

                text("Password:")
                input("Enter password")
            }

            row(spacing: 10) {
                button("Login", onclick: 1)
                button("Cancel", onclick: 2)
            }
        }
    }

    fn on(ev int) {
        is ev {
            1 => {
                // 处理登录
            }
            2 => {
                // 取消
            }
        }
    }
}
```

### 仪表板布局

```auto
type Dashboard {
    fn view() {
        col(spacing: 0, padding: 0) {
            // Header
            container(padding: 20) {
                text("Dashboard")
            }

            // Main content
            row(spacing: 20, padding: 20) {
                // Sidebar
                col(spacing: 10) {
                    text("Navigation")
                    button("Home", onclick: 1)
                    button("Settings", onclick: 2)
                }

                // Content
                col(spacing: 15) {
                    text("Welcome!")
                    text("This is the main content area")
                }
            }
        }
    }
}
```

### 带样式的卡片

```auto
type StyledCard {
    fn view() {
        col(spacing: 15) {
            // Card 1
            container(padding: 20) {
                col(spacing: 10) {
                    text("Card Title")
                    text("Card content goes here...")
                    button("Action", onclick: 1)
                }
            }

            // Card 2
            container(padding: 20) {
                col(spacing: 10) {
                    text("Another Card")
                    text("More content...")
                    button("Click", onclick: 2)
                }
            }
        }
    }
}
```

## 最佳实践

### 1. 使用合适的布局类型

- 垂直列表 → 使用 `col`
- 水平工具栏 → 使用 `row`
- 居中内容 → 使用 `center` 或 `container(center_x: true, center_y: true)`
- 长内容 → 使用 `scrollable`

### 2. 合理设置间距

```auto
// 推荐：有层次感的间距
col(spacing: 20) {
    text("Section")        // 大间距分隔区域

    col(spacing: 5) {      // 小间距分隔相关项
        text("Label")
        input("Value")
    }
}
```

### 3. 避免过深嵌套

```auto
// 不推荐：嵌套太深
col {
    row {
        col {
            row {
                text("Too deep!")
            }
        }
    }
}

// 推荐：扁平化结构
col(spacing: 10) {
    text("Item 1")
    text("Item 2")
    text("Item 3")
}
```

### 4. 使用 container 包装单个元素

```auto
// 当需要给单个元素添加 padding 或居中时
container(padding: 20, center_x: true) {
    text("Centered with padding")
}
```

### 5. 利用自动包装简化代码

```auto
// 简洁：多个顶级节点
fn view() {
    text("Title")
    text("Subtitle")
    button("Action", onclick: 1)
}

// 等价于显式使用 col
fn view() {
    col {
        text("Title")
        text("Subtitle")
        button("Action", onclick: 1)
    }
}
```

## 调试技巧

### 查看生成的代码

```bash
# 生成 Rust 代码
cargo run --package auto-ui-transpiler-cli -- file example.at output.rs

# 查看输出
cat output.rs
```

### 常见问题

**问题**: 子组件没有显示
- 检查是否正确嵌套（子组件应该在父组件的 `{}` 内）
- 确保没有遗漏逗号或括号

**问题**: 间距不生效
- 检查 `spacing` 和 `padding` 的值
- 确保属性在正确的组件上

**问题**: 内容被截断
- 考虑使用 `scrollable` 组件
- 检查父容器的尺寸限制

## 性能优化

1. **避免不必要的嵌套**：扁平化结构可以减少渲染开销
2. **使用 keys**：对于动态列表，将来版本可能需要唯一标识
3. **限制重渲染**：合理组织状态，避免不必要的更新

## 相关文档

- [UI 组件参考](UI_COMPONENTS.md) - 所有可用的 UI 组件
- [快速入门](RUN_QUICKSTART.md) - 如何运行 AutoUI 应用
- [命令参考](COMMAND_RUN.md) - CLI 命令使用说明
