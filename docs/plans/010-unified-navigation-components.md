# Plan 010: Unified Navigation Components

**Status**: ⚠️ Partially Complete - GPUI Backend Has Issues
**Created**: 2025-01-23
**Last Updated**: 2025-01-23
**Priority**: High
**Complexity**: Medium
**Estimated Timeline**: 4-7 days (MVP: 2-3 days)

## Current Status

### ✅ Completed
- **Phase 1 (核心抽象层)**: 完成
  - ✅ `AccordionBuilder`, `SidebarBuilder`, `TabsBuilder`, `NavigationRailBuilder`
  - ✅ 辅助类型: `AccordionItem`, `NavigationItem`, `SidebarPosition`, `TabsPosition`
  - ✅ Builder 模式和方法链
  - ✅ 在 `auto-ui/src/view.rs` 实现
  - ✅ 回调类型: `AccordionToggleCallback`, `TabsSelectCallback`, `NavigationRailSelectCallback`

- **Phase 2 (Iced 后端)**: 完成
  - ✅ 所有组件在 Iced 中正常渲染
  - ✅ 交互功能正常（展开/折叠，切换）
  - ✅ 事件处理和消息传递
  - ✅ 样式支持

- **Phase 4 (Unified 示例)**: 部分完成
  - ✅ `unified-accordion` - Iced 后端工作正常，GPUI 后端有栈溢出
  - ✅ `unified-sidebar` - 两个后端都工作正常
  - ✅ `unified-tabs` - 两个后端都工作正常
  - ✅ `unified-navigation-rail` - 两个后端都工作正常
  - ✅ `unified-gallery` - 使用 Sidebar 组件，布局正确

### ⚠️ Issues

**GPUI 后端栈溢出问题** (详见下方 "Known Issues" 章节)
- `unified-accordion` 在 GPUI 后端运行时出现栈溢出错误
- 错误信息: `thread 'main' has overflowed its stack`
- 退出代码: `0xc00000fd (STATUS_STACK_OVERFLOW)`
- **临时解决方案**: 使用 Iced 后端运行 Accordion 示例

### 📋 In Progress
- **Phase 3 (GPUI 后端)**: 阻塞
  - ✅ 编译通过，代码实现在 `crates/auto-ui-gpui/src/auto_render.rs`
  - ❌ 运行时栈溢出（Accordion 组件）
  - ✅ Tabs, NavigationRail 组件正常工作
  - ⏸️ **Blocked**: 需要解决栈溢出问题才能继续

### 📝 Pending
- **Phase 5 (Gallery 集成)**: 未开始

## Overview

从 Iced Gallery 中提取常见的导航和布局控件，实现为 auto-ui 的统一抽象组件（unified-components），使其可以在 Iced 和 GPUI 两个后端中使用。

## Motivation

### 当前状态

**Iced Gallery 中使用的控件**:
1. **Accordion (手风琴)** - 可展开/折叠的分组列表
   - 侧边栏导航中的分组
   - 每个分组有标题和子项
   - 点击标题可展开/折叠
   - 支持多组同时展开

2. **Sidebar (侧边栏)** - 固定在侧边的导航面板
   - 固定宽度（通常 250-300px）
   - 包含导航内容（Accordion, 菜单等）
   - 响应式：小屏幕可折叠

3. **NavigationRail (导航栏)** - 紧凑的侧边导航
   - 通常只显示图标
   - 悬停时显示文本标签

4. **Tabs (选项卡)** - 水平标签切换
   - 多个标签页
   - 点击切换内容

**现有 unified-components**:
- ✅ Button, Text, Input
- ✅ Select, Checkbox
- ✅ Slider, Progress
- ✅ Table, List
- ❌ **缺少**: Accordion, Sidebar, NavigationRail, Tabs

### 目标

- ✅ 实现 Accordion 作为统一抽象组件
- ✅ 实现 Sidebar/NavigationRail 作为布局组件
- ✅ 实现 Tabs 作为切换组件
- ✅ 为 Iced 和 GPUI 提供后端实现
- ✅ 创建 unified 示例演示这些组件
- ✅ 集成到 Gallery 中

## Architecture

### 1. Accordion 组件

```rust
// view.rs 新增方法
impl View {
    /// 创建 Accordion（手风琴）组件
    ///
    /// # Arguments
    /// * `items` - 手风琴项列表
    ///
    /// # Example
    /// ```rust
    /// View::accordion()
    ///     .items(vec![
    ///         AccordionItem::new("Getting Started", '🏠')
    ///             .children(vec![
    ///                 View::text("Home"),
    ///                 View::text("Hello"),
    ///             ]),
    ///         AccordionItem::new("Basic Widgets", '📦')
    ///             .children(vec![
    ///                 View::text("Button"),
    ///                 View::text("Checkbox"),
    ///             ]),
    ///     ])
    ///     .on_toggle(|index, expanded| Message::GroupToggled(index, expanded))
    ///     .allow_multiple(true)  // 允许多个同时展开
    /// ```
    pub fn accordion() -> AccordionBuilder { ... }
}

pub struct AccordionBuilder<M> {
    items: Vec<AccordionItem<M>>,
    on_toggle: Option<Box<dyn Fn(usize, bool) -> M>>,
    allow_multiple: bool,
    initially_expanded: Option<usize>,
}

pub struct AccordionItem<M> {
    title: String,
    icon: Option<char>,
    children: Vec<View<M>>,
    expanded: bool,
}
```

### 2. Sidebar 组件

```rust
impl View {
    /// 创建侧边栏布局
    ///
    /// # Arguments
    /// * `content` - 侧边栏内容
    /// * `width` - 宽度（px）
    ///
    /// # Example
    /// ```rust
    /// View::sidebar(
    ///     View::accordion()
    ///         .items(items)
    ///         .build(),
    ///     300.0
    /// )
    /// .collapsible(true)  // 可折叠
    /// .responsive(true)   // 响应式
    /// ```
    pub fn sidebar(content: View<M>, width: f32) -> SidebarBuilder<M> { ... }
}

pub struct SidebarBuilder<M> {
    content: View<M>,
    width: f32,
    collapsible: bool,
    responsive: bool,
    position: SidebarPosition,  // Left, Right
}

pub enum SidebarPosition {
    Left,
    Right,
}
```

### 3. Tabs 组件

```rust
impl View {
    /// 创建选项卡组件
    ///
    /// # Arguments
    /// * `tabs` - 选项卡标签列表
    ///
    /// # Example
    /// ```rust
    /// View::tabs(vec!["Home", "Settings", "About"])
    ///     .selected(0)
    ///     .on_select(|index| Message::TabChanged(index))
    ///     .contents(vec![
    ///         View::text("Home Content"),
    ///         View::text("Settings Content"),
    ///         View::text("About Content"),
    ///     ])
    /// ```
    pub fntabs(labels: Vec<String>) -> TabsBuilder<M> { ... }
}

pub struct TabsBuilder<M> {
    labels: Vec<String>,
    contents: Vec<View<M>>,
    selected: usize,
    on_select: Option<Box<dyn Fn(usize) -> M>>,
    position: TabsPosition,  // Top, Bottom, Left, Right
}

pub enum TabsPosition {
    Top,
    Bottom,
    Left,
    Right,
}
```

### 4. NavigationRail 组件

```rust
impl View {
    /// 创建导航栏（紧凑型侧边栏）
    ///
    /// # Arguments
    /// * `items` - 导航项列表
    ///
    /// # Example
    /// ```rust
    /// View::navigation_rail()
    ///     .items(vec![
    ///         NavigationItem::new('🏠', "Home"),
    ///         NavigationItem::new('⚙️', "Settings"),
    ///     ])
    ///     .selected(0)
    ///     .on_select(|index| Message::Navigate(index))
    ///     .width(72.0)
    /// ```
    pub fn navigation_rail() -> NavigationRailBuilder<M> { ... }
}

pub struct NavigationRailBuilder<M> {
    items: Vec<NavigationItem>,
    selected: usize,
    on_select: Option<Box<dyn Fn(usize) -> M>>,
    width: f32,
    show_labels: bool,  // 是否显示文本标签
}

pub struct NavigationItem {
    icon: char,
    label: String,
    badge: Option<String>,
}
```

## Implementation Plan

### Phase 1: 核心抽象层（1天）

**目标**: 在 auto-ui 中定义统一的抽象 API

**任务**:
- [ ] 在 `crates/auto-ui/src/view.rs` 添加新的 Builder 类型：
  - [ ] `AccordionBuilder<M>`
  - [ ] `SidebarBuilder<M>`
  - [ ] `TabsBuilder<M>`
  - [ ] `NavigationRailBuilder<M>`
- [ ] 定义辅助数据结构：
  - [ ] `AccordionItem<M>`
  - [ ] `NavigationItem`
  - [ ] `SidebarPosition`, `TabsPosition`
- [ ] 实现 Builder 模式的方法链
- [ ] 更新 `prelude` 和导出

**验证标准**:
- ✅ 所有 Builder 类型定义完整
- ✅ API 设计简洁易用
- ✅ 编译通过

### Phase 2: Iced 后端实现（1-2天）

**目标**: 为 Iced 实现所有新组件

**任务**:
- [ ] 在 `crates/auto-ui-iced` 实现转换：
  - [ ] `AccordionBuilder` → Iced widget
  - [ ] `SidebarBuilder` → Iced container
  - [ ] `TabsBuilder` → Iced tabs
  - [ ] `NavigationRailBuilder` → Iced rail
- [ ] 实现状态管理（展开/折叠，选中）
- [ ] 实现事件处理（点击，切换）
- [ ] 添加样式支持

**Iced 实现示例**:
```rust
// auto-ui-iced/src/widget/accordion.rs
use iced::widget::{container, column, button, text};

pub struct Accordion<M> {
    items: Vec<AccordionItem<M>>,
    on_toggle: Box<dyn Fn(usize, bool) -> M>,
    allow_multiple: bool,
}

impl<M> Accordion<M> {
    pub fn view(&self) -> iced::Element<M> {
        let mut col = column!();

        for (idx, item) in self.items.iter().enumerate() {
            let header = button(text(item.title))
                .on_press((self.on_toggle)(idx, !item.expanded));

            let children = if item.expanded {
                // Render children
            } else {
                text("")
            };

            col = col.push(header).push(children);
        }

        container(col).into()
    }
}
```

**验证标准**:
- ✅ 所有组件在 Iced 中正常渲染
- ✅ 交互功能正常（展开/折叠，切换）
- ✅ 样式美观

### Phase 3: GPUI 后端实现（1-2天）

**目标**: 为 GPUI 实现所有新组件

**任务**:
- [ ] 在 `crates/auto-ui-gpui` 实现转换：
  - [ ] `AccordionBuilder` → GPUI component
  - [ ] `SidebarBuilder` → GPUI div
  - [ ] `TabsBuilder` → GPUI tabs
  - [ ] `NavigationRailBuilder` → GPUI rail
- [ ] 使用 `gpui_component` 的原生组件（如果可用）
- [ ] 实现状态管理（使用 Entity）
- [ ] 实现事件处理（订阅，点击）

**GPUI 实现示例**:
```rust
// auto-ui-gpui/src/widget/accordion.rs
use gpui::*;
use gpui_component::*;

pub struct Accordion<M> {
    items: Vec<AccordionItem<M>>,
    expanded: Vec<bool>,
    on_toggle: Box<dyn Fn(usize, bool) -> M>,
}

impl<M> Render for Accordion<M> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut v_flex = v_flex();

        for (idx, item) in self.items.iter().enumerate() {
            let is_expanded = self.expanded[idx];
            let idx_clone = idx;

            let header = div()
                .cursor_pointer()
                .on_click(cx.listener(move |_accordion, _event, cx| {
                    // Toggle logic
                }))
                .child(div().text(item.title.clone()));

            v_flex = v_flex.child(header);

            if is_expanded {
                for child in &item.children {
                    v_flex = v_flex.child(child.clone());
                }
            }
        }

        v_flex.into_any()
    }
}
```

**验证标准**:
- ✅ 所有组件在 GPUI 中正常渲染
- ✅ 交互功能正常
- ✅ Entity 生命周期管理正确

### Phase 4: Unified 示例（1天）

**目标**: 创建 unified 示例演示新组件

**任务**:
- [ ] 创建 `examples/unified-accordion`
- [ ] 创建 `examples/unified-sidebar`
- [ ] 创建 `examples/unified-tabs`
- [ ] 创建 `examples/unified-navigation-rail`
- [ ] 每个示例支持 `--features iced` 和 `--features gpui`

**示例结构**:
```
examples/unified-accordion/
├── Cargo.toml
└── src/
    └── main.rs (100行)
```

**示例代码**:
```rust
// unified-accordion/src/main.rs
use auto_ui::{Component, View, App};

#[derive(Debug, Default)]
struct AccordionApp {
    expanded_groups: Vec<bool>,
}

#[derive(Clone, Debug)]
enum Message {
    GroupToggled(usize, bool),
}

impl Component for AccordionApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::GroupToggled(index, expanded) => {
                self.expanded_groups[index] = expanded;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Accordion Example".to_string()))
            .child(
                View::accordion()
                    .items(vec![
                        AccordionItem::new("Getting Started", '🏠')
                            .children(vec![
                                View::text("Home".to_string()),
                                View::text("Hello".to_string()),
                            ]),
                        AccordionItem::new("Basic Widgets", '📦')
                            .children(vec![
                                View::text("Button".to_string()),
                                View::text("Checkbox".to_string()),
                            ]),
                    ])
                    .allow_multiple(true)
                    .on_toggle(|index, expanded| Message::GroupToggled(index, expanded))
                    .build()
            )
            .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    App::run::<AccordionApp>()
}
```

**验证标准**:
- ✅ 所有示例可以在 Iced 中运行
- ✅ 所有示例可以在 GPUI 中运行
- ✅ UI 美观且交互流畅

### Phase 5: Gallery 集成（1天）

**目标**: 在 Iced/GPUI Gallery 中使用新组件

**任务**:
- [ ] 重构 `iced-examples/src/navigation.rs` 使用 unified Accordion
- [ ] 重构 `iced-examples/src/main.rs` 使用 unified Sidebar
- [ ] 为 GPUI Gallery（Plan 009）准备组件
- [ ] 对比 unified 和原生实现

**重构示例**:
```rust
// 之前 (iced-examples/src/navigation.rs)
pub struct Sidebar {
    pub page_groups: Vec<PageGroup>,
    pub display_mode: DisplayMode,
}

// 之后 (使用 unified Accordion)
use auto_ui::{View, AccordionItem};

struct Sidebar {
    accordion: View<Message>,
}

impl Sidebar {
    fn new(page_groups: Vec<PageGroup>) -> Self {
        let items: Vec<AccordionItem<Message>> = page_groups
            .into_iter()
            .map(|group| {
                AccordionItem::new(group.label, group.icon)
                    .children(/* page items as Views */)
            })
            .collect();

        Self {
            accordion: View::accordion()
                .items(items)
                .allow_multiple(true)
                .build(),
        }
    }
}
```

**验证标准**:
- ✅ Gallery 使用 unified 组件后功能正常
- ✅ 代码更简洁（减少重复代码）
- ✅ 后端切换更容易

## File Manifest

### 修改文件

```
crates/auto-ui/src/
├── lib.rs                         # 添加新 Builder 导出
└── view.rs                        # 新增 ~400 行
    ├── AccordionBuilder<M>        # (100行)
    ├── SidebarBuilder<M>          # (80行)
    ├── TabsBuilder<M>             # (100行)
    ├── NavigationRailBuilder<M>   # (80行)
    └── 辅助类型                    # (40行)

crates/auto-ui-iced/src/
├── converter.rs                   # 添加新组件转换逻辑 (~200行)
└── widget/                        # 新增目录
    ├── accordion.rs               # (150行)
    ├── sidebar.rs                 # (100行)
    ├── tabs.rs                    # (150行)
    └── navigation_rail.rs         # (120行)

crates/auto-ui-gpui/src/
├── converter.rs                   # 添加新组件转换逻辑 (~200行)
└── widget/                        # 新增目录
    ├── accordion.rs               # (180行)
    ├── sidebar.rs                 # (120行)
    ├── tabs.rs                    # (150行)
    └── navigation_rail.rs         # (130行)
```

### 新建文件（~12 个）

```
examples/
├── unified-accordion/             # NEW
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                # (100行)
├── unified-sidebar/               # NEW
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                # (120行)
├── unified-tabs/                  # NEW
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                # (110行)
└── unified-navigation-rail/       # NEW
    ├── Cargo.toml
    └── src/
        └── main.rs                # (100行)
```

**总代码量**: ~2,500 行

## Technical Challenges & Solutions

### Challenge 1: Accordion 状态管理

**问题**: 如何统一管理多组的展开/折叠状态？

**解决方案**:
```rust
// 在应用层管理状态
struct App {
    expanded: Vec<bool>,  // 每个组的展开状态
}

impl Component for App {
    fn view(&self) -> View<Message> {
        View::accordion()
            .items(/* ... */)
            .on_toggle(|index, is_expanded| {
                Message::GroupToggled(index, is_expanded)
            })
            .build()
    }

    fn on(&mut self, msg: Message) {
        if let Message::GroupToggled(idx, expanded) = msg {
            self.expanded[idx] = expanded;
        }
    }
}
```

### Challenge 2: Sidebar 响应式布局

**问题**: 不同窗口大小时如何调整 Sidebar？

**解决方案**:
```rust
// 在 auto-ui 层定义响应式策略
pub enum ResponsiveMode {
    Fixed(f32),           // 固定宽度
    Percentage(f32),      // 百分比
    CollapseUnder(f32),   // 小于某宽度时折叠
}

pub struct SidebarBuilder<M> {
    responsive: ResponsiveMode,
    // ...
}
```

### Challenge 3: Tabs 内容管理

**问题**: 如何高效管理多个标签页的内容？

**解决方案**:
```rust
// 只渲染当前选中的标签页
pub struct TabsBuilder<M> {
    labels: Vec<String>,
    contents: Vec<View<M>>,
    selected: usize,
    // ...
}

// 渲染时只显示 selected 的内容
fn render(&self) {
    // 渲染标签栏
    let tab_bar = /* ... */;

    // 只渲染当前内容
    let current_content = &self.contents[self.selected];

    /* 组合 tab_bar 和 current_content */
}
```

### Challenge 4: GPUI Entity 预初始化

**问题**: Accordion 等组件可能需要预初始化子 Entity

**解决方案**:
```rust
impl Accordion for GPUI {
    fn build_entities(&mut self, cx: &mut Context<Self>) {
        // 预初始化所有可展开的内容
        for item in &self.items {
            if item.has_children() {
                // 初始化子 entities
            }
        }
    }
}
```

## API Design Examples

### Accordion 完整示例

```rust
use auto_ui::{View, AccordionItem, Component, App};

#[derive(Default)]
struct MyApp {
    expanded: Vec<bool>,
}

#[derive(Clone, Debug)]
enum Message {
    ToggleGroup(usize, bool),
    SelectItem(String),
}

impl Component for MyApp {
    type Msg = Message;

    fn view(&self) -> View<Message> {
        View::col()
            .padding(20)
            .child(View::text("Navigation".to_string()))
            .child(
                View::accordion()
                    .items(vec![
                        AccordionItem::new("Getting Started", '🏠')
                            .children(vec![
                                View::text("Home".to_string())
                                    .on_click(Message::SelectItem("home".to_string())),
                                View::text("Hello".to_string())
                                    .on_click(Message::SelectItem("hello".to_string())),
                            ]),
                        AccordionItem::new("Components", '📦')
                            .children(vec![
                                View::text("Button".to_string())
                                    .on_click(Message::SelectItem("button".to_string())),
                                View::text("Input".to_string())
                                    .on_click(Message::SelectItem("input".to_string())),
                            ]),
                    ])
                    .allow_multiple(true)
                    .initially_expanded(0)
                    .on_toggle(|idx, expanded| Message::ToggleGroup(idx, expanded))
                    .build()
            )
            .build()
    }

    fn on(&mut self, msg: Message) {
        match msg {
            Message::ToggleGroup(idx, expanded) => {
                self.expanded[idx] = expanded;
            }
            Message::SelectItem(item) => {
                println!("Selected: {}", item);
            }
        }
    }
}

fn main() -> auto_ui::AppResult<()> {
    App::run::<MyApp>()
}
```

### Sidebar 完整示例

```rust
impl Component for MyApp {
    type Msg = Message;

    fn view(&self) -> View<Message> {
        View::row()
            .child(
                // 侧边栏
                View::sidebar(
                    View::accordion()
                        .items(items)
                        .build(),
                    300.0
                )
                .collapsible(true)
                .responsive(true)
                .build()
            )
            .child(
                // 主内容区
                View::scrollable(
                    View::col()
                        .child(self.main_content())
                        .build()
                )
                .build()
            )
            .build()
    }
}
```

### Tabs 完整示例

```rust
impl Component for MyApp {
    type Msg = Message;

    fn view(&self) -> View<Message> {
        View::tabs(vec![
            "Home".to_string(),
            "Settings".to_string(),
            "About".to_string(),
        ])
        .selected(0)
        .on_select(|index| Message::TabChanged(index))
        .contents(vec![
            View::text("Welcome to Home".to_string()),
            View::text("Settings Panel".to_string()),
            View::text("About App".to_string()),
        ])
        .position(TabsPosition::Top)
        .build()
    }
}
```

## Integration with Existing Plans

### Plan 008 (Iced Gallery)
- ✅ Phase 2 完成后可以使用 unified Accordion
- ✅ 重构 navigation.rs 使用统一抽象
- ✅ 减少代码重复

### Plan 009 (GPUI Story)
- ✅ 可以使用 unified Sidebar 组织 stories
- ✅ 可以使用 unified Tabs 切换 story 类型
- ✅ 统一两个 Gallery 的实现

### Future Plans
- unified-drawer - 抽屉组件
- unified-menu - 上下文菜单
- unified-breadcrumb - 面包屑导航
- unified-stepper - 步骤条

## Success Criteria

### Must Have
- ✅ Accordion 在 Iced 和 GPUI 中都可用
- ✅ Sidebar 在 Iced 和 GPUI 中都可用
- ✅ Tabs 在 Iced 和 GPUI 中都可用
- ✅ Unified 示例在两个后端中运行
- ✅ API 设计一致且易用

### Nice to Have
- ✅ NavigationRail 实现
- ✅ 动画支持（展开/折叠动画）
- ✅ 主题定制
- ✅ 键盘导航支持
- ✅ 无障碍访问（ARIA）

## Timeline

- **Phase 1** (抽象层): 1 天
- **Phase 2** (Iced 实现): 1-2 天
- **Phase 3** (GPUI 实现): 1-2 天
- **Phase 4** (Unified 示例): 1 天
- **Phase 5** (Gallery 集成): 1 天

**总计**: 4-7 天（MVP: 2-3 天）

## Dependencies

### 新增依赖

**Iced**:
- 无新依赖（使用现有 Iced widgets）

**GPUI**:
- 复用 `gpui-component`

## Usage Examples

### 运行 Unified 示例

```bash
# Iced 后端
cargo run --package unified-accordion --features iced
cargo run --package unified-sidebar --features iced
cargo run --package unified-tabs --features iced

# GPUI 后端
cargo run --package unified-accordion --features gpui
cargo run --package unified-sidebar --features gpui
cargo run --package unified-tabs --features gpui
```

### 在 Gallery 中使用

```bash
# Iced Gallery (使用 unified Accordion)
cargo run --package iced-examples

# GPUI Gallery (Plan 009, 使用 unified Sidebar)
cargo run --package gpui-gallery
```

## Notes

- 优先实现 Accordion（最重要的导航组件）
- 与 Plan 008 和 Plan 009 紧密集成
- 保持 API 简洁，避免过度设计
- 注重两个后端的一致性
- 参考 Material Design 和 Fluent Design 规范

---

**Document Status**: Ready for Implementation
**Last Updated**: 2025-01-23
**Author**: Claude Sonnet 4.5
**Review Status**: Pending
