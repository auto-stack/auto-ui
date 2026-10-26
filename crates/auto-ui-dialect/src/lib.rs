//! UI owned AST、完整 UI 文法与版本化 codec 的单一来源。
//!
//! `UiDialect` 经中立 `DomainParseContext` 委托 Core expr/type/body，
//! 不复制 Parser/Pratt。调用方以 `Parser::parse_domain(&UiDialect)`
//! 显式组合并取得 `UiCode`，再校验/lowering 后进入执行与生成器；
//! 无进程级文法表或 Core `Stmt` UI 兼容臂。
//!
//! 两个 model 合同面共用一份块循环与字段算法（T-01 D4）：
//! - [`ModelGrammar::Builtin`]：内建完整文法（注解 + `var` + name +
//!   [type] [= expr]，**无裸字段**）——生产默认；
//! - [`ModelGrammar::ExternalSubset`]：PLAN-778 冻结的外部子集（裸字段
//!   允许、注解让行 [`ModelSubset::NotClaimed`]）——仅在显式 pilot
//!   策略下启用（默认 false，不构成默认文法漂移）。
//!
//! 行为冻结要点（迁出时未收紧/未扩展）：
//! - msg：旧 `msg Name {` 名字读弃（Plan 448 A）；quoted variant 合同名；
//!   多 payload（Plan 043 M5 #1）；named payload 经端口零副作用原语
//!   `probe_next_kind` 做原始单 token 探测（Plan 435 挂账②；成功路径
//!   零副作用/幂等、失败路径遵循旧回推前瞻失败消费协议——与旧 lexer
//!   单 token 回推逐字同构，F806-R2-01/R3-01 冻结形态）；
//! - model：`#[primary]`/`#[Consume]`/`#[Provide]`/`#[NavParam]` 注解
//!   （Plan 119/05-Nav），未知注解错误文本与 span 逐字保持；`var`
//!   可变旗标（Plan 130）；空行/逗号宽容；
//! - 错误：关键字消费复刻 `Parser::expect_ident`（"Expected 'x', got
//!   'y'"、纯文本比较、无 kind 检查——桥面 `expect_keyword` 的文本与
//!   检查语义不同，T-01 D3）；`expect` 族经桥面走 `Parser::expect`
//!   同一实现；未知注解经 `syntax_error`（`SyntaxError::Generic` 同文
//!   消息 + `pos_to_span(cur.pos)` 同位）。

pub mod ast;
pub mod codec;
pub mod declarations;
pub mod grammar;
pub mod model;
pub mod route;
pub mod validation;

pub use ast::*;
pub use codec::{decode_ui_code, encode_ui_code, UI_AST_DOMAIN, UI_AST_SCHEMA_VERSION};
pub use model::{UiCode, UiStmt};
pub use route::{RouteDef, RoutesBlock};
pub use validation::validate_ui;
// PLAN-814 R3-03b：UI 文法依赖回归测试组（随迁自 auto-lang-core 门控测试；
// dev-dep 驱动，仅测试编译）。
#[cfg(test)]
mod regression;

pub use declarations::{
    parse_model_block, parse_model_block_subset, parse_msg_decl, ModelGrammar, ModelSubset,
};
pub use grammar::{UiDialect, UiParseContext};
