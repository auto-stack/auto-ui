//! autographics-node2d-pilot — PLAN-016 研究 adapter 库（T-04）。
//!
//! 分段（合同 §2）：load（领域解析+模块表）→ validate（执行前定位诊断）
//! → lower（实例表→单文件 Auto 用户源 + 场景文档）。游戏规则全部落在
//! 生成的真实 Auto 源里；Rust 只做调度、类型化桥与图形服务连接。
//! 本 crate 刻意不依赖 AG driver（tip vs pinned-SDK 两棵依赖树绝不
//! 同图）；生成的源/场景由 driver 侧 node2d_flow 测试作为冻结数据消费。
pub mod load;
pub mod lower;
pub mod scene;
pub mod translate;

pub use load::{load_project, Project, ProjectError};
pub use lower::{lower_project, LowerOutput};
