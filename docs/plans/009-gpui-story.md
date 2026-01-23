# Plan 009: GPUI Story Gallery 实现

**Status**: ✅ Phase 1-3 完成
**Created**: 2025-01-23
**Priority**: High
**Complexity**: Low-Medium
**Estimated Timeline**: 1-2 天

## Overview

基于 `gpui-component::story` 的完整实现，在 `gpui-examples` crate 中创建一个 Gallery 应用，展示所有 auto-ui 组件和统一示例。

## 关键发现

通过分析 `D:/github/gpui-component/crates/story` 的实现，发现：

1. **gpui-component::story 已经是一个完整的 Gallery 实现**
   - 位置：`D:/github/gpui-component/crates/story/src/main.rs`
   - 包含完整的 Gallery UI、Sidebar、Story 系统
   - 已实现 50+ 组件 stories

2. **核心组件**：
   - `Gallery` - 主应用（管理 stories、搜索、导航）
   - `StoryContainer` - Story 包装器
   - `Story` trait - 统一的 Story 接口
   - `Sidebar`, `SidebarGroup`, `SidebarMenu` - 导航组件

3. **关键函数**：
   - `gpui_component_story::init(cx)` - 初始化 story 系统
   - `gpui_component_story::create_new_window()` - 创建窗口
   - `StoryContainer::panel::<T>()` - 创建 story panel

## 实现方案

### 方案选择

**推荐方案**：直接复制 `gpui-component::story` 的代码到 `gpui-examples`

**理由**：
1. gpui-component::story 是完整可用的实现
2. 代码量不大（main.rs ~320 行）
3. 可以完全自定义以展示我们的内容
4. 避免依赖复杂的 workspace 外部路径

### 实现步骤

#### Phase 1: 创建基础结构（2-3 小时）

1. **创建 `src/lib.rs`**
   - 导出 Story trait
   - 导出 StoryContainer
   - 导出辅助函数

2. **创建 `src/main.rs`**
   - 复制 `gpui-component/crates/story/src/main.rs`
   - 修改为展示 auto-ui 相关的 stories
   - 调整窗口标题为 "AutoUI GPUI Gallery"

3. **创建 `src/gallery.rs`**
   - 复制 Gallery 结构
   - 修改 stories 列表以展示我们的内容

**关键文件**：
```
crates/gpui-examples/
├── src/
│   ├── main.rs           # ✅ 新建：Gallery 入口
│   ├── lib.rs            # ✅ 新建：库导出
│   ├── gallery.rs        # ✅ 新建：Gallery 应用
│   └── bin/              # ✅ 保持：现有独立示例
└── Cargo.toml            # ✅ 修改：添加依赖
```

#### Phase 2: 实现 Welcome Story（1 小时）

创建 `src/stories/welcome.rs`：
```rust
use gpui::*;
use gpui_component::*;

pub struct WelcomeStory {
    focus_handle: FocusHandle,
}

impl Story for WelcomeStory {
    fn title() -> &'static str {
        "Welcome to AutoUI"
    }

    fn description() -> &'static str {
        "Unified UI framework for GPUI and Iced"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|cx| Self::new(window, cx))
    }
}
```

#### Phase 3: 添加 auto-ui 组件 Stories（2-3 小时）

创建 stories 展示 auto-ui 组件：

1. **src/stories/mod.rs** - Story 模块
2. **src/stories/button_story.rs** - Button 组件展示
3. **src/stories/select_story.rs** - Select 组件展示（Plan 007）
4. **src/stories/input_story.rs** - Input 组件展示

每个 story 实现 `Story` trait：
```rust
pub trait Story: Render + Sized {
    fn title() -> &'static str;
    fn description() -> &'static str;
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render>;
}
```

#### Phase 4: 集成 unified-* 示例（2-3 小时）

为每个 unified 示例创建 wrapper story：

1. `unified_button_story.rs` - 包装 unified-button
2. `unified_select_story.rs` - 包装 unified-select
3. `unified_gallery_story.rs` - 包装 unified-gallery
4. ...

**策略**：
- 每个 unified example 作为独立的 story
- 使用 `unified-*` 的 main 逻辑作为 story 内容
- 保持统一抽象层的展示

## 详细实现

### 1. 修改 Cargo.toml

```toml
[package]
name = "gpui-examples"
version = "0.1.0"
edition = "2021"

# Gallery 应用
[[bin]]
name = "gallery"
path = "src/main.rs"

# 独立示例（保持现有）
[[bin]]
name = "hello"
path = "src/bin/hello.rs"
# ... 其他 bin

[dependencies]
gpui = "0.2.2"
gpui-component = { workspace = true }
gpui-component-assets = { workspace = true }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }

# Unified examples（作为 libraries）
unified-button = { path = "../../../examples/unified-button" }
unified-select = { path = "../../../examples/unified-select" }
# ...
```

### 2. src/lib.rs

```rust
//! GPUI Examples - Gallery and standalone examples

pub mod gallery;
pub mod stories;

pub use gallery::Gallery;
```

### 3. src/main.rs（简化版）

```rust
use gpui::prelude::*;
use gpui::*;
use gpui_component::{Root, *};
use gpui_component_story::*;

fn main() {
    let app = Application::new();

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window(
            "AutoUI GPUI Gallery",
            move |window, cx| Gallery::view(None, window, cx),
            cx,
        );
    });
}
```

### 4. src/gallery.rs

```rust
use gpui::*;
use gpui_component::*;
use crate::stories::*;

pub struct Gallery {
    stories: Vec<(&'static str, Vec<Entity<StoryContainer>>)>,
    active_group_index: Option<usize>,
    active_index: Option<usize>,
    search_input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl Gallery {
    pub fn new(init_story: Option<&str>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));

        let stories = vec![
            ("Getting Started", vec![
                StoryContainer::panel::<WelcomeStory>(window, cx),
            ]),
            ("Components", vec![
                // 添加 auto-ui 组件 stories
            ]),
            ("Unified Examples", vec![
                // 添加 unified-* 包装器 stories
            ]),
        ];

        // ... 其余实现
    }
}
```

### 5. src/stories/welcome_story.rs

```rust
use gpui::*;
use gpui_component::*;
use gpui_component_story::*;

pub struct WelcomeStory;

impl Story for WelcomeStory {
    fn title() -> &'static str {
        "Welcome to AutoUI"
    }

    fn description() -> &'static str {
        "Unified UI framework for GPUI and Iced backends"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
        })
    }

    fn paddings() -> Pixels {
        px(0.)
    }

    fn zoomable() -> Option<PanelControl> {
        None
    }
}

impl Focusable for WelcomeStory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for WelcomeStory {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .px_12()
            .py_8()
            .child(
                v_flex()
                    .gap_6()
                    .items_center()
                    .child(
                        div().text_4xl().font_weight(FontWeight::BOLD)
                            .child("Welcome to AutoUI")
                    )
                    .child(
                        div().text_xl().text_color(cx.theme().muted_foreground)
                            .child("Unified UI framework for GPUI and Iced")
                    )
            )
    }
}
```

## Stories 组织结构

```
Gallery
├── Getting Started
│   ├── Welcome
│   └── Quick Start
├── Components
│   ├── Button
│   ├── Select (Plan 007 native widget)
│   ├── Input
│   ├── Checkbox
│   └── ...
└── Unified Examples
    ├── unified-button
    ├── unified-select
    ├── unified-gallery
    ├── unified-accordion
    └── ... (所有 13 个 unified 示例)
```

## 依赖关系

### 直接依赖 gpui-component-story

**优点**：
- 最少代码
- 完整功能
- 易于维护

**缺点**：
- 依赖外部路径

### 复制代码

**优点**：
- 完全控制
- 可以自定义
- 不依赖外部路径

**缺点**：
- 需要维护副本

**推荐**：复制核心代码（main.rs + Gallery），依赖 Story trait

## Timeline

- **Phase 1** (基础结构): 2-3 小时
- **Phase 2** (Welcome Story): 1 小时
- **Phase 3** (组件 Stories): 2-3 小时
- **Phase 4** (Unified Examples): 2-3 小时

**总计**: 1-2 天

## Success Criteria

### Must Have
- ✅ Gallery 应用可以启动（`cargo run --bin gallery`）
- ✅ 显示 Welcome Story
- ✅ Sidebar 显示分组
- ✅ 搜索功能工作
- ✅ 至少 5 个 stories（Welcome + 4 个组件/示例）

### Nice to Have
- ✅ 所有 13 个 unified 示例可访问
- ✅ 核心组件有 stories
- ✅ 主题切换工作
- ✅ 键盘导航

## File Manifest

### 新建文件

```
crates/gpui-examples/
├── src/
│   ├── main.rs                  # ✅ 新建：~50 行
│   ├── lib.rs                   # ✅ 新建：~20 行
│   ├── gallery.rs               # ✅ 新建：~300 行（复制 + 修改）
│   └── stories/                 # ✅ 新建目录
│       ├── mod.rs               # ✅ 新建：~150 行
│       ├── welcome_story.rs     # ✅ 新建：~100 行
│       ├── button_story.rs      # ✅ 新建：~200 行
│       ├── select_story.rs      # ✅ 新建：~200 行
│       └── unified/             # ✅ 新建目录
│           ├── mod.rs           # ✅ 新建：~100 行
│           ├── button.rs        # ✅ 新建：~50 行（wrapper）
│           └── select.rs        # ✅ 新建：~50 行（wrapper）
└── Cargo.toml                   # ✅ 修改：添加依赖和 bins
```

**总代码量**: ~1,500 行（大部分是复制/修改）

## 与现有系统集成

1. **独立示例保持不变**：`src/bin/*.rs` 继续可独立运行
2. **Plan 007 Select**：作为 component story 展示
3. **Plan 010 Unified**：作为 unified examples 分组展示
4. **auto-ui-gpui**：组件使用 auto-ui 抽象层

## Notes

- 基于 `gpui-component::story` 的成熟实现
- 渐进式实施，先让基础工作
- 保持向后兼容（独立示例仍可运行）
- 注重用户体验（搜索、导航、响应式）

---

## Phase 1-3 完成总结 ✅

### 完成日期
2025-01-23

### 主要成果

#### 1. Phase 1: 基础结构 ✅
- ✅ 创建 `src/lib.rs` - 库导出模块
- ✅ 创建 `src/main.rs` - Gallery 应用入口
- ✅ 创建 `src/gallery.rs` - Gallery 主应用结构
- ✅ 修改 `Cargo.toml` - 添加 gallery binary 和依赖
- ✅ 编译成功，无警告无错误

#### 2. Phase 2: Welcome Story ✅
- ✅ 创建 `src/stories/welcome_story.rs`
- ✅ 实现 Welcome Story 展示欢迎页面
- ✅ 支持 Focusable trait

#### 3. Phase 3: 组件 Stories ✅
- ✅ 创建 `src/stories/button_story.rs` - Button 组件展示
  - 展示 Primary、Ghost、Danger 三种样式
  - 展示不同尺寸（Large、Default、Small、XSmall）
  - 展示 Disabled 状态
  - 展示交互式点击计数器
  - 展示图标按钮
- ✅ 创建 `src/stories/select_story.rs` - Select 组件展示（Plan 007）
  - 展示下拉选择功能
  - 展示水果选择示例
  - 展示事件订阅和处理
  - 说明原生实现特性
- ✅ 创建 `src/stories/checkbox_story.rs` - Checkbox 组件展示
  - 展示基础复选框
  - 展示 Mood Tracker 示例
  - 展示多选功能

### 技术亮点

1. **正确的 GPUI 模式**
   - 使用 `async move |cx|` 而非 `|cx| async move` 避免类型注解错误
   - 使用 `move |cx|` 在 `app.run()` 中捕获环境
   - 正确导入 traits：`ScrollableElement`, `ButtonVariants`
   - 使用 `disabled(true)` 而非 `disabled()` 传递参数

2. **组件状态管理**
   - 使用 `cx.listener()` 创建事件监听器
   - 使用 `cx.subscribe_in()` 订阅 Select 状态变化
   - 正确实现 `Focusable` trait
   - 使用 `FocusHandle` 管理焦点

3. **布局和样式**
   - 使用 `v_flex()`, `h_flex()` 进行布局
   - 使用 `.gap_*()`, `.p_*()`, `.px_*()`, `.py_*()` 设置间距
   - 使用 `.text_size(px(...))` 设置文字大小（GPUI 不支持 Tailwind 风格方法）
   - 使用 `gpui::Size` 避免类型歧义

### 文件清单

**新建文件**：
```
crates/gpui-examples/
├── src/
│   ├── main.rs                  # ✅ 54 行 - Gallery 入口
│   ├── lib.rs                   # ✅ 11 行 - 库导出
│   ├── gallery.rs               # ✅ 58 行 - Gallery 应用
│   └── stories/
│       ├── mod.rs               # ✅ 15 行 - Story 模块导出
│       ├── welcome_story.rs     # ✅ 59 行 - 欢迎 Story
│       ├── button_story.rs      # ✅ 220 行 - Button Story
│       ├── select_story.rs      # ✅ 182 行 - Select Story (Plan 007)
│       └── checkbox_story.rs    # ✅ 175 行 - Checkbox Story
└── Cargo.toml                   # ✅ 修改 - 添加 gallery binary
```

**总代码量**: ~774 行

### 编译验证

```bash
cargo check --bin gallery
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.19s
```

✅ 所有文件编译通过，无警告无错误

### 下一步行动

#### Phase 4: 集成 unified-* 示例（待实施）
- [ ] 创建 `src/stories/unified/` 目录
- [ ] 为每个 unified 示例创建 wrapper story
- [ ] 在 Gallery 中注册 unified stories
- [ ] 测试所有 unified 示例在 Gallery 中的显示

#### 可选增强功能
- [ ] 实现完整的 Gallery UI（Sidebar、搜索、导航）
- [ ] 添加更多组件 Stories（Slider、Progress、Circle、Layout）
- [ ] 添加主题切换功能
- [ ] 添加键盘导航

### 遇到的问题和解决方案

| 问题 | 解决方案 |
|------|---------|
| `async move` 闭包类型注解错误 | 使用 `async move \|cx\|` 而非 `\|cx\| async move` |
| `Size` 类型歧义 | 使用 `gpui::Size` 明确指定类型 |
| `.primary()` 等方法不可用 | 导入 `gpui_component::button::*` 包含 ButtonVariants trait |
| `.disabled()` 缺少参数 | 使用 `.disabled(true)` 传递布尔值 |
| `overflow_y_scroll()` 不存在 | 导入 `ScrollableElement` trait 并使用 `.overflow_y_scrollbar()` |
| `text_4xl()` 等方法不存在 | 使用 `.text_size(px(...))` 代替 |

---

**Document Status**: Phase 1-3 Completed
**Last Updated**: 2025-01-23
**Author**: Claude Sonnet 4.5
**Review Status**: Completed
