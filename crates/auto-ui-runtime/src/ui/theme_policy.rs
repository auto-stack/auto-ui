//! PLAN-053 T-02：宿主主题策略核心——family/mode/revision 的**校验-提交-通知
//! 单点**。bus 动词（`set_theme_policy`）、config 外写轮询、协议 attach 推送
//! 三条入口全部经本模块的 stage/publish，杜绝双路径语义漂移（P041 等价契约
//! 的策略层）。
//!
//! 分层边界（沿 031 合同）：目录（catalog）由宿主进程 boot 期经 auto-ui-tools
//! 校验 + digest 后注入（纯数据，runtime 零 crypto/零 manifest IO）；选择输
//! 出仍走 031 的 `HostThemeInputs`/`publish` 原子端口（失败保旧）；本模块只
//! 持有**宿主 policy**（持久化 + revision + 会话无关的真源）。
//!
//! 持久化：`DesktopConfig` 新 policy 组（`theme_family/_version/_digest`、
//! `theme_mode`、`theme_rev`）。提交序 = 完整校验 → 改写 cfg 候选字段（零
//! 变更/拒绝时 cfg 不动）→ 调用方 `desktop_config::save` → `publish_staged`
//! 发布内存 revision。落盘失败 = 不 publish（内存 policy 不变、无成功 ack）。

use crate::ui::desktop_config::DesktopConfig;
use std::collections::HashMap;
use std::sync::Mutex;

/// 宿主可见的官方主题包目录（宿主 boot 注入；None = 包缺席/校验失败，
/// family 链整体不可用，候选具名拒绝）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThemePackageCatalog {
    pub owner: String,
    pub package: String,
    pub version: String,
    /// 全文件内容聚合 digest（`theme_package_content_digest` 同算法）。
    pub digest: String,
    pub families: Vec<ThemeFamilyInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeFamilyInfo {
    pub name: String,
    /// 该 family 声明支持的 mode（`light`/`dark`）。
    pub modes: Vec<String>,
    pub default: String,
}

impl ThemePackageCatalog {
    pub fn family(&self, name: &str) -> Option<&ThemeFamilyInfo> {
        self.families.iter().find(|f| f.name == name)
    }
}

/// 已提交的宿主策略（内存镜像 + 恢复/推送的真源）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostPolicy {
    pub family: String,
    pub version: String,
    pub digest: String,
    /// 声明值 `light`|`dark`|`system`（运行派生值不落盘不进本结构）。
    pub mode: String,
    pub rev: u64,
}

impl HostPolicy {
    /// mode → 派生 dark 布尔（`system` 经受控 provider；provider 缺席 =
    /// None = 本帧不发布 mode，沿 031 语义）。
    pub fn resolved_dark(&self) -> Option<bool> {
        resolved_dark(&self.mode)
    }

    /// 推送/会话面快照形态（协议帧、client 端口共用）。
    pub fn snapshot(&self) -> PolicySnapshot {
        PolicySnapshot {
            revision: self.rev,
            family: self.family.clone(),
            version: self.version.clone(),
            digest: self.digest.clone(),
            mode: self.mode.clone(),
        }
    }
}

/// 一份完整主题策略快照（宿主→App 推送载荷的语义形态；协议帧 =
/// [`crate::ui::desktop_protocol::message::ControlMsg::ThemePolicyPush`]，
/// 独立进程 config 轮询臂复用本结构喂 031 端口）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySnapshot {
    pub revision: u64,
    pub family: String,
    pub version: String,
    pub digest: String,
    pub mode: String,
}

impl PolicySnapshot {
    /// 031 端口输入构造：follow 宿主的 App 用（App override 不在宿主 v1 面）。
    pub fn host_inputs(&self) -> auto_ui_compiler::style::theme::HostThemeInputs {
        use auto_ui_compiler::style::theme::{HostThemeInputs, ModeInput, ThemeInput};
        HostThemeInputs {
            app_override: ThemeInput::Unset,
            app_override_mode: ModeInput::Unset,
            follow_host: true,
            host_family: ThemeInput::Value(self.family.clone()),
            host_mode: match self.mode.as_str() {
                "light" => ModeInput::Light,
                "dark" => ModeInput::Dark,
                "system" => ModeInput::System,
                other => ModeInput::Invalid(format!("未知 theme_mode {other:?}")),
            },
        }
    }

    /// 协议帧 record 编码：`rev\x1Ffamily\x1Fversion\x1Fdigest\x1Fmode`
    /// （FIELD_SEP 与 DesktopCommand 同族；五段定长——坏段数/空 family =
    /// None，解析端不猜）。
    pub fn encode_record(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            self.revision,
            '\u{1F}',
            self.family,
            '\u{1F}',
            self.version,
            '\u{1F}',
            self.digest,
            '\u{1F}',
        ) + &self.mode
    }

    /// [`PolicySnapshot::encode_record`] 对偶解析。
    pub fn parse_record(record: &str) -> Option<PolicySnapshot> {
        let mut it = record.split('\u{1F}');
        let (Some(revision), Some(family), Some(version), Some(digest), Some(mode), None) = (
            it.next(),
            it.next(),
            it.next(),
            it.next(),
            it.next(),
            it.next(),
        ) else {
            return None;
        };
        let revision = revision.trim().parse::<u64>().ok()?;
        if family.is_empty() || version.is_empty() || digest.is_empty() {
            return None;
        }
        if !matches!(mode.trim(), "light" | "dark" | "system") {
            return None;
        }
        Some(PolicySnapshot {
            revision,
            family: family.to_string(),
            version: version.to_string(),
            digest: digest.to_string(),
            mode: mode.trim().to_string(),
        })
    }
}

/// `light`/`dark`/`system` → 派生 dark（system 经 `system_prefers_dark`）。
pub fn resolved_dark(mode: &str) -> Option<bool> {
    match mode.trim() {
        "light" => Some(false),
        "dark" => Some(true),
        "system" => crate::ui::system_theme::system_prefers_dark(),
        _ => None,
    }
}

/// 单次策略应用结果（调用方据此决定通知面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyOutcome {
    /// 已提交（cfg 候选字段已改写、调用方落盘后 `publish_staged`）。
    Staged(StagedPolicy),
    /// 与当前已提交策略全等（或 family 链未选）——零变更零落盘，杀写环。
    Unchanged,
    /// 具名拒绝——cfg 零改动、内存 policy 不动。
    Rejected(String),
}

/// 校验通过、待落盘的候选（rev 已分配）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedPolicy {
    pub policy: HostPolicy,
}

struct PolicyState {
    catalog: Option<ThemePackageCatalog>,
    policy: Option<HostPolicy>,
}

static STATE: Mutex<PolicyState> = Mutex::new(PolicyState {
    catalog: None,
    policy: None,
});

fn with_state<R>(f: impl FnOnce(&mut PolicyState) -> R) -> R {
    let mut st = STATE.lock().unwrap();
    f(&mut st)
}

/// 宿主 boot 注入目录（None = 包缺席——family 链降级为具名拒绝，legacy 链不受影响）。
pub fn init_catalog(catalog: Option<ThemePackageCatalog>) {
    with_state(|st| st.catalog = catalog);
}

pub fn catalog() -> Option<ThemePackageCatalog> {
    with_state(|st| st.catalog.clone())
}

/// 当前已提交策略（None = family 链未激活）。
pub fn current() -> Option<HostPolicy> {
    with_state(|st| st.policy.clone())
}

/// boot 恢复：从落盘 config 恢复内存策略（宿主重启/崩溃后从磁盘完整快照
/// 恢复并重发语义的入口）。校验失败（含包缺席）→ 保持 None + 具名诊断，
/// 不降级使用旧值。
pub fn boot_from_config(cfg: &DesktopConfig) -> Result<(), String> {
    let Some(family) = cfg.theme_family.clone() else {
        return Ok(());
    };
    let verdict = validate_fields(
        Some(&family),
        cfg.theme_family_version.as_deref(),
        cfg.theme_family_digest.as_deref(),
        cfg.theme_mode.as_deref(),
    );
    let policy = match verdict {
        Ok(p) => p,
        Err(reason) => {
            eprintln!("[PLAN-053] boot 恢复拒绝：{reason}（保 legacy 链，family 候选需重新选择）");
            return Err(reason);
        }
    };
    let policy = HostPolicy {
        rev: cfg.theme_rev,
        ..policy
    };
    with_state(|st| st.policy = Some(policy));
    // boot 恢复成功 → shell 状态面首推（ServiceTick 消费）。
    mark_shell_dirty();
    Ok(())
}

/// 单点校验：cfg 的候选四元组 → 完整 `HostPolicy`（rev 由 stage/publish 分配）。
/// family 键缺席 = 非候选（Ok(None) 语义由调用方给）。错误全具名：
/// 包缺席 / family 未知 / version 或 digest 不符 / mode 未声明 / mode 缺失坏值。
pub fn validate_fields(
    family: Option<&str>,
    version: Option<&str>,
    digest: Option<&str>,
    mode: Option<&str>,
) -> Result<HostPolicy, String> {
    let Some(family) = family.map(str::trim).filter(|f| !f.is_empty()) else {
        return Err("family 未选择（theme_family 缺席）".to_string());
    };
    let cat = with_state(|st| st.catalog.clone());
    let Some(cat) = cat else {
        return Err("主题包目录不可用（包缺席或校验失败）——候选拒绝".to_string());
    };
    let Some(info) = cat.family(family) else {
        return Err(format!("未知主题 family {family:?}（目录含 {} 个 family）", cat.families.len()));
    };
    let version = version.map(str::trim).filter(|v| !v.is_empty());
    if version != Some(cat.version.as_str()) {
        return Err(format!(
            "包版本不符：候选 {:?} ≠ 目录 {}",
            version.unwrap_or("(缺席)"),
            cat.version
        ));
    }
    let digest = digest.map(str::trim).filter(|d| !d.is_empty());
    if digest != Some(cat.digest.as_str()) {
        return Err("内容 digest 不符（候选与目录快照不一致）——候选拒绝".to_string());
    }
    let Some(mode) = mode.map(str::trim).filter(|m| !m.is_empty()) else {
        return Err(format!("mode 缺失/坏值（family {family:?} 支持 {:?}）", info.modes));
    };
    if !info.modes.iter().any(|m| m == mode) {
        return Err(format!("mode {mode:?} 未被 family {family:?} 声明（支持 {:?}）", info.modes));
    }
    Ok(HostPolicy {
        family: family.to_string(),
        version: cat.version.clone(),
        digest: cat.digest.clone(),
        mode: mode.to_string(),
        rev: 0,
    })
}

/// **唯一提交入口**（bus 动词 / config 外写轮询 / 协议 attach 三路共用）：
/// 校验 cfg 候选 → 通过则把候选字段 + 新 revision + legacy 兼容字段改写进
/// cfg（调用方随后落盘）；全等或未选 → `Unchanged`（零改动）；拒绝 → 具名
/// reason 且 cfg 零改动。**发布内存 revision 必须在落盘成功后调
/// [`publish_staged`]**（写失败 = 内存 policy 不变，沿 §5.2 提交序）。
pub fn stage_candidate(cfg: &mut DesktopConfig) -> PolicyOutcome {
    // family 链未选：零变更（v1 不提供 un-select——shell 选择恒带 family）。
    if cfg.theme_family.is_none() {
        return PolicyOutcome::Unchanged;
    }
    let verdict = validate_fields(
        cfg.theme_family.as_deref(),
        cfg.theme_family_version.as_deref(),
        cfg.theme_family_digest.as_deref(),
        cfg.theme_mode.as_deref(),
    );
    let candidate = match verdict {
        Ok(p) => p,
        Err(reason) => return PolicyOutcome::Rejected(reason),
    };
    // 全等判定**不含 rev**（新候选 rev 恒 0；语义同一 = family/version/digest/mode）。
    let (unchanged, last_rev) = with_state(|st| {
        (
            st.policy.as_ref().map(|p| {
                (&p.family, &p.version, &p.digest, &p.mode)
            }) == Some((
                &candidate.family,
                &candidate.version,
                &candidate.digest,
                &candidate.mode,
            )),
            st.policy.as_ref().map(|p| p.rev).unwrap_or(0),
        )
    });
    if unchanged {
        // 盘面值与内存一致时仍对齐 rev（外部写方回填旧 rev 的收敛）。
        cfg.theme_rev = candidate.rev;
        return PolicyOutcome::Unchanged;
    }
    let rev = last_rev.wrapping_add(1);
    let staged = StagedPolicy {
        policy: HostPolicy { rev, ..candidate },
    };
    // 候选字段 + 宿主 revision 落 cfg（legacy 兼容映射同提交：mode→dark_theme，
    // theme_source=manual 终结 OS 派生覆盖；system 恢复 system 跟随）。
    cfg.theme_family = Some(staged.policy.family.clone());
    cfg.theme_family_version = Some(staged.policy.version.clone());
    cfg.theme_family_digest = Some(staged.policy.digest.clone());
    cfg.theme_mode = Some(staged.policy.mode.clone());
    cfg.theme_rev = rev;
    match staged.policy.mode.as_str() {
        "dark" => {
            cfg.dark_theme = true;
            cfg.theme_source = "manual".to_string();
        }
        "light" => {
            cfg.dark_theme = false;
            cfg.theme_source = "manual".to_string();
        }
        _ => {
            cfg.theme_source = "system".to_string();
        }
    }
    PolicyOutcome::Staged(staged)
}

/// 落盘成功后发布内存 revision + 镜像（返回 rev 供通知面用）。落盘失败
/// **不得**调用（调用方丢弃 stage 出的 cfg 副本即可回滚）。
pub fn publish_staged(staged: StagedPolicy) -> u64 {
    with_state(|st| st.policy = Some(staged.policy.clone()));
    staged.policy.rev
}

// ---------------------------------------------------------------------------
// shell 状态面刷新脏位（session/协议侧非 iced 代码置位，iced ServiceTick
// 消费——无 iced 依赖倒挂）。
// ---------------------------------------------------------------------------

static SHELL_DIRTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 策略/结果面变化（outproc 结果帧等）→ 请求 shell 状态面刷新。
pub fn mark_shell_dirty() {
    SHELL_DIRTY.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// ServiceTick 消费（取走即清）。
pub fn take_shell_dirty() -> bool {
    SHELL_DIRTY.swap(false, std::sync::atomic::Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// 目录发现装载（宿主 boot 与 standalone 跟随武装共用单一实现——解析序
// env `AUTOOS_THEME_PACKAGE_DIR` → env `AUTO_UI_ROOT/packages/themes` →
// 兄弟 `<组根>/auto-ui/packages/themes`（编译期锚 ancestors，组树/主检出
// 双形态命中）；全缺席 = None 注入（family 链具名拒绝降级，legacy 链
// 不受影响）。校验/digest 全走 compiler 包权威模块，无镜像实现。
// ---------------------------------------------------------------------------

/// 目录发现 + 校验 + 目录注入。返回人类可读诊断行（启动日志用）；包缺席
/// 时注入 None 并返回降级诊断。
pub fn init_catalog_from_discovery() -> String {
    use auto_ui_compiler::design_tokens::theme_package::{
        theme_package_content_digest, verify_theme_package, ThemePackageExpectation,
    };
    const OWNER: &str = "auto-ui";
    const PACKAGE: &str = "themes";
    const VERSION: &str = "0.2.0";
    const SCHEMA: u64 = 1;

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = std::env::var("AUTOOS_THEME_PACKAGE_DIR") {
        if !dir.trim().is_empty() {
            candidates.push(std::path::PathBuf::from(dir));
        }
    }
    if let Ok(root) = std::env::var("AUTO_UI_ROOT") {
        if !root.trim().is_empty() {
            candidates.push(std::path::PathBuf::from(root).join("packages").join("themes"));
        }
    }
    // 组兄弟（编译期锚 ancestors 第 4 层 = 组根——crates/<crate> → crates →
    // <repo> → <组根>；worktree/主检出同律，PLAN-002 shell pack 同族先例）。
    if let Some(group) = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .map(|p| p.to_path_buf())
    {
        candidates.push(group.join("auto-ui").join("packages").join("themes"));
    }
    for dir in candidates {
        let expect = ThemePackageExpectation {
            owner: OWNER,
            package: PACKAGE,
            version: VERSION,
            schema: SCHEMA,
        };
        let (verified, digest) = match (
            verify_theme_package(&dir, &expect),
            theme_package_content_digest(&dir),
        ) {
            (Ok(v), Ok(d)) => (v, d),
            _ => continue,
        };
        // family 缺省 mode 从 manifest 读（校验后的能力表只带 modes）。
        let manifest_families = std::fs::read_to_string(dir.join("theme-package.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|v| v.get("families").and_then(|f| f.as_array()).cloned())
            .unwrap_or_default();
        let families: Vec<ThemeFamilyInfo> = manifest_families
            .iter()
            .filter_map(|f| {
                let name = f.get("name")?.as_str()?.to_string();
                let modes = verified.families.get(&name)?;
                Some(ThemeFamilyInfo {
                    name,
                    modes: modes.clone(),
                    default: f
                        .get("default")
                        .and_then(|d| d.as_str())
                        .unwrap_or("light")
                        .to_string(),
                })
            })
            .collect();
        let diag = format!(
            "theme package: {} v{} families={} digest={}..{} root={}",
            verified.package,
            verified.version,
            families.len(),
            &digest[..8],
            &digest[digest.len() - 8..],
            dir.display()
        );
        init_catalog(Some(ThemePackageCatalog {
            owner: verified.owner,
            package: verified.package,
            version: verified.version,
            digest,
            families,
        }));
        return diag;
    }
    init_catalog(None);
    "theme package 未找到/校验失败（AUTOOS_THEME_PACKAGE_DIR/AUTO_UI_ROOT/兄弟 auto-ui 均未命中）——family 链降级"
        .to_string()
}

// ---------------------------------------------------------------------------
// standalone（`auto run -r vm`）跟随腿（PLAN-053 T-04；§5.1 冻结裁定：
// 独立进程复用 config 轮询通道，不新增传输）。`AUTOOS_DESKTOP_CONFIG`
// env 在场 = 显式跟随宿主桌面配置——武装位拉起 400ms ServiceTick 订阅，
// tick 内 poll_external_config 的策略组差异臂（与宿主同一 stage/commit
// 入口）喂本进程组件的 031 端口。env 缺席 = 零开销零行为差。
// ---------------------------------------------------------------------------

static STANDALONE_FOLLOW: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// standalone 跟随武装（run_file_dynamic_ui_inner 入口调用；幂等）。
pub fn arm_standalone_follow() {
    let armed = std::env::var_os("AUTOOS_DESKTOP_CONFIG")
        .map(|p| !p.is_empty())
        .unwrap_or(false);
    if !armed {
        return;
    }
    let diag = init_catalog_from_discovery();
    // eprintln!("[PLAN-053] standalone follow armed: {diag}");
    // 落盘策略恢复（宿主重启/崩溃后从磁盘完整快照恢复语义同源）。
    let cfg = crate::ui::desktop_config::load();
    if let Err(reason) = boot_from_config(&cfg) {
        eprintln!("[PLAN-053] standalone boot 恢复拒绝：{reason}");
    }
    STANDALONE_FOLLOW.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// ServiceTick 订阅门（standalone 且已武装才拉 400ms 拍）。
pub fn standalone_follow_armed() -> bool {
    STANDALONE_FOLLOW.load(std::sync::atomic::Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// App 级评估结果登记（in-proc/独立进程共用的 applied/rejected 读回面）。
// eval_app_design 在整帧发布/拒绝路径各记一笔；宿主状态面据此把每 App
// pending/applied/rejected/unsupported 分列（§5.3），不把发送成功当渲染成功。
// ---------------------------------------------------------------------------

/// scope_id → (是否整帧提交, 具名 detail, 提交时 revision)。
static LAST_EVAL: std::sync::OnceLock<Mutex<HashMap<u64, (bool, String, u64)>>> =
    std::sync::OnceLock::new();

fn eval_map() -> &'static Mutex<HashMap<u64, (bool, String, u64)>> {
    LAST_EVAL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// eval_app_design 的宿主策略评估结果登记（UI 侧唯一记录点）。
pub fn note_scope_eval(scope_id: u64, committed: bool, detail: &str) {
    let rev = with_state(|st| st.policy.as_ref().map(|p| p.rev).unwrap_or(0));
    eval_map()
        .lock()
        .unwrap()
        .insert(scope_id, (committed, detail.to_string(), rev));
}

/// 读回某 scope 最近一次评估结果。
pub fn last_scope_eval(scope_id: u64) -> Option<(bool, String)> {
    eval_map()
        .lock()
        .unwrap()
        .get(&scope_id)
        .map(|(c, d, _)| (*c, d.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> ThemePackageCatalog {
        ThemePackageCatalog {
            owner: "auto-ui".into(),
            package: "themes".into(),
            version: "0.2.0".into(),
            digest: "a".repeat(64),
            families: vec![
                ThemeFamilyInfo {
                    name: "Auto".into(),
                    modes: vec!["light".into(), "dark".into()],
                    default: "light".into(),
                },
                ThemeFamilyInfo {
                    name: "Solarized".into(),
                    modes: vec!["light".into(), "dark".into()],
                    default: "light".into(),
                },
            ],
        }
    }

    /// 串行化守卫：本模块 static 为进程级，用例间互斥（同盘 config 无关——
    /// stage/publish 纯内存）。
    fn lock() -> std::sync::MutexGuard<'static, ()> {
        static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn candidate(family: &str, mode: &str) -> DesktopConfig {
        let mut cfg = DesktopConfig::default();
        cfg.theme_family = Some(family.into());
        cfg.theme_family_version = Some("0.2.0".into());
        cfg.theme_family_digest = Some("a".repeat(64));
        cfg.theme_mode = Some(mode.into());
        cfg
    }

    #[test]
    fn t01_no_catalog_rejects_named() {
        let _g = lock();
        init_catalog(None);
        with_state(|st| st.policy = None);
        let mut cfg = candidate("Auto", "dark");
        match stage_candidate(&mut cfg) {
            PolicyOutcome::Rejected(r) => {
                assert!(r.contains("目录不可用"), "reason={r}");
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
        // 零改动：候选字段保持调用方原值（此处即未选转态不发生）。
        assert_eq!(cfg.theme_rev, 0);
    }

    #[test]
    fn t02_unknown_family_and_bad_digest_reject_without_mutation() {
        let _g = lock();
        init_catalog(Some(catalog()));
        with_state(|st| st.policy = None);
        let mut cfg = candidate("Nope", "dark");
        match stage_candidate(&mut cfg) {
            PolicyOutcome::Rejected(r) => assert!(r.contains("未知主题 family"), "reason={r}"),
            other => panic!("expected Rejected, got {other:?}"),
        }
        let mut cfg = candidate("Auto", "dark");
        cfg.theme_family_digest = Some("b".repeat(64));
        match stage_candidate(&mut cfg) {
            PolicyOutcome::Rejected(r) => assert!(r.contains("digest 不符"), "reason={r}"),
            other => panic!("expected Rejected, got {other:?}"),
        }
        // 拒绝零写入：策略字段未被宿主改写（digest 仍是调用方值、rev 不变）。
        assert_eq!(cfg.theme_family_digest, Some("b".repeat(64)));
        assert_eq!(cfg.theme_rev, 0);
    }

    #[test]
    fn t03_undeclared_mode_rejects() {
        let _g = lock();
        init_catalog(Some(catalog()));
        with_state(|st| st.policy = None);
        let mut cfg = candidate("Auto", "sepia");
        assert!(matches!(stage_candidate(&mut cfg), PolicyOutcome::Rejected(_)));
        let mut cfg = candidate("Auto", "system");
        match stage_candidate(&mut cfg) {
            PolicyOutcome::Rejected(r) => assert!(r.contains("未被 family"), "reason={r}"),
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    #[test]
    fn t04_valid_candidate_stages_rev_and_legacy_compat_then_publishes() {
        let _g = lock();
        init_catalog(Some(catalog()));
        with_state(|st| st.policy = None);
        let mut cfg = candidate("Auto", "dark");
        let staged = match stage_candidate(&mut cfg) {
            PolicyOutcome::Staged(s) => s,
            other => panic!("expected Staged, got {other:?}"),
        };
        assert_eq!(staged.policy.rev, 1);
        assert_eq!(cfg.theme_rev, 1);
        // legacy 兼容映射：dark → dark_theme=true + theme_source=manual。
        assert!(cfg.dark_theme);
        assert_eq!(cfg.theme_source, "manual");
        let rev = publish_staged(staged);
        assert_eq!(rev, 1);
        assert_eq!(
            current(),
            Some(HostPolicy {
                family: "Auto".into(),
                version: "0.2.0".into(),
                digest: "a".repeat(64),
                mode: "dark".into(),
                rev: 1,
            })
        );
        // 切 mode → rev +2；切 family → +3。
        let mut cfg = candidate("Auto", "light");
        let staged = match stage_candidate(&mut cfg) {
            PolicyOutcome::Staged(s) => s,
            other => panic!("expected Staged, got {other:?}"),
        };
        assert_eq!(staged.policy.rev, 2);
        assert!(!cfg.dark_theme, "light 映射 dark_theme=false");
        publish_staged(staged);
        let mut cfg = candidate("Solarized", "dark");
        let staged = match stage_candidate(&mut cfg) {
            PolicyOutcome::Staged(s) => s,
            other => panic!("expected Staged, got {other:?}"),
        };
        assert_eq!(staged.policy.rev, 3);
        publish_staged(staged);
    }

    #[test]
    fn t05_identical_candidate_is_unchanged_no_rev_bump() {
        let _g = lock();
        init_catalog(Some(catalog()));
        with_state(|st| st.policy = None);
        let mut cfg = candidate("Auto", "dark");
        let staged = match stage_candidate(&mut cfg) {
            PolicyOutcome::Staged(s) => s,
            other => panic!("expected Staged, got {other:?}"),
        };
        publish_staged(staged);
        // 同候选重复提交（含外部写回盘面读回）→ Unchanged，rev 不动。
        let mut cfg = candidate("Auto", "dark");
        match stage_candidate(&mut cfg) {
            PolicyOutcome::Unchanged => {}
            other => panic!("expected Unchanged, got {other:?}"),
        }
        assert_eq!(current().unwrap().rev, 1);
        // family 未选 → Unchanged（v1 无 un-select）。
        let mut cfg = DesktopConfig::default();
        assert!(matches!(stage_candidate(&mut cfg), PolicyOutcome::Unchanged));
    }

    #[test]
    fn t06_boot_recovers_from_config_and_rejects_degraded_catalog() {
        let _g = lock();
        init_catalog(Some(catalog()));
        with_state(|st| st.policy = None);
        let cfg = candidate("Auto", "dark");
        boot_from_config(&cfg).unwrap();
        assert_eq!(current().unwrap().mode, "dark");
        // 坏盘面（mode 未声明）→ boot 具名拒绝保 None。
        with_state(|st| st.policy = None);
        let bad = candidate("Auto", "system");
        assert!(boot_from_config(&bad)
            .unwrap_err()
            .contains("未被 family"));
        assert!(current().is_none());
        // 包缺席（宿主降级）→ boot 拒绝保 None + 具名。
        init_catalog(None);
        assert!(boot_from_config(&cfg).is_err());
        assert!(current().is_none());
    }

    #[test]
    fn t07_config_roundtrip_keeps_policy_group() {
        let _g = lock();
        init_catalog(Some(catalog()));
        with_state(|st| st.policy = None);
        let mut cfg = candidate("Solarized", "dark");
        let staged = match stage_candidate(&mut cfg) {
            PolicyOutcome::Staged(s) => s,
            other => panic!("expected Staged, got {other:?}"),
        };
        publish_staged(staged);
        let text = crate::ui::desktop_config::serialize_config(&cfg);
        assert!(text.contains("theme_family : \"Solarized\""));
        assert!(text.contains("theme_rev : 1"));
        let parsed = crate::ui::desktop_config::parse_config(&text);
        assert_eq!(parsed.theme_family.as_deref(), Some("Solarized"));
        assert_eq!(parsed.theme_mode.as_deref(), Some("dark"));
        assert_eq!(parsed.theme_rev, 1);
        // 链未选的序列化不含策略组（升级前文件面一致）。
        let plain = crate::ui::desktop_config::serialize_config(&DesktopConfig::default());
        assert!(!plain.contains("theme_family"));
    }

    #[test]
    fn t08_eval_registry_roundtrip() {
        let _g = lock();
        note_scope_eval(42, true, "");
        assert_eq!(last_scope_eval(42), Some((true, String::new())));
        note_scope_eval(42, false, "mode 未声明");
        assert_eq!(last_scope_eval(42), Some((false, "mode 未声明".into())));
        assert_eq!(last_scope_eval(7), None);
    }
}
