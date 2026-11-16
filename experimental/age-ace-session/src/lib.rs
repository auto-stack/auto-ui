//! PLAN-981（B-SESSION-I，S1）：AGE Session 的 typed 宿主桥接层（独立 workspace）。
//!
//! 实现只有一份：`age_session_core.rs`（driver 也用 `include!` 编译同一文件）。
//! 本 crate 依赖 **live** auto-lang 计划树（组内兄弟仓相对路径），用于 S1/S3 的
//! 独立验证；driver 侧则编译 vendored 到 `.sdk-source` 的 ACE 副本。

include!("age_session_core.rs");
