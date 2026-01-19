# AutoUI 抽象层设计方案

## 1. 设计目标

设计一个**类 ELM 的抽象层**，让 iced 和 gpui-component 都能使用，实现：
- ✅ 统一的组件定义方式
- ✅ 统一的状态管理
- ✅ 统一的消息/事件处理
- ✅ 统一的视图构建 DSL
- ✅ 自动适配不同后端

---

## 2. 核心抽象设计

### 2.1 Component Trait

```rust
/// 核心组件 trait - ELM 架构
pub trait Component: Sized {
    /// 消息类型（必须可克隆）
    type Message: Clone + 'static;

    /// 初始化组件
    fn init() -> Self;

    /// 更新状态（ELM 的 update）
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    /// 渲染视图（ELM 的 view）
    fn view(&self) -> ViewBuilder<Self::Message>;
}
```

**设计要点：**
- 使用 ELM 的 `Model -> Update -> View` 循环
- 消息必须可克隆（支持事件传递）
- `Command` 用于副作用（异步操作、日志等）
- `ViewBuilder` 是抽象的视图构建器

### 2.2 统一的消息类型

```rust
/// 消息 trait - 所有消息都实现它
pub trait Message: Clone + 'static {}

// 为所有满足条件的类型自动实现
impl<T: Clone + 'static> Message for T {}
```

**设计要点：**
- 自动为所有 `Clone + 'static` 类型实现
- 不需要手动实现
- 保持了类型安全

### 2.3 抽象视图构建器

```rust
/// 视图构建器 - 流畅 API
pub struct ViewBuilder<M: Message> {
    node: ViewNode<M>,
}

/// 抽象视图节点
pub enum ViewNode<M: Message> {
    Empty,
    Text {
        content: String,
        style: TextStyle,
    },
    Container {
        child: Box<ViewNode<M>>,
        style: ContainerStyle,
    },
    Row {
        children: Vec<ViewNode<M>>,
        style: RowStyle,
    },
    Column {
        children: Vec<ViewNode<M>>,
        style: ColumnStyle,
    },
    Button {
        label: String,
        on_press: Option<M>,
        style: ButtonStyle,
    },
    Checkbox {
        is_checked: bool,
        label: String,
        on_toggle: Option<M>,
        style: CheckboxStyle,
    },
    Input {
        placeholder: String,
        value: String,
        on_change: Option<M>,
        style: InputStyle,
    },
    // ... 更多组件
}
```

### 2.4 样式系统

```rust
/// 通用样式
pub struct Style {
    pub padding: Option<u16>,
    pub spacing: u16,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub align_x: Align,
    pub align_y: Align,
}

/// 长度单位
pub enum Length {
    Fill,
    Shrink,
    Fixed(u32),
}

/// 对齐方式
pub enum Align {
    Start,
    Center,
    End,
}
```

---

## 3. DSL 设计

### 3.1 构建器方法

```rust
impl<M: Message> ViewBuilder<M> {
    // 基础组件
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            node: ViewNode::Text {
                content: content.into(),
                style: TextStyle::default(),
            },
        }
    }

    pub fn button(label: impl Into<String>) -> Self {
        Self {
            node: ViewNode::Button {
                label: label.into(),
                on_press: None,
                style: ButtonStyle::default(),
            },
        }
    }

    pub fn checkbox(is_checked: bool) -> Self {
        Self {
            node: ViewNode::Checkbox {
                is_checked,
                label: String::new(),
                on_toggle: None,
                style: CheckboxStyle::default(),
            },
        }
    }

    // 布局组件
    pub fn row() -> Self {
        Self {
            node: ViewNode::Row {
                children: Vec::new(),
                style: RowStyle::default(),
            },
        }
    }

    pub fn column() -> Self {
        Self {
            node: ViewNode::Column {
                children: Vec::new(),
                style: ColumnStyle::default(),
            },
        }
    }

    pub fn container(child: Self) -> Self {
        Self {
            node: ViewNode::Container {
                child: Box::new(child.node),
                style: ContainerStyle::default(),
            },
        }
    }

    // 链式方法
    pub fn child(mut self, child: impl Into<ViewBuilder<M>>) -> Self {
        if let ViewNode::Row { ref mut children, .. } = self.node {
            children.push(child.into().node);
        } else if let ViewNode::Column { ref mut children, .. } = self.node {
            children.push(child.into().node);
        }
        self
    }

    pub fn padding(mut self, padding: u16) -> Self {
        // 设置 padding
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        // 设置 spacing
        self
    }

    pub fn center(mut self) -> Self {
        // 居中
        self
    }

    // 事件绑定
    pub fn on_press(mut self, message: M) -> Self {
        if let ViewNode::Button { ref mut on_press, .. } = self.node {
            *on_press = Some(message);
        }
        self
    }

    pub fn on_toggle(mut self, message: M) -> Self {
        if let ViewNode::Checkbox { ref mut on_toggle, .. } = self.node {
            *on_toggle = Some(message);
        }
        self
    }

    pub fn on_change(mut self, message: M) -> Self {
        if let ViewNode::Input { ref mut on_change, .. } = self.node {
            *on_change = Some(message);
        }
        self
    }
}
```

### 3.2 使用示例

```rust
struct Counter {
    value: i64,
}

#[derive(Clone)]
enum Message {
    Increment,
    Decrement,
}

impl Component for Counter {
    type Message = Message;

    fn init() -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Increment => self.value += 1,
            Message::Decrement => self.value -= 1,
        }
        Command::none()
    }

    fn view(&self) -> ViewBuilder<Self::Message> {
        ViewBuilder::column()
            .spacing(10)
            .padding(20)
            .center()
            .child(ViewBuilder::<Message>::button("+").on_press(Message::Increment))
            .child(ViewBuilder::<Message>::text(self.value.to_string()))
            .child(ViewBuilder::<Message>::button("-").on_press(Message::Decrement))
    }
}
```

---

## 4. 后端适配器设计

### 4.1 Iced 适配器

```rust
// crates/auto-ui-iced/src/lib.rs
use auto_ui::{Component, Command, Message as Msg, ViewBuilder};

pub struct IcedAdapter<C: Component<'static>> {
    state: C,
}

impl<C: Component<'static>> IcedAdapter<C> {
    pub fn run() -> iced::Result {
        iced::run(
            |state: &mut C, message: C::Message| {
                state.update(message);
                // 忽略 command，iced 0.14 不需要返回值
            },
            |state| state.view().into_iced_element(),
        )
    }
}

// ViewBuilder 到 iced Element 的转换
trait IntoIcedElement<M: Msg> {
    fn into_iced_element(self) -> iced::Element<'_, M>;
}

impl<M: Msg> IntoIcedElement<M> for ViewBuilder<M> {
    fn into_iced_element(self) -> iced::Element<'_, M> {
        match self.node {
            ViewNode::Text { content, style } => {
                iced::widget::text(content)
                    .size(style.size)
                    .into()
            }
            ViewNode::Button { label, on_press, .. } => {
                let mut btn = iced::widget::button(text(label));
                if let Some(msg) = on_press {
                    btn = btn.on_press(msg);
                }
                btn.into()
            }
            ViewNode::Column { children, style } => {
                let mut col = iced::widget::column([]);
                for child in children {
                    col = col.child(ViewBuilder { node: child }.into_iced_element());
                }
                col
                .padding(style.padding)
                .spacing(style.spacing)
                .align_x(style.align_x.into())
                .into()
            }
            // ... 其他组件
        }
    }
}
```

### 4.2 GPUI 适配器

```rust
// crates/auto-ui-gpui/src/lib.rs
use auto_ui::{Component, Command, Message as Msg, ViewBuilder};

pub struct GpuiAdapter<C: Component> {
    state: C,
}

impl<C: Component> GpuiAdapter<C> {
    pub fn run() {
        Application::new().run(|cx| {
            gpui_component::init(cx);

            cx.spawn(async move |cx| {
                cx.open_window(
                    WindowOptions::default(),
                    |window, cx| {
                        let state = C::init();
                        let entity = cx.new(|cx| Self {
                            state,
                            window,
                            _phantom: std::marker::PhantomData,
                        });
                        cx.new(|cx| Root::new(entity, window, cx))
                    },
                )?;
                Ok(())
            }).detach();
        });
    }
}

impl<C: Component> Render for GpuiAdapter<C> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.state.view().into_gpui_element(cx)
    }
}

// ViewBuilder 到 gpui Element 的转换
trait IntoGpuiElement<M: Msg> {
    fn into_gpui_element(self, cx: &mut Context<GpuiAdapter<C>>) -> impl IntoElement;
}

impl<M: Msg> IntoGpuiElement<M> for ViewBuilder<M> {
    fn into_gpui_element(self, cx: &mut Context<GpuiAdapter<C>>) -> impl IntoElement {
        match self.node {
            ViewNode::Text { content, .. } => {
                div().child(content)
            }
            ViewNode::Button { label, on_press, .. } => {
                let mut btn = gpui_component::Button::new("btn").label(&label);
                if let Some(msg) = on_press {
                    btn = btn.on_click(cx.listener(|state, _, _, cx| {
                        state.update(msg);
                        cx.notify();
                    }));
                }
                btn
            }
            ViewNode::Column { children, style } => {
                let mut col = div().v_flex().gap(style.spacing);
                for child in children {
                    col = col.child(child.into_gpui_element(cx));
                }
                col
            }
            // ... 其他组件
        }
    }
}
```

---

## 5. 使用示例

### 5.1 Counter 组件（统一接口）

```rust
use auto_ui::{Component, Command, Message, ViewBuilder};

struct Counter {
    value: i64,
}

#[derive(Clone)]
enum Msg {
    Increment,
    Decrement,
}

impl Component for Counter {
    type Message = Msg;

    fn init() -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            Msg::Increment => self.value += 1,
            Msg::Decrement => self.value -= 1,
        }
        Command::none()
    }

    fn view(&self) -> ViewBuilder<Self::Message> {
        ViewBuilder::column()
            .spacing(10)
            .padding(20)
            .center()
            .child(ViewBuilder::button("+").on_press(Msg::Increment))
            .child(ViewBuilder::text(self.value.to_string()))
            .child(ViewBuilder::button("-").on_press(Msg::Decrement))
    }
}
```

### 5.2 使用 Iced 后端

```rust
// examples/iced_counter.rs
use auto_ui::iced_adapter::IcedAdapter;

fn main() -> iced::Result {
    IcedAdapter::<Counter>::run()
}
```

### 5.3 使用 GPUI 后端

```rust
// examples/gpui_counter.rs
use auto_ui::gpui_adapter::GpuiAdapter;

fn main() {
    GpuiAdapter::<Counter>::run()
}
```

---

## 6. 实现路线图

### Phase 1: 核心抽象（1-2 周）
- [ ] 定义 `Component` trait
- [ ] 定义 `Message` trait
- [ ] 定义 `Command` enum
- [ ] 定义 `ViewBuilder` 和 `ViewNode`
- [ ] 实现基础构建器方法

### Phase 2: Iced 适配器（1 周）
- [ ] 实现 `IntoIcedElement` trait
- [ ] 转换基础组件
- [ ] 转换布局组件
- [ ] 转换事件绑定
- [ ] 测试 Counter 示例

### Phase 3: GPUI 适配器（1-2 周）
- [ ] 实现 `IntoGpuiElement` trait
- [ ] 转换基础组件
- [ ] 转换布局组件
- [ ] 转换事件绑定（需要处理闭包）
- [ ] 测试 Counter 示例

### Phase 4: 高级特性（2-3 周）
- [ ] 更多组件支持
- [ ] 样式系统完善
- [ ] Command 支持（异步操作）
- [ ] 性能优化

---

## 7. 关键技术点

### 7.1 事件系统转换

**问题**：Iced 使用 Message 枚举，GPUI 使用闭包

**解决方案**：
```rust
// 在 ViewNode 中存储统一的事件描述
pub enum EventHandler<M: Message> {
    Message(M),
    Callback(Box<dyn Fn(&mut dyn Any, &mut AnyContext)>),
}

// GPUI 适配时转换为闭包
fn to_gpui_handler<M: Message>(
    handler: EventHandler<M>,
    cx: &mut Context<...>,
) -> impl Fn(...) {
    match handler {
        EventHandler::Message(msg) => {
            cx.listener(|state, _, _, cx| {
                state.update(msg.clone());
                cx.notify();
            })
        }
        EventHandler::Callback(cb) => {
            // 直接使用闭包
        }
    }
}
```

### 7.2 状态同步

**问题**：两个框架的状态管理方式不同

**解决方案**：
```rust
// Iced: 状态由框架管理
// GPUI: 状态需要手动通知更新

// 统一：使用 PhantomData 包装状态
pub struct StateContainer<S> {
    state: S,
    _phantom: PhantomData<*const ()>,
}
```

### 7.3 生命周期处理

**问题**：Iced 的 `Element<'_>` vs GPUI 的 `impl IntoElement`

**解决方案**：
```rust
// 使用 trait object 统一
pub trait Element<M: Message> {
    fn as_any(&self) -> &dyn Any;
}

// 为每个后端实现
impl<M: Message> Element<M> for iced::Element<'_, M> { ... }
impl<M: Message> Element<M> for gpui::impl IntoElement { ... }
```

---

## 8. 优势

### 8.1 统一开发体验

```rust
// 一套代码，两个后端
struct MyApp;

impl Component for MyApp {
    // ... 实现
}

fn main() {
    #[cfg(feature = "iced")]
    auto_ui::run_iced::<MyApp>();

    #[cfg(feature = "gpui")]
    auto_ui::run_gpui::<MyApp>();
}
```

### 8.2 易于测试

```rust
// 测试不需要 GUI
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let mut counter = Counter::init();
        counter.update(Msg::Increment);
        assert_eq!(counter.value, 1);
    }
}
```

### 8.3 类型安全

```rust
// 编译时检查消息类型
button("Click").on_press(Msg::Click)  // ✅
button("Click").on_press(123)         // ❌ 类型错误
```

---

## 9. 未来扩展

### 9.1 更多后端

- Slint（嵌入式）
- Tauri（Web + Desktop）
- Konva（Canvas）
- Flutter Desktop

### 9.2 高级特性

- 路由系统
- 状态管理器（类似 Redux）
- 异步命令
- 动画系统
- 主题系统

---

## 10. 参考

- [Elm Architecture](https://guide.elm-lang.org/architecture/)
- [Iced GitHub](https://github.com/iced-rs/iced)
- [GPUI-Component GitHub](https://github.com/longbridgeapp/gpui-component)
- [Yew Framework](https://yew.rs/) - 类似的 Web 框架
- [Leptos Framework](https://leptos.rs/) - 响应式框架
