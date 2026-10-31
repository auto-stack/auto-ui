//! node2d-lower：PLAN-016 降低生成器 CLI。
//!
//! --fixtures <dir>/main.at → --out：写 world.at（生成用户源）、
//! scene.agscene、identity.json（源与夹具 sha256 身份）。重新生成必须
//! 字节等价（tools/verify_plan016.py 的再生控制）。
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let entry = args
        .iter()
        .position(|a| a == "--entry")
        .and_then(|i| args.get(i + 1))
        .expect("usage: node2d-lower --entry <dir>/main.at --out <dir>");
    let out = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .expect("usage: node2d-lower --entry <dir>/main.at --out <dir>");
    let project = autographics_node2d_pilot::load_project(std::path::Path::new(entry))
        .unwrap_or_else(|e| {
            eprintln!("node2d-lower: {e}");
            std::process::exit(2);
        });
    let lowered = autographics_node2d_pilot::lower_project(&project)
        .unwrap_or_else(|e| {
            eprintln!("node2d-lower: {e}");
            std::process::exit(3);
        });
    std::fs::create_dir_all(out).expect("out dir");
    std::fs::write(format!("{out}/world.at"), &lowered.user_source).expect("world.at");
    std::fs::write(format!("{out}/scene.agscene"), &lowered.scene_json).expect("scene.agscene");
    let mut identity = String::from("{\n");
    identity.push_str(&format!(
        " \"user_source_sha256\": \"{}\",\n \"user_source_bytes\": {},\n",
        lowered.source_sha256, lowered.source_bytes
    ));
    let mut names: Vec<_> = project.modules.keys().collect();
    names.sort();
    for name in names {
        let bytes = project.modules[name].source.as_bytes();
        identity.push_str(&format!(
            " \"module:{}\": \"{}\",\n",
            name,
            autographics_node2d_pilot::scene::sha256_hex(bytes)
        ));
    }
    identity.push_str(" \"pair_to_instance\": [");
    identity.push_str(
        &lowered
            .pair_to_instance
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );
    identity.push_str("]\n}\n");
    std::fs::write(format!("{out}/identity.json"), identity).expect("identity.json");
    println!(
        "node2d-lower: wrote world.at ({} bytes, sha {}) + scene.agscene",
        lowered.source_bytes, lowered.source_sha256
    );
}
