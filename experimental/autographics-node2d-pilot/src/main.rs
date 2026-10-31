//! node2d-lower：PLAN-016/017 降低生成器 CLI。
//!
//! legacy（默认，PLAN-016 冻结字节）：`--entry <dir>/main.at --out <dir>`
//!   → world.at + scene.agscene + identity.json（verify_plan016 再生控制）。
//! g11-2d（`--profile g11-2d`，PLAN-017 T-02 冻结）：完整 G11/2d 作者入口
//!   → world.at + scene.agscene + identity.json（含 roles/pair 映射、生成器
//!   身份与输入闭包清单）+ closure/（输入模块闭包副本，供 A→B 冷消费）。
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let profile = match args.iter().find(|a| a.as_str() == "--profile") {
        None => autographics_node2d_pilot::Profile::Legacy,
        Some(_) => {
            let i = args.iter().position(|a| a == "--profile").expect("profile");
            match args[i + 1].as_str() {
                "legacy" => autographics_node2d_pilot::Profile::Legacy,
                "g11-2d" => autographics_node2d_pilot::Profile::G112D,
                other => panic!("unknown profile '{other}' (legacy | g11-2d)"),
            }
        }
    };
    let entry = args
        .iter()
        .position(|a| a == "--entry")
        .and_then(|i| args.get(i + 1))
        .expect("usage: node2d-lower [--profile g11-2d] --entry <dir>/main.at --out <dir>");
    let out = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .expect("usage: node2d-lower [--profile g11-2d] --entry <dir>/main.at --out <dir>");
    let project = autographics_node2d_pilot::load_project(std::path::Path::new(entry))
        .unwrap_or_else(|e| {
            eprintln!("node2d-lower: {e}");
            std::process::exit(2);
        });
    std::fs::create_dir_all(out).expect("out dir");

    match profile {
        autographics_node2d_pilot::Profile::Legacy => {
            let lowered = autographics_node2d_pilot::lower_project(&project).unwrap_or_else(|e| {
                eprintln!("node2d-lower: {e}");
                std::process::exit(3);
            });
            std::fs::write(format!("{out}/world.at"), &lowered.user_source).expect("world.at");
            std::fs::write(format!("{out}/scene.agscene"), &lowered.scene_json)
                .expect("scene.agscene");
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
        autographics_node2d_pilot::Profile::G112D => {
            let lowered =
                autographics_node2d_pilot::lower_project_g112d(&project).unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            std::fs::write(format!("{out}/world.at"), &lowered.user_source).expect("world.at");
            std::fs::write(format!("{out}/scene.agscene"), &lowered.scene_json)
                .expect("scene.agscene");
            let mut names: Vec<_> = project.modules.keys().collect();
            names.sort();
            let mut modules = String::new();
            let mut closure = String::new();
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                modules.push_str(&format!(
                    "  \"module:{}\": \"{}\",\n",
                    name,
                    autographics_node2d_pilot::scene::sha256_hex(bytes)
                ));
                closure.push_str(&format!(
                    "  {{\"name\": \"{name}\", \"sha256\": \"{}\", \"bytes\": {}}},\n",
                    autographics_node2d_pilot::scene::sha256_hex(bytes),
                    bytes.len()
                ));
            }
            let mut identity = String::from("{\n");
            identity.push_str(" \"generator\": \"autographics-node2d-pilot plan017 g11-2d\",\n");
            identity.push_str(" \"api\": \"autographics.triggers2d/1\",\n");
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",\n \"user_source_bytes\": {},\n",
                lowered.source_sha256, lowered.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [\n");
            identity.push_str(closure.trim_end_matches(",\n"));
            identity.push_str("\n ],\n");
            identity.push_str(&format!(" \"roles\": {}\n", lowered.roles_json));
            identity.push_str("}\n");
            std::fs::write(format!("{out}/identity.json"), identity).expect("identity.json");
            // 输入闭包副本（A→B 冷消费：B 不借生产者源树）。
            let closure_dir = format!("{out}/closure");
            std::fs::create_dir_all(&closure_dir).expect("closure dir");
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                std::fs::write(format!("{closure_dir}/{name}.at"), bytes).expect("closure file");
            }
            println!(
                "node2d-lower[g11-2d]: wrote world.at ({} bytes, sha {}) + scene.agscene + closure/{}",
                lowered.source_bytes,
                lowered.source_sha256,
                names.len()
            );
        }
    }
}
