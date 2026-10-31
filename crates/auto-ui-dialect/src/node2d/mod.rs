//! Node2D 显式研究领域（auto-ui PLAN-016）。
//!
//! `node`/`view-2d`/`Life.*`/匿名 msg/前导点的研究文法与 AST。仅经
//! [`Node2DDialect`]（`parse_domain` 显式组合）可达；不改变
//! [`crate::UiDialect`] 生产路径、不进入 [`crate::UiCode`]。语义边界
//! 与降层合同见 `docs/plans/attachments/016/work/node2d-contract.md`。

pub mod ast;
pub mod parse;

pub use ast::{
    EventPattern, Handler, Node2DStmt, NodeDecl, OnBlock, Prop, PropValue, View2dBlock,
    View2dElement,
};
pub use parse::Node2DDialect;
