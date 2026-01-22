# 运行 AutoUI 生成的 GPUI 应用 - 问题排查与解决方案

## ✅ 当前状态

两个示例都已成功创建并可以运行：

1. **[run_generated.rs](../crates/auto-ui-gpui/examples/run_generated.rs)** - 简单文本组件
2. **[run_col.rs](../crates/auto-ui-gpui/examples/run_col.rs)** - 带布局的组件

## 🚀 运行命令

### 示例 1：简单文本组件

```bash
cargo run --package auto-ui-gpui --example run_generated
```

### 示例 2：Col 布局组件

```bash
cargo run --package auto-ui-gpui --example run_col
```

## 🐛 遇到的问题和解决方案

### 问题 1：API 不兼容

**错误信息**：
```
error[E0599]: no method named `into_gpui_static` found for enum `auto_ui::View<M>`
error[E0061]: this function takes 2 arguments but 0 arguments were supplied
```

**原因**：
- GPUI 0.2.2 的 API 与预期不同
- 直接使用 `App::new()` 需要复杂的设置

**解决方案**：
使用 `auto_ui_gpui::run_app()` 辅助函数，它简化了 GPUI 应用的启动：

```rust
fn main() -> auto_ui::AppResult<()> {
    auto_ui_gpui::run_app::<MyComponent>("Window Title")
}
```

### 问题 2：Default trait 实现

**错误信息**：
```
the trait `Default` is not implemented for `Hello`
```

**原因**：
- `run_app()` 需要组件实现 `Default` trait
- 生成的代码通常使用 `new()` 构造函数，而不是 `Default`

**解决方案**：
为生成的组件添加自定义 `Default` 实现：

```rust
#[derive(Debug)]  // 移除 Default，手动实现
pub struct Hello {
    pub msg: String,
}

impl Default for Hello {
    fn default() -> Self {
        Self {
            msg: "Hello from Auto Language!".to_string(),
        }
    }
}
```

### 问题 3：类型不匹配

**错误信息**：
```
error[E0308]: mismatched types
   expected `Hello`, found `ColHello`
```

**原因**：
尝试将不同类型的组件存储在同一个 `GpuiComponentState<T>` 中

**解决方案**：
- 为不同的组件创建不同的示例文件
- 或者使用枚举包装多个组件类型

## 📝 完整工作流程

### 1. 编写 Auto 代码

创建 `scratch/my_widget.at`：

```auto
type MyWidget {
    title str = "Hello"

    fn view() {
        text(title)
    }
}
```

### 2. 转译为 Rust

```bash
cargo run --package auto-ui-transpiler-cli -- file scratch/my_widget.at scratch/my_widget.rs
```

生成的代码（简化）：

```rust
use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct MyWidget {
    pub title: String,
}

impl MyWidget {
    pub fn new(title: String) -> Self {
        Self { title }
    }
}

impl Component for MyWidget {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        View::text(&self.title)
    }
}
```

### 3. 创建 GPUI 应用

在 `crates/auto-ui-gpui/examples/` 创建新文件：

```rust
use auto_ui::{Component, View};

// 复制生成的组件代码
// ... (粘贴上面的代码)

// 添加 Default 实现
impl Default for MyWidget {
    fn default() -> Self {
        Self {
            title: "My Default Title".to_string(),
        }
    }
}

// 添加 main 函数
fn main() -> auto_ui::AppResult<()> {
    auto_ui_gpui::run_app::<MyWidget>("My Widget App")
}
```

### 4. 运行应用

```bash
cargo run --package auto-ui-gpui --example my_widget
```

## 🎨 当前支持的节点

### ✅ 已测试

| 节点 | Auto 语法 | 生成的 Rust 代码 | 状态 |
|------|-----------|-----------------|------|
| text | `text(msg)` | `View::text(&self.msg)` | ✅ 可运行 |
| col | `col { ... }` | `View::col().child(...).build()` | ✅ 可运行 |
| row | `row { ... }` | `View::row().child(...).build()` | ✅ 可运行 |

### 🚧 待测试

- button
- input
- checkbox
- radio
- select
- list
- table
- center
- container
- scrollable

## 💡 提示和最佳实践

### 1. 组件初始化

生成的代码通常使用 `new()` 构造函数：

```rust
let widget = MyWidget::new("Custom title".to_string());
```

但 `run_app()` 使用 `Default`：

```rust
impl Default for MyWidget {
    fn default() -> Self {
        Self::new("Default title".to_string())
    }
}
```

### 2. 字符串处理

Auto 语言中的 `str` 类型会被转译为 Rust 的 `String`：

```auto
// Auto 代码
msg str = "Hello"
```

```rust
// 生成的 Rust 代码
pub msg: String,
```

在 View 中使用引用：

```rust
View::text(&self.msg)  // 注意 & 取引用
```

### 3. 嵌套布局

Col 和 Row 可以嵌套：

```auto
col {
    text("Title")
    row {
        text("Left")
        text("Right")
    }
}
```

生成的代码会自动处理嵌套。

### 4. 消息类型

当前生成的组件使用 `()` 作为消息类型：

```rust
type Msg = ();
```

这意味着组件不处理任何事件。未来的 transpiler 版本将支持从 `on()` 方法生成消息枚举。

## 🔍 调试技巧

### 查看生成的 View 结构

在开发过程中，可以打印 View 结构进行调试：

```rust
let view = self.view();
println!("View: {:#?}", view);
```

### 验证组件逻辑

在集成到 GPUI 之前，先运行逻辑测试：

```bash
cargo run --package auto-ui --example verify_generated
```

### 检查编译错误

如果遇到编译错误，检查：

1. ✅ 组件是否实现了 `Component` trait
2. ✅ 组件是否实现了 `Default` trait
3. ✅ `view()` 方法是否返回 `View<Self::Msg>`
4. ✅ `on()` 方法是否与 `Msg` 类型匹配

## 📚 相关文件

- Transpiler: [crates/auto-ui/src/trans/rust_gen.rs](../crates/auto-ui/src/trans/rust_gen.rs)
- GPUI Backend: [crates/auto-ui-gpui/src/lib.rs](../crates/auto-ui-gpui/src/lib.rs)
- 示例代码: [crates/auto-ui-gpui/examples/](../crates/auto-ui-gpui/examples/)
- 测试文件: [scratch/](../scratch/)

## 🎯 下一步

1. ✅ 实现更多节点类型（button, input 等）
2. ✅ 支持 `on()` 方法的代码生成
3. ✅ 添加样式属性的支持
4. ✅ 实现热重载功能
