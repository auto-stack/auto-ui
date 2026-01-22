# 快速开始：运行你的第一个 AutoUI 应用

## 🎯 30 秒快速运行

### 方法 1: 使用 run 命令 (推荐) 🚀

```bash
# 一条命令从 .at 文件直接运行!
cargo run --package auto-ui-transpiler-cli -- run scratch/text_simple.at -b gpui
```

### 方法 2: 运行预生成的示例

```bash
# 1. 运行简单的 Hello 组件
cargo run --package auto-ui-gpui --example run_generated

# 2. 运行带布局的 Col 组件
cargo run --package auto-ui-gpui --example run_col
```

就这么简单!一个 GPUI 窗口会打开,显示从 Auto 语言生成的组件。

> **💡 提示:** 推荐使用 `run` 命令,它会自动完成转译、生成、编译和运行所有步骤。详见 [Run 命令指南](RUN_COMMAND.md)。

## 📝 创建你自己的组件

### 🚀 快速方法 (使用 run 命令)

```bash
# 1. 编写 Auto 代码到 my_app.at

# 2. 一条命令运行!
cargo run --package auto-ui-transpiler-cli -- run scratch/my_app.at -b gpui
```

就这么简单!`run` 命令会自动处理所有步骤。

---

### 📚 详细方法 (手动步骤)

如果你想了解每个步骤的详细信息,或者需要自定义生成的代码,可以按照以下步骤操作:

#### 步骤 1：编写 Auto 代码

创建文件 `scratch/my_app.at`：

```auto
type MyApp {
    greeting str = "Hello"
    name str = "World"

    fn view() {
        col {
            text(greeting)
            text(name)
        }
    }
}
```

### 步骤 2：转译为 Rust

```bash
cargo run --package auto-ui-transpiler-cli -- file scratch/my_app.at scratch/my_app.rs
```

### 步骤 3：查看生成的代码

```bash
cat scratch/my_app.rs
```

你会看到：

```rust
use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct MyApp {
    pub greeting: String,
    pub name: String,
}

impl MyApp {
    pub fn new(greeting: String, name: String) -> Self {
        Self { greeting, name }
    }
}

impl Component for MyApp {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(0)
            .padding(0)
            .child(View::text(&self.greeting))
            .child(View::text(&self.name))
            .build()
    }
}
```

### 步骤 4：创建 GPUI 应用

在 `crates/auto-ui-gpui/examples/` 创建 `my_app.rs`：

```rust
use auto_ui::{Component, View};

// 粘贴上面生成的代码

// 添加 Default 实现
impl Default for MyApp {
    fn default() -> Self {
        Self {
            greeting: "Hello".to_string(),
            name: "AutoUI!".to_string(),
        }
    }
}

// 添加 main 函数
fn main() -> auto_ui::AppResult<()> {
    auto_ui_gpui::run_app::<MyApp>("My First AutoUI App")
}
```

### 步骤 5：运行

```bash
cargo run --package auto-ui-gpui --example my_app
```

## 🎨 支持的组件

### 文本组件

```auto
text(msg)
```

### 布局组件

```auto
col {
    text("First")
    text("Second")
    text("Third")
}
```

```auto
row {
    text("Left")
    text("Right")
}
```

### 嵌套布局

```auto
col {
    text("Title")
    row {
        text("Item 1")
        text("Item 2")
    }
}
```

## 🔧 故障排查

### 问题：找不到 `run_app` 函数

确保导入了正确的模块：

```rust
use auto_ui::{Component, View};
// run_app 在 auto_ui_gpui 中，不需要显式导入
```

### 问题：没有实现 Default

为你的组件添加 Default 实现：

```rust
impl Default for MyApp {
    fn default() -> Self {
        Self::new(
            "Default value".to_string(),
            "Another default".to_string()
        )
    }
}
```

### 问题：窗口不显示

- 确保没有编译错误
- 检查控制台是否有运行时错误
- 在 Windows 上，可能需要等待几秒钟让窗口出现

## 📚 更多示例

查看 `crates/auto-ui-gpui/examples/` 目录：

- `run_generated.rs` - 简单的文本组件
- `run_col.rs` - 带布局的组件
- 更多样例正在添加中...

## 🚀 下一步

1. 尝试不同的布局组合
2. 修改默认值查看效果
3. 等待更多节点类型的实现（button, input 等）
4. 查看 [完整文档](GPUI_TROUBLESHOOTING.md) 了解详细信息

## 💡 提示

- 生成的组件代码可以在多个 GPUI 应用中重用
- 使用 `Default` trait 设置合理的默认值
- 布局组件（col, row）可以无限嵌套
- 当前所有生成的组件都使用 `()` 作为消息类型（无事件处理）
