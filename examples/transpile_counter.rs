// Example: Transpile and run counter.at
//
// This example demonstrates how to use the AutoUI transpiler to convert
// Auto language .at files into Rust code, then run them.

use auto_ui::trans::transpile_file;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AutoUI Transpiler Example ===\n");

    // Path to the Auto language file
    let at_file = "../../scratch/counter.at";
    let rs_file = "counter_generated.rs";

    println!("📝 Reading Auto file: {}", at_file);

    // Transpile the .at file to Rust code
    match transpile_file(at_file, Some(rs_file)) {
        Ok(generated_code) => {
            println!("✅ Transpilation successful!\n");
            println!("📄 Generated Rust code saved to: {}", rs_file);
            println!("\n--- Generated Code ---\n");
            println!("{}", generated_code);
            println!("\n--- End of Generated Code ---\n");

            // Note: The generated code would need to be integrated into a proper
            // application to actually run. The transpiler creates Component
            // implementations that can be used with any backend (GPUI, Iced, etc.)

            println!("\n💡 To use the generated component:");
            println!("   1. Copy the generated code into your project");
            println!("   2. Integrate it with your chosen backend (GPUI, Iced, etc.)");
            println!("   3. Create an instance and call component.view()");

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Transpilation failed: {}", e);
            eprintln!("\n💡 This is expected during development!");
            eprintln!("   The transpiler is implemented but may need adjustments");
            eprintln!("   to match the exact Auto language syntax in your .at files.");

            Err(e.into())
        }
    }
}
