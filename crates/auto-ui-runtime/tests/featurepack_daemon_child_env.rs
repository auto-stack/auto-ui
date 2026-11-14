//! PLAN-083 R-1/F-3：宿主后端配置的**子进程投递**门禁（进程级 oracle）。
//!
//! 复审指出：`host_backend_env` 的单元用例只验证"env 袋子"，没有证明值真的
//! 到达子进程、也没证明宿主进程全局未被改写。本档用两个真子进程分别携带
//! A/B 两份宿主配置（含各自的受管凭据），并注入一条**宿主侧陈旧同名值**，
//! 断言：
//! 1. A 子进程只见 A 的地址/凭据，B 只见 B 的（互不污染）；
//! 2. 陈旧继承值被剔除（`remove` 生效），未投递键为空；
//! 3. 宿主进程自身的 env 未被改写（不靠全局 set_var 冒充隔离）。
#![cfg(windows)]

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use auto_ui_runtime::ui::osconfig_daemon::{apply_host_env, HostBackendEnv};

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "autoui-083-childenv-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// 子进程 helper：把指定键的当前 env 值写入 dump 文件（无 `FP083_ENV_DUMP`
/// 时是普通空转测试）。
#[test]
fn env_dump_child() {
    let Ok(dump) = std::env::var("FP083_ENV_DUMP") else {
        return;
    };
    let keys = std::env::var("FP083_ENV_KEYS").unwrap_or_default();
    let mut out = String::new();
    for key in keys.split(',').filter(|key| !key.is_empty()) {
        // `<unset>` 区分"未设置"与"空串"。
        let value = std::env::var(key).unwrap_or_else(|_| "<unset>".to_string());
        out.push_str(&format!("{key}={value}\n"));
    }
    std::fs::write(&dump, out).unwrap();
}

fn host_env(addr: &str, cred: &str) -> HostBackendEnv {
    HostBackendEnv {
        entries: vec![
            ("FP083_ADDR".to_string(), addr.to_string()),
            ("FP083_CRED".to_string(), cred.to_string()),
        ],
        remove: vec!["FP083_ADDR".to_string(), "FP083_CRED".to_string()],
        credential_env: Some("FP083_CRED".to_string()),
    }
}

fn spawn_dump_child(env: &HostBackendEnv, dump: &Path, keys: &str) -> std::process::Child {
    let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
    cmd.args(["--exact", "env_dump_child", "--nocapture"])
        .env("FP083_ENV_DUMP", dump)
        .env("FP083_ENV_KEYS", keys)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    apply_host_env(&mut cmd, env);
    cmd.spawn().expect("spawn env dump child")
}

fn wait_child(mut child: std::process::Child) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("env dump child 超时");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn read_dump(path: &Path) -> String {
    let mut raw = String::new();
    std::fs::File::open(path)
        .unwrap_or_else(|err| panic!("dump 不可读 {}: {err}", path.display()))
        .read_to_string(&mut raw)
        .unwrap();
    raw
}

#[test]
fn child_env_delivery_is_per_child_and_never_global() {
    let root = temp_root("ab");
    let cred_a = "aa".repeat(32);
    let cred_b = "bb".repeat(32);
    // 宿主侧陈旧同名值：子进程必须看不到它。
    std::env::set_var("FP083_CRED", "stale-parent-value");
    std::env::set_var("FP083_ADDR", "http://stale");
    // R-2/F-5：**只出现在 remove、不出现在 entries** 的陈旧键——让
    // `env_remove` 成为承重步骤（否则 entries 覆盖会让 remove 可被删掉而不红）。
    std::env::set_var("FP083_STALE", "parent-stale-only");

    let dump_a = root.join("a.env");
    let dump_b = root.join("b.env");
    let keys = "FP083_ADDR,FP083_CRED,FP083_UNSET,FP083_STALE";
    let mut env_a = host_env("http://127.0.0.1:17819", &cred_a);
    env_a.remove.push("FP083_STALE".to_string());
    let mut env_b = host_env("http://127.0.0.1:17820", &cred_b);
    env_b.remove.push("FP083_STALE".to_string());
    let child_a = spawn_dump_child(&env_a, &dump_a, keys);
    let child_b = spawn_dump_child(&env_b, &dump_b, keys);
    wait_child(child_a);
    wait_child(child_b);

    let a = read_dump(&dump_a);
    let b = read_dump(&dump_b);
    assert!(a.contains("FP083_ADDR=http://127.0.0.1:17819"), "A 地址: {a}");
    assert!(a.contains(&format!("FP083_CRED={cred_a}")), "A 凭据: {a}");
    assert!(b.contains("FP083_ADDR=http://127.0.0.1:17820"), "B 地址: {b}");
    assert!(b.contains(&format!("FP083_CRED={cred_b}")), "B 凭据: {b}");
    assert!(!a.contains(&cred_b) && !b.contains(&cred_a), "A/B 凭据互不可见");
    assert!(!a.contains("stale-parent-value") && !b.contains("stale-parent-value"),
        "陈旧继承值必须被剔除（remove 生效）：A={a} B={b}");
    assert!(a.contains("FP083_UNSET=<unset>"), "未投递键为空: {a}");
    assert!(
        a.contains("FP083_STALE=<unset>") && b.contains("FP083_STALE=<unset>"),
        "只在 remove 里的继承键必须被剔除（F-5 承重断言）：A={a} B={b}"
    );

    // 宿主进程全局未被改写（隔离不是靠 set_var 全局冒充）。
    assert_eq!(std::env::var("FP083_CRED").as_deref(), Ok("stale-parent-value"));
    assert_eq!(std::env::var("FP083_ADDR").as_deref(), Ok("http://stale"));
    assert_eq!(std::env::var("FP083_STALE").as_deref(), Ok("parent-stale-only"));
    std::env::remove_var("FP083_CRED");
    std::env::remove_var("FP083_ADDR");
    std::env::remove_var("FP083_STALE");
    let _ = std::fs::remove_dir_all(&root);
}
