//! PLAN-112 T-02 第 5 项：VM 宿主结束进程授权门（`auto.sys.kill` 2431）。
//!
//! 合并 VM 路线（`auto run -r vm`）把前端与 `back.api` 装进同一个 VM，
//! `kill_process` 直达 `shim_sys_kill`，不经 `back_proxy` 的 HTTP 会话代理。
//! 破坏性操作的具名拒绝路径因此挂在本模块：runner 在进入解释器前按宿主
//! 策略注册进程默认 gate（core `vm::host_capabilities::SysKillGate`），
//! VM 构造期快照进实例。
//!
//! 策略语义与 `back_proxy::SysKillPolicy` 同源（同三态、同稳定码
//! `sys.kill.permission_denied`）；两个门分属两条调用链，互不替代。
//!
//! 宿主入口（CLI 面）：`AUTO_UI_VM_SYS_KILL_POLICY`
//! - 未设置 / 空 / `allow-all`：不登记（= 迁移前无门行为逐字不变）；
//! - `deny` / `deny:<reason>`：一律具名拒；
//! - `allowed-pids:<pid,…>`：清单内放行、其余具名拒（空清单/非法 pid 具名失败）。

use std::collections::BTreeSet;
use std::sync::Arc;

use auto_lang_core::vm::host_capabilities::{SysKillDecision, SysKillGate};

use crate::back_proxy::SysKillPolicy;

/// 宿主结束进程策略的 CLI 环境变量。
pub const VM_SYS_KILL_POLICY_ENV: &str = "AUTO_UI_VM_SYS_KILL_POLICY";

/// `deny` 无显式 reason 时的默认原因（仍随稳定码一起具名）。
pub const VM_SYS_KILL_DEFAULT_REASON: &str =
    "the VM host holds no process-kill authorization";

/// `SysKillPolicy` → core 授权门适配。策略持有 owned 值，不借用调用方。
pub struct PolicyGate {
    policy: SysKillPolicy,
}

impl PolicyGate {
    pub fn new(policy: SysKillPolicy) -> Self {
        Self { policy }
    }
}

impl SysKillGate for PolicyGate {
    fn decision(&self, pid: i32) -> SysKillDecision {
        match &self.policy {
            SysKillPolicy::AllowAll => SysKillDecision::Allow,
            SysKillPolicy::Deny { reason } => SysKillDecision::Deny {
                reason: reason.clone(),
            },
            SysKillPolicy::AllowedPids(allowed) => {
                if allowed.contains(&pid) {
                    SysKillDecision::Allow
                } else {
                    SysKillDecision::Deny {
                        reason: format!("pid {pid} is not in the host authorization list"),
                    }
                }
            }
        }
    }
}

/// 按策略建 gate（`AllowAll` 也建；调用方决定是否登记）。
pub fn gate_from_policy(policy: SysKillPolicy) -> Arc<dyn SysKillGate> {
    Arc::new(PolicyGate::new(policy))
}

/// 解析环境变量值。`None`/空/`allow-all` → `AllowAll`（= 无门）。
pub fn parse_policy(raw: Option<&str>) -> Result<SysKillPolicy, String> {
    let Some(text) = raw.map(str::trim).filter(|text| !text.is_empty()) else {
        return Ok(SysKillPolicy::AllowAll);
    };
    if text.eq_ignore_ascii_case("allow-all") {
        return Ok(SysKillPolicy::AllowAll);
    }
    if text.eq_ignore_ascii_case("deny") {
        return Ok(SysKillPolicy::Deny {
            reason: VM_SYS_KILL_DEFAULT_REASON.to_string(),
        });
    }
    if let Some(reason) = text.strip_prefix("deny:") {
        let reason = reason.trim();
        if reason.is_empty() {
            return Err(format!(
                "invalid {VM_SYS_KILL_POLICY_ENV}='{text}': deny requires a non-empty reason"
            ));
        }
        return Ok(SysKillPolicy::Deny {
            reason: reason.to_string(),
        });
    }
    if let Some(list) = text.strip_prefix("allowed-pids:") {
        if list.trim().is_empty() {
            return Err(format!(
                "invalid {VM_SYS_KILL_POLICY_ENV}='{text}': allowed-pids requires at least one pid"
            ));
        }
        let mut allowed = BTreeSet::new();
        for token in list.split(',').map(str::trim) {
            if token.is_empty() {
                return Err(format!(
                    "invalid {VM_SYS_KILL_POLICY_ENV}='{text}': empty pid entry"
                ));
            }
            let pid: i32 = token.parse().map_err(|_| {
                format!("invalid {VM_SYS_KILL_POLICY_ENV}='{text}': pid '{token}' is not an integer")
            })?;
            if pid <= 0 {
                return Err(format!(
                    "invalid {VM_SYS_KILL_POLICY_ENV}='{text}': pid '{token}' must be positive"
                ));
            }
            allowed.insert(pid);
        }
        if allowed.is_empty() {
            return Err(format!(
                "invalid {VM_SYS_KILL_POLICY_ENV}='{text}': allowed-pids requires at least one pid"
            ));
        }
        return Ok(SysKillPolicy::AllowedPids(allowed));
    }
    Err(format!(
        "invalid {VM_SYS_KILL_POLICY_ENV}='{text}': expected allow-all | deny[:<reason>] | allowed-pids:<pid,...>"
    ))
}

/// 按环境变量登记/清除进程默认 gate。返回 Err 时调用方应中止启动
/// （非法策略不得静默退回无门）。
pub fn install_vm_sys_kill_gate_from_env() -> Result<(), String> {
    let raw = std::env::var(VM_SYS_KILL_POLICY_ENV).ok();
    let policy = parse_policy(raw.as_deref())?;
    match policy {
        SysKillPolicy::AllowAll => {
            auto_lang_core::vm::host_capabilities::clear_process_default_sys_kill_gate();
        }
        policy => {
            auto_lang_core::vm::host_capabilities::set_process_default_sys_kill_gate(
                gate_from_policy(policy),
            );
        }
    }
    Ok(())
}

/// 只读查询：当前进程默认是否登记了门（诊断/测试用）。
pub fn process_default_gate_installed() -> bool {
    auto_lang_core::vm::host_capabilities::process_default_sys_kill_gate().is_some()
}
