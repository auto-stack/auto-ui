# Unified Styling System Usage Guide

Complete guide to using the unified Tailwind CSS-inspired styling system in AutoUI.

## Table of Contents

- [Quick Start](#quick-start)
- [Basic Concepts](#basic-concepts)
- [Style Class Reference](#style-class-reference)
- [API Patterns](#api-patterns)
- [Common Scenarios](#common-scenarios)
- [Best Practices](#best-practices)
- [Backend Compatibility](#backend-compatibility)
- [Troubleshooting](#troubleshooting)

---

## Quick Start

### 1. Basic Usage

```rust
use auto_ui::{Component, View};

fn view(&self) -> View<Self::Msg> {
    View::col()
        .style("gap-4 p-6 bg-white rounded-lg")
        .child(View::text_styled("Hello", "text-2xl font-bold"))
        .child(View::button("Click", Msg::Click))
        .build()
}
```

### 2. Style String Format

Style strings use space-separated Tailwind CSS classes:

```rust
"gap-4 p-6 bg-white rounded-lg shadow-md flex items-center"
```

Each class represents a style property:
- `gap-4` - gap between children
- `p-6` - padding
- `bg-white` - background color
- `rounded-lg` - border radius
- `shadow-md` - shadow
- `flex` - flexbox layout
- `items-center` - center items cross-axis

---

## Basic Concepts

### Style Parsing

Style strings are parsed at compile time into type-safe `StyleClass` enums:

```rust
use auto_ui::style::Style;

let style = Style::parse("p-4 bg-white flex").unwrap();
// Returns Style { classes: [Padding(Fixed(4)), BackgroundColor(White), Flex] }
```

### Zero-Cost Abstraction

Style parsing happens at compile time - zero runtime overhead:

```rust
// This is compiled to:
// Style { classes: [Padding(Fixed(4)), BackgroundColor(White)] }
let style = Style::parse("p-4 bg-white").unwrap();
```

### Style Application

Styles are applied by backend adapters (GPUI, Iced):

```rust
// 1. Parse style string
let style = Style::parse("p-4 bg-white").unwrap();

// 2. Convert to backend-specific format
let gpui_style = GpuiStyle::from_style(&style);

// 3. Apply to component
div().p(gpui_style.padding).bg(gpui_style.background_color)
```

---

## Style Class Reference

### L1: Core Features (30% Coverage)

#### Spacing

**Padding** (`p-*`, `px-*`, `py-*`)

```rust
"p-4"   // padding: 16px (all sides)
"p-6"   // padding: 24px
"px-4"  // padding-left: 16px, padding-right: 16px
"py-2"  // padding-top: 8px, padding-bottom: 8px
```

**Gap** (`gap-*`)

```rust
"gap-2"  // gap: 8px
"gap-4"  // gap: 16px
"gap-6"  // gap: 24px
```

**Sizing Formula**: `value * 4px`
- `p-1` = 4px
- `p-2` = 8px
- `p-4` = 16px
- `p-6` = 24px
- `p-8` = 32px

#### Colors

**Background Colors** (`bg-{color}-{shade}`)

```rust
"bg-white"           // #ffffff
"bg-gray-100"        // #f3f4f6
"bg-blue-500"        // #3b82f6
"bg-red-500"         // #ef4444
"bg-green-500"       // #22c55e
```

**Text Colors** (`text-{color}-{shade}`)

```rust
"text-white"         // #ffffff
"text-gray-900"      // #111827
"text-blue-600"      // #2563eb
"text-red-500"       // #ef4444
```

**Color Palette**:
- Gray: 50-900 (light to dark)
- Blue, Red, Green, Yellow, etc.: 100-900

#### Layout

**Flexbox**

```rust
"flex"         // display: flex
"flex-row"     // flex-direction: row
"flex-col"     // flex-direction: column
"flex-1"       // flex: 1 1 0%
```

**Alignment**

```rust
"items-center"  // align-items: center
"items-start"   // align-items: flex-start
"items-end"     // align-items: flex-end

"justify-center"   // justify-content: center
"justify-between"  // justify-content: space-between
"justify-start"    // justify-content: flex-start
"justify-end"      // justify-content: flex-end
```

#### Sizing

```rust
"w-full"   // width: 100%
"w-64"     // width: 256px (64 * 4)
"h-full"   // height: 100%
"h-32"     // height: 128px (32 * 4)
```

#### Border Radius

```rust
"rounded"       // border-radius: 4px
"rounded-sm"    // border-radius: 2px
"rounded-md"    // border-radius: 4px
"rounded-lg"    // border-radius: 8px
"rounded-xl"    // border-radius: 12px
"rounded-2xl"   // border-radius: 16px
"rounded-full"  // border-radius: 9999px
```

---

### L2: Important Features (40% Coverage)

#### Directional Padding

```rust
"px-4"  // padding-left: 16px, padding-right: 16px
"py-2"  // padding-top: 8px, padding-bottom: 8px
```

#### Margin (GPUI only)

```rust
"m-4"    // margin: 16px (all sides)
"mx-2"   // margin-left: 8px, margin-right: 8px
"my-2"   // margin-top: 8px, margin-bottom: 8px
```

**⚠️ Note**: Iced backend doesn't support margin

#### Typography

**Font Size**

```rust
"text-xs"    // font-size: 12px
"text-sm"    // font-size: 14px
"text-base"  // font-size: 16px
"text-lg"    // font-size: 18px
"text-xl"    // font-size: 20px
"text-2xl"   // font-size: 24px
"text-3xl"   // font-size: 30px
```

**Font Weight**

```rust
"font-normal"  // font-weight: 400
"font-medium"  // font-weight: 500
"font-bold"    // font-weight: 700
```

**Text Alignment**

```rust
"text-left"    // text-align: left
"text-center"  // text-align: center
"text-right"   // text-align: right
```

#### Border

```rust
"border"              // border: 1px solid
"border-0"            // border: 0
"border-gray-200"     // border-color: #e5e7eb
"border border-red-500"  // border with color
```

---

### L3: Advanced Features (20% Coverage)

#### Shadow

```rust
"shadow"      // box-shadow: default
"shadow-sm"   // box-shadow: small
"shadow-md"   // box-shadow: medium
"shadow-lg"   // box-shadow: large
"shadow-xl"   // box-shadow: extra large
"shadow-2xl"  // box-shadow: 2x large
"shadow-none" // box-shadow: none
```

#### Opacity

```rust
"opacity-0"    // opacity: 0
"opacity-25"   // opacity: 0.25
"opacity-50"   // opacity: 0.50
"opacity-75"   // opacity: 0.75
"opacity-100"  // opacity: 1.00
```

#### Position

```rust
"relative"   // position: relative
"absolute"   // position: absolute
"z-0"        // z-index: 0
"z-10"       // z-index: 10
"z-50"       // z-index: 50
```

**⚠️ Note**: Iced backend doesn't support `absolute` or `z-index`

#### Overflow

```rust
"overflow-auto"     // overflow: auto
"overflow-hidden"   // overflow: hidden
"overflow-scroll"   // overflow: scroll
"overflow-visible"  // overflow: visible
"overflow-x-auto"   // overflow-x: auto
"overflow-y-auto"   // overflow-y: auto
```

#### Grid Layout

```rust
"grid"          // display: grid
"grid-cols-2"   // grid-template-columns: repeat(2, 1fr)
"grid-cols-3"   // grid-template-columns: repeat(3, 1fr)
"grid-rows-2"   // grid-template-rows: repeat(2, 1fr)
"col-span-2"    // grid-column: span 2
"row-span-2"    // grid-row: span 2
"col-start-2"   // grid-column-start: 2
"row-start-1"   // grid-row-start: 1
```

**⚠️ Note**: Iced backend doesn't support grid layout

---

## API Patterns

### Pattern 1: Styled Containers

```rust
View::container(child)
    .style("p-6 bg-white rounded-lg shadow-md")
    .build()
```

### Pattern 2: Styled Layouts

```rust
View::row()
    .style("gap-4 p-4 bg-blue-50 flex items-center")
    .child(item1)
    .child(item2)
    .build()

View::col()
    .style("gap-4 p-6 flex flex-col")
    .child(item1)
    .child(item2)
    .build()
```

### Pattern 3: Styled Text

```rust
View::text_styled(
    "Hello World",
    "text-2xl font-bold text-center text-blue-600"
)
```

### Pattern 4: Styled Buttons

```rust
View::button_styled(
    "Click Me",
    Msg::Click,
    "px-6 py-3 bg-blue-500 text-white rounded-lg font-bold hover:bg-blue-600"
)
```

### Pattern 5: Styled Inputs

```rust
View::input("Enter email")
    .value(self.email.clone())
    .on_change(Msg::EmailChanged)
    .style("px-4 py-2 border border-gray-300 rounded")
    .build()
```

### Pattern 6: Complex Nested Styles

```rust
View::col()
    .style("gap-6 p-8 bg-gray-50 min-h-screen")
    .child(
        View::container(
            View::col()
                .style("gap-4")
                .child(View::text_styled("Card Title", "text-xl font-bold"))
                .child(View::text("Card content"))
                .build()
        )
        .style("p-6 bg-white rounded-lg shadow-md")
        .build()
    )
    .build()
```

---

## Common Scenarios

### Scenario 1: Card Component

```rust
View::col()
    .style("gap-3 p-6 bg-white rounded-lg shadow-md border border-gray-200")
    .child(View::text_styled("Card Title", "text-lg font-bold text-gray-800"))
    .child(View::text_styled("Card description", "text-sm text-gray-600"))
    .child(
        View::button_styled(
            "Action",
            Msg::Action,
            "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        )
    )
    .build()
```

### Scenario 2: Navigation Bar

```rust
View::row()
    .style("gap-4 px-6 py-3 bg-white border-b border-gray-200 flex items-center justify-between")
    .child(
        View::col()
            .style("gap-1")
            .child(View::text_styled("Brand", "text-xl font-bold text-blue-600"))
            .child(View::text_styled("Tagline", "text-xs text-gray-500"))
            .build()
    )
    .child(
        View::row()
            .style("gap-2")
            .child(View::button_styled("Home", Msg::Home, "px-3 py-1 text-gray-700 hover:bg-gray-100"))
            .child(View::button_styled("About", Msg::About, "px-3 py-1 text-gray-700 hover:bg-gray-100"))
            .build()
    )
    .build()
```

### Scenario 3: Form Layout

```rust
View::col()
    .style("gap-4 p-6 bg-white rounded-lg shadow-md")
    .child(View::text_styled("Sign Up", "text-2xl font-bold text-gray-800"))
    .child(
        View::col()
            .style("gap-2")
            .child(View::text_styled("Email", "text-sm font-medium text-gray-700"))
            .child(
                View::input("you@example.com")
                    .value(self.email.clone())
                    .style("px-3 py-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500")
                    .build()
            )
            .build()
    )
    .child(
        View::col()
            .style("gap-2")
            .child(View::text_styled("Password", "text-sm font-medium text-gray-700"))
            .child(
                View::input("••••••••")
                    .password()
                    .value(self.password.clone())
                    .style("px-3 py-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500")
                    .build()
            )
            .build()
    )
    .child(
        View::button_styled(
            "Sign Up",
            Msg::Submit,
            "w-full py-2 bg-blue-500 text-white rounded font-medium hover:bg-blue-600"
        )
    )
    .build()
```

### Scenario 4: Dashboard Layout

```rust
View::row()
    .style("flex h-screen bg-gray-100")
    .child(
        View::col()
            .style("w-64 bg-gray-800 text-white p-4 flex flex-col gap-4")
            .child(View::text_styled("Dashboard", "text-xl font-bold"))
            .child(View::text_styled("Menu Item 1", "text-gray-300 hover:text-white"))
            .child(View::text_styled("Menu Item 2", "text-gray-300 hover:text-white"))
            .build()
    )
    .child(
        View::col()
            .style("flex-1 p-6 overflow-auto")
            .child(View::text_styled("Welcome!", "text-2xl font-bold"))
            .child(
                View::row()
                    .style("gap-4 grid grid-cols-3")
                    .children(stats_cards)
                    .build()
            )
            .build()
    )
    .build()
```

### Scenario 5: Alert/Notification

```rust
View::container(
    View::col()
        .style("gap-2")
        .child(View::text_styled("Success!", "font-bold text-green-800"))
        .child(View::text_styled("Your changes have been saved.", "text-sm text-green-700"))
        .build()
)
.style("p-4 bg-green-100 border-l-4 border-green-500 rounded-r")
.build()
```

---

## Best Practices

### 1. Composition Over Repetition

❌ **Avoid**: Repeating style strings

```rust
View::col()
    .style("gap-4 p-6 bg-white rounded-lg shadow-md")
    .child(item1)
    .child(item2)
    .child(item3)
    .build()

View::col()
    .style("gap-4 p-6 bg-white rounded-lg shadow-md")  // Repeated
    .child(item4)
    .child(item5)
    .build()
```

✅ **Prefer**: Extract common styles

```rust
fn card_style() -> &'static str {
    "gap-4 p-6 bg-white rounded-lg shadow-md"
}

View::col().style(card_style()).child(item1).child(item2).build()
View::col().style(card_style()).child(item4).child(item5).build()
```

### 2. Logical Grouping

Group related styles:

```rust
// ✅ Good: Layout → Spacing → Colors → Effects
"flex flex-col items-center gap-4 p-6 bg-white rounded-lg shadow-md"

// ❌ Bad: Random order
"shadow-md bg-white flex rounded-lg p-6 gap-4 items-center flex-col"
```

**Recommended Order**:
1. Layout (`flex`, `grid`, etc.)
2. Alignment (`items-center`, `justify-center`, etc.)
3. Spacing (`gap-*`, `p-*`, `m-*`)
4. Sizing (`w-*`, `h-*`)
5. Colors (`bg-*`, `text-*`)
6. Borders (`border`, `rounded-*`)
7. Effects (`shadow`, `opacity`)

### 3. Responsive Spacing Scale

Use consistent spacing scale (4px base unit):

```rust
// ✅ Good: Consistent spacing
"gap-2 p-4"   // 8px gap, 16px padding

// ❌ Bad: Arbitrary values
"gap-3 p-7"   // 12px gap, 28px padding (non-standard)
```

**Standard Scale**: 0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24
(Pixels: 0, 4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96)

### 4. Color Consistency

Use semantic colors from palette:

```rust
// ✅ Good: Semantic colors
"text-gray-800"   // Primary text
"text-gray-600"   // Secondary text
"bg-blue-500"     // Primary action
"bg-red-500"      // Destructive action

// ❌ Bad: Arbitrary colors
"text-gray-843"   // Non-standard shade
"bg-blue-123"     // Non-standard shade
```

### 5. Visual Hierarchy

Use typography and spacing to create hierarchy:

```rust
View::col()
    .style("gap-3 p-6")  // Consistent spacing
    .child(View::text_styled("Heading", "text-2xl font-bold text-gray-800"))      // Large, bold
    .child(View::text_styled("Subheading", "text-lg font-medium text-gray-700"))  // Medium, medium
    .child(View::text_styled("Body text", "text-base text-gray-600"))             // Base, regular
    .child(View::text_styled("Caption", "text-sm text-gray-500"))                 // Small, light
    .build()
```

### 6. Border Radius Consistency

Use consistent border radius scale:

```rust
// Small elements
"rounded-sm"   // Buttons, tags

// Cards, containers
"rounded-lg"   // Cards, panels

// Pills, badges
"rounded-full" // Circular elements
```

### 7. Shadow Usage

Use shadows for depth:

```rust
// Subtle elevation
"shadow-sm"    // Cards, buttons

// Medium elevation
"shadow-md"    // Dropdowns, tooltips

// High elevation
"shadow-lg"    // Modals, popovers

// Don't overuse
// ❌ "shadow-xl" on every element
```

### 8. Performance Considerations

- ✅ **Parse styles once, reuse**: Use `Style::parse()` for static styles
- ✅ **Keep style strings short**: Prefer `"p-4 bg-white"` over verbose alternatives
- ✅ **Avoid dynamic style construction**: Don't build style strings at runtime

```rust
// ✅ Good: Static style string
View::col().style("p-4 bg-white").build()

// ⚠️ Acceptable: Pre-parsed style
let style = Style::parse("p-4 bg-white").unwrap();
View::col().with_style(style).build()

// ❌ Bad: Runtime string construction
let padding = format!("p-{}", value);
View::col().style(&padding).build()
```

---

## Backend Compatibility

### Feature Matrix

| Feature | GPUI | Iced | Notes |
|---------|------|------|-------|
| **Padding** (`p-*`, `px-*`, `py-*`) | ✅ Full | ✅ Full | |
| **Gap** (`gap-*`) | ✅ Full | ✅ Full | |
| **Colors** (`bg-*`, `text-*`) | ✅ Full | ✅ Full | |
| **Layout** (`flex`, `items-*`, `justify-*`) | ✅ Full | ✅ Full | |
| **Sizing** (`w-*`, `h-*`) | ✅ Full | ✅ Full | |
| **Border Radius** (`rounded-*`) | ✅ Full | ✅ Full | |
| **Margin** (`m-*`, `mx-*`, `my-*`) | ✅ Full | ❌ None | Use nested containers for Iced |
| **Typography** (`text-*`, `font-*`) | ✅ Full | ✅ Full | |
| **Border** (`border`, `border-color`) | ✅ Full | ✅ Full | |
| **Shadow** (`shadow-*`) | ✅ Full | ⚠️ Limited | Iced has basic shadow support |
| **Opacity** (`opacity-*`) | ✅ Full | ✅ Full | |
| **Position** (`relative`, `absolute`, `z-*`) | ✅ Full | ❌ None | Iced doesn't support absolute |
| **Overflow** (`overflow-*`) | ✅ Full | ✅ Full | |
| **Grid** (`grid`, `grid-cols-*`) | ✅ Full | ❌ None | Use flex for Iced |

### Graceful Degradation

When using features not supported by a backend, styles are silently ignored:

```rust
// Works perfectly on GPUI
// On Iced: margin is ignored, rest applies normally
View::col()
    .style("m-4 p-4 bg-white")  // m-4 ignored on Iced
    .child(...)
    .build()
```

### Cross-Backend Best Practices

1. **Test on both backends** if you target both GPUI and Iced
2. **Avoid unsupported features** for Iced if cross-platform compatibility is required
3. **Use fallback layouts** (nested containers instead of margin)
4. **Document backend-specific features** in code comments

---

## Troubleshooting

### Issue: Styles Not Applying

**Symptoms**: Component appears unstyled

**Causes**:
1. Missing `.build()` call
2. Invalid style string
3. Backend doesn't support feature
4. Style priority conflict

**Solutions**:
```rust
// ✅ Always call .build()
View::col().style("p-4 bg-white").child(...).build()

// ✅ Validate style string
let style = Style::parse("p-4 bg-white").unwrap();
dbg!(style);

// ✅ Check backend compatibility
// See Backend Compatibility matrix above
```

### Issue: Style Parse Error

**Symptoms**: Compiler error about invalid style

**Causes**:
1. Typo in style class name
2. Unsupported style class
3. Invalid numeric value

**Solutions**:
```rust
// ❌ Wrong: typo in "bg"
"bg-whte"  // Should be "bg-white"

// ❌ Wrong: unsupported class
"animate-bounce"  // Not implemented yet

// ❌ Wrong: invalid value
"p-99"  // Only p-0 through p-96 supported
```

### Issue: Inconsistent Appearance

**Symptoms**: Different appearance on GPUI vs Iced

**Causes**:
1. Backend doesn't support feature
2. Different default styling
3. Color mapping differences

**Solutions**:
```rust
// ✅ Use cross-platform compatible styles
"p-4 bg-white rounded"  // Works on both backends

// ⚠️ Backend-specific features
"m-4 absolute z-10"  // GPUI only
```

### Issue: Performance Degradation

**Symptoms**: Slow rendering, high memory usage

**Causes**:
1. Parsing styles at runtime
2. Excessive style string length
3. Too many styled components

**Solutions**:
```rust
// ✅ Parse once, reuse
static CARD_STYLE: &str = "p-6 bg-white rounded-lg shadow-md";
View::col().style(CARD_STYLE).build()

// ✅ Keep style strings concise
"p-4 bg-white"  // Good
"p-4 px-4 py-4 pt-4 pr-4 pb-4 pl-4 bg-white bg-#ffffff"  // Bad - redundant
```

---

## Resources

- [Migration Guide](./migration-guide.md) - Migrate from legacy API
- [Plan 004: Unified Styling System](../plans/004-unified-styling-system.md) - System design
- [Plan 005: Style System Integration](../plans/005-style-system-integration.md) - Implementation details
- [Examples](../../crates/auto-ui/examples/) - See `styling_showcase.rs` for complete examples
- [Tailwind CSS Documentation](https://tailwindcss.com/docs) - Reference for class names

---

**Last Updated**: 2025-01-21
**Version**: v0.1 (90% Tailwind CSS Coverage)
