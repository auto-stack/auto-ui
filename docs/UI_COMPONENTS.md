# AutoUI 组件参考

本文档展示了 AutoUI 中所有可用的 UI 组件及其语法。

## 布局组件

### col / column (垂直布局)
```auto
col {
    text("Item 1")
    text("Item 2")
}
```

带属性:
```auto
col(spacing: 10, padding: 20) {
    text("Item 1")
    text("Item 2")
}
```

### row (水平布局)
```auto
row {
    text("Left")
    text("Right")
}
```

带属性:
```auto
row(spacing: 10, padding: 20) {
    text("Left")
    text("Right")
}
```

### center (居中布局)
```auto
center {
    text("Centered content")
}
```

### container (容器)
```auto
container(padding: 20) {
    text("Content")
}
```

### scrollable (滚动容器)
```auto
scrollable {
    text("Scrollable content")
}
```

## 基础组件

### text / label (文本)
```auto
text("Hello, World!")
label("This is a label")
```

动态文本:
```auto
type MyComponent {
    message str = "Hello"

    fn view() {
        col {
            text(self.message)  // 显示字段值
            text(count)         // 自动转换为字符串
        }
    }
}
```

### button (按钮)
```auto
button("Click me", onclick: 1)
```

带事件处理:
```auto
type Counter {
    fn view() {
        col {
            button("Increment", onclick: 1)
            button("Decrement", onclick: 2)
        }
    }

    fn on(ev int) {
        is ev {
            1 => {
                // 处理 Increment 点击
            }
            2 => {
                // 处理 Decrement 点击
            }
        }
    }
}
```

## 表单组件

### input (文本输入)
```auto
input("Enter text...")
```

带占位符:
```auto
input("Username")
input("Password")
```

### checkbox (复选框)
```auto
checkbox("Accept terms", is_checked: false)
checkbox("Subscribe", is_checked: true)
```

### radio (单选按钮)
```auto
radio("Option 1", is_selected: true)
radio("Option 2", is_selected: false)
radio("Option 3", is_selected: false)
```

### select (下拉选择)
```auto
select("Default option")
```

## 高级组件

### list (列表)
```auto
list(spacing: 10) {
    text("Item 1")
    text("Item 2")
}
```

### table (表格)
```auto
table(spacing: 5, col_spacing: 10) {
    // 表格内容
}
```

## 完整示例

### 计数器组件
```auto
type Counter {
    count int = 0

    fn view() {
        col {
            text("Counter")
            text(count)
            row {
                button("+", onclick: 1)
                button("-", onclick: 2)
            }
        }
    }

    fn on(ev int) {
        is ev {
            1 => {
                count += 1
            }
            2 => {
                count -= 1
            }
        }
    }
}
```

### 表单组件
```auto
type Form {
    username str = ""
    email str = ""
    accept_terms bool = false

    fn view() {
        col {
            text("User Registration")

            text("Username:")
            input("Enter username")

            text("Email:")
            input("Enter email")

            checkbox("I accept the terms", is_checked: self.accept_terms)

            button("Submit", onclick: 1)
        }
    }

    fn on(ev int) {
        is ev {
            1 => {
                // 提交表单
            }
        }
    }
}
```

### 布局示例
```auto
type LayoutDemo {
    fn view() {
        col(spacing: 20, padding: 10) {
            text("Header")

            row {
                text("Left")
                text("Center")
                text("Right")
            }

            center {
                text("Centered Content")
            }

            container(padding: 10) {
                text("Container Content")
            }
        }
    }
}
```

## 属性参考

### 通用属性
- `spacing: <number>` - 子元素间距
- `padding: <number>` - 内边距
- `style: "<css>"` - CSS 样式

### Button 属性
- `onclick: <value>` - 点击事件消息值

### Checkbox 属性
- `is_checked: <boolean>` - 是否选中

### Radio 属性
- `is_selected: <boolean>` - 是否选中

### Input 属性
- `value: <string>` - 输入值
- `placeholder: <string>` - 占位符文本

### Container 属性
- `padding: <number>` - 内边距
- `width: <number>` - 宽度
- `height: <number>` - 高度
- `center_x: <boolean>` - 水平居中
- `center_y: <boolean>` - 垂直居中

### Scrollable 属性
- `width: <number>` - 宽度
- `height: <number>` - 高度

## 事件处理

AutoUI 使用消息传递模式处理事件：

```auto
type MyComponent {
    fn view() {
        col {
            button("Action 1", onclick: 1)
            button("Action 2", onclick: 2)
            button("Custom", onclick: 99)
        }
    }

    fn on(ev int) {
        is ev {
            1 => {
                // 处理 Action 1
            }
            2 => {
                // 处理 Action 2
            }
            99 => {
                // 处理自定义操作
            }
        }
    }
}
```

## 状态管理

组件状态使用字段定义：

```auto
type StatefulComponent {
    count int = 0
    message str = "Hello"
    enabled bool = true

    fn on(ev int) {
        is ev {
            1 => {
                count += 1
                message = "Updated"
                enabled = false
            }
        }
    }
}
```

## 类型系统

支持的基本类型:
- `int` / `i32` - 整数
- `str` / `String` - 字符串
- `bool` - 布尔值

## 运行示例

使用 CLI 工具运行示例:

```bash
# 生成 Rust 代码
cargo run --package auto-ui-transpiler-cli -- file example.at example.rs

# 一键运行
cargo run --package auto-ui-transpiler-cli -- run example.at -b gpui
```
