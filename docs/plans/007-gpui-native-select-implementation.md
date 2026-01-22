# Plan 007: Native GPUI Select Widget Implementation

**Status**: 📋 Planning
**Created**: 2025-01-23
**Priority**: High
**Complexity**: High

## Overview

Implement native `gpui_component::select::Select` widget support in the GPUI backend adapter, enabling proper dropdown functionality for the unified-select example with the GPUI backend.

## Problem Statement

### Current State
- ✅ Iced backend: Full working dropdown with `iced::widget::pick_list`
- ❌ GPUI backend: Shows text display instead of actual dropdown widget
- ✅ Callback API: Already implemented - callbacks receive selected values
- ❌ GPUI adapter: Cannot render native Select widget due to architectural limitations

### Root Cause

The GPUI adapter's `into_gpui_impl_with_context()` method lacks the necessary context to create GPUI's Select widget:

```rust
fn into_gpui_impl_with_context<C>(
    self,
    state: &mut GpuiComponentState<C>,
    cx: &mut Context<GpuiComponentState<C>>,  // ❌ Missing Window!
) -> AnyElement
```

But GPUI's Select widget requires:
1. `&mut Window` parameter for creating SelectState
2. Entity creation: `cx.new(|cx| SelectState::new(...))`
3. Event subscriptions: `cx.subscribe_in(&select_state, window, handler)`
4. SelectItem trait implementation for options

### Why Framework-Specific Example Works

The framework-specific GPUI example ([gpui-examples/src/bin/select.rs](../../crates/gpui-examples/src/bin/select.rs)) works because it has full access to Window and Context:

```rust
impl SelectExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let select_state = cx.new(|cx| {
            SelectState::new(languages, Some(indexPath), window, cx)
        });

        cx.subscribe_in(&select_state, window, Self::on_select_event)
            .detach();
        // ...
    }
}
```

## Architecture Analysis

### Current GPUI Adapter Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ run_app()                                                  │
│   └─> cx.new(|_| GpuiComponentState::new(component))        │
│       └─> Window opened                                    │
│           └─> cx.new(|cx| Root::new(state, window, cx))      │
│               └─> state.render(window, cx)                   │
│                   └─> component.view()                      │
│                       └─> view.render_gpui_with(state, cx)   │
│                           └─> ❌ No Window!          │
└─────────────────────────────────────────────────────────────┘
```

### Required Architecture for Select Widget

```
┌─────────────────────────────────────────────────────────────┐
│ run_app()                                                  │
│   └─> cx.new(|_| GpuiComponentState::new(component))        │
│       ├─> Create SelectState entities HERE ✅                │
│       │   - During component initialization                   │
│       │   - Has access to window & cx                       │
│       │   - Can subscribe to events                          │
│   └─> Store entities in state.select_states HashMap          │
│       └─> Window opened                                    │
│           └─> cx.new(|cx| Root::new(state, window, cx))      │
│               └─> state.render(window, cx)                   │
│                   └─> component.view()                      │
│                       └─> view.render_gpui_with(state, cx)   │
│                           ├─> state.get_select_entity()   │
│                           ├─> Select::new(&entity)      │
│                           └─> ✅ Native Select widget!│
└─────────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Extend GpuiComponentState with Select State Management

**Goal**: Create and cache SelectState entities during component initialization.

#### 1.1 Add Select State Creation Method

**File**: `crates/auto-ui-gpui/src/auto_render.rs`

```rust
impl<C: Component> GpuiComponentState<C> {
    /// Create a SelectState entity for the given options
    /// Must be called during component initialization (not render!)
    pub fn create_select_state(
        &mut self,
        key: String,
        options: Vec<String>,
        selected_index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<Vec<String>> {
        let state = cx.new(|cx| {
            SelectState::new(
                options,
                selected_index.map(|i| IndexPath::default().row(i)),
                window,
                cx,
            )
        });

        self.select_states.insert(key.clone(), state.clone());

        // Subscribe to selection changes
        let key_clone = key.clone();
        std::mem::forget(
            cx.subscribe_in(&state, window, move |comp, entity, event, cx| {
                if let SelectEvent::Confirm(value) = event {
                    // Trigger component update via callback
                    // This requires storing callback with the state
                }
                cx.notify();
            }).detach()
        );

        state
    }
}
```

**Problem**: How to trigger component message when Select fires event?

#### 1.2 Store Callback with Select State

We need to associate callbacks with SelectState entities. Options:

**Option A**: Create a wrapper struct
```rust
struct SelectStateWithCallback<M> {
    state: Entity<SelectState<Vec<String>>>,
    callback: SelectCallback<M>,
}
```

**Option B**: Use a separate HashMap
```rust
select_callbacks: HashMap<String, SelectCallback<M>>,
```

**Option C**: Store callback in a parallel entity (complex)

**Decision**: Use Option A - create wrapper for better organization.

#### 1.3 Update get_or_create_select_state

```rust
pub fn get_or_create_select_state(
    &mut self,
    key: String,
    options: Vec<String>,
    selected_index: Option<usize>,
    callback: Option<SelectCallback<C::Msg>>,
    window: &mut Window,
    cx: &mut Context<Self>,
) -> Entity<SelectState<Vec<String>>> {
    // Always create new state with callback if provided
    let state = cx.new(|cx| {
        SelectState::new(
            options.clone(),
            selected_index.map(|i| IndexPath::default().row(i)),
            window,
            cx,
        )
    });

    if let Some(cb) = callback {
        // Subscribe to events
        std::mem::forget(
            cx.subscribe_in(&state, window, move |comp, _entity, event, cx| {
                if let SelectEvent::Confirm(value) = event {
                    let index = options.iter()
                        .position(|s| s.as_str() == *value)
                        .unwrap_or(0);
                    let msg = cb.call(index, value);
                    comp.handle(msg);
                    cx.notify();
                }
            }).detach()
        );
    }

    self.select_states.insert(key, state.clone());
    state
}
```

### Phase 2: Modify run_app to Pre-initialize Select States

**Goal**: Parse the View tree, find all Select widgets, and create their entities upfront.

**File**: `crates/auto-ui-gpui/src/lib.rs`

#### 2.1 Add Pre-initialization Method

```rust
impl<C: Component + 'static> GpuiComponentState<C>
where
    C::Msg: Clone + Debug + 'static,
{
    /// Scan the view tree and pre-create all SelectState entities
    pub fn preinitialize_selects(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = self.component.view();
        self.scan_view_for_selects(view, window, cx);
    }

    fn scan_view_for_selects(
        &mut self,
        view: View<C::Msg>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match view {
            View::Select { options, selected_index, on_select, .. } => {
                // Generate unique key for this select widget
                let key = format!("select_{:?}", options);

                // Create the entity
                self.get_or_create_select_state(
                    key,
                    options,
                    selected_index,
                    on_select,
                    window,
                    cx,
                );
            }
            View::Row { children, .. } | View::Column { children, .. } => {
                for child in children {
                    self.scan_view_for_selects(child, window, cx);
                }
            }
            View::Container { child, .. } => {
                self.scan_view_for_selects(*child.clone(), window, cx);
            }
            // ... handle other View variants
            _ => {}
        }
    }
}
```

#### 2.2 Update run_app

```rust
pub fn run_app<C>(title: &str) -> auto_ui::AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + 'static,
{
    let app = gpui::Application::new();

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions { /* ... */ },
                |window, cx| {
                    // Create the state
                    let mut state = cx.new(|_| GpuiComponentState::new(C::default()));

                    // ✅ NEW: Pre-initialize Select states
                    state.update(cx, |state, cx| {
                        state.preinitialize_selects(window, cx);
                    });

                    // Build the UI
                    cx.new(|cx| Root::new(state, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });

    Ok(())
}
```

### Phase 3: Update Rendering to Use Cached Entities

**File**: `crates/auto-ui-gpui/src/auto_render.rs`

#### 3.1 Modify View::Select Rendering

```rust
View::Select { options, selected_index, on_select, style } => {
    // Generate key for this select
    let key = format!("select_{:?}", options);

    // Retrieve cached SelectState entity
    if let Some(select_state) = state.select_states.get(&key) {
        // ✅ Use native Select widget!
        let mut select = Select::new(select_state)
            .placeholder("Select an option");

        // Apply unified styling if present
        if let Some(style) = style {
            select = apply_style_to_select(select, &style);
        }

        select.into_any()
    } else {
        // Fallback: no pre-initialized state (shouldn't happen)
        let selected = selected_index
            .and_then(|i| options.get(i).cloned())
            .unwrap_or_default();

        div().child(format!("Select: {}", selected)).into_any()
    }
}
```

#### 3.2 Add Style Application Function

```rust
fn apply_style_to_select(
    mut select: Select<SelectState<Vec<String>>>,
    style: &Style,
) -> Select<SelectState<Vec<String>>> {
    for class in &style.classes {
        match class {
            StyleClass::Size(size) => {
                select = match size {
                    SizeValue::Small => select.with_size(gpui_component::Size::Small),
                    SizeValue::Medium => select.with_size(gpui_component::Size::Medium),
                    SizeValue::Large => select.with_size(gpui_component::Size::Large),
                };
            }
            StyleClass::Disabled => {
                select = select.disabled(true);
            }
            // Handle more style classes...
            _ => {}
        }
    }
    select
}
```

### Phase 4: Implement SelectItem Trait

**Problem**: `SelectState<Vec<String>>` requires String to implement SelectItem.

**Solution**: String already implements SelectItem (lines 69-91 in gpui-component select.rs), so we're good!

### Phase 5: Handle Dynamic Updates

**Problem**: What if options change at runtime?

**Solution**:
1. Detect if options changed (compare with cached)
2. Re-create SelectState entity with new options
3. Update subscription with new callback

```rust
fn get_or_create_select_state(
    &mut self,
    key: String,
    options: Vec<String>,
    selected_index: Option<usize>,
    callback: Option<SelectCallback<C::Msg>>,
    window: &mut Window,
    cx: &mut Context<Self>,
) -> Entity<SelectState<Vec<String>>> {
    // Check if we need to recreate
    let needs_recreate = if let Some(existing) = self.select_states.get(&key) {
        let current_options = existing.read(cx).get_items_count(0, cx);
        // Simple check: do lengths match?
        current_options != options.len()
    } else {
        true
    };

    if needs_recreate {
        // Recreate state with new options
        self.create_select_state(key, options, selected_index, callback, window, cx)
    } else {
        // Return existing
        self.select_states.get(&key).cloned().unwrap()
    }
}
```

## Implementation Order

### Step 1: Extend GpuiComponentState
- [ ] Add select_callbacks HashMap
- [ ] Implement create_select_state() with event subscription
- [ ] Update get_or_create_select_state() with callback support
- [ ] Add preinitialize_selects() method
- [ ] Implement scan_view_for_selects() recursive scanner

### Step 2: Integrate into run_app
- [ ] Modify run_app to call preinitialize_selects()
- [ ] Test that states are created before rendering

### Step 3: Update Rendering
- [ ] Modify View::Select rendering to use cached entities
- [ ] Implement apply_style_to_select()
- [ ] Add fallback for missing entities

### Step 4: Testing
- [ ] Test unified-select with GPUI backend
- [ ] Verify callback receives correct values
- [ ] Verify subscription triggers component updates
- [ ] Test dynamic option changes (if implemented)

## Technical Challenges

### Challenge 1: Callback Storage with Entity
- **Problem**: SelectState doesn't have a field for callbacks
- **Solution**: Store callbacks separately in GpuiComponentState
- **Status**: ⚠️ Needs implementation

### Challenge 2: Message Type Erasure
- **Problem**: Event subscription doesn't know about Component::Msg
- **Solution**: Use dynamic dispatch or trait objects
- **Status**: ⚠️ Needs investigation

### Challenge 3: Lifecycle Management
- **Problem**: When to recreate SelectState entities?
- **Solution**: Detect option changes and recreate
- **Status**: ⚠️ Needs implementation

### Challenge 4: Window Parameter Access
- **Problem**: render_gpui_with() doesn't have Window
- **Solution**: Pre-create entities in run_app before rendering
- **Status**: ✅ Solved by pre-initialization approach

## Alternative Approaches

### Alternative A: Create Select Wrapper Entity
```rust
struct ManagedSelect<M> {
    select_state: Entity<SelectState<Vec<String>>>,
    _phantom: PhantomData<M>,
}

impl<M> Render for ManagedSelect<M> {
    fn render(&mut self, window: &mut Window, cx: &Context<Self>) -> impl IntoElement {
        Select::new(&self.select_state)
            .window_size(window.size())
            .into_any_element()
    }
}
```
**Pros**: Encapsulates Select with callback
**Cons**: More complex wrapper hierarchy

### Alternative B: Lazy State Creation
Create SelectState on first render, then cache for subsequent renders.
**Pros**: Simpler initial setup
**Cons**: First frame won't have proper dropdown

### Alternative C: Custom Select Widget
Build our own dropdown widget that doesn't need entity management.
**Pros**: Full control, simpler architecture
**Cons**: Reimplementing existing functionality, maintenance burden

**Decision**: Stick with native Select widget (Alternative A/Pre-initialization approach)

## Testing Strategy

### Unit Tests
- [ ] Test select state creation
- [ ] Test callback invocation
- [ ] Test option change detection

### Integration Tests
- [ ] unified-select with GPUI backend
- [ ] Verify dropdown appears and functions
- [ ] Verify value updates on selection
- [ ] Verify callback receives correct (index, value)

### Comparison Tests
- [ ] Compare GPUI and Iced backend behavior
- [ ] Ensure identical functionality

## Success Criteria

### Must Have
- ✅ Dropdown widget appears (not just text)
- ✅ Clicking opens dropdown list
- ✅ Selecting option updates component state
- ✅ Callback receives correct index and value
- ✅ Works with unified-select example

### Nice to Have
- ✅ Keyboard navigation (arrow keys)
- ✅ Search/filter functionality
- ✅ Dynamic option updates
- ✅ Disabled state support
- ✅ Custom styling support

## Dependencies

### Internal
- `gpui_component::select::*` - Already imported
- `gpui::Entity`, `gpui::Context` - Already used
- `gpui::Window` - Available in run_app

### External
- None (all dependencies already in workspace)

## Timeline Estimate

- Phase 1: 2-3 hours (entity management foundation)
- Phase 2: 1-2 hours (integration into run_app)
- Phase 3: 2-3 hours (rendering updates)
- Phase 4: <1 hour (SelectItem already done)
- Phase 5: 2-3 hours (dynamic updates, testing)

**Total**: 8-12 hours of focused development

## Risks & Mitigation

### Risk 1: Breaking Existing Code
- **Mitigation**: Add feature flag or use opt-in via new method
- **Impact**: Low - changes are internal to GPUI adapter

### Risk 2: Performance Overhead
- **Mitigation**: Only pre-initialize when Select widgets exist
- **Impact**: Low - minimal overhead

### Risk 3: Complex Event Subscriptions
- **Mitigation**: Test thoroughly with multiple Select widgets
- **Impact**: Medium - event bugs are tricky to debug

### Risk 4: Lifecycle Edge Cases
- **Mitigation**: Add comprehensive tests for dynamic scenarios
- **Impact**: Medium - may require iteration

## Open Questions

1. **How to handle duplicate Select widgets with same options?**
   - Option: Use unique IDs (View::Select with id field)
   - Option: Create separate state for each instance

2. **How to clean up SelectState entities?**
   - Option: Let them persist for component lifetime
   - Option: Implement cleanup when Select widgets are removed

3. **Should we support searchable Select?**
   - Requires SearchableVec wrapper
   - Adds complexity but better UX

4. **How to handle SelectItem for custom types?**
   - Currently only String supported
   - Would need generic SelectItem wrapper

## References

- GPUI Select implementation: `D:/github/gpui-component/crates/ui/src/select.rs`
- GPUI Select example: `crates/gpui-examples/src/bin/select.rs`
- GPUI Slider implementation (similar pattern): `crates/auto-ui-gpui/src/auto_render.rs` (slider caching)

## Next Steps

1. **Review and approve this plan**
2. **Implement Phase 1** (entity management)
3. **Test state creation** with simple example
4. **Implement Phase 2** (run_app integration)
5. **Implement Phase 3** (rendering updates)
6. **Full testing** with unified-select example
7. **Documentation** of usage patterns

## Notes

- This is a significant architectural change that requires careful testing
- The pre-initialization pattern can be applied to other widgets (future work)
- Consider adding instrumentation/logging for debugging entity lifecycle
- May need to adjust GPUI Component library version if API changes are needed

---

**Document Status**: Ready for Implementation
**Last Updated**: 2025-01-23
**Author**: Claude Sonnet 4.5
**Review Status**: Pending
