//! PLAN-031 T-04（work）——UI tools 主题包输入闸门：官方身份常量 + 具名拒绝。
//! PLAN-036 T-02——语义身份/解析精度/快照安全/receipt 消费失败语义。

use std::path::Path;

use auto_ui_tools::theme_package::{
    app_theme_dep_path, publish_theme_package_snapshot, read_theme_inputs_identity,
    read_theme_inputs_receipt, theme_file_sha256, theme_package_content_digest,
    theme_package_identity, verified_theme_input_identity, verify_app_theme_inputs,
    verify_official_theme_package, verify_theme_package, write_theme_inputs_receipt,
    ThemePackageError, ThemePackageExpectation,
};

fn pkg() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/themes")
}

#[test]
fn tools_input_accepts_official_package_and_reports_families() {
    let p = verify_official_theme_package(&pkg()).expect("官方包应通过 tools 输入闸门");
    assert_eq!(p.package, "themes");
    assert_eq!(p.version, "0.2.0");
    assert!(p.families.contains_key("Auto") && p.families.contains_key("Solarized"));
}

#[test]
fn tools_input_rejects_wrong_version_with_named_error() {
    let err = verify_theme_package(
        &pkg(),
        &ThemePackageExpectation {
            owner: "auto-ui",
            package: "themes",
            version: "0.1.0",
            schema: 1,
        },
    )
    .expect_err("错版必须拒绝");
    match err {
        ThemePackageError::FieldMismatch { field, .. } => assert_eq!(field, "version"),
        other => panic!("错版必须 FieldMismatch(version)，得到 {other:?}"),
    }
}


#[test]
fn snapshot_publish_is_independent_and_minimal() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("vendor/themes");
    let p = publish_theme_package_snapshot(&pkg(), &dest).expect("snapshot publish");
    assert_eq!(p.root, dest);
    verify_official_theme_package(&dest).expect("snapshot must verify independently");

    let mut names: Vec<String> = std::fs::read_dir(&dest)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec!["pac.at", "theme-package.json", "themes.at"],
        "snapshot must contain only manifest + listed files"
    );
}

#[test]
fn snapshot_publish_refuses_tampered_source_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("themes");
    std::fs::create_dir_all(&src).unwrap();
    for f in ["theme-package.json", "themes.at", "pac.at"] {
        std::fs::copy(pkg().join(f), src.join(f)).unwrap();
    }
    let original = std::fs::read(src.join("themes.at")).unwrap();
    let mut bad = original.clone();
    bad[original.len() / 2] ^= 0x01;
    std::fs::write(src.join("themes.at"), bad).unwrap();

    let dest = dir.path().join("vendor/themes");
    let err = publish_theme_package_snapshot(&src, &dest)
        .expect_err("tampered source must be refused");
    assert!(matches!(err, ThemePackageError::HashMismatch { .. }), "{err:?}");
    assert!(!dest.exists(), "refusal must not create a snapshot dir");
}
#[test]
fn content_digest_is_location_independent_and_content_sensitive() {
    let d1 = theme_package_content_digest(&pkg()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("vendor/themes");
    publish_theme_package_snapshot(&pkg(), &copy).unwrap();
    assert_eq!(
        theme_package_content_digest(&copy).unwrap(),
        d1,
        "digest must not depend on the absolute root"
    );
    let themes_at = copy.join("themes.at");
    let original = std::fs::read(&themes_at).unwrap();
    let mut changed = original.clone();
    changed[original.len() / 2] ^= 0x01;
    std::fs::write(&themes_at, changed).unwrap();
    assert_ne!(theme_package_content_digest(&copy).unwrap(), d1);
}
// ── PLAN-031 T-07（V-09）：生成准入 + 生成输入 receipt ──────────────────

/// 保留 mtime（Windows 下用 std File::set_modified，不引额外依赖）。
fn set_mtime(path: &Path, t: std::time::SystemTime) {
    let f = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    f.set_modified(t).unwrap();
}
#[test]
fn app_theme_inputs_admit_official_package_and_receipt_records_digest() {
    let app = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui/010-contact-form")
        .canonicalize()
        .expect("010 例程目录");
    let root = verify_app_theme_inputs(&app)
        .expect("声明官方包就必须校验通过")
        .expect("应从 pac dep 解析出包根");
    assert_eq!(root, pkg().canonicalize().unwrap());

    let out = tempfile::tempdir().unwrap();
    let id = write_theme_inputs_receipt(out.path(), &app)
        .expect("receipt 可写")
        .expect("010 声明官方包，应返回统一输入身份");
    assert_eq!(id.content_digest, theme_package_content_digest(&pkg()).unwrap());
    assert_eq!(id.identity_digest, theme_package_identity(&pkg()).unwrap());
    assert_eq!(
        read_theme_inputs_receipt(out.path()).as_deref(),
        Some(id.content_digest.as_str()),
        "receipt 读回必须等于写入摘要"
    );
    assert_eq!(
        read_theme_inputs_identity(out.path()).as_deref(),
        Some(id.identity_digest.as_str()),
        "receipt 必须携带语义身份"
    );
}

#[test]
fn tampered_package_is_rejected_at_generation_admission() {
    let dir = tempfile::tempdir().unwrap();
    let pkg_dir = dir.path().join("themes");
    publish_theme_package_snapshot(&pkg(), &pkg_dir).unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir_all(app.join("src/front")).unwrap();
    std::fs::write(
        app.join("pac.at"),
        "name: \"t\"\nversion: \"0.1.0\"\ndep themes {\n    path: \"../themes\"\n}\n",
    )
    .unwrap();

    assert!(
        verify_app_theme_inputs(&app).unwrap().is_some(),
        "合法包应通过生成准入"
    );

    // 内容改但清单 hash 未更新 = 篡改 → 具名拒绝、不落半成品
    let themes_at = pkg_dir.join("themes.at");
    let mut bytes = std::fs::read(&themes_at).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x01;
    std::fs::write(&themes_at, &bytes).unwrap();
    let err = verify_app_theme_inputs(&app).expect_err("篡改必须拒绝");
    assert!(
        matches!(err, ThemePackageError::HashMismatch { .. }),
        "应具名为 HashMismatch，得到 {err:?}"
    );
}

/// V-09 核心：同名主题**内容变更且保留 mtime**（清单 hash 同步更新=合法发布）
/// 也必须让生成输入摘要变化；内容不变时摘要稳定。
#[test]
fn receipt_follows_content_not_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let pkg_dir = dir.path().join("themes");
    publish_theme_package_snapshot(&pkg(), &pkg_dir).unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir_all(app.join("src/front")).unwrap();
    std::fs::write(
        app.join("pac.at"),
        "name: \"t\"\nversion: \"0.1.0\"\ndep themes {\n    path: \"../themes\"\n}\n",
    )
    .unwrap();
    let out = tempfile::tempdir().unwrap();

    let r1 = write_theme_inputs_receipt(out.path(), &app)
        .unwrap()
        .expect("声明官方包应有身份");

    // 未变：再写一次摘要稳定
    let r_same = write_theme_inputs_receipt(out.path(), &app)
        .unwrap()
        .expect("声明官方包应有身份");
    assert_eq!(r1, r_same, "内容未变时统一输入身份必须稳定");

    // 合法内容变更：改色 + 同步清单 sha256 + 还原 mtime
    let themes_at = pkg_dir.join("themes.at");
    let manifest_path = pkg_dir.join("theme-package.json");
    let mtime_themes = std::fs::metadata(&themes_at).unwrap().modified().unwrap();
    let mtime_manifest = std::fs::metadata(&manifest_path).unwrap().modified().unwrap();

    let src = std::fs::read_to_string(&themes_at).unwrap();
    let new_src = src.replace("primary: \"#8b5cf6\"", "primary: \"#123456\"");
    assert_ne!(src, new_src, "夹具必须真的改了内容");
    std::fs::write(&themes_at, &new_src).unwrap();

    let new_hash = theme_file_sha256(&themes_at).unwrap();
    let manifest_text = std::fs::read_to_string(&manifest_path).unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(&manifest_text).unwrap();
    let files = manifest.get_mut("files").unwrap().as_array_mut().unwrap();
    let mut patched = false;
    for f in files.iter_mut() {
        if f.get("path").and_then(|p| p.as_str()) == Some("themes.at") {
            f["sha256"] = serde_json::Value::String(new_hash.clone());
            patched = true;
        }
    }
    assert!(patched, "清单必须列出 themes.at");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap()).unwrap();

    // 保留 mtime（证明新鲜度按内容而不是时间戳）
    set_mtime(&themes_at, mtime_themes);
    set_mtime(&manifest_path, mtime_manifest);
    assert_eq!(
        std::fs::metadata(&themes_at).unwrap().modified().unwrap(),
        mtime_themes
    );

    let r2 = write_theme_inputs_receipt(out.path(), &app)
        .unwrap()
        .expect("声明官方包应有身份");
    assert_ne!(
        r1.content_digest, r2.content_digest,
        "内容变化（mtime 不变）必须让生成输入摘要变化"
    );
    assert_ne!(
        r1.identity_digest, r2.identity_digest,
        "内容变化必须同时让语义身份变化"
    );
    assert_eq!(
        read_theme_inputs_receipt(out.path()).as_deref(),
        Some(r2.content_digest.as_str())
    );
}

// ── PLAN-036 T-02：语义身份 / 解析精度 / 快照安全 / 消费失败语义 ──────────

/// V-05：语义身份规范化（字段顺序/空白/families 排列不敏感）且对能力元数据
/// 敏感；**内容摘要与语义身份分离**——只改清单能力（如 default 翻转、module、
/// families）时内容摘要不变（OS053 冻结算法兼容），语义身份必变（复用失效）。
#[test]
fn identity_is_canonical_and_capability_sensitive() {
    let d0 = theme_package_identity(&pkg()).unwrap();
    assert_eq!(
        verified_theme_input_identity(&pkg()).unwrap().identity_digest,
        d0,
        "统一身份入口与 compiler 单源一致"
    );

    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("themes");
    publish_theme_package_snapshot(&pkg(), &copy).unwrap();

    // 1) 非语义重排：紧凑序列化（字段顺序/空白规范化，families 保序）→ 双摘要均不变。
    let manifest_path = copy.join("theme-package.json");
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    std::fs::write(&manifest_path, serde_json::to_string(&v).unwrap()).unwrap();
    verify_official_theme_package(&copy).expect("重排后仍合法");
    assert_eq!(theme_package_identity(&copy).unwrap(), d0, "清单重排不改语义身份");
    assert_eq!(theme_package_content_digest(&copy).unwrap(), theme_package_content_digest(&pkg()).unwrap());

    // 2) 能力变化：default light→dark（只动清单，不动文件）→ 内容摘要不变、身份必变。
    let mut v2: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    for fam in v2.get_mut("families").unwrap().as_array_mut().unwrap() {
        if fam.get("name").and_then(|n| n.as_str()) == Some("Auto") {
            fam["default"] = serde_json::Value::String("dark".into());
        }
    }
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&v2).unwrap()).unwrap();
    verify_official_theme_package(&copy).expect("default 变化仍为合法包（现 verifier 不校验 default）");
    let content_only = theme_package_content_digest(&copy).unwrap();
    assert_eq!(
        content_only,
        theme_package_content_digest(&pkg()).unwrap(),
        "只改能力元数据时内容摘要必须不变（OS053 冻结算法兼容）"
    );
    assert_ne!(
        theme_package_identity(&copy).unwrap(),
        d0,
        "能力（default）变化必须让语义身份变化——旧 receipt 据此失效"
    );

    // 3) 模块名变化：身份必变。
    let mut v3 = v2.clone();
    v3["module"] = serde_json::Value::String("themes2".into());
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&v3).unwrap()).unwrap();
    assert_ne!(theme_package_identity(&copy).unwrap(), d0);
}

/// V-01：dep 名恰为 `themes` 才算官方声明——近似名（`mythemes`）、前缀词
/// （`deploy:`）、其他 dep 块内 `themes` 字样不误命中；未声明 = 准入 Ok(None)、
/// receipt 写入 Ok(None) 且零文件写入。
#[test]
fn dep_scan_matches_exact_name_only_and_undeclared_stays_legacy() {
    let dir = tempfile::tempdir().unwrap();
    let pkg_dir = dir.path().join("themes");
    publish_theme_package_snapshot(&pkg(), &pkg_dir).unwrap();

    let mk_app = |name: &str, body: String| {
        let app = dir.path().join(name);
        std::fs::create_dir_all(app.join("src/front")).unwrap();
        std::fs::write(app.join("pac.at"), body).unwrap();
        app
    };

    // 近似名：目录就是官方包，但 dep 名不是 `themes` → 不算声明。
    let app = mk_app(
        "near",
        format!("name: \"n\"\nversion: \"0.1.0\"\ndep mythemes {{\n    path: \"../themes\"\n}}\n"),
    );
    assert!(app_theme_dep_path(&app).is_none(), "近似名不误命中");
    assert!(verify_app_theme_inputs(&app).unwrap().is_none(), "近似名=未声明");
    let out = tempfile::tempdir().unwrap();
    let r = write_theme_inputs_receipt(out.path(), &app).unwrap();
    assert!(r.is_none(), "未声明=Ok(None)");
    assert!(!out.path().join(".theme-inputs.json").exists(), "未声明零写入");

    // deploy: 前缀词不误命中；其他 dep 块含 themes 字样不误命中。
    let app2 = mk_app(
        "words",
        "name: \"w\"\nversion: \"0.1.0\"\ndeploy: \"themes-something\"\ndep utils {\n    note: \"themes\"\n}\n"
            .to_string(),
    );
    assert!(app_theme_dep_path(&app2).is_none(), "deploy:/其他dep块不误命中");

    // 恰名 `dep themes {` 正常命中（真实 010 已在既有用例覆盖，这里补内联块形式）。
    let app3 = mk_app(
        "exact",
        format!("name: \"e\"\nversion: \"0.1.0\"\ndep themes{{\n    path: \"../themes\"\n}}\n"),
    );
    assert!(app_theme_dep_path(&app3).is_some(), "内联 dep themes 块（同行开括号）应命中");
}

/// V-07：发布原语拒绝非空目标——未知内容零破坏、零合并。
#[test]
fn snapshot_publish_refuses_nonempty_dest_and_preserves_foreign_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("vendor/themes");
    std::fs::create_dir_all(&dest).unwrap();
    let foreign = dest.join("foreign.txt");
    std::fs::write(&foreign, b"someone else's data").unwrap();

    let err = publish_theme_package_snapshot(&pkg(), &dest)
        .expect_err("非空目标必须拒绝");
    assert!(
        matches!(err, ThemePackageError::BadManifest(ref m) if m.contains("非空")),
        "应具名拒绝非空目标，得到 {err:?}"
    );
    assert_eq!(
        std::fs::read(&foreign).unwrap(),
        b"someone else's data",
        "未知内容必须原样保留"
    );
    assert!(!dest.join("theme-package.json").exists(), "拒绝时不得写入任何包文件");

    // 空目录仍可发布（先清空）。
    std::fs::remove_file(&foreign).unwrap();
    publish_theme_package_snapshot(&pkg(), &dest).expect("清空后可发布");
}

/// V-06：receipt 写失败 = 消费失败（具名 Err）；声明但篡改 = 具名 Err；
/// 旧格式 receipt（无 identity 字段）读回身份为 None——不得当 fresh。
#[test]
fn receipt_failures_are_named_and_legacy_receipt_is_never_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let pkg_dir = dir.path().join("themes");
    publish_theme_package_snapshot(&pkg(), &pkg_dir).unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir_all(app.join("src/front")).unwrap();
    std::fs::write(
        app.join("pac.at"),
        "name: \"t\"\nversion: \"0.1.0\"\ndep themes {\n    path: \"../themes\"\n}\n",
    )
    .unwrap();

    // out_dir 实为一个普通文件 → 写 receipt 必失败且具名。
    let blocked = dir.path().join("blocked-out");
    std::fs::write(&blocked, b"not a dir").unwrap();
    let err = write_theme_inputs_receipt(&blocked, &app)
        .expect_err("receipt 写失败必须报错（消费失败）");
    assert!(
        matches!(err, ThemePackageError::MissingFile(ref p) if p == ".theme-inputs.json"),
        "写失败应具名，得到 {err:?}"
    );

    // 旧 receipt（无 identity 字段）：内容摘要可读，身份不可读 → 调用方必须重验。
    let out = tempfile::tempdir().unwrap();
    write_theme_inputs_receipt(out.path(), &app).unwrap().expect("some");
    let legacy = out.path().join(".theme-inputs.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&legacy).unwrap()).unwrap();
    let mut pkg_val = v.get("package").unwrap().clone();
    if let Some(m) = pkg_val.as_object_mut() {
        m.remove("identity");
    }
    let mut legacy_doc = v.clone();
    legacy_doc["package"] = pkg_val;
    std::fs::write(&legacy, serde_json::to_string_pretty(&legacy_doc).unwrap()).unwrap();
    assert!(read_theme_inputs_receipt(out.path()).is_some(), "旧 receipt 内容摘要仍可读");
    assert!(
        read_theme_inputs_identity(out.path()).is_none(),
        "旧 receipt 无身份字段 → 必须按不新鲜处理（安全重验），绝不当 fresh"
    );
}