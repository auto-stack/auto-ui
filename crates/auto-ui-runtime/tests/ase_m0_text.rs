//! PLAN-1022 ASE M0 文字浏览器永久 case（auto-ui 侧消费面）。
//!
//! 分层：CPU 面（恒跑）——同一宿主 frame 轮换 A→空→B（AV 真值文本，中文/
//! ASCII、字面量+函数叶，经真实 ui980 FFI 提交）、反硬编码（换合法根/函数名/
//! 改文字仍工作）、改A不改B、非法→修复、未知 app 零执行（代次不动、零
//! session）、导航代次与迟到装载整票拒绝（不覆盖新页、零 UI 效果）、关闭/
//! 重开与 20 轮 owner 释放（无存活 session 遗留）；窗口面
//! （`AUTO_UI_ASE_M0_E2E=1` 排窗，单实例串行）——同一真窗内轮换 + 证据 JSON。
//!
//! 准备（全部档，不在 case 计时内；2026-11-14 测试纪律——日常用例零 rustc）：
//! `cargo build -p auto-ui-runtime --features ace-bridge --example
//! ase_m0_text_window` 后运行 `ase_m0_text_window.exe --prep-only`（ui980
//! provider 预构建/新鲜度缓存；窗口档另需该 exe 本体）。
//!
//! 真实页面 fixtures（V-01/V-02）经**显式路径**消费组内 auto-lang 兄弟树
//! `experimental/ase-m0/fixtures/text-{a,b}/main.at`（`ASE_M0_TEXT_ROOT`
//! 可覆盖根；缺文件=具名准备失败，不静默跳过）。

#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_node_desktop::{current_session, host_session_stats};
use auto_ui_runtime::ui::ase_m0_frame::{AseFrameHost, AsePage};
use std::path::PathBuf;
use std::time::Instant;

/// 中文页（与 lang 树 fixtures/text-a/main.at 同构；根/函数名不同以测反硬编码）。
const PAGE_A_SRC: &str = "fn headline_a() str {\n    return \"第一页·中文标题\"\n}\n\nnode TextPageA {\n    view {\n        text \"ASE M0 页面甲\" {}\n        text headline_a() {}\n    }\n}\n";

/// ASCII 页（与 fixtures/text-b/main.at 同构）。
const PAGE_B_SRC: &str = "fn body_line() str {\n    return \"page B ascii body line\"\n}\n\nnode SecondTextRoot {\n    view {\n        text \"ASE M0 page B\" {}\n        text body_line() {}\n    }\n}\n";

const A_TEXTS: [&str; 2] = ["ASE M0 页面甲", "第一页·中文标题"];
const B_TEXTS: [&str; 2] = ["ASE M0 page B", "page B ascii body line"];

/// 换合法根/函数名/改文字的变体（反硬编码 oracle 输入）。
const PAGE_A_RENAMED_SRC: &str = "fn other_caption() str {\n    return \"换名后的函数文本\"\n}\n\nnode AnotherRoot {\n    view {\n        text \"换根页面\" {}\n        text other_caption() {}\n    }\n}\n";

fn temp_pages(tag: &str, srcs: &[(&str, &str)]) -> (tempdir::TempDirGuard, Vec<AsePage>) {
    let dir = std::env::temp_dir().join(format!("p1022-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp pages dir");
    let pages = srcs
        .iter()
        .map(|(key, src)| {
            let path = dir.join(format!("{key}.at"));
            std::fs::write(&path, src).expect("write page source");
            AsePage {
                key: key.to_string(),
                path: path.to_string_lossy().to_string(),
            }
        })
        .collect();
    (tempdir::TempDirGuard(dir), pages)
}

mod tempdir {
    use std::path::PathBuf;
    /// 用例收尾删除临时目录（不依赖外部 crate）。
    pub struct TempDirGuard(pub PathBuf);
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn display_texts(session: i32) -> Vec<String> {
    let (texts, _) = host_session_display_or_panic(session);
    texts.into_iter().next().unwrap_or_default()
}

fn host_session_display_or_panic(session: i32) -> (Vec<Vec<String>>, Vec<Vec<(String, String)>>) {
    auto_ui_runtime::ui::ace_node_desktop::host_session_display(session)
        .unwrap_or_else(|| panic!("session {session} display model"))
}

fn current_texts() -> Option<Vec<String>> {
    current_session(|s| s.texts.clone()).map(|t| t.into_iter().next().unwrap_or_default())
}

/// 真实 fixture 页面（显式路径；具名准备失败不静默）。
fn fixture_pages() -> (PathBuf, Vec<AsePage>) {
    let root = match std::env::var("ASE_M0_TEXT_ROOT") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            manifest.join("../../../auto-lang/experimental/ase-m0/fixtures")
        }
    };
    let pages = ["a", "b"]
        .iter()
        .map(|key| {
            let path = root.join(format!("text-{key}")).join("main.at");
            assert!(
                path.is_file(),
                "preparation failure: fixture page {} missing (ASE_M0_TEXT_ROOT or the \
                 group sibling auto-lang tree)",
                path.display()
            );
            AsePage {
                key: key.to_string(),
                path: path.to_string_lossy().to_string(),
            }
        })
        .collect();
    (root, pages)
}

/// V-02 核心：同一宿主 frame 轮换 A→空→B；两页 AV 真值文本（字面量+函数叶，
/// 中文/ASCII）经真实 ui980 FFI 进显示模型；关闭=空 frame；旧 session 冻结。
#[test]
fn p1022_v02_same_host_rotation_texts_through_real_ffi() {
    let started = Instant::now();
    let (_guard, pages) = temp_pages("v02", &[("a", PAGE_A_SRC), ("b", PAGE_B_SRC)]);
    let mut host = AseFrameHost::new(pages);

    let s1 = host.open("a").expect("open a");
    assert_eq!(
        current_texts(),
        Some(A_TEXTS.iter().map(|s| s.to_string()).collect())
    );
    assert_eq!(
        display_texts(s1),
        A_TEXTS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "page A AV texts through real ui980 FFI (literal + fn leaf, Chinese)"
    );

    host.close().expect("close a");
    assert_eq!(current_texts(), None, "closed frame is empty");
    assert!(host_session_stats(s1).expect("stats").closed);

    let s2 = host.open("b").expect("open b");
    assert_eq!(
        current_texts(),
        Some(B_TEXTS.iter().map(|s| s.to_string()).collect())
    );
    assert_eq!(
        display_texts(s2),
        B_TEXTS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "page B AV texts through real ui980 FFI (ASCII)"
    );
    let s1_stats = host_session_stats(s1).unwrap();
    assert!(s1_stats.closed, "old page session stays closed");
    assert_eq!(s1_stats.applies, host_session_stats(s1).unwrap().applies);
    assert!(host.errors.is_empty(), "errors: {:?}", host.errors);
    assert!(
        host.nav_generation() >= 3,
        "open/close/open advanced generations"
    );
    eprintln!("p1022_v02 elapsed: {:?}", started.elapsed());
}

/// V-02 反硬编码：换合法根名/函数名/改文字后，同入口仍装载并显示新真值
/// （适配与宿主不锚定根名、函数名或字面文本）。
#[test]
fn p1022_v02_legal_rename_and_text_change_still_works() {
    let started = Instant::now();
    let (_guard, pages) = temp_pages("v02r", &[("a", PAGE_A_RENAMED_SRC)]);
    let mut host = AseFrameHost::new(pages);
    let s = host.open("a").expect("renamed page loads");
    assert_eq!(
        display_texts(s),
        vec!["换根页面".to_string(), "换名后的函数文本".to_string()],
        "renamed root/fn and changed texts display through the same lane"
    );
    assert!(host.errors.is_empty(), "errors: {:?}", host.errors);
    eprintln!("p1022_v02r elapsed: {:?}", started.elapsed());
}

/// V-03：改A不改B；非法A（无 node 候选）具名拒且不半挂载（frame 不变、零
/// 新 session）；修复后真实重开；未知 app 零执行（代次不动、零装载）。
#[test]
fn p1022_v03_modify_a_not_b_invalid_repair_unknown() {
    let started = Instant::now();
    let modified_a = PAGE_A_SRC.replace("第一页·中文标题", "改后的中文标题");
    let (_guard, pages) = temp_pages(
        "v03",
        &[
            ("a", &modified_a),
            ("b", PAGE_B_SRC),
            ("bad", "fn main() int { return 11 }\n"),
        ],
    );
    let mut host = AseFrameHost::new(pages);

    // 改 A 不改 B：各自显示模型独立。
    let sa = host.open("a").expect("open modified a");
    assert_eq!(
        display_texts(sa),
        vec!["ASE M0 页面甲".to_string(), "改后的中文标题".to_string()],
        "modified A displays its own truth"
    );
    let sb = host.open("b").expect("open b");
    assert_eq!(
        display_texts(sb),
        B_TEXTS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "B unchanged by A's edit"
    );

    // 非法 A：装载期具名拒；frame 保持 B；零新 session。
    let sessions_before = host.evidence()["sessions"].as_array().unwrap().len();
    let err = host
        .open("bad")
        .expect_err("source without a node candidate must refuse");
    assert!(
        err.contains("no AGE node candidate"),
        "named refusal: {err}"
    );
    assert_eq!(
        current_texts(),
        Some(B_TEXTS.iter().map(|s| s.to_string()).collect())
    );
    assert_eq!(
        host.evidence()["sessions"].as_array().unwrap().len(),
        sessions_before,
        "refused open creates zero sessions"
    );

    // 修复：换回合法源后同入口真实打开。
    let repaired = "node Repaired {\n    view {\n        text \"修复页\" {}\n    }\n}\n";
    let bad_path = host
        .pages()
        .iter()
        .find(|p| p.key == "bad")
        .map(|p| PathBuf::from(&p.path))
        .expect("bad page registered");
    std::fs::write(&bad_path, repaired).expect("repair in place");
    let s_fix = host.open("bad").expect("repaired source opens");
    assert_eq!(display_texts(s_fix), vec!["修复页".to_string()]);

    // 未知 app：零执行（代次不动、零装载、具名拒）。
    let gen_before = host.nav_generation();
    let sessions_before = host.evidence()["sessions"].as_array().unwrap().len();
    let err = host.open("zzz").expect_err("unknown app must refuse");
    assert!(err.contains("unknown app key"), "named refusal: {err}");
    assert_eq!(
        host.nav_generation(),
        gen_before,
        "unknown app advances nothing"
    );
    assert_eq!(
        host.evidence()["sessions"].as_array().unwrap().len(),
        sessions_before,
        "unknown app executes zero"
    );
    assert!(host.current_key() == Some("bad"));
    eprintln!("p1022_v03 elapsed: {:?}", started.elapsed());
}

/// V-04：导航代次——A 的装载结果迟到于 B 的打开 → 整票拒绝（不 open、零
/// UI 效果、不覆盖 B 的 frame）；关窗后迟到同样拒绝；A→B→A 全轮换可用。
#[test]
fn p1022_v04_generations_late_reject_and_aba_cycle() {
    let started = Instant::now();
    let (_guard, pages) = temp_pages("v04", &[("a", PAGE_A_SRC), ("b", PAGE_B_SRC)]);
    let mut host = AseFrameHost::new(pages);

    // A 装载完成（ticket gen1）→ 期间 B 打开（gen2）→ A 挂载迟到。
    let ticket = host.begin_open("a").expect("begin load a");
    let sb = host.open("b").expect("open b (advances generation)");
    let applies_b_before = host_session_stats(sb).unwrap().applies;
    let err = host
        .mount(ticket)
        .expect_err("stale load result must be rejected");
    assert!(
        err.contains("stale load result"),
        "named stale rejection: {err}"
    );
    assert_eq!(
        current_texts(),
        Some(B_TEXTS.iter().map(|s| s.to_string()).collect()),
        "late A does not overwrite B's frame"
    );
    assert_eq!(
        host_session_stats(sb).unwrap().applies,
        applies_b_before,
        "stale mount has zero UI effect"
    );
    let stale_records: Vec<_> = host
        .nav_log
        .iter()
        .filter(|r| r.outcome.starts_with("stale-rejected"))
        .collect();
    assert_eq!(
        stale_records.len(),
        1,
        "one stale-rejected record: {:?}",
        host.nav_log
    );

    // 关窗后迟到：begin(其余页) → close（代次再进）→ mount 拒。
    let ticket2 = host.begin_open("a").expect("begin load a again");
    host.close().expect("close b (advances generation)");
    let err = host
        .mount(ticket2)
        .expect_err("post-close late mount refused");
    assert!(err.contains("stale load result"), "{err}");
    assert_eq!(current_texts(), None, "frame stays empty after close+late");

    // A→B→A：每次轮换旧 session 关闭、新 session 显示新页。
    let s1 = host.open("a").expect("open a");
    let s2 = host.open("b").expect("open b");
    let s3 = host.open("a").expect("open a again");
    assert_ne!(s3, s1);
    assert_ne!(s3, s2);
    assert!(host_session_stats(s1).unwrap().closed);
    assert!(host_session_stats(s2).unwrap().closed);
    assert!(!host_session_stats(s3).unwrap().closed);
    assert_eq!(
        current_texts(),
        Some(A_TEXTS.iter().map(|s| s.to_string()).collect()),
        "A→B→A ends on A's frame"
    );
    eprintln!("p1022_v04 elapsed: {:?}", started.elapsed());
}

/// V-05（CPU 部分）：打开/关闭 20 轮——每轮关闭后空 frame，全程无存活
/// session 遗留（全部 closed），owner 只有一个；重开=新 session、显示模型
/// 重新提交（applies 重记账），旧 session 冻结。
#[test]
fn p1022_v05_close_rounds_owner_release_and_reopen() {
    let started = Instant::now();
    let (_guard, pages) = temp_pages("v05", &[("a", PAGE_A_SRC)]);
    let mut host = AseFrameHost::new(pages);

    let mut seen = Vec::new();
    for round in 0..20 {
        let s = host.open("a").expect("open round");
        seen.push(s);
        assert_eq!(
            current_texts(),
            Some(A_TEXTS.iter().map(|s| s.to_string()).collect()),
            "round {round} shows page A"
        );
        host.close().expect("close round");
        assert_eq!(
            current_texts(),
            None,
            "round {round} frame empty after close"
        );
    }
    for (round, s) in seen.iter().enumerate() {
        let stats = host_session_stats(*s).unwrap_or_else(|| panic!("session {s} stats"));
        assert!(
            stats.closed,
            "round {round} session released (no live owner left)"
        );
    }
    assert_eq!(
        host.nav_generation(),
        40,
        "each accepted open/close advanced once"
    );

    // 重开：同页新 session；显示模型重提交；旧 session 冻结。
    let s1 = host.open("a").expect("reopen base");
    let applies_s1 = host_session_stats(s1).unwrap().applies;
    let s2 = host.reopen().expect("reopen");
    assert_ne!(s1, s2, "reopen is a new session handle");
    assert!(
        host_session_stats(s1).unwrap().closed,
        "old generation closed"
    );
    assert!(
        host_session_stats(s2).unwrap().applies >= applies_s1,
        "reopen resubmits the display model over FFI"
    );
    assert_eq!(
        display_texts(s2),
        A_TEXTS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "reopened frame shows the same AV truth"
    );
    assert!(host.errors.is_empty(), "errors: {:?}", host.errors);
    eprintln!("p1022_v05 elapsed: {:?}", started.elapsed());
}

/// V-01/V-02（正式来源）：真实 fixture 页面（组内 auto-lang 兄弟树显式路径）
/// 经同一公共面装载，AV 真值文本字节级一致（中文/ASCII、函数叶）。
#[test]
fn p1022_v01_real_fixture_pages_via_explicit_paths() {
    let started = Instant::now();
    let (_root, pages) = fixture_pages();
    let mut host = AseFrameHost::new(pages);
    let sa = host.open("a").expect("fixture a loads");
    assert_eq!(
        display_texts(sa),
        A_TEXTS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "fixture text-a AV texts byte-exact"
    );
    let sb = host.open("b").expect("fixture b loads");
    assert_eq!(
        display_texts(sb),
        B_TEXTS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "fixture text-b AV texts byte-exact"
    );
    assert!(host.errors.is_empty(), "errors: {:?}", host.errors);
    eprintln!("p1022_v01 elapsed: {:?}", started.elapsed());
}

/// V-02/V-05（窗口面，排窗 `AUTO_UI_ASE_M0_E2E=1`）：同一真实宿主窗口内
/// 脚本轮换 打开A→关闭→打开B→重开→关闭；帧数证明单窗持续重渲染（非每页
/// 新窗）；证据 JSON 断言代次/会话/文本。真鼠标/键盘与手工 resize 归手工
/// 档（与自动注入分账）。
#[test]
fn p1022_window_e2e_same_host_rotation() {
    if std::env::var("AUTO_UI_ASE_M0_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p1022_window_e2e_same_host_rotation: not scheduled (set AUTO_UI_ASE_M0_E2E=1 \
             after the prep stage; window single-instance discipline)"
        );
        return;
    }
    let started = Instant::now();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest
        .join("../../target/debug/examples/ase_m0_text_window.exe")
        .canonicalize()
        .unwrap();
    assert!(
        exe.is_file(),
        "preparation failure: {} missing — run the prep stage first",
        exe.display()
    );
    let (_root, pages) = fixture_pages();
    let tmp = std::env::temp_dir().join(format!("p1022-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let ev_path = tmp.join("evidence.json");

    let out = std::process::Command::new(&exe)
        .arg("--page")
        .arg(format!("a={}", pages[0].path))
        .arg("--page")
        .arg(format!("b={}", pages[1].path))
        .arg("--evidence")
        .arg(&ev_path)
        .arg("--script")
        .arg("3:open:a,9:close,13:open:b,19:reopen,25:close")
        .arg("--hold-ms")
        .arg("4000")
        .arg("--min-frames")
        .arg("12")
        .current_dir(&tmp)
        .output()
        .expect("spawn window exe");
    assert!(
        out.status.success(),
        "window exe failed: {} / {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let ev: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ev_path).unwrap()).unwrap();

    // 单窗持续重渲染：同一窗口跨轮换渲染 ≥12 帧（每页新窗/第二事件环不成立）。
    assert!(
        ev["frames_rendered"].as_u64().unwrap_or(0) >= 12,
        "one real window re-rendered across rotation: {}",
        ev["frames_rendered"]
    );
    assert!(
        ev["errors"].as_array().unwrap().is_empty(),
        "errors: {}",
        ev["errors"]
    );

    // 导航轨迹：open a → close → open b(rotate) → reopen → close，代次 1..5。
    let nav: Vec<&str> = ev["nav_log"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["action"].as_str().unwrap())
        .collect();
    assert_eq!(
        nav,
        vec!["open", "close", "open", "reopen", "close"],
        "{nav:?}"
    );
    let sessions = ev["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 3, "a; b; b-reopen = three sessions");
    let texts_of = |s: &serde_json::Value| -> Vec<String> {
        s["texts"]
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        texts_of(&sessions[0]),
        A_TEXTS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "session 1 is page A"
    );
    assert_eq!(
        texts_of(&sessions[1]),
        B_TEXTS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "session 2 is page B"
    );
    for s in sessions.iter().take(2) {
        assert_eq!(s["closed"], true, "rotated pages closed: {s}");
    }
    assert_eq!(ev["active"], serde_json::Value::Null, "final frame empty");
    assert_eq!(
        ev["nav_generation"].as_u64().unwrap(),
        5,
        "five accepted navigations"
    );
    let _ = std::fs::remove_dir_all(&tmp);
    eprintln!("p1022_window_e2e elapsed: {:?}", started.elapsed());
}
