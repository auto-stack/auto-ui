# Plan 008: Iced Gallery Implementation

**Status**: 📋 Planning
**Created**: 2025-01-23
**Priority**: High
**Complexity**: Medium
**Estimated Timeline**: 5-9 days (MVP: 3-4 days)

## Overview

实现一个基于 Iced 的统一 Gallery 应用，参考 [fluent_iced_gallery](D:\github\fluent_iced_gallery) 的设计，展示所有 Iced 控件示例和最佳实践。

## Motivation

### 当前状态
- **iced-examples**: 9个独立示例（hello, counter, button, checkbox, select, dropdown, slider, progress, todos）
- **缺少**: 统一的 Gallery 展示界面
- **用户体验**: 需要分别运行各个示例，难以浏览和对比

### 目标
- ✅ 创建统一的 Gallery 应用，侧边栏导航展示所有示例
- ✅ 支持主题切换（Light/Dark）
- ✅ 响应式设计，小屏幕自动切换到紧凑模式
- ✅ 补充缺失的控件示例（Radio, Text Input, Modal, Tabs 等）
- ✅ 保持独立示例的向后兼容性

## Architecture

### ⚠️ 关键架构原则

**重要**: 本计划遵循以下架构原则：

1. **不创建新的 crate**: Gallery 集成到现有的 `iced-examples` crate 中
2. **main.rs 作为 Gallery 入口**: `iced-examples/src/main.rs` 作为 Gallery 应用的入口点
3. **独立示例保持独立**: 现有的 `examples/` 目录下的独立示例继续保持可独立运行
4. **模块化设计**: Gallery 逻辑作为库模块，可被 main.rs 调用，独立示例也可选择性引用

```
iced-examples/
├── src/
│   ├── main.rs                    # ✅ Gallery 应用入口（默认运行）
│   ├── gallery.rs                 # NEW: 核心 Gallery 逻辑
│   ├── navigation.rs              # NEW: 侧边导航组件
│   ├── theme/                     # NEW: 主题系统
│   │   ├── mod.rs
│   │   ├── theme.rs
│   │   ├── light.rs
│   │   └── dark.rs
│   └── page/                      # NEW: 页面组件
│       ├── mod.rs
│       ├── home.rs
│       ├── button.rs
│       ├── checkbox.rs
│       └── ...
└── examples/                      # ✅ 保持现有独立示例
    ├── hello.rs
    ├── counter.rs
    └── ...
```

### 1. 项目结构

```
auto-ui/
├── crates/
│   └── iced-examples/             # 现有 crate，扩展功能
│       ├── Cargo.toml             # 添加 gallery 相关依赖
│       ├── src/
│       │   ├── main.rs            # ✅ Gallery 应用入口
│       │   ├── gallery.rs         # NEW: 核心 Gallery 逻辑
│       │   ├── navigation.rs      # NEW: 侧边导航组件
│       │   ├── theme/             # NEW: 主题系统
│       │   │   ├── mod.rs
│       │   │   ├── theme.rs
│       │   │   ├── light.rs
│       │   │   └── dark.rs
│       │   └── page/              # NEW: 页面组件
│       │       ├── mod.rs
│       │       ├── home.rs
│       │       ├── button.rs
│       │       ├── checkbox.rs
│       │       ├── radio.rs       # NEW
│       │       ├── text_input.rs  # NEW
│       │       ├── select.rs
│       │       ├── dropdown.rs
│       │       ├── slider.rs
│       │       ├── modal.rs       # NEW
│       │       ├── tabs.rs        # NEW
│       │       └── ...
│       └── examples/              # ✅ 保持现有独立示例
│           ├── hello.rs           # 可独立运行
│           ├── counter.rs         # 可独立运行
│           └── ...
└── examples/                       # 保持现有结构
```

### 2. 核心数据结构

```rust
// app.rs
pub struct Gallery {
    current_page: Page,
    side_nav_display_mode: DisplayMode,
    theme: Theme,
    window_size: iced::Size,
}

// gallery.rs
#[derive(Clone, Debug)]
pub enum Page {
    Home,
    Button(page::button::Button),
    Checkbox(page::checkbox::Checkbox),
    // ... 其他页面
}

#[derive(Clone, Debug)]
pub struct PageGroup {
    pub icon: char,
    pub label: String,
    pub page_items: Vec<PageItem>,
}

#[derive(Clone, Debug)]
pub struct PageItem {
    pub label: String,
    pub page: Page,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayMode {
    Compact,  // < 1000px 宽度，仅显示图标
    Full,     // >= 1000px 宽度，显示图标+文本
}
```

### 3. 消息系统

```rust
#[derive(Clone, Debug)]
pub enum Message {
    // 导航消息
    PageSelected(Page),
    PageGroupToggled(String),
    SideNavModeToggled,

    // 主题消息
    ThemeChanged(Theme),

    // 页面特定消息
    ButtonsPage(buttons::Message),
    CheckboxPage(checkbox::Message),
    // ... 其他页面消息
}
```

## Implementation Plan

### Phase 1: 基础 Gallery 框架（1-2天）

**目标**: 在 iced-examples 中搭建基本的 Gallery 应用结构和导航系统

**任务**:
- [ ] 在 `iced-examples/src/` 下创建 gallery 模块结构
- [ ] 修改 `main.rs` 为 Gallery 应用入口
- [ ] 实现 `Gallery` 核心逻辑（gallery.rs）
- [ ] 实现 `Page` 枚举和所有页面定义
- [ ] 实现侧边导航组件（navigation.rs）
- [ ] 实现主页页面（page/home.rs）
- [ ] 实现基础主题系统（theme/mod.rs, theme/theme.rs）
- [ ] 测试 Gallery 启动

**关键文件**:
- `crates/iced-examples/src/main.rs` - 修改为 Gallery 入口
- `crates/iced-examples/src/gallery.rs` - 核心逻辑
- `crates/iced-examples/src/navigation.rs` - 导航组件
- `crates/iced-examples/Cargo.toml` - 添加依赖

**验证标准**:
- ✅ Gallery 应用可以启动
- ✅ 侧边栏显示分组
- ✅ 主页可以正常显示
- ✅ 主题切换功能正常
- ✅ 独立示例仍可运行

### Phase 2: 集成现有示例（1-2天）

**目标**: 将现有的 9 个示例集成到 Gallery 中

**任务**:
- [ ] 创建页面组件：
  - [ ] `page/button.rs` - 按钮示例
  - [ ] `page/checkbox.rs` - 复选框示例
  - [ ] `page/select.rs` - 选择器示例
  - [ ] `page/dropdown.rs` - 下拉菜单示例
  - [ ] `page/slider.rs` - 滑块示例
  - [ ] `page/progress.rs` - 进度条示例
  - [ ] `page/todos.rs` - 待办事项示例
  - [ ] `page/layout.rs` - 布局示例
  - [ ] `page/circle.rs` - 圆形示例
- [ ] 将独立示例代码重构为可复用模块
- [ ] 在 Gallery 中注册所有页面
- [ ] 测试所有页面在 Gallery 中的渲染

**重构策略**:
```rust
// 之前: iced-examples/src/bin/button.rs
fn main() -> iced::Result {
    // ...
}

// 之后: iced-examples/src/examples/button.rs (库模块)
pub fn view() -> Element<Message> {
    // ...
}

// page/button.rs
pub fn view() -> Element<Message> {
    crate::examples::button::view()
}
```

**验证标准**:
- ✅ 所有 9 个现有示例在 Gallery 中可用
- ✅ 页面切换流畅无卡顿
- ✅ 示例功能与独立运行时一致
- ✅ 保持独立示例仍可运行

### Phase 3: 补充缺失控件（2-3天）

**目标**: 添加缺失的重要控件示例

**P0 优先级**:
- [ ] **Radio** - 单选按钮组
  - 示例：颜色选择、选项组
- [ ] **Text Input** - 文本输入框
  - 示例：单行输入、密码输入、多行输入
- [ ] **Modal** - 模态对话框
  - 示例：确认对话框、表单对话框
- [ ] **Tabs** - 选项卡
  - 示例：基础选项卡、可关闭选项卡

**P1 优先级**（时间允许）:
- [ ] **Toggle** - 开关按钮
- [ ] **Badge/Tag** - 徽章标签
- [ ] **Tooltip** - 提示框
- [ ] **Card** - 卡片容器
- [ ] **Toast** - 通知提示

**验证标准**:
- ✅ 至少完成 P0 的所有 4 个控件
- ✅ 每个控件有清晰的示例和说明
- ✅ 控件与主题系统兼容
- ✅ 交互功能完整（动画、状态管理）

### Phase 4: 完善功能和样式（1-2天）

**目标**: 提升 Gallery 的用户体验和视觉效果

**任务**:
- [ ] 响应式布局
  - 窗口宽度 < 1000px 自动切换到 Compact 模式
  - 侧边栏可折叠/展开
- [ ] 代码示例展示
  - 在每个页面底部显示相关代码片段
  - 代码高亮和复制按钮
- [ ] 键盘快捷键
  - `Ctrl+1-9` 快速切换页面
  - `Ctrl+B` 切换侧边栏
  - `Ctrl+T` 切换主题
- [ ] 改进主题样式
  - 完善 Fluent Design Light/Dark 主题
  - 确保所有组件主题适配
- [ ] 添加动画和过渡效果
  - 页面切换动画
  - 按钮悬停效果
  - 侧边栏展开/折叠动画

**验证标准**:
- ✅ 响应式设计正常工作
- ✅ 键盘快捷键全部可用
- ✅ 代码示例清晰可读
- ✅ 主题切换平滑无闪烁
- ✅ 动画流畅不影响性能

## File Manifest

### 新建文件（~25 个）

**注意**: 所有文件都在现有的 `crates/iced-examples/` 目录下

```
crates/iced-examples/
├── Cargo.toml                           # ✅ 修改：添加 gallery 相关依赖
├── src/
│   ├── main.rs                          # ✅ 修改：Gallery 应用入口
│   ├── gallery.rs                       # NEW: 核心逻辑（200行）
│   ├── navigation.rs                    # NEW: 导航组件（300行）
│   ├── theme/
│   │   ├── mod.rs                       # NEW: 主题模块（50行）
│   │   ├── theme.rs                     # NEW: 主题定义（80行）
│   │   ├── light.rs                     # NEW: 浅色主题（200行）
│   │   └── dark.rs                      # NEW: 深色主题（200行）
│   ├── page/
│   │   ├── mod.rs                       # NEW: 页面模块（100行）
│   │   ├── home.rs                      # NEW: 主页（100行）
│   │   ├── button.rs                    # NEW: 按钮页面（150行）
│   │   ├── checkbox.rs                  # NEW: 复选框页面（150行）
│   │   ├── select.rs                    # NEW: 选择器页面（150行）
│   │   ├── dropdown.rs                  # NEW: 下拉菜单页面（150行）
│   │   ├── slider.rs                    # NEW: 滑块页面（150行）
│   │   ├── progress.rs                  # NEW: 进度条页面（150行）
│   │   ├── todos.rs                     # NEW: 待办事项页面（200行）
│   │   ├── layout.rs                    # NEW: 布局页面（200行）
│   │   ├── circle.rs                    # NEW: 圆形页面（100行）
│   │   ├── radio.rs                     # NEW: 单选按钮（150行）
│   │   ├── text_input.rs                # NEW: 文本输入（200行）
│   │   ├── modal.rs                     # NEW: 模态框（250行）
│   │   └── tabs.rs                      # NEW: 选项卡（200行）
│   └── examples/                        # ✅ 保持：独立示例
│       ├── hello.rs                     # 保持不变
│       ├── counter.rs                   # 保持不变
│       └── ...                           # 其他独立示例保持不变
└── README.md                             # ✅ 修改：添加 Gallery 文档
```

**总代码量**: ~2,500 行（不含现有示例）

### 修改文件

```
crates/iced-examples/Cargo.toml          # 添加 gallery 相关依赖
crates/iced-examples/src/main.rs         # 改为 Gallery 入口
CLAUDE.md                                # 添加 Gallery 文档
```

## Technical Challenges & Solutions

### Challenge 1: 消息类型统一

**问题**: 不同页面有不同的 Message 类型，如何统一？

**解决方案**:
```rust
pub enum Message {
    PageSelected(Page),
    ButtonPage(buttons::Message),
    CheckboxPage(checkbox::Message),
    // ...
}

impl Application {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ButtonPage(msg) => {
                if let Page::Button(page) = &mut self.current_page {
                    page.update(msg)
                }
            }
            // ...
        }
    }
}
```

### Challenge 2: 示例代码复用

**问题**: 如何避免将独立示例代码复制到 Gallery？

**解决方案**:
1. 将独立示例重构为库模块（`src/examples/`）
2. Gallery 页面组件调用库模块
3. 保持 `[[bin]]` 配置用于独立运行

### Challenge 3: 主题系统集成

**问题**: 如何确保所有控件都支持主题？

**解决方案**:
```rust
pub trait StyleSheet {
    type Style: Default;
    fn appearance(&self, style: &Theme) -> Self::Style;
}

// 为每个组件实现 StyleSheet
impl StyleSheet for Button {
    type Style = button::Style;
    fn appearance(&self, theme: &Theme) -> Self::Style {
        match theme {
            Theme::Light => button::primary(),
            Theme::Dark => button::secondary(),
        }
    }
}
```

### Challenge 4: 响应式布局

**问题**: 如何在不同窗口大小下调整布局？

**解决方案**:
```rust
pub fn view(&self) -> Element<Message> {
    let display_mode = if self.window_size.width < 1000.0 {
        DisplayMode::Compact
    } else {
        DisplayMode::Full
    };

    let sidebar = sidebar(&self.pages, display_mode);
    // ...
}

// 订阅窗口大小变化事件
fn subscription(&self) -> Subscription<Message> {
    iced::event::listen_with(
        iced::event::listening_to(std::time::Duration::from_millis(100)),
        || Event::WindowEvent(iced::window::Event::Resized { .. })
    ).map(|_| Message::CheckWindowSize)
}
```

## Usage

### 运行 Gallery

```bash
# 构建并运行
cargo run --package iced-gallery

# 或直接运行 binary
cargo run --bin iced-gallery
```

### 添加新页面

1. 在 `page/` 创建新页面文件
2. 在 `gallery.rs` 的 `Page` 枚举添加变体
3. 在 `pages()` 函数注册页面
4. 实现页面特定的 `update()` 和 `view()`

## Success Criteria

### Must Have
- ✅ Gallery 应用启动正常
- ✅ 侧边栏显示所有页面分组
- ✅ 页面切换流畅无错误
- ✅ 所有现有 9 个示例集成
- ✅ 主题切换功能正常（Light/Dark）
- ✅ 响应式布局（< 1000px 紧凑模式）

### Nice to Have
- ✅ Radio, Text Input, Modal, Tabs 示例
- ✅ 代码示例展示
- ✅ 键盘快捷键
- ✅ 页面切换动画
- ✅ 搜索功能

### Future Work
- 更多控件示例（Badge, Tooltip, Toast, Card）
- 控件交互指南和最佳实践
- 性能基准测试
- 国际化支持

## Timeline

- **Phase 1** (基础框架): 1-2 天
- **Phase 2** (集成示例): 1-2 天
- **Phase 3** (补充控件): 2-3 天
- **Phase 4** (完善功能): 1-2 天

**总计**: 5-9 天（MVP: 3-4 天）

## Dependencies

### 新增依赖

```toml
[dependencies]
iced = { workspace = true }
serde = { version = "1.0", features = ["derive"] }
```

### 复用现有

- `iced` workspace dependency
- 现有 `iced-examples` 代码

## Integration Points

### 与现有系统集成

1. **iced-examples**: 重构为库 + bin 双模式
2. **unified-examples**: 可选集成用于对比
3. **CLAUDE.md**: 更新文档说明 Gallery 用法

### 未来扩展

1. **auto-ui-iced adapter**: 展示抽象层组件
2. **unified-* 示例**: 可添加对比展示
3. **auto-lang**: 展示 .at 文件编译结果

## Notes

- 参考 fluent_iced_gallery 的成熟模式
- 保持代码简洁，避免过度工程
- 优先实现核心功能，渐进增强
- 注重用户体验（性能、美观、易用性）
- 所有独立示例保持可独立运行

---

**Document Status**: Ready for Implementation
**Last Updated**: 2025-01-23
**Author**: Claude Sonnet 4.5
**Review Status**: Pending

---

# Plan 009: GPUI Story Implementation

**Status**: 📋 Planning
**Created**: 2025-01-23
**Priority**: High
**Complexity**: Medium-High
**Estimated Timeline**: 12-17 days (MVP: 6-8 days)

## Overview

实现一个基于 GPUI 的统一 Story/Gallery 系统，参考 [gpui-component story](D:\github\gpui-component\crates\story) 的设计，展示所有 auto-ui 组件、统一示例和原生 GPUI 组件。

## Motivation

### 当前状态
- **gpui-examples**: 9个独立示例（与 iced-examples 相同）
- **unified-examples**: 13个统一示例展示跨后端能力
- **缺少**: 统一的 Story/Gallery 展示界面
- **Plan 007**: 已实现 native Select widget，需要展示平台

### 目标
- ✅ 创建统一的 Story 系统，类似 Storybook
- ✅ 展示三种类型的内容：
  1. auto-ui 抽象组件（Story 展示）
  2. unified-* 示例（跨后端演示）
  3. 原生 GPUI 组件（直接调用）
- ✅ 侧边栏分组、搜索功能
- ✅ 主题切换、配置管理
- ✅ 保持独立示例向后兼容

## Architecture

### 1. 项目结构

```
auto-ui/
├── crates/
│   ├── auto-ui-gpui/              # 现有 GPUI adapter
│   ├── gpui-examples/             # 现有框架特定示例（保持不变）
│   └── auto-ui-gpui-story/        # NEW: Story 系统库
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs             # 公共 API（50行）
│           ├── story.rs           # Story trait 定义（100行）
│           ├── story_container.rs # StoryContainer 包装器（150行）
│           ├── gallery.rs         # 主 Gallery 应用（250行）
│           ├── sidebar.rs         # 侧边栏组件（350行）
│           ├── preview.rs         # 预览区域（200行）
│           ├── theme.rs           # 主题管理（150行）
│           └── stories/           # Story 实现
│               ├── mod.rs           # Story 注册（150行）
│               ├── welcome.rs       # 欢迎页面（100行）
│               ├── components/      # auto-ui 组件 stories
│               │   ├── mod.rs
│               │   ├── button.rs     # （200行）
│               │   ├── input.rs      # （200行）
│               │   ├── select.rs     # （200行）
│               │   ├── checkbox.rs   # （150行）
│               │   ├── slider.rs     # （150行）
│               │   ├── layout.rs     # （200行）
│               │   └── table.rs      # （250行）
│               ├── examples/        # unified-* 包装器
│               │   ├── mod.rs
│               │   ├── counter.rs    # （50行）
│               │   ├── select.rs     # （50行）
│               │   ├── todos.rs      # （50行）
│               │   └── ...           # （其他 unified 示例）
│               └── native/          # 原生 GPUI stories
│                   ├── mod.rs
│                   ├── button.rs     # （150行）
│                   ├── input.rs      # （150行）
│                   └── ...
└── examples/
    └── gpui-gallery/              # NEW: Gallery 应用
        ├── Cargo.toml
        └── src/
            └── main.rs            # 入口点（100行）
```

### 2. 核心 Story Trait

```rust
// story.rs
use gpui::*;
use auto_ui::Component;

/// Story trait - 所有展示组件的统一接口
pub trait Story: Render {
    /// Story 显示的标题
    fn title() -> &'static str
    where
        Self: Sized;

    /// Story 描述
    fn description() -> &'static str
    where
        Self: Sized;

    /// 创建新实例
    fn new() -> Self
    where
        Self: Sized;

    /// 激活/失活回调
    fn on_active(&mut self, active: bool, window: &mut Window, cx: &mut Context<Self>) {
        let _ = (active, window, cx);
    }
}

/// 为所有 Component 提供 blanket 实现
impl<C> Story for C
where
    C: Component + Render + Default + 'static,
    C::Msg: Clone + std::fmt::Debug + 'static,
{
    fn title() -> &'static str {
        std::any::type_name::<C>()
            .split("::")
            .last()
            .unwrap_or("Unknown")
    }

    fn description() -> &'static str {
        "AutoUI Component Example"
    }

    fn new() -> Self {
        Self::default()
    }
}
```

### 3. StoryContainer

```rust
// story_container.rs
use gpui::*;
use crate::story::Story;

/// 包装 Story 提供面板功能
pub struct StoryContainer<S: Story> {
    story: S,
    is_active: bool,
    title: &'static str,
    description: &'static str,
}

impl<S: Story> StoryContainer<S> {
    pub fn new(story: S) -> Self {
        Self {
            story,
            is_active: false,
            title: S::title(),
            description: S::description(),
        }
    }

    pub fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_active {
            self.is_active = true;
            self.story.on_active(true, window, cx);
        }
    }
}

impl<S: Story> Render for StoryContainer<S> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.story.render(window, cx)
    }
}

/// 辅助函数：创建 story panel
pub fn panel<S: Story + 'static>() -> StoryContainer<S> {
    StoryContainer::new(S::new())
}
```

### 4. Gallery 应用

```rust
// gallery.rs
use gpui::*;
use gpui_component::*;
use crate::sidebar::Sidebar;

pub struct Gallery {
    sidebar: Entity<Sidebar>,
    active_story: Option<Box<dyn std::any::Any>>,
    theme: Theme,
}

impl Gallery {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(|cx| Sidebar::new(cx));

        Self {
            sidebar,
            active_story: None,
            theme: Theme::Dark,
        }
    }

    pub fn register_group(&mut self, name: &str, stories: Vec<StoryEntry>, cx: &mut Context<Self>) {
        self.sidebar.update(cx, |sidebar, cx| {
            sidebar.add_group(name, stories, cx);
        });
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .h_flex()
            .child(
                // 侧边栏（300px）
                div()
                    .w(px(300.0))
                    .h_full()
                    .border_r_1()
                    .border_color(gpui::rgb(0x333333))
                    .child(self.sidebar.clone())
            )
            .child(
                // 预览区域
                div()
                    .flex_1()
                    .h_full()
                    .overflow_scrollbar()
                    .child(/* active_story */)
            )
    }
}
```

## Implementation Plan

### Phase 1: 核心 Story 系统（2-3天）

**目标**: 实现 Story trait 和基础 Gallery 框架

**任务**:
- [ ] 创建 `auto-ui-gpui-story` crate
- [ ] 实现 `Story` trait（story.rs）
- [ ] 实现 `StoryContainer`（story_container.rs）
- [ ] 实现 `Sidebar` 基础组件（sidebar.rs）
- [ ] 实现 `Gallery` 应用骨架（gallery.rs）
- [ ] 实现 `WelcomeStory`
- [ ] 测试基本渲染

**验证**:
- ✅ Story trait 编译通过
- ✅ Gallery 应用可以启动
- ✅ WelcomeStory 正常显示
- ✅ 侧边栏显示分组

### Phase 2: 首批 Stories（2天）

**目标**: 实现三类 story 的示例

**任务**:
- [ ] Type A: `ButtonStory` - auto-ui 抽象组件
- [ ] Type B: `UnifiedCounterStory` - unified-* 包装器
- [ ] Type C: `NativeTextStory` - 原生 GPUI 文本组件
- [ ] 实现故事选择和激活逻辑
- [ ] 测试三类 story 渲染

**验证**:
- ✅ 三种 story 类型都能正常工作
- ✅ Sidebar 列表正确
- ✅ 点击 story 可以切换预览
- ✅ 激活/失活回调正常

### Phase 3: Gallery 功能（2-3天）

**目标**: 完善 Gallery UI 和交互

**任务**:
- [ ] 实现搜索功能（sidebar 过滤）
- [ ] 实现分组展开/折叠
- [ ] 实现主题选择器（Dark/Light）
- [ ] 实现字体大小调整
- [ ] 添加键盘导航（上下箭头切换）
- [ ] 优化预览区域布局

**验证**:
- ✅ 搜索可以过滤 story
- ✅ 主题切换正常工作
- ✅ 键盘导航流畅
- ✅ 布局响应式调整

### Phase 4: Story 扩展（3-4天）

**目标**: 集成所有现有示例

**任务**:
- [ ] 创建所有 unified-* 示例的 Type B 包装器：
  - unified-select ✅ (已有 native select widget)
  - unified-counter
  - unified-todos
  - unified-input
  - unified-checkbox
  - unified-radio
  - unified-slider
  - unified-progress
  - unified-layout
  - unified-scroll
  - unified-list
  - unified-table
  - unified-container
- [ ] 创建 auto-ui 组件的 Type A stories：
  - Button, Input, Select, Checkbox, Slider, Layout, Table, Progress
- [ ] 组织成逻辑分组（Getting Started, Components, Unified Examples, Advanced）

**验证**:
- ✅ 所有 unified-* 示例可访问
- ✅ 核心组件都有 story
- ✅ 分组合理清晰
- ✅ 至少 20+ stories

### Phase 5: 高级 Stories（2-3天）

**目标**: 添加原生 GPUI 组件 stories

**任务**:
- [ ] Type C stories:
  - DatePicker（如果可用）
  - Tree（文件树）
  - Modal（对话框）
  - Popover（弹出层）
  - Tooltip（提示框）
  - CodeEditor（代码编辑器，如果简单）
- [ ] 交互式控件（knobs）
- [ ] 源代码查看功能

**验证**:
- ✅ 至少 5 个原生组件 stories
- ✅ 控件交互功能正常
- ✅ 源代码显示正确

### Phase 6: 文档和完善（1-2天）

**任务**:
- [ ] 创建 `gpui-gallery` binary
- [ ] 编写 README
- [ ] 添加 "如何添加 story" 指南
- [ ] 更新 CLAUDE.md
- [ ] 添加截图和示例

**验证**:
- ✅ 文档完整清晰
- ✅ 新开发者可以添加 story
- ✅ Gallery 示例运行正常

## File Manifest

### 新建文件（~25 个）

```
crates/auto-ui-gpui-story/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                  # 公共 API（50行）
│   ├── story.rs                # Story trait（100行）
│   ├── story_container.rs      # StoryContainer（150行）
│   ├── gallery.rs              # Gallery 应用（250行）
│   ├── sidebar.rs              # Sidebar UI（350行）
│   ├── preview.rs              # 预览区域（200行）
│   ├── theme.rs                # 主题管理（150行）
│   └── stories/
│       ├── mod.rs              # Story 注册（150行）
│       ├── welcome.rs          # 欢迎 story（100行）
│       ├── components/
│       │   ├── mod.rs
│       │   ├── button.rs        # （200行）
│       │   ├── input.rs         # （200行）
│       │   ├── select.rs        # （200行）
│       │   ├── checkbox.rs      # （150行）
│       │   ├── slider.rs        # （150行）
│       │   ├── layout.rs        # （200行）
│       │   └── table.rs         # （250行）
│       ├── examples/
│       │   ├── mod.rs           # 包装器注册（100行）
│       │   ├── counter.rs       # （50行）
│       │   ├── select.rs        # （50行）
│       │   ├── todos.rs         # （50行）
│       │   └── ...              # 其他 unified 包装器
│       └── native/
│           ├── mod.rs
│           ├── button.rs        # （150行）
│           ├── input.rs         # （150行）
│           └── ...

examples/gpui-gallery/
├── Cargo.toml
├── README.md
└── src/
    └── main.rs                  # 入口点（100行）
```

**总代码量**: ~3,500 行

## Technical Challenges & Solutions

### Challenge 1: Entity 类型擦除

**问题**: GPUI 需要具体的 Entity 类型，但我们要存储异构 stories

**解决方案**:
```rust
active_story: Option<Box<dyn std::any::Any>>,

// 使用时 downcast
if let Some(story) = self.active_story.as_ref() {
    if let Some(typed) = story.downcast_ref::<StoryContainer<ButtonStory>>() {
        // 使用 typed story
    }
}
```

### Challenge 2: 消息类型多样性

**问题**: 不同 story 有不同的 Message 类型

**解决方案**: Stories 内部管理消息，Gallery 不需要知道具体消息类型

### Challenge 3: Unified Example 依赖

**问题**: unified-* 示例是独立的 binaries

**解决方案**:
1. 转换为 libraries（`[[lib]]`）
2. 在 `auto-ui-gpui-story/Cargo.toml` 添加依赖

### Challenge 4: Select Widget Entity 生命周期

**问题**: Plan 007 的 Select 需要 pre-initialization

**解决方案**: 在 `Story::on_active()` 中初始化
```rust
fn on_active(&mut self, active: bool, window: &mut Window, cx: &mut Context<Self>) {
    if active {
        // Pre-initialize Select states
    }
}
```

## Integration with Plan 007

Plan 007 的 native Select widget 完全兼容此 Story 系统：

1. **Entity Pre-initialization**: 在 `Story::on_active()` 中进行
2. **Window Access**: 回调参数提供 window 和 cx
3. **事件订阅**: 每个 story 管理自己的 entities
4. **状态缓存**: 每个 story 有自己的 `GpuiComponentState`

## Success Criteria

### Must Have
- ✅ Gallery 应用启动正常
- ✅ Sidebar 显示所有 story 分组
- ✅ 点击 story 显示预览
- ✅ 搜索功能正常
- ✅ 所有 13 个 unified-* 示例可访问
- ✅ 核心组件有 stories
- ✅ 主题切换工作

### Nice to Have
- ✅ 交互式控件（knobs）
- ✅ 源代码查看
- ✅ 全屏模式
- ✅ 性能指标显示
- ✅5+ 原生 GPUI 组件 stories

## Timeline

- **Phase 1** (核心系统): 2-3 天
- **Phase 2** (首批 Stories): 2 天
- **Phase 3** (Gallery 功能): 2-3 天
- **Phase 4** (Story 扩展): 3-4 天
- **Phase 5** (高级 Stories): 2-3 天
- **Phase 6** (文档): 1-2 天

**总计**: 12-17 天（MVP: 6-8 天）

## Dependencies

```toml
[dependencies]
auto-ui = { path = "../auto-ui" }
auto-ui-gpui = { path = "../auto-ui-gpui" }
gpui = "0.2.2"
gpui-component = { workspace = true }
gpui-storybook = { workspace = true }

# Unified examples
unified-select = { path = "../../../examples/unified-select" }
unified-counter = { path = "../../../examples/unified-counter" }
# ...
```

## Integration Points

1. **auto-ui-gpui**: 复用 `GpuiComponentState` 和 Plan 007 的 Select 实现
2. **unified-examples**: 转换为 libraries 并添加依赖
3. **gpui-component**: 直接使用原生组件
4. **gpui-storybook**: 参考其 Story 系统设计

## Notes

- 三种 story 类型提供灵活性
- 与 Plan 007 的 Select widget 无缝集成
- 保持向后兼容（独立示例仍可运行）
- 渐进式实施，MVP 优先
- 注重用户体验和性能

---

**Document Status**: Ready for Implementation
**Last Updated**: 2025-01-23
**Author**: Claude Sonnet 4.5
**Review Status**: Pending
