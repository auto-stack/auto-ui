# Unified Input Demo

This example demonstrates text input fields for data entry with various configurations. **The same code works with BOTH Iced and GPUI backends!**

## Features

- Multiple input fields (username, email, password, bio)
- Real-time form summary display
- Password masking
- Clear form functionality
- **Unified API** - Same code, different backends!

## Running

### With Iced backend (default)
```bash
cargo run --package unified-input
# Or explicitly:
cargo run --package unified-input --features iced
```

### With GPUI backend
```bash
cargo run --package unified-input --features gpui
```

## Code Structure

The example demonstrates:

1. **Component trait** - Single implementation for both backends
2. **View abstraction** - Declarative UI using `View<Message>`
3. **Message handling** - Enum-based messages for state updates
4. **Auto-conversion** - Automatic conversion to backend-specific code

## Key Differences from Backend-Specific Versions

### Before (GPUI-specific)
- Required manual `InputRenderer` struct with `Render` trait
- Manual GPUI application initialization
- Backend-specific code in the example

### Before (Iced-specific)
- Direct call to `iced::run()`
- Manual view function
- Backend-specific code in the example

### After (Unified) ✅
- Only `Component` trait implementation
- Simple `run_app()` call
- **No backend-specific code!**

## Benefits

- ✅ **Less code** - No manual Render implementation
- ✅ **Simpler** - No backend-specific initialization
- ✅ **Unified** - Same code works on both backends
- ✅ **Type-safe** - Compile-time message checking
