//! Node2D 显式研究领域（auto-ui PLAN-016）。
//!
//! `node`/`view-2d`/`Life.*`/匿名 msg/前导点的研究文法与 AST。仅经
//! [`Node2DDialect`]（`parse_domain` 显式组合）可达；不改变
//! [`crate::UiDialect`] 生产路径、不进入 [`crate::UiCode`]。语义边界
//! 与降层合同见 `docs/plans/attachments/016/work/node2d-contract.md`。

pub mod ast;
pub mod game;
pub mod parse;
pub mod components2d;
pub mod presentation;

pub use ast::{
    Camera3dBlock, Character3dBlock, EventPattern, Handler, Node2DStmt, NodeDecl, OnBlock, Prop,
    PropValue, View2dBlock, View2dElement, View3dBlock, View3dElement,
};
pub use game::{Game2dAudio, Game2dBlock, Game2dCue, Game2dHud, Game2dShape, Game2dSlot,
    Game2dText, Game2dTrigger, Game2dTriggerKind};
pub use parse::Node2DDialect;
pub use components2d::{CmpAtlas, CmpPatrol, CmpSpawn, Components2dBlock};
pub use presentation::{PresHud, PresSlot, PresTrigger, Presentation2dBlock};
