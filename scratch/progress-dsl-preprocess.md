# DSL Preprocessor MVP Progress

## Date: 2025-01-26

## Summary

The DSL preprocessor is now functional and can transform DSL-style `.at` files into standard Auto syntax that can be parsed and transpiled to Rust code.

## Completed

1. **DSL Preprocessor Implementation** (`dsl_preprocess.rs`)
   - `msg X {}` → `enum X {}`
   - `widget X { model {} view {} on {} style {} }` → `type X { fields fn view() fn on() }`
   - `button +` → `button "+"`
   - `h2 > text` → `h2 "text"` (uses regular strings to avoid parser bug)
   - `app X {}` → `fn main() { app("X") {} }`
   - Indentation preservation in nested blocks

2. **Bug Fixes**
   - Fixed `transform_dot_field` to not transform `CounterMsg.Inc` into `CounterMsgself.Inc`
   - Fixed indentation in `reindent_block` to preserve relative indentation
   - Fixed template literal bug by using regular strings

3. **Known Parser Bugs in auto-lang**
   - Template literals followed by another element cause parser error
     - Workaround: Use regular strings `"..."` instead of backticks `` `...` ``
   - Enum variants in pattern matching (`Msg.Inc => { }`) cause parser error
     - Workaround: Skip `on` block transformation for now, add TODO comment

## Test Results

- All 5 DSL preprocess unit tests pass
- DSL-to-Rust transpilation works (generates struct and Component impl)
- Generated code has limitations:
  - `h2` widget not recognized by code generator
  - Button widgets not fully generated in view()

## Remaining Work

1. **Code Generator Enhancements**
   - Add support for `h2` widget in `rust_gen.rs`
   - Fix button widget generation
   - Add `center`, `app` widget support

2. **auto-lang Parser Fixes** (upstream)
   - Template literal parsing bug
   - Enum variant pattern matching bug

3. **Backend Integration**
   - GPUI backend rendering
   - Iced backend rendering

## Files Modified

- `crates/auto-ui/src/trans/dsl_preprocess.rs` - Main preprocessor
- `crates/auto-ui/src/trans/api.rs` - Added `transpile_dsl`, `transpile_auto`
- `crates/auto-ui/src/trans/mod.rs` - Module exports
- `crates/auto-ui/examples/debug_ast.rs` - Updated to use new Parser API
- `crates/auto-ui/examples/test_counter_dsl.rs` - Test example

## Next Steps

1. Add `h2` widget support to Rust code generator
2. Fix button onclick event binding in generated code
3. Test end-to-end with GPUI runtime
