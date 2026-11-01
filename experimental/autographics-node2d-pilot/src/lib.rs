//! autographics-node2d-pilot — PLAN-016/017 研究 adapter 库。
//!
//! 分段（016 合同 §2，017 扩展）：load（领域解析+模块表）→ validate
//! （执行前定位诊断）→ lower（实例表→单文件 Auto 用户源 + 场景文档）。
//! 游戏规则全部落在生成的真实 Auto 源里；Rust 只做调度、类型化桥与
//! 图形服务连接。本 crate 刻意不依赖 AG driver（tip vs pinned-SDK
//! 两棵依赖树绝不同图）；生成的源/场景由 driver 侧测试作为冻结数据消费。
//!
//! PLAN-017：`Profile::G112D`（`--profile g11-2d` 显式选择）为完整
//! G11/2d 作者入口（声明驱动模板/角色/事件/调度）；`Profile::Legacy`
//! 保持 016 最小切片逐字节兼容（verify_plan016 再生控制），不被自动升级。
pub mod load;
pub mod lower;
pub mod lower_basic2d;
pub mod lower_components;
pub mod lower_g11;
pub mod scene;
pub mod translate;

/// 降层 profile（PLAN-017 T-02 冻结；PLAN-020 增补 components-2d；
/// PLAN-021 增补 queries-2d）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// PLAN-016 最小切片：字节兼容，服务映射不变。
    Legacy,
    /// PLAN-017 完整 G11/2d 作者入口。
    G112D,
    /// PLAN-019 基础二维：scene-2d/character-2d/camera-2d。
    Basic2D(lower_basic2d::BasicProfile),
    /// PLAN-020 有状态组件：components-2d（T-02 冻结的两套既有机制）。
    Components2D,
    /// PLAN-021 typed 二维查询：queries-2d（components-2d 超集 + query2d
    /// 服务/具名行/typed 结果；api/运行时装配不变）。
    Queries2D,
}

pub use load::{load_project, Project, ProjectError};
pub use lower::{lower_project, LowerOutput};
pub use lower_basic2d::{lower_project_basic, BasicProfile, LowerOutputBasic2D};
pub use lower_components::{lower_project_components, lower_project_queries};
pub use lower_g11::{lower_project_g112d, LowerOutputG11};
