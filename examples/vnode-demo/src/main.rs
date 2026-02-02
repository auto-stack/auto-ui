// VNode 渲染演示
//
// 展示 VNode 架构的核心功能：
// 1. 创建 VTree
// 2. 渲染各种 VNodeKind
// 3. 展示布局能力

use auto_ui::{Component, View};
use auto_ui::vnode::{VNode, VNodeId, VNodeKind, VNodeProps, VTree};
use gpui::*;

fn main() {
    auto_ui_gpui::run_app::<VNodeDemoApp>("VNode 架构演示 - Plan 012")
}

#[derive(Clone)]
struct VNodeDemoApp {
    vtree: VTree,
}

impl Default for VNodeDemoApp {
    fn default() -> Self {
        println!("🎨 VNode 演示应用启动");
        println!("✅ 创建示例 VTree...");

        let vtree = create_demo_vtree();
        println!("✅ VTree 创建成功: {} 个节点", vtree.stats().total_nodes);

        Self { vtree }
    }
}

impl Component for VNodeDemoApp {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(16)
            .padding(20)
            .child(View::text("🎨 VNode 架构演示"))
            .child(View::text(format!("✅ 节点总数: {}", self.vtree.stats().total_nodes)))
            .child(View::text(format!("📊 文本节点: {}", self.vtree.stats().text_nodes)))
            .child(View::text(format!("🔘 按钮节点: {}", self.vtree.stats().button_nodes)))
            .child(View::text("────────────────────────"))
            .child(View::text("支持的控件:"))
            .child(View::text("  • Text, Button, Input"))
            .child(View::text("  • Checkbox, Radio, Slider"))
            .child(View::text("  • ProgressBar, List, Table"))
            .child(View::text("  • Column, Row, Container, Center"))
            .child(View::text("────────────────────────"))
            .child(View::text("提示：运行单元测试查看完整功能"))
            .child(View::text("cargo test --package auto-ui vnode"))
            .build()
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
