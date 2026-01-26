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
};

/// GPUI 动态解释器组件
///
/// 此组件负责：
/// 1. 加载并解释 Auto 代码
/// 2. 将求值的 Node 转换为 View
/// 3. 在 GPUI 中渲染 View
/// 4. 处理用户交互事件
pub struct DynamicInterpreterComponent {
    /// 解释器桥梁
    #[cfg(feature = "interpreter")]
    bridge: Arc<RwLock<InterpreterBridge>>,

    /// Auto 文件路径
    file_path: PathBuf,

    /// 当前视图
    #[cfg(feature = "interpreter")]
    current_view: Option<View<DynamicMessage>>,

    /// 焦点句柄
    focus_handle: FocusHandle,

    /// 错误信息（如果有）
    error: Option<String>,
}

impl DynamicInterpreterComponent {
    /// 从 .at 文件创建新组件
    #[cfg(feature = "interpreter")]
    pub fn from_file(path: impl Into<PathBuf>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let path = path.into();
        let bridge = Arc::new(RwLock::new(InterpreterBridge::new()));

        // 尝试加载文件
        let mut component = Self {
            bridge: bridge.clone(),
            file_path: path.clone(),
            current_view: None,
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
        // TODO: 传递 metadata 参数以支持类型化消息
        let view = convert_node_dynamic(&node, None)
            .map_err(|e| format!("转换视图失败: {}", e))?;

        self.current_view = Some(view);
        self.error = None;

        // 通知 GPUI 需要重新渲染
        cx.notify();

        Ok(())
    }

    /// 重新加载文件（热重载）
    #[cfg(feature = "interpreter")]
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        if let Err(e) = self.load_file(&self.file_path, cx) {
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

        self.current_view = Some(view);
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

        // 否则渲染当前视图
        #[cfg(feature = "interpreter")]
        {
            if let Some(ref view) = self.current_view {
                self.render_view(view.clone(), cx)
            } else {
                // 加载中...
                div()
                    .size_full()
                    .bg(rgb(0x1a1a1a))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("⏳ 正在加载...")
            }
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
    /// 渲染单个 View 节点
    #[cfg(feature = "interpreter")]
    fn render_view(&mut self, view: View<DynamicMessage>, cx: &mut Context<Self>) -> AnyElement {
        match view {
            View::Empty => div().into_any(),

            View::Text { content, style } => {
                // TODO: 应用 style
                div()
                    .text_sm()
                    .child(content)
                    .into_any()
            }

            View::Button { label, onclick, style } => {
                // TODO: 应用 style
                let bridge = self.bridge.clone();
                div()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x3c3c3c))
                    .border_1()
                    .border_color(rgb(0x6c6c6c))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|div| {
                        div.bg(rgb(0x4c4c4c))
                    })
                    .child(label)
                    .on_click(cx.listener(move |_this, _event, _window, _cx| {
                        // 分发事件
                        if let Ok(mut b) = bridge.write() {
                            let _ = b.handle_message(onclick.clone());
                        }
                    }))
                    .into_any()
            }

            View::Center { children, .. } => {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .children(
                        children.into_iter()
                            .map(|child| self.render_view(child, cx))
                    )
                    .into_any()
            }

            View::Column { spacing, children, .. } => {
                div()
                    .flex()
                    .flex_col()
                    .gap(spacing as f32)
                    .children(
                        children.into_iter()
                            .map(|child| self.render_view(child, cx))
                    )
                    .into_any()
            }

            View::Row { spacing, children, .. } => {
                div()
                    .flex()
                    .flex_row()
                    .gap(spacing as f32)
                    .children(
                        children.into_iter()
                            .map(|child| self.render_view(child, cx))
                    )
                    .into_any()
            }

            View::Container { children, .. } => {
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .children(
                        children.into_iter()
                            .map(|child| self.render_view(child, cx))
                    )
                    .into_any()
            }

            View::Scrollable { children, .. } => {
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .overflow_y_scroll()
                    .children(
                        children.into_iter()
                            .map(|child| self.render_view(child, cx))
                    )
                    .into_any()
            }

            View::Input { placeholder, on_change, .. } => {
                let bridge = self.bridge.clone();
                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x2a2a2a))
                    .border_1()
                    .border_color(rgb(0x4a4a4a))
                    .rounded_md()
                    .text_sm()
                    .child(placeholder)
                    .on_click(cx.listener(move |_this, _event, _window, _cx| {
                        // TODO: 实现输入处理
                        if let Ok(mut b) = bridge.write() {
                            let _ = b.handle_message(on_change.clone());
                        }
                    }))
                    .into_any()
            }

            View::Checkbox { label, checked, on_toggle, .. } => {
                let bridge = self.bridge.clone();
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w_4()
                            .h_4()
                            .border_1()
                            .border_color(if checked { rgb(0x3b82f6) } else { rgb(0x6c6c6c) })
                            .bg(if checked { rgb(0x3b82f6) } else { rgb(0x2a2a2a) })
                            .rounded_sm()
                    )
                    .child(label)
                    .on_click(cx.listener(move |_this, _event, _window, _cx| {
                        if let Some(ref msg) = on_toggle {
                            if let Ok(mut b) = bridge.write() {
                                let _ = b.handle_message(msg.clone());
                            }
                        }
                    }))
                    .into_any()
            }

            View::Radio { label, checked, on_select, .. } => {
                let bridge = self.bridge.clone();
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w_4()
                            .h_4()
                            .border_1()
                            .border_color(if checked { rgb(0x3b82f6) } else { rgb(0x6c6c6c) })
                            .rounded_full()
                    )
                    .child(label)
                    .on_click(cx.listener(move |_this, _event, _window, _cx| {
                        if let Some(ref msg) = on_select {
                            if let Ok(mut b) = bridge.write() {
                                let _ = b.handle_message(msg.clone());
                            }
                        }
                    }))
                    .into_any()
            }

            View::Select { options, selected, .. } => {
                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x2a2a2a))
                    .border_1()
                    .border_color(rgb(0x4a4a4a))
                    .rounded_md()
                    .text_sm()
                    .child(
                        selected
                            .and_then(|i| options.get(i))
                            .cloned()
                            .unwrap_or_else(|| "Select...".to_string())
                    )
                    .into_any()
            }

            View::List { children, .. } => {
                div()
                    .flex()
                    .flex_col()
                    .children(
                        children.into_iter()
                            .map(|child| self.render_view(child, cx))
                    )
                    .into_any()
            }

            View::Table { children, .. } => {
                div()
                    .flex()
                    .flex_col()
                    .children(
                        children.into_iter()
                            .map(|child| self.render_view(child, cx))
                    )
                    .into_any()
            }
        }
    }
}
