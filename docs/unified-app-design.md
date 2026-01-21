# 统一 App 抽象设计

## 设计目标

创建一个统一的抽象层，让开发者可以使用相同的 `Component` 代码运行在不同的后端（Iced, GPUI 等）上，通过特性标志选择后端。

## 架构方案

由于循环依赖问题（auto-ui 不能依赖后端 crate，后端又依赖 auto-ui），采用了文档化的模式而不是统一的 `App::run()` 函数。

## 使用方式

### 1. Iced 后端

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
auto-ui = "0.1"
auto-ui-iced = "0.1"
```

在 `main.rs` 中：

```rust
use auto_ui::{Component, View};
use auto_ui_iced::run_app;

struct MyComponent;

impl Component for MyComponent {
    type Msg = MyMessage;
    fn on(&mut self, msg: Self::Msg) { /* ... */ }
    fn view(&self) -> View<Self::Msg> { /* ... */ }
}

fn main() -> auto_ui::AppResult<()> {
    run_app::<MyComponent>()
}
```

### 2. GPUI 后端

GPUI 需要手动实现 `Render` trait，参考 `auto-ui-gpui-examples/src/bin/counter.rs`。

### 3. 统一 main() 函数（推荐）

你可以在自己的项目中创建一个统一的 main 函数：

```rust
use auto_ui::{Component, View};

struct MyComponent;
impl Component for MyComponent {
    type Msg = ();
    fn on(&mut self, _msg: Self::Msg) {}
    fn view(&self) -> View<Self::Msg> { View::text("Hello") }
}

fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "iced")]
    return auto_ui_iced::run_app::<MyComponent>();

    #[cfg(feature = "gpui")]
    return auto_ui_gpui::run_app::<MyComponent>();

    #[cfg(not(any(feature = "iced", feature = "gpui")))]
    Err("No backend enabled".into())
}
```

然后在运行时选择后端：

```bash
# 使用 Iced 后端
cargo run --features iced

# 使用 GPUI 后端
cargo run --features gpui
```

## 设计决策

### 为什么不使用 `auto_ui::App::run()`？

1. **循环依赖**：`auto-ui` 依赖 `auto-ui-iced`/`auto-ui-gpui`，它们又依赖 `auto-ui`
2. **架构差异**：GPUI 使用闭包处理事件，Iced 使用消息枚举，无法完全统一
3. **灵活性**：让用户在自己的项目中选择后端，更容易扩展

### 推荐模式

在你的应用项目中，创建一个统一的 `main.rs`，使用条件编译选择后端。这样：
- ✅ `Component` 代码只需要写一次
- ✅ 通过特性标志选择后端
- ✅ 无循环依赖
- ✅ 易于扩展到新后端

## 示例项目结构

```
my-app/
├── Cargo.toml          # 添加 auto-ui-iced 和 auto-ui-gpui 依赖
├── src/
│   └── main.rs         # 统一的 main() 函数
└── components/
    └── my_component.rs # Component 实现（后端无关）
```

## 总结

虽然我们没有实现完全透明的 `App::run()` 统一接口，但通过：
1. 统一的 `Component` trait
2. 各后端的 `run_app()` 函数
3. 条件编译的模式

开发者可以轻松地使用相同的业务逻辑代码，在不同后端之间切换。
