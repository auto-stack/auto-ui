# 运行从 Auto 语言生成的组件

本指南说明如何将从 Auto 语言（.at 文件）转译生成的 Rust 组件代码运行起来。

## 快速开始

### 1. 生成组件代码

```bash
# 生成简单的 Hello 组件
cargo run --package auto-ui-transpiler-cli -- file scratch/text_simple.at scratch/text_simple.rs

# 生成带布局的组件
cargo run --package auto-ui-transpiler-cli -- file scratch/col_test.at scratch/col_test.rs
```

### 2. 验证生成的代码（逻辑测试）

```bash
cargo run --package auto-ui --example verify_generated
```

这会验证组件的结构、字段访问和 View 生成是否正确。

### 3. 运行完整的 GPUI 应用

有三种方式运行 GPUI 应用：

#### 方式 A：使用 prepared example（推荐）

```bash
# 将 GPUI 应用示例复制到 examples 目录
copy scratch\hello_gpui_app.rs crates\auto-ui-gpui\examples\hello_auto.rs

# 运行
cargo run --package auto-ui-gpui --example hello_auto
```

#### 方式 B：修改现有 example

将生成的组件代码复制到 `crates/auto-ui-gpui/examples/` 下的某个示例中，然后运行。

#### 方式 C：创建新的 binary

```bash
# 在 crates/auto-ui-gpui/ 下创建 src/bin/hello_auto.rs
# 然后运行：
cargo run --package auto-ui-gpui --bin hello_auto
```

## 完整工作流程示例

### 步骤 1：编写 Auto 代码

创建文件 `scratch/my_widget.at`：

```auto
type MyWidget {
    title str = "Hello AutoUI"
    count int = 0

    fn view() {
        col {
            text(title)
            text(count)
        }
    }
}
```

### 步骤 2：转译为 Rust

```bash
cargo run --package auto-ui-transpiler-cli -- file scratch/my_widget.at scratch/my_widget.rs
```

### 步骤 3：查看生成的代码

```bash
cat scratch/my_widget.rs
```

你会看到类似这样的代码：

```rust
use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct MyWidget {
    pub title: String,
    pub count: i32,
}

impl MyWidget {
    pub fn new(title: String, count: i32) -> Self {
        Self { title, count }
    }
}

impl Component for MyWidget {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(0)
            .padding(0)
            .child(View::text(&self.title))
            .child(View::text(&self.count.to_string()))
            .build()
    }
}
```

### 步骤 4：创建 GPUI 应用

创建文件 `crates/auto-ui-gpui/examples/my_widget_app.rs`：

```rust
use auto_ui::{Component, View};
use auto_ui_gpui::GpuiComponentState;
use gpui::{App, AppContext, Context, Window, WindowOptions};

// [粘贴从步骤 3 生成的组件代码]

// GPUI 应用状态
struct MyAppState {
    widget_state: GpuiComponentState<MyWidget>,
}

impl MyAppState {
    fn new() -> Self {
        let widget = MyWidget::new("My First Widget".to_string(), 42);
        Self {
            widget_state: GpuiComponentState::new(widget),
        }
    }
}

impl gpui::Render for MyAppState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let view = self.widget_state.component().view();
        view.into_gpui_static()
    }
}

fn main() {
    App::new().run(move |cx: &mut AppContext| {
        let window_options = WindowOptions {
            window_bounds: Some(gpui::Bounds {
                origin: gpui::Point { x: 100.0, y: 100.0 },
                size: gpui::Size { width: 800.0, height: 600.0 },
            }),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("My Widget App".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(window_options, |cx| {
            let state = MyAppState::new();
            cx.new_view(|_cx| state)
        })
        .unwrap();
    });
}
```

### 步骤 5：运行应用

```bash
cargo run --package auto-ui-gpui --example my_widget_app
```

## 当前已实现的节点

目前 transpiler 已经支持以下节点的代码生成：

### ✅ 已实现
- **text**: `text(msg)` → `View::text(&self.msg)`
- **col/row**: 布局容器，支持嵌套子节点
  ```auto
  col {
      text("First")
      text("Second")
  }
  ```

### 🚧 待实现
- **button**: 带点击事件的按钮
- **input**: 文本输入框
- **checkbox**: 复选框
- **radio**: 单选按钮
- **select**: 下拉选择
- **list**: 列表
- **table**: 表格
- **center/container/scrollable**: 容器组件

## 故障排查

### 问题 1：编译错误 "cannot find Component"

确保 `auto-ui` 依赖已正确配置：

```toml
[dependencies]
auto-ui = { path = "../auto-ui", features = ["gpui"] }
```

### 问题 2：GPUI 窗口不显示

检查 GPUI 是否正确初始化。确保在 `App::new().run()` 闭包中创建了窗口。

### 问题 3：生成的 View 为空

检查 `.at` 文件语法，确保：
- `view()` 方法有正确的返回值
- 节点名称拼写正确（区分大小写）
- 有对应的生成器实现

## 下一步

1. **实现更多节点**：继续完成 button、input 等节点的代码生成
2. **支持消息传递**：实现 `on()` 方法的代码生成
3. **样式支持**：添加内联样式的解析和生成
4. **热重载**：开发时自动转译和重载

## 相关文件

- 转译器实现：`crates/auto-ui/src/trans/rust_gen.rs`
- GPUI 后端：`crates/auto-ui-gpui/`
- 测试文件：`scratch/*.at` 和 `scratch/*.rs`
- 示例应用：`crates/auto-ui-gpui/examples/*.rs`
