//! PLAN-981（B-SESSION-I，S2）：driver 内的 AGE typed 宿主入口。
//!
//! 实现只有一份：与 bridge crate 共享 `age_session_core.rs`（`include!`），
//! 两边只差依赖图（这里用 vendored 到 `.sdk-source` 的 ACE）。

include!("../../age-ace-session/src/age_session_core.rs");
