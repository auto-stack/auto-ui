//! GPUI 动态解释器组件
//!
//! 此组件提供 GPUI 渲染支持，将 `View<DynamicMessage>` 渲染为实际的 GPUI 元素。
//!
//! ## 使用示例
//!
//! ```ignore
//! use auto_ui_gpui::interpreter_component::DynamicInterpreterComponent;
//!
//! let component = DynamicInterpreterComponent::new(
//!     "examples/counter.at",
//!     window,
//!     cx
//! );
//!
//! // 在 GPUI 应用中使用
//! cx.new(|cx| component)
//! ```

use gpui::{prelude::*, *};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// 导入 auto-ui 的类型
#[cfg(feature = "interpreter")]
use auto_ui::{
    interpreter::{InterpreterBridge, DynamicMessage},
    node_converter::convert_node_dynamic,
    view::View,
    vnode_converter::view_to_vtree,
    vnode::{VTree, VNodeId},
};

/// GPUI 动态解释器组件
///
/// 此组件负责：
/// 1. 加载并解释 Auto 代码
/// 2. 将求值的 Node 转换为 View
/// 3. 将 View 转换为 VTree（扁平化）
/// 4. 在 GPUI 中渲染 VTree
/// 5. 处理用户交互事件
pub struct DynamicInterpreterComponent {
    /// 解释器桥梁
    #[cfg(feature = "interpreter")]
    bridge: Arc<RwLock<InterpreterBridge>>,

    /// Auto 文件路径
    file_path: PathBuf,

    /// 当前虚拟节点树（Plan 012: VNode 架构）
    #[cfg(feature = "interpreter")]
    vtree: Option<VTree>,

    /// 焦点句柄
    focus_handle: FocusHandle,

    /// 错误信息（如果有）
    error: Option<String>,
}

impl DynamicInterpreterComponent {
    /// 从 .at 文件创建新组件
    #[cfg(feature = "interpreter")]
    pub fn from_file(path: impl Into<PathBuf>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let path = path.into();
        let bridge = Arc::new(RwLock::new(InterpreterBridge::new()));

        // 尝试加载文件
        let mut component = Self {
            bridge: bridge.clone(),
            file_path: path.clone(),
            vtree: None,  // Plan 012: 使用 VTree 而不是 View
            focus_handle: cx.focus_handle(),
            error: None,
        };

        // 加载并解释文件
        if let Err(e) = component.load_file(&path, cx) {
            component.error = Some(format!("加载失败: {}", e));
        }

        component
    }

    /// 加载并解释 Auto 文件
    #[cfg(feature = "interpreter")]
    fn load_file(&mut self, path: &PathBuf, cx: &mut Context<Self>) -> Result<(), String> {
        let mut bridge = self.bridge.write()
            .map_err(|e| format!("获取解释器锁失败: {}", e))?;

        // 加载文件
        bridge.load_file(path)
            .map_err(|e| format!("解释失败: {}", e))?;

        // 获取主视图
        let node = bridge.get_main_view()
            .map_err(|e| format!("获取视图失败: {}", e))?;

        // 转换 Node → View<DynamicMessage>
        let view = convert_node_dynamic(&node, None)
            .map_err(|e| format!("转换视图失败: {}", e))?;

        // Plan 012: 将 View 转换为 VTree（扁平化）
        let vtree = view_to_vtree(view);

        self.vtree = Some(vtree);
        self.error = None;

        // 通知 GPUI 需要重新渲染
        cx.notify();

        Ok(())
    }

    /// 重新加载文件（热重载）
    #[cfg(feature = "interpreter")]
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let path = self.file_path.clone();
        if let Err(e) = self.load_file(&path, cx) {
            self.error = Some(format!("重载失败: {}", e));
            cx.notify();
        }
    }

    /// 处理动态消息事件
    #[cfg(feature = "interpreter")]
    fn handle_message(&mut self, msg: DynamicMessage, cx: &mut Context<Self>) {
        let mut bridge = match self.bridge.write() {
            Ok(b) => b,
            Err(e) => {
                self.error = Some(format!("获取解释器锁失败: {}", e));
                cx.notify();
                return;
            }
        };

        if let Err(e) = bridge.handle_message(msg) {
            self.error = Some(format!("处理事件失败: {}", e));
            cx.notify();
            return;
        }

        // 重新获取视图（状态可能已改变）
        let node = match bridge.get_main_view() {
            Ok(n) => n,
            Err(e) => {
                self.error = Some(format!("获取更新后的视图失败: {}", e));
                cx.notify();
                return;
            }
        };

        let view = match convert_node_dynamic(&node, None) {
            Ok(v) => v,
            Err(e) => {
                self.error = Some(format!("转换更新后的视图失败: {}", e));
                cx.notify();
                return;
            }
        };

        // Plan 012: 将 View 转换为 VTree
        let vtree = view_to_vtree(view);

        self.vtree = Some(vtree);
        self.error = None;
        cx.notify();
    }
}

impl Focusable for DynamicInterpreterComponent {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DynamicInterpreterComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 如果有错误，显示错误信息
        if let Some(ref error) = self.error {
            return div()
                .size_full()
                .bg(rgb(0x1a1a1a))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .p_4()
                        .bg(rgb(0x3c1a1a))
                        .text_color(rgb(0xff6b6b))
                        .child(format!("❌ {}", error))
                );
        }

        // Plan 012 Phase 3: 渲染 VTree 并集成事件处理
        #[cfg(feature = "interpreter")]
        {
            if let Some(vtree) = &self.vtree {
                // 验证树结构
                if let Err(e) = vtree.validate() {
                    return div()
                        .size_full()
                        .bg(rgb(0x1a1a1a))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(0xff6b6b))
                        .child(format!("❌ 树结构验证失败: {}", e));
                }

                // 获取根节点并渲染
                if let Some(root) = vtree.root() {
                    // 将 AnyElement 包装在 Div 中
                    return div().child(self.render_vnode_with_events(root.id, vtree, cx));
                }
            }

            // 加载中...
            return div()
                .size_full()
                .bg(rgb(0x1a1a1a))
                .flex()
                .items_center()
                .justify_center()
                .child("⏳ 正在加载...");
        }

        #[cfg(not(feature = "interpreter"))]
        {
            div()
                .size_full()
                .bg(rgb(0x1a1a1a))
                .flex()
                .items_center()
                .justify_center()
                .child("⚠️ 解释器功能未启用。请启用 'interpreter' feature。")
        }
    }
}

impl DynamicInterpreterComponent {
    /// 带事件处理的 VNode 渲染（Plan 012 Phase 3）
    #[cfg(feature = "interpreter")]
    fn render_vnode_with_events(&self, node_id: VNodeId, vtree: &VTree, cx: &mut Context<Self>) -> AnyElement {
        use auto_ui::vnode::{VNodeKind, VNodeProps};

        let node = match vtree.get(node_id) {
            Some(n) => n,
            None => {
                return div()
                    .text_color(rgb(0xff6b6b))
                    .child(format!("❌ 节点 {} 不存在", node_id))
                    .into_any()
            }
        };

        match &node.kind {
            VNodeKind::Text => {
                let content = match &node.props {
                    VNodeProps::Text { content } => content.clone(),
                    VNodeProps::Empty => String::new(),
                    _ => String::from("(无效)"),
                };
                div().text_sm().child(content).into_any()
            }

            VNodeKind::Button => {
                let label = match &node.props {
                    VNodeProps::Button { label } => label.clone(),
                    _ => String::from("Button"),
                };
                div()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x3b82f6))
                    .rounded_md()
                    .cursor_pointer()
                    .child(label)
                    .into_any()
            }

            VNodeKind::Column => {
                let (spacing, _padding) = match &node.props {
                    VNodeProps::Layout { spacing, padding } => (*spacing, *padding),
                    _ => (10, 0),
                };

                let mut col = div()
                    .flex()
                    .flex_col()
                    .gap(px(spacing as f32));

                for child_id in &node.children {
                    col = col.child(self.render_vnode_with_events(*child_id, vtree, cx));
                }

                col.into_any()
            }

            VNodeKind::Row => {
                let (spacing, _padding) = match &node.props {
                    VNodeProps::Layout { spacing, padding } => (*spacing, *padding),
                    _ => (10, 0),
                };

                let mut row = div()
                    .flex()
                    .flex_row()
                    .gap(px(spacing as f32));

                for child_id in &node.children {
                    row = row.child(self.render_vnode_with_events(*child_id, vtree, cx));
                }

                row.into_any()
            }

            VNodeKind::Container => {
                let (_padding, _center_x, _center_y) = match &node.props {
                    VNodeProps::Container {
                        padding,
                        center_x,
                        center_y,
                    } => (*padding, *center_x, *center_y),
                    _ => (0, false, false),
                };

                let mut container = div().flex().size_full();

                if let Some(child_id) = node.children.first() {
                    container = container.child(self.render_vnode_with_events(*child_id, vtree, cx));
                }

                container.into_any()
            }

            VNodeKind::Scrollable => {
                let mut scrollable = div()
                    .flex()
                    .flex_col()
                    .size_full();

                if let Some(child_id) = node.children.first() {
                    scrollable = scrollable.child(self.render_vnode_with_events(*child_id, vtree, cx));
                }

                scrollable.into_any_element()
            }

            VNodeKind::Input => {
                let placeholder = match &node.props {
                    VNodeProps::Input { placeholder, .. } => placeholder.clone(),
                    _ => String::new(),
                };
                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x2a2a2a))
                    .border_1()
                    .border_color(rgb(0x4a4a4a))
                    .rounded_md()
                    .text_sm()
                    .child(format!("{}: {}", placeholder, "(输入框)"))
                    .into_any()
            }

            VNodeKind::Checkbox => {
                let (label, is_checked) = match &node.props {
                    VNodeProps::Checkbox { label, is_checked } => (label.clone(), *is_checked),
                    _ => (String::new(), false),
                };
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .child(
                        div()
                            .w_4()
                            .h_4()
                            .border_1()
                            .border_color(if is_checked {
                                rgb(0x3b82f6)
                            } else {
                                rgb(0x6c6c6c)
                            })
                            .bg(if is_checked {
                                rgb(0x3b82f6)
                            } else {
                                rgb(0x2a2a2a)
                            })
                            .rounded_sm(),
                    )
                    .child(label)
                    .into_any()
            }

            VNodeKind::Radio => {
                let (label, is_selected) = match &node.props {
                    VNodeProps::Radio { label, is_selected } => (label.clone(), *is_selected),
                    _ => (String::new(), false),
                };
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .child(
                        div()
                            .w_4()
                            .h_4()
                            .border_1()
                            .border_color(if is_selected {
                                rgb(0x3b82f6)
                            } else {
                                rgb(0x6c6c6c)
                            })
                            .rounded_full(),
                    )
                    .child(label)
                    .into_any()
            }

            VNodeKind::Select => {
                let (options, selected_index) = match &node.props {
                    VNodeProps::Select {
                        options,
                        selected_index,
                    } => (options.clone(), *selected_index),
                    _ => (vec![], None),
                };
                let selected = selected_index
                    .and_then(|i| options.get(i))
                    .cloned()
                    .unwrap_or_else(|| "Select...".to_string());
                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x2a2a2a))
                    .border_1()
                    .border_color(rgb(0x4a4a4a))
                    .rounded_md()
                    .text_sm()
                    .child(selected)
                    .into_any()
            }

            VNodeKind::List => {
                let spacing = match &node.props {
                    VNodeProps::List { spacing } => *spacing,
                    _ => 8,
                };

                let mut list = div().flex().flex_col().gap(px(spacing as f32));
                for child_id in &node.children {
                    list = list.child(self.render_vnode_with_events(*child_id, vtree, cx));
                }
                list.into_any()
            }

            VNodeKind::Table => {
                let (_spacing, _col_spacing) = match &node.props {
                    VNodeProps::Table {
                        spacing,
                        col_spacing,
                    } => (*spacing, *col_spacing),
                    _ => (5, 10),
                };

                let mut table = div().flex().flex_col();
                for child_id in &node.children {
                    let child = match vtree.get(*child_id) {
                        Some(c) => c,
                        None => continue,
                    };

                    let mut row = div().flex().flex_row().gap_2();
                    for cell_id in &child.children {
                        row = row.child(self.render_vnode_with_events(*cell_id, vtree, cx));
                    }
                    table = table.child(row);
                }
                table.into_any()
            }

            VNodeKind::Slider => {
                let (min, max, value, _step) = match &node.props {
                    VNodeProps::Slider {
                        min,
                        max,
                        value,
                        step,
                    } => (*min, *max, *value, *step),
                    _ => (0.0, 100.0, 50.0, None),
                };

                let range = max - min;
                let percentage = ((value - min) / range).clamp(0.0, 1.0);

                div()
                    .h(px(16.0))
                    .w(px(300.0))
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(6.0))
                            .h(px(4.0))
                            .w(px(300.0))
                            .bg(rgb(0x333333))
                            .rounded_md(),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(6.0))
                            .h(px(4.0))
                            .w(px(percentage * 300.0))
                            .bg(rgb(0x3b82f6))
                            .rounded_md(),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(percentage * 300.0 - 8.0))
                            .top(px(0.0))
                            .w(px(16.0))
                            .h(px(16.0))
                            .bg(rgb(0xffffff))
                            .rounded_full()
                            .border_2()
                            .border_color(rgb(0x3b82f6))
                            .shadow_lg(),
                    )
                    .into_any()
            }

            VNodeKind::ProgressBar => {
                let progress = match &node.props {
                    VNodeProps::ProgressBar { progress } => *progress,
                    _ => 0.0,
                };

                let filled_width = (progress * 200.0) as f32;

                div()
                    .w(px(200.0))
                    .h(px(20.0))
                    .bg(rgb(0x222222))
                    .border_1()
                    .border_color(rgb(0x444444))
                    .child(div().w(px(filled_width)).h(px(20.0)).bg(rgb(0x3b82f6)))
                    .into_any()
            }

            VNodeKind::Center => {
                let mut center = div().flex().items_center().justify_center().size_full();
                if let Some(child_id) = node.children.first() {
                    center = center.child(self.render_vnode_with_events(*child_id, vtree, cx));
                }
                center.into_any()
            }

            // 高级组件占位符
            VNodeKind::Accordion | VNodeKind::Sidebar | VNodeKind::Tabs | VNodeKind::NavigationRail => {
                div()
                    .text_color(rgb(0xf59e0b))
                    .child(format!("🔧 {:?} 组件暂未实现", node.kind))
                    .into_any()
            }
        }
    }
}



