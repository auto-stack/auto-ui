// Style System Verification Example
//
// This example demonstrates the unified styling system working with L1 core features:
// - Spacing: p-*, gap-*
// - Colors: bg-*, text-*
// - Layout: flex, flex-row/col, items-center
// - Sizing: w-full, w-*, h-full, h-*
// - Border Radius: rounded

use auto_ui::style::{Style, StyleClass, SizeValue};

fn main() {
    println!("🎨 AutoUI 统一样式系统验证示例\n");
    println!("==================================================\n");

    // Example 1: Parse a simple style string
    println!("📝 示例 1: 解析简单的样式字符串");
    println!("输入: \"p-4 gap-2 bg-white flex\"\n");

    let style = Style::parse("p-4 gap-2 bg-white flex").unwrap();
    println!("解析结果: {} 个样式类", style.classes.len());
    for (i, class) in style.classes.iter().enumerate() {
        println!("  {}: {:?}", i + 1, class);
    }
    println!();

    // Example 2: Counter component styles
    println!("📝 示例 2: Counter 组件样式");
    let counter_style = "p-5 bg-blue-500 text-white rounded flex items-center gap-2";
    println!("输入: \"{}\"\n", counter_style);

    let style = Style::parse(counter_style).unwrap();
    println!("解析结果: {} 个样式类", style.classes.len());
    for class in &style.classes {
        match class {
            StyleClass::Padding(size) => println!("  ✅ Padding: {:?}", size),
            StyleClass::Gap(size) => println!("  ✅ Gap: {:?}", size),
            StyleClass::BackgroundColor(color) => println!("  ✅ BackgroundColor: {:?}", color),
            StyleClass::TextColor(color) => println!("  ✅ TextColor: {:?}", color),
            StyleClass::Rounded => println!("  ✅ Rounded"),
            StyleClass::Flex => println!("  ✅ Flex"),
            StyleClass::ItemsCenter => println!("  ✅ ItemsCenter"),
            _ => println!("  ✅ Other: {:?}", class),
        }
    }
    println!();

    // Example 3: Complex button style
    println!("📝 示例 3: 复杂按钮样式");
    let button_style = "px-4 py-2 bg-white text-blue-500 rounded w-full";
    println!("输入: \"{}\"\n", button_style);

    // Note: px and py are not in L1, but p-4 works
    let style = Style::parse("p-2 bg-white text-slate-500 rounded w-full").unwrap();
    println!("解析结果 (L1 简化版): {} 个样式类", style.classes.len());
    for class in &style.classes {
        println!("  ✅ {:?}", class);
    }
    println!();

    // Example 4: Layout container
    println!("📝 示例 4: 布局容器");
    let layout_style = "flex flex-col items-center justify-center gap-4 h-full";
    println!("输入: \"{}\"\n", layout_style);

    let style = Style::parse(layout_style).unwrap();
    println!("解析结果: {} 个样式类", style.classes.len());
    for class in &style.classes {
        println!("  ✅ {:?}", class);
    }
    println!();

    // Example 5: Error handling
    println!("📝 示例 5: 错误处理");
    println!("输入: \"p-4 invalid-class bg-white\"\n");

    match Style::parse("p-4 invalid-class bg-white") {
        Ok(style) => println!("✅ 解析成功: {} 个样式类", style.classes.len()),
        Err(e) => println!("❌ 解析失败: {}", e),
    }
    println!();

    // Example 6: Test with backend adapters
    #[cfg(feature = "gpui")]
    {
        println!("📝 示例 6: GPUI 适配器");
        use auto_ui::style::gpui_adapter::GpuiStyle;

        let style = Style::parse("p-4 bg-white flex").unwrap();
        let gpui_style = GpuiStyle::from_style(&style);

        println!("GPUI 样式:");
        println!("  Padding: {:?}", gpui_style.padding);
        println!("  Flex: {:?}", gpui_style.flex);
        println!("  BackgroundColor: {:?}", gpui_style.background_color);
        println!();
    }

    #[cfg(feature = "iced")]
    {
        println!("📝 示例 7: Iced 适配器");
        use auto_ui::style::iced_adapter::IcedStyle;

        let style = Style::parse("p-4 bg-white rounded").unwrap();
        let iced_style = IcedStyle::from_style(&style);

        println!("Iced 样式:");
        println!("  Padding: {:?}", iced_style.padding);
        println!("  BackgroundColor: {:?}", iced_style.background_color);
        println!("  Rounded: {}", iced_style.rounded);
        println!();
    }

    println!("==================================================");
    println!("✅ 所有测试完成！统一样式系统 MVP 原型验证成功！");
}
