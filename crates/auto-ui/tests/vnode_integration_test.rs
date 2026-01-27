//! VNode 系统集成测试
//!
//! 测试 VNode 与现有 View 系统的完整集成
//!
//! ## 测试覆盖
//!
//! - 完整的 View → VTree 转换
//! - 复杂嵌套结构的处理
//! - 树结构完整性验证
//! - 与现有 API 的兼容性

use auto_ui::prelude::*;
use auto_ui::vnode_converter::view_to_vtree;
use auto_ui::vnode::{VNodeKind};

/// 测试消息类型
#[derive(Debug, Clone, Copy)]
enum TestMessage {
    Click,
    Change,
    Submit,
}

#[test]
fn test_complete_view_conversion() {
    // 构建一个复杂的 View 树
    let view: View<TestMessage> = View::Column {
        children: vec![
            View::Text {
                content: "Title".to_string(),
                style: None,
            },
            View::Button {
                label: "Click Me".to_string(),
                onclick: TestMessage::Click,
                style: None,
            },
            View::Row {
                children: vec![
                    View::Text {
                        content: "Left".to_string(),
                        style: None,
                    },
                    View::Text {
                        content: "Right".to_string(),
                        style: None,
                    },
                ],
                spacing: 10,
                padding: 0,
                style: None,
            },
        ],
        spacing: 20,
        padding: 10,
        style: None,
    };

    // 转换为 VTree
    let tree = view_to_vtree(view);

    // 验证结构
    assert_eq!(tree.node_count(), 6); // Column + Text + Button + Row + 2 Text

    let root = tree.root().unwrap();
    assert_eq!(root.kind, VNodeKind::Column);
    assert_eq!(root.children.len(), 3);

    // 验证 Row 子树
    let row_id = root.children[2];
    let row = tree.get(row_id).unwrap();
    assert_eq!(row.children.len(), 2);
}

#[test]
fn test_preserves_structure_integrity() {
    // 测试转换后的树结构完整性
    let view: View<TestMessage> = View::Container {
        child: Box::new(View::Text {
            content: "Container Content".to_string(),
            style: None,
        }),
        padding: 20,
        width: None,
        height: None,
        center_x: true,
        center_y: true,
        style: None,
    };

    let tree = view_to_vtree(view);

    assert_eq!(tree.node_count(), 2);

    let root = tree.root().unwrap();
    assert_eq!(root.kind, VNodeKind::Container);

    let child_id = root.children[0];
    let child = tree.get(child_id).unwrap();
    assert_eq!(child.kind, VNodeKind::Text);
    assert_eq!(child.parent, Some(root.id));
}

#[test]
fn test_deeply_nested_structure() {
    // 测试深度嵌套结构的处理
    let view: View<TestMessage> = View::Column {
        children: vec![
            View::Row {
                children: vec![View::Container {
                    child: Box::new(View::Text {
                        content: "Deep".to_string(),
                        style: None,
                    }),
                    padding: 10,
                    width: None,
                    height: None,
                    center_x: false,
                    center_y: false,
                    style: None,
                }],
                spacing: 5,
                padding: 0,
                style: None,
            },
        ],
        spacing: 0,
        padding: 0,
        style: None,
    };

    let tree = view_to_vtree(view);

    // 验证深度
    assert_eq!(tree.depth(), 4); // Column → Row → Container → Text
    assert_eq!(tree.node_count(), 4);
}

#[test]
fn test_large_tree_performance() {
    // 测试较大树的性能和正确性
    let children = (0..100)
        .map(|i| {
            View::Text {
                content: format!("Item {}", i),
                style: None,
            }
        })
        .collect();

    let view: View<TestMessage> = View::Column {
        children,
        spacing: 5,
        padding: 0,
        style: None,
    };

    let tree = view_to_vtree(view);

    // 1 Column + 100 Text = 101 节点
    assert_eq!(tree.node_count(), 101);
    assert_eq!(tree.depth(), 2); // Column → Text

    // 验证树的完整性
    assert!(tree.validate().is_ok());
}

#[test]
fn test_mixed_component_types() {
    // 测试混合不同组件类型的树
    let view: View<TestMessage> = View::Column {
        children: vec![
            View::Text {
                content: "Header".to_string(),
                style: None,
            },
            View::Button {
                label: "Submit".to_string(),
                onclick: TestMessage::Submit,
                style: None,
            },
            View::Input {
                placeholder: "Enter text".to_string(),
                value: "".to_string(),
                on_change: None,
                width: None,
                password: false,
                style: None,
            },
            View::Checkbox {
                is_checked: false,
                label: "Remember me".to_string(),
                on_toggle: None,
                style: None,
            },
            View::Scrollable {
                child: Box::new(View::Text {
                    content: "Scrollable content".to_string(),
                    style: None,
                }),
                width: None,
                height: None,
                style: None,
            },
        ],
        spacing: 10,
        padding: 20,
        style: None,
    };

    let tree = view_to_vtree(view);

    // Column + 5 direct children + 1 Scrollable content = 7 节点
    assert_eq!(tree.node_count(), 7);

    // 验证统计信息
    let stats = tree.stats();
    assert_eq!(stats.total_nodes, 7);
    assert_eq!(stats.text_nodes, 2); // Header + Scrollable content
    assert_eq!(stats.button_nodes, 1);
    assert_eq!(stats.layout_nodes, 1); // Column
}

#[test]
fn test_tree_with_lists() {
    // 测试包含列表的树
    let items = vec![
        View::Text {
            content: "Item 1".to_string(),
            style: None,
        },
        View::Text {
            content: "Item 2".to_string(),
            style: None,
        },
        View::Text {
            content: "Item 3".to_string(),
            style: None,
        },
    ];

    let view: View<TestMessage> = View::List {
        items,
        spacing: 8,
        style: None,
    };

    let tree = view_to_vtree(view);

    // 1 List + 3 Text = 4 节点
    assert_eq!(tree.node_count(), 4);

    let root = tree.root().unwrap();
    assert_eq!(root.kind, VNodeKind::List);
    assert_eq!(root.children.len(), 3);
}

#[test]
fn test_tree_with_table() {
    // 测试包含表格的树
    let headers = vec![
        View::Text {
            content: "Name".to_string(),
            style: None,
        },
        View::Text {
            content: "Value".to_string(),
            style: None,
        },
    ];

    let rows = vec![vec![
        View::Text {
            content: "Row 1".to_string(),
            style: None,
        },
        View::Text {
            content: "100".to_string(),
            style: None,
        },
    ]];

    let view: View<TestMessage> = View::Table {
        headers,
        rows,
        spacing: 5,
        col_spacing: 10,
        style: None,
    };

    let tree = view_to_vtree(view);

    // 1 Table + 2 headers + 2 cells = 5 节点
    assert_eq!(tree.node_count(), 5);

    let root = tree.root().unwrap();
    assert_eq!(root.kind, VNodeKind::Table);
}

#[test]
fn test_tree_stats_accuracy() {
    // 测试树统计信息的准确性
    let view: View<TestMessage> = View::Column {
        children: vec![
            View::Text {
                content: "Title".to_string(),
                style: None,
            },
            View::Button {
                label: "Button 1".to_string(),
                onclick: TestMessage::Click,
                style: None,
            },
            View::Button {
                label: "Button 2".to_string(),
                onclick: TestMessage::Click,
                style: None,
            },
            View::Row {
                children: vec![
                    View::Text {
                        content: "A".to_string(),
                        style: None,
                    },
                    View::Text {
                        content: "B".to_string(),
                        style: None,
                    },
                ],
                spacing: 5,
                padding: 0,
                style: None,
            },
        ],
        spacing: 10,
        padding: 0,
        style: None,
    };

    let tree = view_to_vtree(view);
    let stats = tree.stats();

    assert_eq!(stats.total_nodes, 7); // Column + Title + 2 Buttons + Row + 2 Text
    assert_eq!(stats.text_nodes, 3);
    assert_eq!(stats.button_nodes, 2);
    assert_eq!(stats.layout_nodes, 2); // Column + Row
    assert_eq!(stats.leaf_nodes, 5); // 3 Text + 2 Button
    assert_eq!(stats.max_depth, 3); // Column → Row → Text
}

#[test]
fn test_empty_and_simple_views() {
    // 测试边界情况：空视图和简单视图
    let empty: View<TestMessage> = View::Empty;
    let tree = view_to_vtree(empty);
    assert_eq!(tree.node_count(), 1);

    let simple: View<TestMessage> = View::Text {
        content: "Hello".to_string(),
        style: None,
    };
    let tree = view_to_vtree(simple);
    assert_eq!(tree.node_count(), 1);
    assert!(tree.validate().is_ok());
}

#[test]
fn test_select_conversion() {
    // 测试下拉选择框的转换
    let view: View<TestMessage> = View::Select {
        options: vec![
            "Option 1".to_string(),
            "Option 2".to_string(),
            "Option 3".to_string(),
        ],
        selected_index: Some(1),
        on_select: None,
        style: None,
    };

    let tree = view_to_vtree(view);

    assert_eq!(tree.node_count(), 1);
    let root = tree.root().unwrap();
    assert_eq!(root.kind, VNodeKind::Select);
}

#[test]
fn test_slider_conversion() {
    // 测试滑块的转换
    let view: View<TestMessage> = View::Slider {
        min: 0.0,
        max: 100.0,
        value: 75.0,
        step: Some(5.0),
        on_change: |_| TestMessage::Change,
        style: None,
    };

    let tree = view_to_vtree(view);

    assert_eq!(tree.node_count(), 1);
    let root = tree.root().unwrap();
    assert_eq!(root.kind, VNodeKind::Slider);
}

#[test]
fn test_progress_bar_conversion() {
    // 测试进度条的转换
    let view: View<TestMessage> = View::ProgressBar {
        progress: 0.6,
        style: None,
    };

    let tree = view_to_vtree(view);

    assert_eq!(tree.node_count(), 1);
    let root = tree.root().unwrap();
    assert_eq!(root.kind, VNodeKind::ProgressBar);
}

#[test]
fn test_complex_form_structure() {
    // 测试复杂表单结构的转换
    let view: View<TestMessage> = View::Column {
        children: vec![
            View::Text {
                content: "User Registration".to_string(),
                style: None,
            },
            View::Input {
                placeholder: "Username".to_string(),
                value: "".to_string(),
                on_change: None,
                width: None,
                password: false,
                style: None,
            },
            View::Input {
                placeholder: "Password".to_string(),
                value: "".to_string(),
                on_change: None,
                width: None,
                password: true,
                style: None,
            },
            View::Checkbox {
                is_checked: false,
                label: "Accept terms".to_string(),
                on_toggle: None,
                style: None,
            },
            View::Button {
                label: "Register".to_string(),
                onclick: TestMessage::Submit,
                style: None,
            },
        ],
        spacing: 15,
        padding: 20,
        style: None,
    };

    let tree = view_to_vtree(view);

    // Column + 5 children = 6 节点
    assert_eq!(tree.node_count(), 6);

    // 验证树的完整性
    assert!(tree.validate().is_ok());
    assert_eq!(tree.depth(), 2);
}

#[test]
fn test_radio_conversion() {
    // 测试单选框的转换
    let view: View<TestMessage> = View::Radio {
        label: "Option A".to_string(),
        is_selected: true,
        on_select: None,
        style: None,
    };

    let tree = view_to_vtree(view);

    assert_eq!(tree.node_count(), 1);
    let root = tree.root().unwrap();
    assert_eq!(root.kind, VNodeKind::Radio);
}

#[test]
fn test_multiple_containers() {
    // 测试多个嵌套容器的转换
    let view: View<TestMessage> = View::Container {
        child: Box::new(View::Container {
            child: Box::new(View::Container {
                child: Box::new(View::Text {
                    content: "Nested deeply".to_string(),
                    style: None,
                }),
                padding: 5,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: None,
            }),
            padding: 10,
            width: None,
            height: None,
            center_x: false,
            center_y: false,
            style: None,
        }),
        padding: 15,
        width: None,
        height: None,
        center_x: false,
        center_y: false,
        style: None,
    };

    let tree = view_to_vtree(view);

    // 3 Containers + 1 Text = 4 节点
    assert_eq!(tree.node_count(), 4);
    assert_eq!(tree.depth(), 4);

    // 验证所有容器都是父子关系
    let root = tree.root().unwrap();
    let mut current_id = root.id;

    for _ in 0..3 {
        let node = tree.get(current_id).unwrap();
        assert_eq!(node.kind, VNodeKind::Container);
        assert!(!node.children.is_empty());
        current_id = node.children[0];
    }

    // 最后应该是 Text 节点
    let text = tree.get(current_id).unwrap();
    assert_eq!(text.kind, VNodeKind::Text);
}
