# Plan: Vue.js Backend for AutoUI Transpiler

## Goal

Implement a transpiler backend that generates Vue 3 Single File Components (SFC) from Auto language code, enabling frontend web development with the Auto DSL syntax.

---

## Phase 1: Foundation & Architecture ✅ **COMPLETED** (2025-01-26)

### 1.1 Project Setup
- [x] Create new module `trans/vue_gen.rs`
- [x] Add feature flag `vue-backend` in Cargo.toml
- [x] Define `VueCodeGenerator` struct with state tracking

### 1.2 Core Generator Structure
```rust
pub struct VueCodeGenerator {
    current_widget: Option<String>,
    imports: HashSet<String>,
    component_refs: Vec<String>,
    fields: Vec<(String, String, Option<String>)>,
    has_emit: bool,
    emit_events: Vec<String>,
    handlers: Vec<(String, String)>,
    template_nodes: Vec<String>,
    tailwind_classes: String,
}
```

### 1.3 API Entry Point
- [x] Add `transpile_vue_ast(ast: &Code) -> Result<String, String>`
- [x] Add `transpile_vue_file(input_path, output_path)`
- [x] Add `transpile_vue_dsl(source, output_path)`
- [x] Add `transpile_vue_auto(source, output_path)`

### 1.4 Implementation Notes
- `Member` in auto-lang is a struct with `name`, `ty`, `value` fields (not an enum)
- `Arg::Pair(name, value)` for named arguments
- `Expr::Float(f, _)` takes two fields (value and span)
- `AutoStr` needs `.to_string()` conversion to Rust `String`

### Files Created/Modified
- `crates/auto-ui/src/trans/vue_gen.rs` - Main Vue code generator (~490 lines)
- `crates/auto-ui/src/trans/api.rs` - Added `vue` module with transpile functions
- `crates/auto-ui/src/trans/mod.rs` - Added `vue_gen` module and re-export
- `crates/auto-ui/Cargo.toml` - Added `vue-backend` feature flag
- `crates/auto-ui/examples/test_counter_vue.rs` - Test example

---

## Phase 2: Auto → Vue Mapping Design

### 2.1 Widget → Vue SFC Structure

**Auto Input:**
```auto
widget Counter {
    model {
        count int
    }
    
    view {
        col {
            button "+" { onclick: CounterMsg.Inc }
            h2 `Count: ${count}`
            button "-" { onclick: CounterMsg.Dec }
        }
    }
    
    on {
        CounterMsg.Inc => { .count += 1 }
        CounterMsg.Dec => { .count -= 1 }
    }
    
    style {
        px-4 py-4
    }
}
```

**Vue 3 Output:**
```vue
<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)

const emit = defineEmits<{
  inc: []
  dec: []
}>()

function onInc() {
  count.value += 1
}

function onDec() {
  count.value -= 1
}
</script>

<template>
  <div class="flex flex-col px-4 py-4">
    <button @click="onInc">+</button>
    <h2>Count: {{ count }}</h2>
    <button @click="onDec">-</button>
  </div>
</template>

<style scoped>
/* Optional scoped styles */
</style>
```

### 2.2 Message Enum → Emits

| Auto Concept | Vue Equivalent |
|-------------|----------------|
| `msg X { A, B }` | `defineEmits<{ a: [], b: [] }>()` |
| `onclick: X.A` | `@click="emit('a')"` |
| Internal state change | Local function call |

### 2.3 Model Fields → Reactive State

| Auto Type | Vue Type |
|-----------|----------|
| `count int` | `const count = ref<number>(0)` |
| `name str` | `const name = ref<string>('')` |
| `items list` | `const items = ref<any[]>([])` |
| `data map` | `const data = reactive({})` |

### 2.4 UI Components → HTML/Tailwind

| Auto Node | Vue Template |
|-----------|--------------|
| `col { }` | `<div class="flex flex-col">` |
| `row { }` | `<div class="flex flex-row">` |
| `center { }` | `<div class="flex items-center justify-center">` |
| `button "label" { onclick: M }` | `<button @click="handler">label</button>` |
| `h1/h2/h3 "text"` | `<h1>text</h1>` |
| `text(content)` | `<span>{{ content }}</span>` |
| `input { bind: value }` | `<input v-model="value" />` |
| `ChildWidget(prop: val)` | `<ChildWidget :prop="val" />` |

### 2.5 Style Handling

- Auto `style { px-4 py-4 }` → Tailwind CSS classes
- Support inline styles as fallback
- Optional: Generate `<style scoped>` blocks

---

## Phase 3: Implementation Steps

### 3.1 Basic SFC Generation
- [ ] Generate `<script setup>` boilerplate
- [ ] Generate `<template>` section
- [ ] Generate empty `<style scoped>` section

### 3.2 Model/State Generation
- [ ] Parse `model {}` block fields
- [ ] Generate `ref()` declarations with type annotations
- [ ] Handle default values

### 3.3 Template Generation
- [ ] Implement `generate_vue_template(node: &Node)`
- [ ] Handle nested nodes recursively
- [ ] Map component names to HTML elements
- [ ] Generate Tailwind classes from style blocks

### 3.4 Event/Message Handling
- [ ] Parse `msg X {}` enum declarations
- [ ] Generate `defineEmits()` call
- [ ] Parse `on {}` pattern matching
- [ ] Generate event handler functions

### 3.5 Child Components
- [ ] Detect child widget instantiations
- [ ] Generate `import` statements
- [ ] Generate component props binding
- [ ] Handle event forwarding (`@inc="emit('inc')"`)

### 3.6 App Entry Point
- [ ] Handle `app X {}` declaration
- [ ] Generate `App.vue` root component
- [ ] Optionally generate `main.ts` entry file

---

## Phase 4: Advanced Features

### 4.1 Props Support
```auto
widget Button {
    props {
        label str
        variant str = "primary"
    }
}
```
→
```vue
<script setup lang="ts">
const props = defineProps<{
  label: string
  variant?: string
}>()
</script>
```

### 4.2 Two-Way Binding
```auto
input { bind: searchText }
```
→
```vue
<input v-model="searchText" />
```

### 4.3 Conditional Rendering
```auto
if showDetails {
    DetailPanel {}
}
```
→
```vue
<DetailPanel v-if="showDetails" />
```

### 4.4 List Rendering
```auto
for item in items {
    ItemCard(item: item)
}
```
→
```vue
<ItemCard 
  v-for="item in items" 
  :key="item.id"
  :item="item"
/>
```

### 4.5 Slots
```auto
widget Card {
    slot content
}
```
→
```vue
<slot name="content"></slot>
```

---

## Phase 5: Testing & Examples

### 5.1 Test Cases
- [ ] Simple counter widget
- [ ] Todo list with items
- [ ] Form with inputs and validation
- [ ] Nested component hierarchy
- [ ] Event propagation

### 5.2 Example Applications
- [ ] Counter app (basic)
- [ ] Todo app (lists, events)
- [ ] Form app (inputs, validation)
- [ ] Dashboard (multiple components)

### 5.3 Output Verification
- [ ] Generate `.vue` files
- [ ] Test with Vite dev server
- [ ] Verify TypeScript types
- [ ] Check Tailwind styling

---

## Phase 6: Developer Experience

### 6.1 CLI Integration
```bash
# Transpile single file
autoc transpile counter.at --backend vue --output counter.vue

# Transpile directory
autoc transpile ./src --backend vue --output ./vue-components
```

### 6.2 Hot Reload Support
- Watch `.at` files for changes
- Regenerate `.vue` files
- Trigger Vite HMR

### 6.3 Source Maps (Future)
- Generate source maps for debugging
- Map Vue errors back to Auto source

---

## File Structure

```
crates/auto-ui/
├── src/trans/
│   ├── mod.rs
│   ├── api.rs              # Add transpile_to_vue()
│   ├── vue_gen.rs          # NEW: Vue code generator
│   └── vue_template.rs     # NEW: Vue template generator
├── tests/
│   └── vue_transpiler_test.rs  # NEW
└── examples/
    └── vue/
        ├── counter.at      # Example Auto files
        ├── counter.vue     # Generated output
        └── app.vue         # Generated root
```

---

## Technical Decisions

### 1. TypeScript vs JavaScript
**Decision:** Generate TypeScript by default (`<script setup lang="ts">`)
- Better type safety
- Better IDE support
- Can be stripped for production

### 2. Tailwind vs Inline CSS
**Decision:** Generate Tailwind classes by default
- Matches Auto's design intent (utility classes)
- Option to generate inline styles for no-build setups

### 3. Composition API vs Options API
**Decision:** Use Composition API with `<script setup>`
- Modern Vue 3 standard
- Better type inference
- More flexible code organization

### 4. Single File vs Split Output
**Decision:** Generate Single File Components (`.vue`)
- Standard Vue convention
- Self-contained components
- Better tooling support

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Auto parser bugs affecting output | Create comprehensive test suite |
| Complex event handling patterns | Start simple, iterate |
| Vue-specific features not in Auto | Define clear subset of supported features |
| Performance with large templates | Profile and optimize generation |

---

## Success Criteria

1. Can transpile `counter.at` to working `Counter.vue`
2. Generated components work in Vite project
3. Events propagate correctly between components
4. Tailwind styles applied correctly
5. TypeScript types are valid

---

## Estimated Timeline

| Phase | Duration |
|-------|----------|
| Phase 1: Foundation | 1-2 days |
| Phase 2: Design | 1 day |
| Phase 3: Implementation | 3-5 days |
| Phase 4: Advanced | 2-3 days |
| Phase 5: Testing | 1-2 days |
| Phase 6: DX | 1-2 days |
| **Total** | **9-15 days** |

---

## Next Steps

1. Review and approve this plan
2. Start Phase 1: Create `vue_gen.rs` skeleton
3. Implement basic Counter example end-to-end
4. Iterate on remaining features

---

## Open Questions

1. **Q: Should we support both Tailwind and CSS output?**
   - Recommendation: Start with Tailwind only, add CSS option later

2. **Q: How to handle Auto's type system in TypeScript?**
   - `int` → `number`
   - `str` → `string`
   - `list` → `Array<T>`
   - Custom types → `interface`

3. **Q: Should we generate a complete Vue project or just components?**
   - Recommendation: Just components initially, project scaffolding as separate tool
