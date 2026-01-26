// 最小化动态解释器测试
//
// 这个示例验证解释器核心功能：
// 1. 加载 .at 文件
// 2. 使用 InterpreterBridge 解释
// 3. 获取 Node 结果
// 4. 转换为 View

use auto_ui::interpreter::InterpreterBridge;
use auto_ui::node_converter;
use std::path::Path;

fn main() {
    println!("🚀 动态解释器测试");
    println!("{}", "=".repeat(50));

    // 创建解释器桥梁
    let mut bridge = InterpreterBridge::new();

    // 测试 1: 加载并解释简单的代码
    println!("\n📝 测试 1: 简单代码解释");
    let simple_code = r#"
        center {
            label("Hello, World!") {}
        }
    "#;

    println!("代码: {}", simple_code.trim());
    match bridge.interpret(simple_code) {
        Ok(_) => println!("✅ 解释成功"),
        Err(e) => println!("❌ 解释失败: {}", e),
    }

    // 获取视图节点
    println!("\n📊 测试 2: 获取视图节点");
    match bridge.get_main_view() {
        Ok(node) => {
            println!("✅ 获取 Node 成功");
            println!("   Node 类型: {}", node.name);
            println!("   Args: {} 个", node.args.args.len());
            println!("   Kids: {} 个", node.kids_len());

            // 测试 3: 转换 Node 到 View
            println!("\n🔄 测试 3: 转换 Node 到 View<String>");
            match node_converter::convert_node(&node) {
                Ok(view) => {
                    println!("✅ 转换成功");
                    println!("   View 类型: {:?}", std::mem::discriminant(&view));
                    print_view_tree(&view, 1);
                }
                Err(e) => println!("❌ 转换失败: {}", e),
            }
        }
        Err(e) => println!("❌ 获取 Node 失败: {}", e),
    }

    // 测试 4: 加载文件
    println!("\n📄 测试 4: 加载 counter.at 文件");
    let mut bridge2 = InterpreterBridge::new();
    match bridge2.load_file(Path::new("counter.at")) {
        Ok(_) => println!("✅ 文件加载成功"),
        Err(e) => println!("❌ 文件加载失败: {}", e),
    }

    println!("\n{}", "=".repeat(50));
    println!("✨ 测试完成");
}

fn print_view_tree(view: &auto_ui::view::View<String>, indent: usize) {
    let prefix = "  ".repeat(indent);
    match view {
        auto_ui::view::View::Text { content, .. } => {
            println!("{}Text: {}", prefix, content);
        }
        auto_ui::view::View::Button { label, .. } => {
            println!("{}Button: {}", prefix, label);
        }
        auto_ui::view::View::Column { children, .. } => {
            println!("{}Column ({} children)", prefix, children.len());
            for child in children {
                print_view_tree(child, indent + 1);
            }
        }
        auto_ui::view::View::Row { children, .. } => {
            println!("{}Row ({} children)", prefix, children.len());
            for child in children {
                print_view_tree(child, indent + 1);
            }
        }
        auto_ui::view::View::Container { child, .. } => {
            println!("{}Container", prefix);
            print_view_tree(child, indent + 1);
        }
        auto_ui::view::View::Empty => {
            println!("{}Empty", prefix);
        }
        _ => {
            println!("{}Other: {:?}", prefix, std::mem::discriminant(view));
        }
    }
}
