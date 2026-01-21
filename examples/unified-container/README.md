# Unified Container Demo

This example demonstrates container styling and layout options. **The same code works with BOTH Iced and GPUI backends!**

## Features

- **Padding** - Add internal spacing to containers
- **Sizing** - Control width and height of containers
- **Centering** - Center content horizontally and/or vertically
- **Nested Containers** - Combine multiple containers for complex layouts
- **Unified API** - Same code, different backends!

## Running

### With Iced backend (default)
```bash
cargo run --package unified-container
# Or explicitly:
cargo run --package unified-container --features iced
```

### With GPUI backend
```bash
cargo run --package unified-container --features gpui
```

## Code Structure

The example demonstrates:

1. **Container widget** - Padding, sizing, and centering options
2. **Navigation** - Switch between different container examples
3. **View methods** - Separate methods for each example type
4. **Auto-conversion** - Automatic conversion to backend-specific code

## Key Differences from Backend-Specific Versions

### Before (GPUI-specific) - 344 lines
- Required manual `ContainerRenderer` struct with `Render` trait
- Manual GPUI application initialization
- 170+ lines of backend-specific rendering code

### Before (Iced-specific) - 187 lines
- Direct call to `iced::run()`
- Manual view function
- Backend-specific code in the example

### After (Unified) - 209 lines ✅
- Only `Component` trait implementation
- Simple `run_app()` call
- **No backend-specific code!**
- **39% less code** than GPUI version
- **Same functionality** as both versions combined

## Benefits

- ✅ **Less code** - No manual Render implementation
- ✅ **Simpler** - No backend-specific initialization
- ✅ **Unified** - Same code works on both backends
- ✅ **Reusable** - Container logic is backend-agnostic
