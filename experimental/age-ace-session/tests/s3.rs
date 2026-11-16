//! PLAN-981（B-SESSION-I，S3）：同 PID body 更新（不调用 cargo/rustc）。
//!
//! 证明三件事：
//! 1. 有效编辑在**同一 Session 身份**下产生新结果（PID 不变、generation 递增）；
//! 2. 错误编辑**保留旧代**（观测值仍是旧值），修复后恢复更新；
//! 3. 更新路径在**没有工具链**的 PATH 下也能完成——用"剥掉 PATH"实测证明它不调用
//!    cargo/rustc（ACE 的编译/检查是进程内的）。
use age_ace_session::AgeSession;

const V1: &str = "node N {\n    model {\n        count int = 1\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
const V2: &str = "node N {\n    model {\n        count int = 7\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
const BAD: &str = "node N {\n    view {\n        text\n    }\n}\n";

fn observed_count(s: &AgeSession) -> i32 {
    s.observe().expect("observation").components[0].instance_states[0][0]
}

/// 有效编辑：同 PID、generation 递增、观测值更新。
#[test]
fn s3_effective_edit_updates_same_pid() {
    let mut s = AgeSession::open(V1, 1).expect("open");
    let pid = s.pid();
    assert_eq!(observed_count(&s), 1);
    let gen = s.update_body(V2).expect("valid edit");
    assert_eq!(gen, 1, "generation counts successful updates");
    assert_eq!(s.pid(), pid, "same PID identity across a body edit");
    assert_eq!(observed_count(&s), 7, "the new body takes effect");
}

/// 错误编辑：保留旧代（值不变、generation 不涨），修复后恢复。
#[test]
fn s3_bad_edit_keeps_the_old_generation_then_recovers() {
    let mut s = AgeSession::open(V2, 1).expect("open");
    let before = observed_count(&s);
    let ds = match s.update_body(BAD) {
        Ok(_) => panic!("a broken body must be rejected"),
        Err(ds) => ds,
    };
    assert!(ds.iter().any(|d| d.starts_with("source.")), "{ds:?}");
    assert_eq!(
        s.generation(),
        0,
        "a failed edit does not advance the generation"
    );
    assert_eq!(
        observed_count(&s),
        before,
        "the old generation stays observable"
    );
    s.update_body(V1).expect("a fixed body is accepted");
    assert_eq!(observed_count(&s), 1, "the fixed body takes effect");
}

/// 无工具链 PATH 下更新仍成功：证明不调用 cargo/rustc（进程内编译/检查）。
#[test]
fn s3_update_needs_no_cargo_or_rustc_on_path() {
    // 该用例独占性地改写 PATH：测试二进制内其它用例只做进程内编译，不受影响。
    let saved = std::env::var_os("PATH");
    std::env::set_var("PATH", "");
    let mut s = AgeSession::open(V1, 1).expect("open without a toolchain on PATH");
    let updated = s.update_body(V2);
    if let Some(p) = saved {
        std::env::set_var("PATH", p);
    }
    let gen = updated.expect("the edit compiles without cargo/rustc");
    assert_eq!(gen, 1);
    assert_eq!(observed_count(&s), 7);
}
