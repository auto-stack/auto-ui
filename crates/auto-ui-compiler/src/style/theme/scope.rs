//! PLAN-031 T-03：App/组件主题作用域（ThemeScope）。
//!
//! 架构位置（§5.2）：App session / 组件实例持有自己的选择、目录与局部设计
//! 登记；builder 在构建一帧时**临时安装**当前 scope，旧 style 读取函数沿
//! 严格入栈/退出恢复的 guard 读取这一帧的有效快照。TLS 仍服务 legacy
//! 单应用路径（未安装 scope 时行为不变），但新多 App 路径不再依赖线程局部
//! 唯一选择——两个 scope 的发布/失效互不影响。
//!
//! 本类型是 compiler 的纯合同（零 runtime 依赖）：runtime 的
//! `DynamicComponent` 持有它并在 `view*` 入口安装 guard；Vue/宿主投影在后续
//! 切片接线同一快照。

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use auto_parser_api::ast::Expr;

use super::{ActiveSpec, ThemeCandidate};
use crate::design_tokens::registry::TokenName;
use crate::design_tokens::theme_tree::{self, ThemeTree, Tree};

/// 一帧构建期内安装的 scope guard：Drop 弹出栈顶并恢复上一帧/宿主状态。
/// 嵌套安装按栈序恢复，失败/panic 退出同样经 Drop 恢复。
pub struct InstalledThemeScope;

thread_local! {
    static SCOPE_STACK: RefCell<Vec<Rc<ThemeScope>>> = const { RefCell::new(Vec::new()) };
}

/// 安装 scope（入栈）；返回的 guard 存活期内该线程的 legacy 主题访问器都读
/// 这个 scope。guard 必须比构建动作活得久。
pub fn install_theme_scope(scope: Rc<ThemeScope>) -> InstalledThemeScope {
    SCOPE_STACK.with(|s| s.borrow_mut().push(scope));
    InstalledThemeScope
}

impl Drop for InstalledThemeScope {
    fn drop(&mut self) {
        SCOPE_STACK.with(|s| {
            s.borrow_mut().pop();
        });
    }
}

/// 当前安装的 scope（`None` = legacy 线程局部路径）。
pub(crate) fn current_scope() -> Option<Rc<ThemeScope>> {
    SCOPE_STACK.with(|s| s.borrow().last().cloned())
}

struct ScopeState {
    /// 主题 family（内置/合成/注册树）。
    family: ActiveSpec,
    /// 解析后的模式（light=false / dark=true）。
    dark: bool,
    /// 本 scope 的发布 revision（替代进程级 THEME_EPOCH 的新基准）。
    revision: u32,
    /// 本 scope 可见的主题目录（同名不同来源不互相冲掉）。
    catalog: BTreeMap<String, ThemeTree>,
    /// design 块登记（theme/mode 选择器）。
    design: Vec<(String, Expr)>,
}

/// App/组件级主题作用域：选择、目录、局部设计与 revision 的 owner。
pub struct ThemeScope {
    inner: RefCell<ScopeState>,
}

impl ThemeScope {
    /// 从当前 legacy 槽位捕获快照：宿主 CLI/config 已设置的 family/mode
    /// 与已注册目录在 scope 建立时继承，之后本 scope 独立演进。
    pub fn capture() -> Rc<Self> {
        let family = super::active_theme();
        let dark = super::DARK_MODE.with(|d| d.get());
        let catalog = super::THEME_TREES.with(|t| t.borrow().clone());
        Rc::new(Self {
            inner: RefCell::new(ScopeState {
                family,
                dark,
                revision: 0,
                catalog,
                design: Vec::new(),
            }),
        })
    }

    /// 当前 family 名（查找标签，不是身份）。
    pub fn family_name(&self) -> String {
        self.inner.borrow().family.name()
    }

    /// 解析后的模式。
    pub fn dark_mode(&self) -> bool {
        self.inner.borrow().dark
    }

    /// 本 scope 的发布 revision（只在 family/mode 实际变化时 +1）。
    pub fn revision(&self) -> u32 {
        self.inner.borrow().revision
    }

    /// 合并本 scope 的主题目录（同名覆盖仅影响本 scope）。
    pub fn register_trees(&self, trees: BTreeMap<String, ThemeTree>) {
        self.inner.borrow_mut().catalog.extend(trees);
    }

    /// 本 scope 目录内的树（看不见别的 scope/legacy 的同名条目）。
    pub fn lookup_tree(&self, name: &str) -> Option<ThemeTree> {
        self.inner.borrow().catalog.get(name).cloned()
    }

    /// 登记 design 块条目（theme/mode 选择器）。
    pub fn set_design_entries(&self, entries: Vec<(String, Expr)>) {
        self.inner.borrow_mut().design = entries;
    }

    /// 本 scope 的 design 块条目。
    pub fn design_entries(&self) -> Vec<(String, Expr)> {
        self.inner.borrow().design.clone()
    }

    /// 本 scope 的树族按模式重建的合成树（槽引用消费面）。
    pub fn active_tree(&self, is_dark: bool) -> Option<Tree> {
        let st = self.inner.borrow();
        match &st.family {
            ActiveSpec::Tree(tree) => theme_tree::compose_mode(tree, is_dark).ok(),
            _ => None,
        }
    }

    /// 本 scope 的活动语义色解析。
    pub fn resolve_rgb(&self, token: TokenName, is_dark: bool) -> Option<(u8, u8, u8)> {
        let st = self.inner.borrow();
        st.family.resolve(token, is_dark)
    }

    /// 本 scope 的活动合成主题（code_editor 等消费面）。
    pub fn composed(&self) -> Option<Arc<crate::design_tokens::decl::ComposedTheme>> {
        let st = self.inner.borrow();
        match &st.family {
            ActiveSpec::Composed(c) => Some(c.clone()),
            _ => None,
        }
    }

    /// 切换 family（内置/树/合成）；只在身份变化时推进 revision。
    pub fn set_family(&self, spec: ActiveSpec) -> bool {
        let mut st = self.inner.borrow_mut();
        if super::same_identity(&st.family, &spec) {
            return false;
        }
        st.family = spec;
        st.revision = st.revision.wrapping_add(1);
        true
    }

    /// 设置模式；只在变化时推进 revision。
    pub fn set_dark(&self, dark: bool) -> bool {
        let mut st = self.inner.borrow_mut();
        if st.dark == dark {
            return false;
        }
        st.dark = dark;
        st.revision = st.revision.wrapping_add(1);
        true
    }

    /// 完整候选一次校验/提交（R-02 语义，作用域版）：失败零写入，
    /// 合法 family+mode 只推进一个 revision。
    pub fn publish(
        &self,
        theme: Option<ThemeCandidate>,
        dark: Option<bool>,
    ) -> Result<bool, String> {
        let mut st = self.inner.borrow_mut();
        let next_dark = dark.unwrap_or(st.dark);
        // 1) 先校验（只读），非法组合不写任何槽位。
        let next_family = match theme {
            Some(ThemeCandidate::Tree(tree)) => {
                if !theme_tree::mode_available(&tree, next_dark) {
                    let want = if next_dark { "dark" } else { "light" };
                    return Err(format!(
                        "主题 `{}` 未声明 {want} 模式——拒绝换肤，保留旧有效设计（C-07）",
                        tree.name
                    ));
                }
                Some(ActiveSpec::Tree(tree))
            }
            Some(ThemeCandidate::Builtin(spec)) => Some(ActiveSpec::Builtin(spec)),
            None => None,
        };
        // 2) 再提交（family 与 mode 同一边界，revision 只 +1）。
        let mut changed = false;
        if let Some(next) = next_family {
            if !super::same_identity(&st.family, &next) {
                st.family = next;
                changed = true;
            }
        }
        if next_dark != st.dark {
            st.dark = next_dark;
            changed = true;
        }
        if changed {
            st.revision = st.revision.wrapping_add(1);
        }
        Ok(changed)
    }
}
