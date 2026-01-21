# Migration Guide: Legacy API to Unified Styling System

This guide helps you migrate from the legacy hardcoded styling API to the new unified Tailwind CSS-inspired styling system.

## Overview

**Legacy API** (still supported):
```rust
View::col()
    .spacing(10)
    .padding(20)
    .child(View::text("Hello"))
    .build()
```

**New Unified Styling API** (recommended):
```rust
View::col()
    .style("gap-2.5 p-5 bg-white flex items-center")
    .child(View::text_styled("Hello", "text-lg font-bold"))
    .build()
```

## Key Differences

| Legacy API | Unified Styling API |
|-----------|---------------------|
| `.spacing(10)` | `.style("gap-2.5")` |
| `.padding(20)` | `.style("p-5")` |
| `.width(100)` | `.style("w-25")` |
| `.center_x()` + `.center_y()` | `.style("flex items-center justify-center")` |
| No text styling | `View::text_styled(content, style)` |
| Limited colors | Full color palette |
| No borders | `.style("border rounded")` |

## Migration Scenarios

### Scenario 1: Layout Containers

#### Before (Legacy API)
```rust
View::col()
    .spacing(15)
    .padding(20)
    .child(View::text("Hello"))
    .child(View::button("Click", Msg::Click))
    .build()
```

#### After (Unified Styling)
```rust
View::col()
    .style("gap-3.75 p-5 flex")
    .child(View::text("Hello"))
    .child(View::button("Click", Msg::Click))
    .build()
```

**Benefits**:
- ✅ More flexible (can add colors, borders, etc.)
- ✅ Consistent with Tailwind CSS
- ✅ Single `.style()` call instead of multiple chained methods

### Scenario 2: Container with Centering

#### Before (Legacy API)
```rust
View::container(child)
    .padding(20)
    .width(Some(400))
    .center_x()
    .center_y()
    .build()
```

#### After (Unified Styling)
```rust
View::container(child)
    .style("p-5 w-100 flex items-center justify-center")
    .build()
```

**Benefits**:
- ✅ More concise (5 lines → 3 lines)
- ✅ Familiar flexbox syntax
- ✅ Easier to understand layout intent

### Scenario 3: Button Styling

#### Before (Legacy API)
```rust
// No styling support - buttons have default appearance
View::button("Click me", Msg::Click)
```

#### After (Unified Styling)
```rust
View::button_styled(
    "Click me",
    Msg::Click,
    "px-4 py-2 bg-blue-500 text-white rounded-lg font-bold hover:bg-blue-600"
)
```

**Benefits**:
- ✅ Full control over button appearance
- ✅ Support for colors, padding, borders, shadows
- ✅ Hover states (in full implementation)

### Scenario 4: Text Styling

#### Before (Legacy API)
```rust
// No text styling support
View::text("Hello World")
```

#### After (Unified Styling)
```rust
View::text_styled(
    "Hello World",
    "text-2xl font-bold text-center text-blue-600"
)
```

**Benefits**:
- ✅ Typography control (size, weight, alignment)
- ✅ Text colors
- ✅ Consistent with web development practices

### Scenario 5: Complex Layout

#### Before (Legacy API)
```rust
View::row()
    .spacing(10)
    .padding(15)
    .child(View::col()
        .spacing(8)
        .padding(10)
        .child(View::text("Title"))
        .child(View::text("Description"))
        .build()
    )
    .child(View::button("Action", Msg::Action))
    .build()
```

#### After (Unified Styling)
```rust
View::row()
    .style("gap-2.5 p-3.75 bg-white rounded-lg shadow")
    .child(View::col()
        .style("gap-2 p-2.5")
        .child(View::text_styled("Title", "text-lg font-bold text-gray-800"))
        .child(View::text_styled("Description", "text-sm text-gray-600"))
        .build()
    )
    .child(View::button_styled(
        "Action",
        Msg::Action,
        "px-3 py-1.5 bg-blue-500 text-white rounded"
    ))
    .build()
```

**Benefits**:
- ✅ Complete styling control
- ✅ Visual hierarchy through styling
- ✅ Professional appearance with shadows and rounded corners

## Step-by-Step Migration Process

### Step 1: Identify Components to Migrate

Start with components that would benefit most from enhanced styling:
- Buttons (add colors, padding, borders)
- Text (add typography, colors)
- Layouts (add backgrounds, shadows, spacing)

### Step 2: Create Style Strings

Convert legacy numeric values to Tailwind CSS classes:

| Legacy Value | Tailwind Class |
|--------------|---------------|
| `spacing(8)` | `gap-2` |
| `spacing(10)` | `gap-2.5` |
| `spacing(16)` | `gap-4` |
| `padding(20)` | `p-5` |
| `width(400)` | `w-100` |
| `center_x()` | `items-center` (with `flex`) |
| `center_y()` | `justify-center` (with `flex`) |

**Conversion formula**: `legacy_value ÷ 4 = tailwind_units`
- Example: `spacing(20)` → `gap-5` (20 ÷ 4 = 5)

### Step 3: Replace API Calls

**Pattern 1: Layout spacing/padding**
```rust
// Before
.spacing(10)
.padding(20)

// After
.style("gap-2.5 p-5")
```

**Pattern 2: Container sizing/centering**
```rust
// Before
.width(Some(400))
.center_x()
.center_y()

// After
.style("w-100 flex items-center justify-center")
```

**Pattern 3: Text elements**
```rust
// Before
View::text("Hello")

// After
View::text_styled("Hello", "text-lg font-bold")
```

**Pattern 4: Button elements**
```rust
// Before
View::button("Click", Msg::Click)

// After
View::button_styled("Click", Msg::Click, "px-4 py-2 bg-blue-500 text-white rounded")
```

### Step 4: Test Incrementally

1. **Migrate one component at a time**
2. **Run examples to verify visual appearance**
3. **Check both GPUI and Iced backends** (if applicable)

```bash
# Test your migrated component
cargo run --package auto-ui --example your_component

# Test with GPUI backend
cargo run --package auto-ui-gpui --example your_component
```

### Step 5: Enhance with Additional Styling

Once basic migration is complete, add enhanced styling:

```rust
// Basic migration
View::col()
    .style("gap-4 p-6")
    .child(View::text("Title"))
    .build()

// Enhanced with colors and effects
View::col()
    .style("gap-4 p-6 bg-white rounded-lg shadow-lg")
    .child(View::text_styled("Title", "text-2xl font-bold text-gray-800"))
    .build()
```

## Common Migration Patterns

### Card Component

```rust
// Before
View::col()
    .spacing(10)
    .padding(15)
    .child(View::text("Card Title"))
    .child(View::text("Card content"))
    .build()

// After
View::col()
    .style("gap-2.5 p-3.75 bg-white rounded-lg shadow-md border border-gray-200")
    .child(View::text_styled("Card Title", "text-lg font-bold text-gray-800"))
    .child(View::text_styled("Card content", "text-sm text-gray-600"))
    .build()
```

### Sidebar Layout

```rust
// Before
View::row()
    .spacing(0)
    .child(
        View::col()
            .spacing(10)
            .padding(20)
            .child(View::text("Sidebar"))
            .build()
    )
    .child(
        View::container(main_content)
            .padding(20)
            .build()
    )
    .build()

// After
View::row()
    .style("flex")
    .child(
        View::col()
            .style("gap-2.5 p-5 w-64 bg-gray-800 text-white flex")
            .child(View::text_styled("Sidebar", "text-xl font-bold"))
            .build()
    )
    .child(
        View::container(main_content)
            .style("flex-1 p-5 bg-gray-50")
            .build()
    )
    .build()
```

### Form with Styled Inputs

```rust
// Before
View::col()
    .spacing(10)
    .child(View::text("Email"))
    .child(View::input("Enter email").value(email).build())
    .child(View::button("Submit", Msg::Submit))
    .build()

// After
View::col()
    .style("gap-2.5 p-6 bg-white rounded-lg shadow-md")
    .child(View::text_styled("Email", "text-sm font-medium text-gray-700"))
    .child(
        View::input("Enter email")
            .value(email)
            .style("px-3 py-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500")
            .build()
    )
    .child(
        View::button_styled(
            "Submit",
            Msg::Submit,
            "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 font-medium"
        )
    )
    .build()
```

## Style Priority Rules

When both legacy fields and Style objects are present, Style takes priority:

```rust
View::col()
    .spacing(10)        // ❌ Ignored (style has gap-4)
    .padding(20)        // ❌ Ignored (style has p-6)
    .style("gap-4 p-6") // ✅ Applied
    .child(...)
    .build()
```

**Best Practice**: Use either legacy API OR unified styling, not both.

## Backward Compatibility

The legacy API is fully supported. You can:

1. **Keep existing code unchanged** - no breaking changes
2. **Migrate gradually** - migrate components one at a time
3. **Mix APIs** - use legacy for simple components, unified for complex ones

```rust
// This is valid - mixing old and new APIs
View::col()
    .spacing(10)  // Legacy API
    .child(
        View::button_styled(  // New API
            "Styled Button",
            Msg::Click,
            "px-4 py-2 bg-blue-500 text-white rounded"
        )
    )
    .child(
        View::text("Plain text")  // Legacy API
    )
    .build()
```

## Quick Reference Card

### Spacing Conversion

| Legacy | Unified |
|--------|---------|
| `spacing(4)` | `gap-1` |
| `spacing(8)` | `gap-2` |
| `spacing(12)` | `gap-3` |
| `spacing(16)` | `gap-4` |
| `spacing(20)` | `gap-5` |
| `padding(4)` | `p-1` |
| `padding(8)` | `p-2` |
| `padding(16)` | `p-4` |

### Sizing Conversion

| Legacy | Unified |
|--------|---------|
| `width(Some(100))` | `w-25` |
| `width(Some(400))` | `w-100` |
| `height(Some(200))` | `h-50` |

### Layout Conversion

| Legacy | Unified |
|--------|---------|
| `center_x()` | `items-center` (requires `flex`) |
| `center_y()` | `justify-center` (requires `flex`) |
| No equivalent | `flex flex-row` / `flex flex-col` |

## Troubleshooting

### Issue: Styles Not Applying

**Problem**: Styles don't appear to be working

**Solutions**:
1. Check that backend adapter supports the style classes
2. Verify style string is valid (no typos)
3. Ensure `.build()` is called after `.style()`
4. Check for style priority conflicts

```rust
// ❌ Wrong - build() missing
View::col().style("gap-4").child(...)

// ✅ Correct - build() called
View::col().style("gap-4").child(...).build()
```

### Issue: Invalid Style String

**Problem**: Compiler error about invalid style

**Solutions**:
1. Check for typos in style class names
2. Verify style class is supported (see [style-system-usage.md](./style-system-usage.md))
3. Use `Style::parse()` for testing

```rust
// Test style parsing
let style = Style::parse("p-4 bg-white flex").unwrap();
dbg!(style);
```

### Issue: Different Appearance on Different Backends

**Problem**: Component looks different on GPUI vs Iced

**Solutions**:
1. Iced doesn't support: `margin`, `absolute`, `z-index`, `grid`
2. Use fallback layouts for Iced
3. Check [Backend Compatibility](#backend-compatibility) section

## Resources

- [Style System Usage Guide](./style-system-usage.md) - Complete style class reference
- [Plan 004: Unified Styling System](../plans/004-unified-styling-system.md) - System design
- [Plan 005: Style System Integration](../plans/005-style-system-integration.md) - Integration details
- [Examples](../../crates/auto-ui/examples/) - See `styling_showcase.rs` for complete examples

## Support

If you encounter issues during migration:

1. Check example code in `crates/auto-ui/examples/`
2. Review Plan 004 for style class reference
3. Run tests: `cargo test --package auto-ui`
4. Check backend compatibility matrix
