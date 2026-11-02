# 009-article-feed — Blog Article Card Feed

A vertical feed of three blog article cards, each showing title, excerpt, and author/date footer.

## Concepts

- Nested col/row composition
- Text content binding to model fields
- Repeated card pattern
- Constrained-width layout with `max-w-2xl`

## Source

See `front/app.at`:

```auto
widget App {
    model {
        var title1 str = "Getting Started with AutoLang"
        var excerpt1 str = "Learn the basics of AutoLang and build your first cross-platform app."
        var author1 str = "Alice"
        var date1 str = "Apr 10, 2026"
        var title2 str = "Cross-Platform UI with AURA"
        var excerpt2 str = "Build once, deploy everywhere. AURA widgets compile to Vue, Compose, ArkTS, and GPUI."
        var author2 str = "Bob"
        var date2 str = "Apr 12, 2026"
        var title3 str = "Advanced Patterns in AutoLang"
        var excerpt3 str = "Take your skills to the next level with tasks, specs, and the type system."
        var author3 str = "Charlie"
        var date3 str = "Apr 14, 2026"
    }

    view {
        col {
            text "Latest Articles"
            col {
                col {
                    col {
                        text .title1
                        text .excerpt1
                        row {
                            text .author1
                            text .date1
                            class: "gap-2 items-center"
                        }
                        class: "p-4 gap-2"
                    }
                    class: "bg-white rounded-lg shadow overflow-hidden"
                }
                col {
                    col {
                        text .title2
                        text .excerpt2
                        row {
                            text .author2
                            text .date2
                            class: "gap-2 items-center"
                        }
                        class: "p-4 gap-2"
                    }
                    class: "bg-white rounded-lg shadow overflow-hidden"
                }
                col {
                    col {
                        text .title3
                        text .excerpt3
                        row {
                            text .author3
                            text .date3
                            class: "gap-2 items-center"
                        }
                        class: "p-4 gap-2"
                    }
                    class: "bg-white rounded-lg shadow overflow-hidden"
                }
                class: "gap-6"
            }
            class: "w-full max-w-2xl p-8 gap-6"
        }
    }
}
```

## How to Run

```bash
cd examples/ui/009-article-feed
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- Nested col/row: card outer shell > content column > author footer row
- Text content: binding model strings to text elements with `.title1`, `.excerpt1`
- Repeated card pattern: three identical card structures with different data
- Width constraint: `max-w-2xl` keeps the feed readable on wide screens
- Card styling: `bg-white rounded-lg shadow overflow-hidden` for card appearance
