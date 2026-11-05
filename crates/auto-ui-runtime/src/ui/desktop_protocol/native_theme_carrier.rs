//! PLAN-055 T-03（auto-os 055 载体收口）：a2r 静态组件的主题策略采纳面。
//!
//! 053 的主题链（031 scope + eval 注册表 + `theme_capable` caps 门）全部
//! 挂在 [`crate::ui::dynamic::DynamicComponent`]；a2r 生成 exe 的静态
//! App 走 `Component` 缺省 = 恒不支持 → Hello 不声明 `theme-policy/1` →
//! 宿主零推（G-2 真载体死点）。本载体在 client 侧把静态组件包一层：
//! - 生成期嵌入的官方包身份/族表（`EmbeddedThemePackage`）；
//! - push 校验（family 注册 / version / digest / mode 声明，具名拒绝保旧
//!   ——校验失败零全局变化）；
//! - 应用 = `set_theme_tree` + `set_dark_mode`（render 全局单源，帧随
//!   后重建）；
//! - 读回 = 帧后置 applied（运行时泵 push_frame 先于 status 读回，
//!   view() 首帧翻转 pending——"帧评估后 applied"同语义）。
//!
//! stale/旧 session 分列、重连取最新快照都是宿主侧锚（`theme_pushed_rev`
//! + attach 三路推）——载体零参与；系统事件腿：mode=system 在应用时经
//! `system_prefers_dark()` 解析，真实 OS 变更经宿主 revision 广播重推
//! （每推重解析），无需载体自行订阅。

use crate::ui::component::Component;
use crate::ui::theme_policy::PolicySnapshot;
use crate::ui::View;

/// 生成期嵌入的官方主题包身份 + 族表 + themes.at 原文。
///
/// 由 a2r 生成器在 `dep themes` 准入（`verify_app_theme_inputs`）通过后
/// 发射：digest = 冻结内容聚合摘要（与宿主 catalog 同一聚合函数、同一
/// 内容 → 相等）；families 来自包清单（name, modes, default）。
#[derive(Debug)]
pub struct EmbeddedThemePackage {
    pub owner: &'static str,
    pub package: &'static str,
    pub version: &'static str,
    pub digest: &'static str,
    /// (family name, declared modes, default mode)
    pub families: &'static [(&'static str, &'static [&'static str], &'static str)],
    /// vendor/themes/themes.at 原文（生成期内联；启动期解析注册树）。
    pub themes_source: &'static str,
}

/// a2r 静态组件的主题策略载体（包装任意生成 App）。
#[derive(Debug)]
pub struct NativeThemeCarrier<C: Component> {
    inner: C,
    pkg: Option<&'static EmbeddedThemePackage>,
    /// 最近一次成功应用的推送 revision。
    applied_rev: u64,
    /// 已应用待首帧（运行时泵 push_frame → view() → 读回——首帧翻转）。
    /// Cell：view(&self) 内翻转——运行时单线程泵，无竞争。
    pending_frame: std::cell::Cell<bool>,
    /// 最近一次推送的读回 `(status, reason)`。
    last: std::cell::RefCell<Option<(String, String)>>,
}

impl<C: Component> NativeThemeCarrier<C> {
    /// 无主题包包装（行为与裸组件一致，恒不支持推送）。
    pub fn bare(inner: C) -> Self {
        Self {
            inner,
            pkg: None,
            applied_rev: 0,
            pending_frame: std::cell::Cell::new(false),
            last: std::cell::RefCell::new(None),
        }
    }

    /// 嵌入官方包：启动期解析 themes.at 注册主题树（与 VM 轨
    /// `collect_named_themes` + `register_theme_trees` 同一收集/注册面）。
    /// 解析/校验失败 = 具名 Err（生成 exe 启动退出留痕，不静默降级）。
    pub fn with_package(
        inner: C,
        pkg: &'static EmbeddedThemePackage,
    ) -> Result<Self, String> {
        let ast = {
            let session = auto_lang_core::session::CompilerSession::ui();
            let mut parser = auto_lang_core::Parser::from(pkg.themes_source)
                .with_session(session);
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .map(auto_ui_dialect::UiCode)
                .map_err(|e| format!("嵌入式主题包解析失败: {e:?}"))?
        };
        let trees = auto_ui_compiler::design_tokens::theme_tree::collect_named_themes(
            std::path::Path::new("vendor/themes"),
            pkg.themes_source,
        )
        .map_err(|e| format!("嵌入式主题包主题树收集失败: {e:?}"))?;
        if trees.is_empty() {
            return Err("嵌入式主题包未声明任何具名主题".to_string());
        }
        // 包清单声明的族必须有对应主题树（族/树一致门——清单撒谎具名失败）。
        for (name, _, _) in pkg.families {
            if !trees.contains_key(*name) {
                return Err(format!("包清单族 {name:?} 无对应主题树"));
            }
        }
        auto_ui_compiler::style::theme::register_theme_trees(trees);
        eprintln!(
            "[native-theme] 嵌入包注册 owner={} pkg={} v={} digest={}… families={}",
            pkg.owner,
            pkg.package,
            pkg.version,
            &pkg.digest[..pkg.digest.len().min(12)],
            pkg.families.len()
        );
        Ok(Self {
            inner,
            pkg: Some(pkg),
            applied_rev: 0,
            pending_frame: std::cell::Cell::new(false),
            last: std::cell::RefCell::new(None),
        })
    }

    fn reject(&self, reason: String) {
        *self.last.borrow_mut() = Some(("rejected".to_string(), reason));
    }
}

impl<C: Component> Component for NativeThemeCarrier<C> {
    type Msg = C::Msg;

    fn on(&mut self, msg: Self::Msg) {
        self.inner.on(msg);
    }

    fn view(&self) -> View<Self::Msg> {
        // 首帧翻转：运行时泵在 on_theme_policy 后 push_frame（render_frame
        // → view()）再读回状态——置 applied 时新全局已进帧（"帧后 applied"）。
        if self.pending_frame.replace(false) {
            *self.last.borrow_mut() = Some(("applied".to_string(), String::new()));
        }
        self.inner.view()
    }

    fn tick_msg(&self) -> Option<Self::Msg> {
        self.inner.tick_msg()
    }

    fn tick_interval_ms(&self) -> Option<u32> {
        self.inner.tick_interval_ms()
    }

    fn fire_due_timers(&mut self) -> bool {
        self.inner.fire_due_timers()
    }

    fn key_bindings(&self) -> std::collections::HashMap<String, String> {
        self.inner.key_bindings()
    }

    fn key_message(&self, key: &str) -> Option<Self::Msg> {
        self.inner.key_message(key)
    }

    fn drain_desktop_commands(&mut self) -> Vec<String> {
        self.inner.drain_desktop_commands()
    }

    fn drain_bitmap_uploads(&mut self) -> Vec<crate::ui::desktop_protocol::endpoint::BitmapUpload> {
        self.inner.drain_bitmap_uploads()
    }

    fn apply_state_snapshot(
        &mut self,
        revision: u64,
        fields: &[(String, auto_val::Value)],
    ) -> bool {
        self.inner.apply_state_snapshot(revision, fields)
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_val::Value> {
        self.inner.state_snapshot()
    }

    fn theme_capable(&self) -> bool {
        self.pkg.is_some()
    }

    fn on_theme_policy(
        &mut self,
        snap: &PolicySnapshot,
    ) -> Result<(), String> {
        eprintln!(
            "[native-theme] push rev={} family={} version={} digest={}… mode={}",
            snap.revision,
            snap.family,
            snap.version,
            &snap.digest[..snap.digest.len().min(12)],
            snap.mode
        );
        let Some(pkg) = &self.pkg else {
            return Err("unsupported: 该载体未嵌入主题包".to_string());
        };
        // 校验序（全部具名；任一失败 = 零全局变化保旧）：
        let declared = pkg.families.iter().find(|(n, _, _)| *n == snap.family);
        let Some((_, modes, _)) = declared else {
            self.reject(format!("family {} 未在嵌入包 {} 中声明", snap.family, pkg.package));
            return Ok(());
        };
        if snap.version != pkg.version {
            self.reject(format!(
                "包版本不符：推送 {} vs 嵌入 {}（拒绝保旧）",
                snap.version, pkg.version
            ));
            return Ok(());
        }
        if snap.digest != pkg.digest {
            self.reject(format!(
                "内容 digest 不符：推送 {}… vs 嵌入 {}…（拒绝保旧）",
                &snap.digest[..snap.digest.len().min(12)],
                &pkg.digest[..pkg.digest.len().min(12)],
            ));
            return Ok(());
        }
        if !modes.contains(&snap.mode.as_str()) {
            self.reject(format!(
                "mode {:?} 未被 family {:?} 声明（modes: {:?}）",
                snap.mode, snap.family, modes
            ));
            return Ok(());
        }
        // 应用：族树激活 + 模式全局（render 全局单源；THEME_EPOCH 失效
        // 回路随之走 view 重建）。system 按当前系统解析——真实 OS 变更
        // 经宿主 revision 重推，每推重解析。
        let Some(tree) = auto_ui_compiler::style::theme::lookup_theme_tree(&snap.family) else {
            self.reject(format!("family {} 主题树缺席（注册回退？）", snap.family));
            return Ok(());
        };
        let dark = match snap.mode.as_str() {
            "dark" => true,
            "light" => false,
            // 系统偏好缺席（查询失败/非 Windows）= 具名拒绝（不猜默认）。
            "system" => match crate::ui::system_theme::system_prefers_dark() {
                Some(d) => d,
                None => {
                    self.reject("system 偏好查询缺席（platform provider 无读数）".to_string());
                    return Ok(());
                }
            },
            other => {
                self.reject(format!("未知 theme_mode {other:?}"));
                return Ok(());
            }
        };
        if !auto_ui_compiler::style::theme::set_theme_tree(tree) {
            // 同名同树零变化（epoch 不动）——仍按成功应用记账。
            eprintln!(
                "[native-theme] family {} 树激活零变化（同值幂等）",
                snap.family
            );
        }
        auto_ui_compiler::style::theme::set_dark_mode(dark);
        self.applied_rev = snap.revision;
        self.pending_frame.set(true);
        *self.last.borrow_mut() = Some(("pending".to_string(), format!("rev {}", snap.revision)));
        Ok(())
    }

    fn theme_policy_status(&self) -> Option<(String, String)> {
        self.last.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct DummyMsg;
    #[derive(Debug)]
    struct Dummy;
    impl Component for Dummy {
        type Msg = DummyMsg;
        fn on(&mut self, _msg: Self::Msg) {}
        fn view(&self) -> View<Self::Msg> {
            View::Empty
        }
    }

    /// 业务状态保持载体（换肤不重置业务——PLAN-055 V-05 载体面语义）。
    #[derive(Debug)]
    struct Stateful {
        elapsed: u64,
    }
    impl Component for Stateful {
        type Msg = DummyMsg;
        fn on(&mut self, _msg: Self::Msg) {}
        fn view(&self) -> View<Self::Msg> {
            View::Empty
        }
    }

    #[test]
    fn carrier_push_preserves_inner_business_state() {
        let mut c = NativeThemeCarrier::with_package(Stateful { elapsed: 1500 }, pkg())
            .expect("package");
        // 推送 + 帧翻 applied——内部业务字段不得被换肤触碰/重置。
        c.on_theme_policy(&snap(2, "Auto", "0.2.0", "9fa07742dc68aa53", "dark"))
            .unwrap();
        let _ = c.view();
        assert_eq!(c.theme_policy_status().unwrap().0, "applied");
        assert_eq!(c.inner.elapsed, 1500, "换肤不重置业务状态");
        // 二次推送（mode 翻转）同样保持。
        c.on_theme_policy(&snap(3, "Auto", "0.2.0", "9fa07742dc68aa53", "light"))
            .unwrap();
        let _ = c.view();
        assert_eq!(c.inner.elapsed, 1500, "二次换肤仍不重置");
    }

    fn pkg() -> &'static EmbeddedThemePackage {
        // 泄漏静态（测试一次性）——EmbeddedThemePackage 需 'static 引用。
        Box::leak(Box::new(EmbeddedThemePackage {
            owner: "auto-ui",
            package: "themes",
            version: "0.2.0",
            digest: "9fa07742dc68aa53",
            families: &[("Auto", &["light", "dark"], "light")],
            themes_source: "pub theme Auto = {\n    tokens: {\n        colors: {\n            background: \"#ffffff\"\n            foreground: \"#09090b\"\n        }\n    }\n}\n",
        }))
    }

    fn snap(rev: u64, family: &str, version: &str, digest: &str, mode: &str) -> PolicySnapshot {
        PolicySnapshot {
            revision: rev,
            family: family.into(),
            version: version.into(),
            digest: digest.into(),
            mode: mode.into(),
        }
    }

    #[test]
    fn carrier_rejects_named_and_keeps_old() {
        let mut c = NativeThemeCarrier::with_package(Dummy, pkg()).expect("package");
        assert!(c.theme_capable());
        // 未知 family。
        c.on_theme_policy(&snap(1, "Solarized", "0.2.0", "9fa07742dc68aa53", "dark"))
            .unwrap();
        assert_eq!(
            c.theme_policy_status(),
            Some(("rejected".into(), "family Solarized 未在嵌入包 themes 中声明".into()))
        );
        // 版本不符。
        c.on_theme_policy(&snap(2, "Auto", "0.3.0", "9fa07742dc68aa53", "dark"))
            .unwrap();
        assert_eq!(
            c.theme_policy_status().unwrap().0,
            "rejected",
            "版本不符拒绝"
        );
        // digest 不符。
        c.on_theme_policy(&snap(3, "Auto", "0.2.0", "000000000000", "dark"))
            .unwrap();
        assert_eq!(
            c.theme_policy_status().unwrap().0,
            "rejected",
            "digest 不符拒绝"
        );
        // mode 未声明。
        c.on_theme_policy(&snap(4, "Auto", "0.2.0", "9fa07742dc68aa53", "sepia"))
            .unwrap();
        assert_eq!(
            c.theme_policy_status().unwrap().0,
            "rejected",
            "mode 未声明拒绝"
        );
    }

    #[test]
    fn carrier_applies_then_frame_flips_applied() {
        let mut c = NativeThemeCarrier::with_package(Dummy, pkg()).expect("package");
        assert_eq!(c.theme_policy_status(), None, "首推前无可读回");
        c.on_theme_policy(&snap(7, "Auto", "0.2.0", "9fa07742dc68aa53", "dark"))
            .unwrap();
        // 帧前 = pending；帧（view）后 = applied。
        assert_eq!(c.theme_policy_status().unwrap().0, "pending");
        let _ = c.view();
        assert_eq!(
            c.theme_policy_status(),
            Some(("applied".into(), String::new()))
        );
        assert_eq!(c.applied_rev, 7);
        // 裸包装恒不支持。
        let bare = NativeThemeCarrier::<Dummy>::bare(Dummy);
        assert!(!bare.theme_capable());
    }
}
