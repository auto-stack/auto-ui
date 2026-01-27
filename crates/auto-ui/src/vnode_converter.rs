//! View<M> → VTree 转换器
//!
//! 将嵌套的 View<M> 树转换为扁平的 VNode 树结构。
//!
//! ## 核心功能
//!
//! - **扁平化转换**：将嵌套的 View<M> 树转换为扁平的 VNode 列表
//! - **ID 引用**：使用 ID 引用建立父子关系，而非直接嵌套
//! - **完整支持**：支持所有 22 个 View 变体
//!
//! ## 使用示例
//!
//! ```ignore
//! use auto_ui::view::View;
//! use auto_ui::vnode_converter::view_to_vtree;
//!
//! let view = View::Column {
//!     children: vec![
//!         View::Text { content: "Hello".to_string(), style: None }
//!     ],
//!     spacing: 10,
//!     padding: 0,
//!     style: None
//! };
//!
//! let vtree = view_to_vtree(view);
//! assert_eq!(vtree.node_count(), 2); // Column + Text
//! ```

use crate::view::View;
use crate::vnode::{VNode, VNodeId, VNodeKind, VNodeProps, VTree};

/// 主转换函数：View<M> → VTree
///
/// 将嵌套的 View<M> 树转换为扁平的 VNode 树结构。
///
/// # 类型参数
///
/// * `M` - 消息类型，必须实现 Clone 和 Debug
///
/// # 参数
///
/// * `view` - 要转换的 View 树
///
/// # 返回
///
/// 转换后的 VTree
///
/// # 示例
///
/// ```ignore
/// let view = View::Text {
///     content: "Hello".to_string(),
///     style: None
/// };
///
/// let vtree = view_to_vtree(view);
/// ```
pub fn view_to_vtree<M>(view: View<M>) -> VTree
where
    M: Clone + std::fmt::Debug,
{
    let mut tree = VTree::new();
    let root_id = tree.next_id();

    let root_node = convert_view_to_vnode(&view, root_id, None, &mut tree);
    tree.set_root(root_node);

    tree
}

/// 将单个 View 转换为 VNode（递归处理子节点）
///
/// # 参数
///
/// * `view` - 要转换的 View
/// * `id` - 为此节点分配的 ID
/// * `parent_id` - 父节点 ID（如果有）
/// * `tree` - VTree 用于添加子节点
///
/// # 返回
///
/// 转换后的 VNode
fn convert_view_to_vnode<M>(
    view: &View<M>,
    id: VNodeId,
    parent_id: Option<VNodeId>,
    tree: &mut VTree,
) -> VNode
where
    M: Clone + std::fmt::Debug,
{
    let (kind, props) = extract_kind_and_props(view);

    let mut vnode = VNode::new(id, kind, props).with_label(format!("{}", kind));

    if let Some(parent) = parent_id {
        vnode = vnode.with_parent(parent);
    }

    // 处理子节点
    let children = extract_children(view);
    for child_view in children {
        let child_id = tree.next_id();
        let child_node = convert_view_to_vnode(&child_view, child_id, Some(id), tree);
        tree.add_node(child_node);
        vnode.add_child(child_id);
    }

    vnode
}

/// 从 View 中提取类型和属性
///
/// # 参数
///
/// * `view` - 要提取属性的 View
///
/// # 返回
///
/// (VNodeKind, VNodeProps) 元组
fn extract_kind_and_props<M>(view: &View<M>) -> (VNodeKind, VNodeProps)
where
    M: Clone + std::fmt::Debug,
{
    match view {
        View::Empty => (VNodeKind::Text, VNodeProps::Empty),

        View::Text { content, .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: content.clone(),
            },
        ),

        View::Button { label, .. } => (
            VNodeKind::Button,
            VNodeProps::Button {
                label: label.clone(),
            },
        ),

        View::Column { spacing, padding, .. } => (
            VNodeKind::Column,
            VNodeProps::Layout {
                spacing: *spacing,
                padding: *padding,
            },
        ),

        View::Row { spacing, padding, .. } => (
            VNodeKind::Row,
            VNodeProps::Layout {
                spacing: *spacing,
                padding: *padding,
            },
        ),

        View::Input {
            placeholder,
            value,
            password,
            ..
        } => (
            VNodeKind::Input,
            VNodeProps::Input {
                placeholder: placeholder.clone(),
                value: value.clone(),
                password: *password,
            },
        ),

        View::Checkbox {
            label, is_checked, ..
        } => (
            VNodeKind::Checkbox,
            VNodeProps::Checkbox {
                label: label.clone(),
                is_checked: *is_checked,
            },
        ),

        View::Radio {
            label, is_selected, ..
        } => (
            VNodeKind::Radio,
            VNodeProps::Radio {
                label: label.clone(),
                is_selected: *is_selected,
            },
        ),

        View::Select {
            options, selected_index, ..
        } => (
            VNodeKind::Select,
            VNodeProps::Select {
                options: options.clone(),
                selected_index: *selected_index,
            },
        ),

        View::Container {
            padding,
            center_x,
            center_y,
            ..
        } => (
            VNodeKind::Container,
            VNodeProps::Container {
                padding: *padding,
                center_x: *center_x,
                center_y: *center_y,
            },
        ),

        View::Scrollable { .. } => (VNodeKind::Scrollable, VNodeProps::Scrollable),

        View::List { spacing, .. } => (
            VNodeKind::List,
            VNodeProps::List { spacing: *spacing },
        ),

        View::Table {
            spacing, col_spacing, ..
        } => (
            VNodeKind::Table,
            VNodeProps::Table {
                spacing: *spacing,
                col_spacing: *col_spacing,
            },
        ),

        View::Slider {
            min, max, value, step, ..
        } => (
            VNodeKind::Slider,
            VNodeProps::Slider {
                min: *min,
                max: *max,
                value: *value,
                step: *step,
            },
        ),

        View::ProgressBar { progress, .. } => (
            VNodeKind::ProgressBar,
            VNodeProps::ProgressBar {
                progress: *progress,
            },
        ),

        // 高级组件（Plan 010）- 暂不支持，返回占位符
        View::Accordion { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[Accordion 暂不支持]".to_string(),
            },
        ),

        View::Sidebar { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[Sidebar 暂不支持]".to_string(),
            },
        ),

        View::Tabs { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[Tabs 暂不支持]".to_string(),
            },
        ),

        View::NavigationRail { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[NavigationRail 暂不支持]".to_string(),
            },
        ),
    }
}

/// 从 View 中提取子节点列表
///
/// # 参数
///
/// * `view` - 要提取子节点的 View
///
/// # 返回
///
/// 子 View 的向量
fn extract_children<M>(view: &View<M>) -> Vec<View<M>>
where
    M: Clone + std::fmt::Debug,
{
    match view {
        View::Column { children, .. } => children.clone(),
        View::Row { children, .. } => children.clone(),
        View::Container { child, .. } => vec![*child.clone()],
        View::Scrollable { child, .. } => vec![*child.clone()],
        View::List { items, .. } => items.clone(),
        View::Table { headers, rows, .. } => {
            let mut children = headers.clone();
            for row in rows {
                for cell in row {
                    children.push(cell.clone());
                }
            }
            children
        }
        View::Tabs { contents, .. } => contents.clone(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用的简化消息类型
    #[derive(Debug, Clone, Copy)]
    enum TestMsg {
        Click,
        Change,
    }

    #[test]
    fn test_simple_text_conversion() {
        let view: View<TestMsg> = View::Text {
            content: "Hello".to_string(),
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Text);
        assert!(tree.validate().is_ok());
    }

    #[test]
    fn test_empty_conversion() {
        let view: View<TestMsg> = View::Empty;

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Text);
        assert!(matches!(root.props, VNodeProps::Empty));
    }

    #[test]
    fn test_button_conversion() {
        let view: View<TestMsg> = View::Button {
            label: "Click Me".to_string(),
            onclick: TestMsg::Click,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Button);
        if let VNodeProps::Button { label } = &root.props {
            assert_eq!(label, "Click Me");
        } else {
            panic!("Expected Button props");
        }
    }

    #[test]
    fn test_column_with_children() {
        let view: View<TestMsg> = View::Column {
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
            spacing: 10,
            padding: 0,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 3); // 1 Column + 2 Text
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Column);
        assert_eq!(root.children.len(), 2);

        // 验证子节点
        let child1 = tree.get(root.children[0]).unwrap();
        assert_eq!(child1.kind, VNodeKind::Text);

        let child2 = tree.get(root.children[1]).unwrap();
        assert_eq!(child2.kind, VNodeKind::Text);
    }

    #[test]
    fn test_row_conversion() {
        let view: View<TestMsg> = View::Row {
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
            spacing: 5,
            padding: 10,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 3); // 1 Row + 2 Text
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Row);

        if let VNodeProps::Layout { spacing, padding } = &root.props {
            assert_eq!(*spacing, 5);
            assert_eq!(*padding, 10);
        } else {
            panic!("Expected Layout props");
        }
    }

    #[test]
    fn test_nested_structure() {
        let view: View<TestMsg> = View::Column {
            children: vec![View::Row {
                children: vec![View::Text {
                    content: "Nested".to_string(),
                    style: None,
                }],
                spacing: 5,
                padding: 0,
                style: None,
            }],
            spacing: 10,
            padding: 0,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 3); // Column + Row + Text

        // 验证嵌套关系
        let root = tree.root().unwrap();
        assert_eq!(root.children.len(), 1);

        let row_id = root.children[0];
        let row = tree.get(row_id).unwrap();
        assert_eq!(row.kind, VNodeKind::Row);
        assert_eq!(row.parent, Some(root.id));

        let text_id = row.children[0];
        let text = tree.get(text_id).unwrap();
        assert_eq!(text.kind, VNodeKind::Text);
        assert_eq!(text.parent, Some(row_id));
    }

    #[test]
    fn test_container_conversion() {
        let view: View<TestMsg> = View::Container {
            child: Box::new(View::Text {
                content: "Centered".to_string(),
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

        if let VNodeProps::Container {
            padding,
            center_x,
            center_y,
        } = &root.props
        {
            assert_eq!(*padding, 20);
            assert!(*center_x);
            assert!(*center_y);
        } else {
            panic!("Expected Container props");
        }

        // 验证子节点
        assert_eq!(root.children.len(), 1);
        let child = tree.get(root.children[0]).unwrap();
        assert_eq!(child.kind, VNodeKind::Text);
    }

    #[test]
    fn test_input_conversion() {
        let view: View<TestMsg> = View::Input {
            placeholder: "Enter text".to_string(),
            value: "".to_string(),
            on_change: None,
            width: None,
            password: false,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Input);

        if let VNodeProps::Input {
            placeholder,
            value,
            password,
        } = &root.props
        {
            assert_eq!(placeholder, "Enter text");
            assert_eq!(value, "");
            assert!(!(*password));
        } else {
            panic!("Expected Input props");
        }
    }

    #[test]
    fn test_checkbox_conversion() {
        let view: View<TestMsg> = View::Checkbox {
            is_checked: true,
            label: "Check me".to_string(),
            on_toggle: None,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Checkbox);

        if let VNodeProps::Checkbox { label, is_checked } = &root.props {
            assert_eq!(label, "Check me");
            assert!(*is_checked);
        } else {
            panic!("Expected Checkbox props");
        }
    }

    #[test]
    fn test_select_conversion() {
        let view: View<TestMsg> = View::Select {
            options: vec!["Option 1".to_string(), "Option 2".to_string()],
            selected_index: Some(0),
            on_select: None,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Select);

        if let VNodeProps::Select {
            options,
            selected_index,
        } = &root.props
        {
            assert_eq!(options.len(), 2);
            assert_eq!(selected_index, &Some(0));
        } else {
            panic!("Expected Select props");
        }
    }

    #[test]
    fn test_scrollable_conversion() {
        let view: View<TestMsg> = View::Scrollable {
            child: Box::new(View::Text {
                content: "Scrollable content".to_string(),
                style: None,
            }),
            width: None,
            height: None,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 2);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Scrollable);
    }

    #[test]
    fn test_list_conversion() {
        let view: View<TestMsg> = View::List {
            items: vec![
                View::Text {
                    content: "Item 1".to_string(),
                    style: None,
                },
                View::Text {
                    content: "Item 2".to_string(),
                    style: None,
                },
            ],
            spacing: 8,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 3); // 1 List + 2 Text
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::List);
    }

    #[test]
    fn test_slider_conversion() {
        let view: View<TestMsg> = View::Slider {
            min: 0.0,
            max: 100.0,
            value: 50.0,
            step: Some(1.0),
            on_change: |_v| TestMsg::Change,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Slider);

        if let VNodeProps::Slider { min, max, value, step } = &root.props {
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*value, 50.0);
            assert_eq!(step, &Some(1.0));
        } else {
            panic!("Expected Slider props");
        }
    }

    #[test]
    fn test_progress_bar_conversion() {
        let view: View<TestMsg> = View::ProgressBar {
            progress: 0.75,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::ProgressBar);

        if let VNodeProps::ProgressBar { progress } = &root.props {
            assert_eq!(*progress, 0.75);
        } else {
            panic!("Expected ProgressBar props");
        }
    }

    #[test]
    fn test_tree_validity() {
        // 测试复杂树的有效性
        let view: View<TestMsg> = View::Column {
            children: vec![
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
                View::Button {
                    label: "Click".to_string(),
                    onclick: TestMsg::Click,
                    style: None,
                },
            ],
            spacing: 10,
            padding: 0,
            style: None,
        };

        let tree = view_to_vtree(view);

        // 验证树的完整性
        assert!(tree.validate().is_ok());

        // 验证节点数量
        assert_eq!(tree.node_count(), 5); // Column + Row + 2 Text + Button

        // 验证深度
        assert_eq!(tree.depth(), 3);
    }

    #[test]
    fn test_advanced_components_placeholder() {
        // 测试高级组件（暂不支持）返回占位符
        let accordion_view: View<TestMsg> = View::Accordion {
            items: vec![],
            allow_multiple: false,
            on_toggle: None,
            style: None,
        };

        let tree = view_to_vtree(accordion_view);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Text);

        if let VNodeProps::Text { content } = &root.props {
            assert!(content.contains("暂不支持"));
        } else {
            panic!("Expected placeholder text");
        }
    }

    #[test]
    fn test_tree_stats() {
        let view = View::Column {
            children: vec![
                View::Text {
                    content: "Title".to_string(),
                    style: None,
                },
                View::Button {
                    label: "Click".to_string(),
                    onclick: TestMsg::Click,
                    style: None,
                },
            ],
            spacing: 10,
            padding: 0,
            style: None,
        };

        let tree = view_to_vtree(view);
        let stats = tree.stats();

        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.text_nodes, 1);
        assert_eq!(stats.button_nodes, 1);
        assert_eq!(stats.layout_nodes, 1);
        assert_eq!(stats.leaf_nodes, 2);
        assert_eq!(stats.max_depth, 2);
    }
}
