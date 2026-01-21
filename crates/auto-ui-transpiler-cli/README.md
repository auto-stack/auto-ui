# AutoUI Transpiler CLI

Command-line tool to transpile Auto language (.at) files to AutoUI Rust code.

## Installation

The CLI tool is built as part of the AutoUI workspace:

```bash
cargo build --package auto-ui-transpiler
```

This creates the binary at `target/debug/auto-ui-transpile` (or `.exe` on Windows).

## Usage

### Basic Commands

```bash
# Show help
auto-ui-transpile

# Transpile a file and output to stdout
auto-ui-transpile transpile input.at --stdout

# Transpile a file to a specific output file
auto-ui-transpile transpile input.at output.rs

# Show information about a .at file
auto-ui-transpile info input.at
```

### Examples

```bash
# View generated code from hello.at
cargo run --package auto-ui-transpiler -- transpile scratch/hello.at --stdout

# Generate a Rust file
cargo run --package auto-ui-transpiler -- transpile scratch/counter.at counter_generated.rs

# Analyze a file without transpiling
cargo run --package auto-ui-transpiler -- info scratch/hello.at
```

## Current Status

**⚠️ Development Status**: The CLI tool is fully functional, but the underlying transpiler is still in development.

### What Works

- ✅ CLI argument parsing and command handling
- ✅ File reading and validation
- ✅ Info command (shows file statistics)
- ✅ Output to stdout or file

### Known Limitations

The transpiler currently has limited support for Auto language syntax:

- ❌ The `widget` keyword is not supported - use `type ... is Widget`
- ❌ The `app` keyword is not supported
- ❌ Custom `use` statements (like `use auto.ui: ...`) may fail
- ❌ Some Auto language features may not parse correctly

The transpiler is actively being developed. Test cases in `crates/auto-ui/tests/transpiler_test.rs` demonstrate the expected syntax.

### Expected Syntax

For best results, use this syntax pattern:

```auto
type MyWidget is Widget {
    field_name str = "default value"
    count int = 0

    fn view() {
        col {
            text(field_name) {}
            text(count) {}
        }
    }
}
```

## Development

### Project Structure

```
crates/auto-ui-transpiler-cli/
├── Cargo.toml          # Package configuration
├── README.md           # This file
└── src/
    └── main.rs         # CLI implementation
```

### Dependencies

- `auto-ui` with `transpiler` feature
- `clap` for CLI argument parsing
- `anyhow` for error handling

### Testing

Run the transpiler tests:

```bash
cargo test --package auto-ui --features transpiler
```

## Next Steps

To improve the transpiler:

1. **Extend auto-lang parser**: Support `widget` and `app` keywords as aliases
2. **Improve error messages**: Show more helpful parsing errors
3. **Add watch mode**: Auto-transpile on file changes
4. **Integration tests**: More comprehensive test coverage

## Related Documentation

- [AutoUI README](../../README.md)
- [Transpiler API](../auto-ui/src/trans/api.rs)
- [Code Generator](../auto-ui/src/trans/rust_gen.rs)
- [Transpiler Tests](../auto-ui/tests/transpiler_test.rs)
