// VNode 渲染演示
//
// 展示 VNode 架构的核心功能：
// 1. 创建 VTree
// 2. 渲染各种 VNodeKind
// 3. 展示布局能力

use auto_ui::vnode::{VNode, VNodeId, VNodeKind, VNodeProps, VTree};
use gpui::*;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point { x: px(100.0), y: px(100.0) },
                    size: Size { width: px(900.0), height: px(700.0) },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("VNode 架构演示 - Plan 012".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                cx.new_view(|cx| VNodeDemoApp::new(window, cx))
            },
        )
    });
}

struct VNodeDemoApp {
    vtree: VTree,
}

impl VNodeDemoApp {
    fn new(_window: &mut Window, _cx: &mut ViewContext<Self>) -> Self {
        println!("🎨 VNode 演示应用启动");
        println!("✅ 创建示例 VTree...");

        let vtree = create_demo_vtree();
        println!("✅ VTree 创建成功: {} 个节点", vtree.stats().node_count);

        Self { vtree }
    }
}

impl Render for VNodeDemoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x1a1a1a))
            .flex()
            .flex_col()
            .child(
                // 顶部工具栏
                div()
                    .w_full()
                    .h_10()
                    .bg(rgb(0x2a2a2a))
                    .border_b_1()
                    .border_color(rgb(0x3a3a3a))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .child("🎨 VNode 架构演示")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .child(format!("节点数: {}", self.vtree.stats().node_count))
                    )
            )
            .child(
                // 主内容区
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_row()
                    .child(
                        // 左侧面板 - 树结构可视化
                        div()
                            .w(px(250.0))
                            .border_r_1()
                            .border_color(rgb(0x3a3a3a))
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .p_3()
                                    .bg(rgb(0x252525))
                                    .border_b_1()
                                    .border_color(rgb(0x3a3a3a))
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .child("📊 VTree 结构")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .overflow_scroll()
                                    .p_3()
                                    .child(self.render_tree_view())
                            )
                    )
                    .child(
                        // 右侧面板 - 渲染结果
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .p_3()
                                    .bg(rgb(0x252525))
                                    .border_b_1()
                                    .border_color(rgb(0x3a3a3a))
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .child("🖼️ 渲染结果")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .overflow_hidden()
                                    .child(self.render_vtree_content(cx))
                            )
                    )
            )
    }
}

impl VNodeDemoApp {
    /// 渲染树结构视图
    fn render_tree_view(&self) -> impl IntoElement {
        if let Some(root) = self.vtree.root() {
            self.render_node_tree(root.id, 0)
        } else {
            div().child("空树")
        }
    }

    /// 递归渲染节点树
    fn render_node_tree(&self, node_id: VNodeId, depth: usize) -> impl IntoElement {
        let node = match self.vtree.get(node_id) {
            Some(n) => n,
            None => return div().child("节点不存在"),
        };

        let indent = depth * 16;
        let kind_name = format!("{:?}", node.kind);
        let short_name = kind_name.split('(').next().unwrap_or(&kind_name);

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .ml(px(indent as f32))
                    .my_1()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_family("monospace")
                            .text_color(match node.kind {
                                VNodeKind::Text => rgb(0x9cdcfe),
                                VNodeKind::Button => rgb(0x4ec9b0),
                                VNodeKind::Column | VNodeKind::Row => rgb(0xdcdcaa),
                                VNodeKind::Container => rgb(0x569cd6),
                                _ => rgb(0xd4d4d4),
                            })
                            .child(format!("└─ {}", short_name))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x666666))
                            .child(format!("#{}", node.id.as_u64()))
                    )
            )
            .children(
                node.children
                    .iter()
                    .map(|child_id| self.render_node_tree(*child_id, depth + 1))
            )
    }

    /// 渲染 VTree 内容
    fn render_vtree_content(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        if let Some(root) = self.vtree.root() {
            self.render_vnode(root.id, cx)
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child("空树")
        }
    }

    /// 递归渲染 VNode
    fn render_vnode(&self, node_id: VNodeId, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let node = match self.vtree.get(node_id) {
            Some(n) => n,
            None => {
                return div()
                    .text_color(rgb(0xff6b6b))
                    .child(format!("❌ 节点 {} 不存在", node_id.as_u64()))
            }
        };

        match &node.kind {
            VNodeKind::Text => {
                let content = match &node.props {
                    VNodeProps::Text { content } => content.clone(),
                    _ => String::new(),
                };
                div().text_sm().child(content)
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
            }

            VNodeKind::Column => {
                let (spacing, _padding) = match &node.props {
                    VNodeProps::Layout { spacing, padding } => (*spacing, *padding),
                    _ => (10, 0),
                };

                let mut col = div().flex().flex_col().gap(px(spacing as f32));
                for child_id in &node.children {
                    col = col.child(self.render_vnode(*child_id, cx));
                }
                col
            }

            VNodeKind::Row => {
                let (spacing, _padding) = match &node.props {
                    VNodeProps::Layout { spacing, padding } => (*spacing, *padding),
                    _ => (10, 0),
                };

                let mut row = div().flex().flex_row().gap(px(spacing as f32));
                for child_id in &node.children {
                    row = row.child(self.render_vnode(*child_id, cx));
                }
                row
            }

            VNodeKind::Container => {
                let padding = match &node.props {
                    VNodeProps::Container { padding, .. } => *padding,
                    _ => 0,
                };

                let mut container = div()
                    .flex()
                    .size_full()
                    .p(padding as u32);
                if let Some(child_id) = node.children.first() {
                    container = container.child(self.render_vnode(*child_id, cx));
                }
                container
            }

            VNodeKind::Center => {
                let mut center = div().flex().items_center().justify_center().size_full();
                if let Some(child_id) = node.children.first() {
                    center = center.child(self.render_vnode(*child_id, cx));
                }
                center
            }

            VNodeKind::Input => {
                let (placeholder, value, _password) = match &node.props {
                    VNodeProps::Input {
                        placeholder,
                        value,
                        password,
                    } => (placeholder.clone(), value.clone(), *password),
                    _ => (String::new(), String::new(), false),
                };

                let display_text = if value.is_empty() {
                    placeholder.clone()
                } else {
                    value.clone()
                };

                div()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x2a2a2a))
                    .border_1()
                    .border_color(rgb(0x4a4a4a))
                    .rounded_md()
                    .text_sm()
                    .child(display_text)
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
            }

            VNodeKind::List => {
                let spacing = match &node.props {
                    VNodeProps::List { spacing } => *spacing,
                    _ => 8,
                };

                let mut list = div().flex().flex_col().gap(px(spacing as f32));
                for child_id in &node.children {
                    list = list.child(self.render_vnode(*child_id, cx));
                }
                list
            }

            VNodeKind::Table => {
                let (_spacing, col_spacing) = match &node.props {
                    VNodeProps::Table {
                        spacing,
                        col_spacing,
                    } => (*spacing, *col_spacing),
                    _ => (5, 10),
                };

                let mut table = div().flex().flex_col();
                for child_id in &node.children {
                    let child = match self.vtree.get(*child_id) {
                        Some(c) => c,
                        None => continue,
                    };

                    let mut row = div().flex().flex_row().gap(px(col_spacing as f32));
                    for cell_id in &child.children {
                        row = row.child(self.render_vnode(*cell_id, cx));
                    }
                    table = table.child(row);
                }
                table
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
            }

            VNodeKind::ProgressBar => {
                let progress = match &node.props {
                    VNodeProps::ProgressBar { progress } => *progress,
                    _ => 0.0,
                };

                let filled_width = (progress * 300.0) as f32;

                div()
                    .w(px(300.0))
                    .h(px(20.0))
                    .bg(rgb(0x222222))
                    .border_1()
                    .border_color(rgb(0x444444))
                    .child(div().w(px(filled_width)).h(px(20.0)).bg(rgb(0x3b82f6)))
            }

            _ => {
                div()
                    .text_color(rgb(0xf59e0b))
                    .child(format!("🔧 {:?} 组件暂未实现", node.kind))
            }
        }
    }
}

/// 创建演示用的 VTree
fn create_demo_vtree() -> VTree {
    let mut vtree = VTree::new();

    // 根节点 - Column
    let root_id = vtree.next_id();
    let root = VNode::new(
        root_id,
        VNodeKind::Column,
        VNodeProps::Layout {
            spacing: 20,
            padding: 20,
        },
    );
    vtree.set_root(root);

    // 辅助函数：添加子节点
    fn add_child(vtree: &mut VTree, parent_id: VNodeId, node: VNode) {
        let child_id = node.id;
        vtree.add_node(node);
        if let Some(parent) = vtree.get_mut(parent_id) {
            parent.add_child(child_id);
        }
    }

    // 标题
    let title_id = vtree.next_id();
    let title = VNode::new(
        title_id,
        VNodeKind::Text,
        VNodeProps::Text {
            content: "VNode 架构渲染演示".to_string(),
        },
    );
    add_child(&mut vtree, root_id, title);

    // 副标题
    let subtitle_id = vtree.next_id();
    let subtitle = VNode::new(
        subtitle_id,
        VNodeKind::Text,
        VNodeProps::Text {
            content: "展示 Plan 012 Phase 1-2 的成果".to_string(),
        },
    );
    add_child(&mut vtree, root_id, subtitle);

    // 控件展示 - Row
    let controls_row_id = vtree.next_id();
    let controls_row = VNode::new(
        controls_row_id,
        VNodeKind::Row,
        VNodeProps::Layout {
            spacing: 15,
            padding: 10,
        },
    );
    add_child(&mut vtree, root_id, controls_row);

    // 按钮
    let button_id = vtree.next_id();
    let button = VNode::new(
        button_id,
        VNodeKind::Button,
        VNodeProps::Button {
            label: "点击按钮".to_string(),
        },
    );
    add_child(&mut vtree, controls_row_id, button);

    // 输入框
    let input_id = vtree.next_id();
    let input = VNode::new(
        input_id,
        VNodeKind::Input,
        VNodeProps::Input {
            placeholder: "输入示例".to_string(),
            value: String::new(),
            password: false,
        },
    );
    add_child(&mut vtree, controls_row_id, input);

    // 复选框
    let checkbox_id = vtree.next_id();
    let checkbox = VNode::new(
        checkbox_id,
        VNodeKind::Checkbox,
        VNodeProps::Checkbox {
            label: "记住我".to_string(),
            is_checked: true,
        },
    );
    add_child(&mut vtree, controls_row_id, checkbox);

    // 单选框
    let radio_id = vtree.next_id();
    let radio = VNode::new(
        radio_id,
        VNodeKind::Radio,
        VNodeProps::Radio {
            label: "选项 A".to_string(),
            is_selected: true,
        },
    );
    add_child(&mut vtree, controls_row_id, radio);

    // 滑块容器
    let slider_container_id = vtree.next_id();
    let slider_container = VNode::new(
        slider_container_id,
        VNodeKind::Container,
        VNodeProps::Container {
            padding: 10,
            center_x: false,
            center_y: false,
        },
    );
    add_child(&mut vtree, root_id, slider_container);

    // 滑块标题
    let slider_label_id = vtree.next_id();
    let slider_label = VNode::new(
        slider_label_id,
        VNodeKind::Text,
        VNodeProps::Text {
            content: "音量控制".to_string(),
        },
    );
    add_child(&mut vtree, slider_container_id, slider_label);

    // 滑块
    let slider_id = vtree.next_id();
    let slider = VNode::new(
        slider_id,
        VNodeKind::Slider,
        VNodeProps::Slider {
            min: 0.0,
            max: 100.0,
            value: 65.0,
            step: None,
        },
    );
    add_child(&mut vtree, slider_container_id, slider);

    // 进度条容器
    let progress_container_id = vtree.next_id();
    let progress_container = VNode::new(
        progress_container_id,
        VNodeKind::Container,
        VNodeProps::Container {
            padding: 10,
            center_x: false,
            center_y: false,
        },
    );
    add_child(&mut vtree, root_id, progress_container);

    // 进度条标题
    let progress_label_id = vtree.next_id();
    let progress_label = VNode::new(
        progress_label_id,
        VNodeKind::Text,
        VNodeProps::Text {
            content: "下载进度: 75%".to_string(),
        },
    );
    add_child(&mut vtree, progress_container_id, progress_label);

    // 进度条
    let progress_bar_id = vtree.next_id();
    let progress_bar = VNode::new(
        progress_bar_id,
        VNodeKind::ProgressBar,
        VNodeProps::ProgressBar { progress: 0.75 },
    );
    add_child(&mut vtree, progress_container_id, progress_bar);

    // 列表演示
    let list_id = vtree.next_id();
    let list = VNode::new(list_id, VNodeKind::List, VNodeProps::List { spacing: 8 });
    add_child(&mut vtree, root_id, list);

    // 列表项
    for i in 1..=3 {
        let item_id = vtree.next_id();
        let item = VNode::new(
            item_id,
            VNodeKind::Text,
            VNodeProps::Text {
                content: format!("列表项 {}", i),
            },
        );
        add_child(&mut vtree, list_id, item);
    }

    // 表格演示
    let table_id = vtree.next_id();
    let table = VNode::new(
        table_id,
        VNodeKind::Table,
        VNodeProps::Table {
            spacing: 5,
            col_spacing: 15,
        },
    );
    add_child(&mut vtree, root_id, table);

    // 表格数据
    let headers = vec!["姓名", "年龄", "城市"];
    let data = vec![
        vec!["张三", "25", "北京"],
        vec!["李四", "30", "上海"],
    ];

    // 表头行
    let header_row_id = vtree.next_id();
    let header_row = VNode::new(header_row_id, VNodeKind::Row, VNodeProps::Layout {
        spacing: 0,
        padding: 0,
    });
    add_child(&mut vtree, table_id, header_row);

    for header in &headers {
        let cell_id = vtree.next_id();
        let cell = VNode::new(
            cell_id,
            VNodeKind::Text,
            VNodeProps::Text {
                content: format!("**{}**", header),
            },
        );
        add_child(&mut vtree, header_row_id, cell);
    }

    // 数据行
    for row_data in &data {
        let row_id = vtree.next_id();
        let row = VNode::new(row_id, VNodeKind::Row, VNodeProps::Layout {
            spacing: 0,
            padding: 0,
        });
        add_child(&mut vtree, table_id, row);

        for cell_data in row_data {
            let cell_id = vtree.next_id();
            let cell = VNode::new(
                cell_id,
                VNodeKind::Text,
                VNodeProps::Text {
                    content: cell_data.to_string(),
                },
            );
            add_child(&mut vtree, row_id, cell);
        }
    }

    // 底部说明
    let footer_id = vtree.next_id();
    let footer = VNode::new(
        footer_id,
        VNodeKind::Container,
        VNodeProps::Container {
            padding: 20,
            center_x: true,
            center_y: false,
        },
    );
    add_child(&mut vtree, root_id, footer);

    let footer_text_id = vtree.next_id();
    let footer_text = VNode::new(
        footer_text_id,
        VNodeKind::Text,
        VNodeProps::Text {
            content: "✨ 所有控件均通过 VNode 架构渲染".to_string(),
        },
    );
    add_child(&mut vtree, footer_id, footer_text);

    vtree
}
