//! PLAN-112 T-02 第 5 项：VM 宿主结束进程授权门（auto-ui 宿主面）。
//!
//! 判据：
//! - `AUTO_UI_VM_SYS_KILL_POLICY` 三态解析；非法值具名失败（不静默退回无门）；
//! - 登记走 core `SysKillGate` 进程默认槽，`allow-all`/未设置时清除；
//! - 经真实 VM 执行 `sys.kill`：deny 具名拒且目标存活、allowlist 内外分流、
//!   allow 真实结束；稳定码与 back_proxy 门同一字面量。
//!
//! 目标进程一律测试自建单进程（`ping`/`sleep`，不经 shell 包一层）。

use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, MutexGuard};

use auto_ui_runtime::back_proxy::SysKillPolicy;
use auto_ui_runtime::vm_sys_kill::{
    install_vm_sys_kill_gate_from_env, parse_policy, process_default_gate_installed,
    VM_SYS_KILL_POLICY_ENV,
};

static ENV: Mutex<()> = Mutex::new(());

fn env_lock() -> MutexGuard<'static, ()> {
    ENV.lock().unwrap_or_else(|e| e.into_inner())
}

struct RestoreEnv {
    previous: Option<String>,
}

impl RestoreEnv {
    /// 在持锁状态下设置 env 并登记清理（Drop 恢复原值/清除）。
    fn set(value: Option<&str>) -> Self {
        let previous = std::env::var(VM_SYS_KILL_POLICY_ENV).ok();
        match value {
            Some(v) => std::env::set_var(VM_SYS_KILL_POLICY_ENV, v),
            None => std::env::remove_var(VM_SYS_KILL_POLICY_ENV),
        }
        Self { previous }
    }
}

impl Drop for RestoreEnv {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => std::env::set_var(VM_SYS_KILL_POLICY_ENV, v),
            None => std::env::remove_var(VM_SYS_KILL_POLICY_ENV),
        }
        auto_lang_core::vm::host_capabilities::clear_process_default_sys_kill_gate();
    }
}

fn spawn_target() -> Child {
    let mut cmd = if cfg!(windows) {
        Command::new("ping")
    } else {
        Command::new("sleep")
    };
    if cfg!(windows) {
        cmd.args(["-n", "120", "127.0.0.1"]);
    } else {
        cmd.arg("120");
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn isolated target process")
}

fn cleanup(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn kill_program(pid: i32) -> String {
    format!(
        r#"
fn main() {{
    let ok = sys.kill({pid})
    if ok {{ print("killed") }} else {{ print("not-killed") }}
}}
"#
    )
}

#[test]
fn p112_parse_policy_three_states_and_invalid() {
    assert_eq!(parse_policy(None).unwrap(), SysKillPolicy::AllowAll);
    assert_eq!(parse_policy(Some("   ")).unwrap(), SysKillPolicy::AllowAll);
    assert_eq!(parse_policy(Some("allow-all")).unwrap(), SysKillPolicy::AllowAll);

    match parse_policy(Some("deny")).unwrap() {
        SysKillPolicy::Deny { reason } => assert!(!reason.is_empty()),
        other => panic!("deny must parse to Deny, got {other:?}"),
    }
    match parse_policy(Some("deny:no authorization")).unwrap() {
        SysKillPolicy::Deny { reason } => assert_eq!(reason, "no authorization"),
        other => panic!("deny:<reason> must keep the reason, got {other:?}"),
    }
    match parse_policy(Some("allowed-pids:12, 34")).unwrap() {
        SysKillPolicy::AllowedPids(pids) => {
            assert!(pids.contains(&12) && pids.contains(&34) && pids.len() == 2);
        }
        other => panic!("allowed-pids must parse to AllowedPids, got {other:?}"),
    }

    for (raw, needle) in [
        ("bogus", "allow-all"),
        ("deny:", "non-empty reason"),
        ("allowed-pids:", "at least one pid"),
        ("allowed-pids:1,x", "'x'"),
        ("allowed-pids:0", "must be positive"),
    ] {
        let err = parse_policy(Some(raw)).expect_err(raw);
        assert!(
            err.contains(VM_SYS_KILL_POLICY_ENV),
            "invalid policy must name the env var; raw={raw} err={err}"
        );
        assert!(
            err.contains(needle),
            "invalid policy must explain the expected shape; raw={raw} err={err}"
        );
    }
}

#[test]
fn p112_install_registers_and_clears_gate() {
    let _guard = env_lock();

    // 未设置：无门，且清除此前可能残留的登记。
    let _env = RestoreEnv::set(None);
    install_vm_sys_kill_gate_from_env().unwrap();
    assert!(!process_default_gate_installed(), "unset policy must leave no gate");

    std::env::set_var(VM_SYS_KILL_POLICY_ENV, "deny");
    install_vm_sys_kill_gate_from_env().unwrap();
    assert!(process_default_gate_installed(), "deny must register the gate");

    std::env::set_var(VM_SYS_KILL_POLICY_ENV, "allow-all");
    install_vm_sys_kill_gate_from_env().unwrap();
    assert!(
        !process_default_gate_installed(),
        "explicit allow-all must clear a previously registered gate"
    );

    std::env::set_var(VM_SYS_KILL_POLICY_ENV, "not-a-policy");
    assert!(
        install_vm_sys_kill_gate_from_env().is_err(),
        "invalid policy must fail install instead of silently opening the gate"
    );
}

#[test]
fn p112_deny_gate_spares_target_through_the_vm() {
    let _guard = env_lock();
    let _env = RestoreEnv::set(Some("deny:host holds no process-kill authorization"));
    install_vm_sys_kill_gate_from_env().unwrap();

    let mut target = spawn_target();
    let pid = target.id() as i32;
    let err = auto_lang_core::run_autovm_capture(&kill_program(pid))
        .err()
        .expect("deny must surface as a VM error")
        .to_string();
    assert!(
        err.contains(auto_lang_core::vm::host_capabilities::SYS_KILL_PERMISSION_DENIED),
        "denial must carry the stable code; err={err}"
    );
    assert!(
        err.contains("host holds no process-kill authorization"),
        "denial must carry the host reason; err={err}"
    );
    assert!(err.contains(&pid.to_string()), "denial must name the pid; err={err}");
    assert!(
        matches!(target.try_wait(), Ok(None)),
        "denied request must not touch the target"
    );
    cleanup(target);
}

#[test]
fn p112_allowed_pids_gate_splits_through_the_vm() {
    let _guard = env_lock();
    let mut granted = spawn_target();
    let mut ungranted = spawn_target();
    let (pid_granted, pid_ungranted) = (granted.id() as i32, ungranted.id() as i32);
    let _env = RestoreEnv::set(Some(&format!("allowed-pids:{pid_granted}")));
    install_vm_sys_kill_gate_from_env().unwrap();

    let err = auto_lang_core::run_autovm_capture(&kill_program(pid_ungranted))
        .err()
        .expect("ungranted pid must be denied")
        .to_string();
    assert!(err.contains("authorization list"), "err={err}");
    assert!(matches!(ungranted.try_wait(), Ok(None)), "ungranted target must survive");

    let (res, stdout) = auto_lang_core::run_autovm_capture(&kill_program(pid_granted))
        .expect("granted pid must be killed");
    assert!(
        res.contains("killed") || stdout.contains("killed"),
        "res={res:?} stdout={stdout:?}"
    );
    assert!(
        granted.try_wait().expect("wait granted").is_some(),
        "granted target must terminate"
    );
    cleanup(granted);
    cleanup(ungranted);
}
