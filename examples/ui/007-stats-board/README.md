# 007-stats-board — Dashboard Metrics Board

A dashboard showing four metric cards (Revenue, Users, Orders, Growth) in a horizontal row.

## Concepts

- Data display with model state
- Repeated `col` card pattern
- `row` layout with flex-1 equal-width cards
- Shadow and rounded styling on cards

## Source

See `front/app.at`:

```auto
widget App {
    model {
        var title1 str = "Revenue"
        var value1 str = "$256K"
        var title2 str = "Users"
        var value2 str = "1,453"
        var title3 str = "Orders"
        var value3 str = "83M"
        var title4 str = "Growth"
        var value4 str = "+12%"
    }

    view {
        col {
            text "Dashboard Overview"
            row {
                col {
                    text .title1
                    text .value1
                    class: "bg-white rounded-lg shadow p-4 flex-1"
                }
                col {
                    text .title2
                    text .value2
                    class: "bg-white rounded-lg shadow p-4 flex-1"
                }
                col {
                    text .title3
                    text .value3
                    class: "bg-white rounded-lg shadow p-4 flex-1"
                }
                col {
                    text .title4
                    text .value4
                    class: "bg-white rounded-lg shadow p-4 flex-1"
                }
                class: "gap-4"
            }
            class: "w-full p-8 gap-6"
        }
    }
}
```

## How to Run

```bash
cd examples/ui/007-stats-board
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- Model state: `var` bindings for each metric title and value
- Data display: binding text content to model fields with `.title1`, `.value1`
- Repeated card pattern: four `col` cards with identical structure
- Row layout: `row` with `flex-1` cards for equal-width distribution
- Card styling: `bg-white rounded-lg shadow p-4` for a clean card appearance
