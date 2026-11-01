//! node2d-lower：PLAN-016/017/019 降低生成器 CLI。
//!
//! legacy（默认，PLAN-016 冻结字节）：`--entry <dir>/main.at --out <dir>`
//!   → world.at + scene.agscene + identity.json（verify_plan016 再生控制）。
//! g11-2d（`--profile g11-2d`，PLAN-017 T-02 冻结）：完整 G11/2d 作者入口
//!   → world.at + scene.agscene + identity.json（含 roles/pair 映射、生成器
//!   身份与输入闭包清单）+ closure/（输入模块闭包副本，供 A→B 冷消费）。
//! scene-2d/character-2d/camera-2d（`--profile ...`，PLAN-019 T-02 冻结）：
//!   基础二维作者入口 → scene.agscene + identity.json + closure/；
//!   character-2d/camera-2d 另有 world.at。
use autographics_node2d_pilot::{BasicProfile, Profile};

fn usage() -> ! {
    eprintln!(
        "usage: node2d-lower [--profile legacy|g11-2d|scene-2d|character-2d|camera-2d] --entry <dir>/main.at --out <dir>"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let profile = match args.iter().find(|a| a.as_str() == "--profile") {
        None => Profile::Legacy,
        Some(_) => {
            let i = args.iter().position(|a| a == "--profile").expect("profile");
            match args.get(i + 1).map(|s| s.as_str()) {
                Some("legacy") => Profile::Legacy,
                Some("g11-2d") => Profile::G112D,
                Some("scene-2d") => Profile::Basic2D(BasicProfile::Scene2D),
                Some("character-2d") => Profile::Basic2D(BasicProfile::Character2D),
                Some("camera-2d") => Profile::Basic2D(BasicProfile::Camera2D),
                Some("components-2d") => Profile::Components2D,
                _ => usage(),
            }
        }
    };
    let entry = args
        .iter()
        .position(|a| a == "--entry")
        .and_then(|i| args.get(i + 1))
        .unwrap_or_else(|| usage());
    let out = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .unwrap_or_else(|| usage());
    let project = autographics_node2d_pilot::load_project(std::path::Path::new(entry))
        .unwrap_or_else(|e| {
            eprintln!("node2d-lower: {e}");
            std::process::exit(2);
        });
    std::fs::create_dir_all(out).expect("out dir");

    match profile {
        Profile::Legacy => {
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
        Profile::G112D => {
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
        Profile::Components2D => {
            let lowered = autographics_node2d_pilot::lower_project_components(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            std::fs::write(
                format!("{out}/world.at"),
                lowered.user_source.expect("source"),
            )
            .expect("world.at");
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
            identity
                .push_str(" \"generator\": \"autographics-node2d-pilot plan020 components-2d\",\n");
            identity.push_str(" \"api\": \"autographics.objects2d/1\",\n");
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",\n \"user_source_bytes\": {},\n",
                lowered.source_sha256, lowered.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [\n");
            identity.push_str(closure.trim_end_matches(",\n"));
            identity.push_str("\n ],\n");
            identity.push_str(&format!(" \"roles\": {},\n", lowered.roles_json));
            // PLAN-020 T-06 runner provenance (factual, per product): the
            // scene binds behavior.source to the world sha; the flow harness
            // consumes the frozen files via include_str! and feeds world.at
            // to scripts/<sha>.at; the load path hash-verifies every
            // scripts/<64hex>.at against its file name, so a damaged world
            // copy is rejected at load ("conflict: immutable source").
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = match product.as_str() {
                "helpers" => {
                    "experimental/autographics-node2d-pilot/tests/node_components_gates.rs (fixture-level; no driver flow consumer)"
                }
                "probe" | "budget" | "budget-plus" | "lifecycle" | "lifecycle-doublepress"
                | "envelope" | "init-fault" => {
                    "experimental/autographics-auto-driver/tests/node_components_flow.rs"
                }
                _ => "unrecorded (this out dir has no registered flow consumer)",
            };
            identity.push_str(" \"provenance\": {\n");
            identity.push_str(&format!("  \"product\": \"{product}\",\n"));
            identity.push_str(&format!("  \"consumer\": \"{consumer}\",\n"));
            identity.push_str(
                "  \"consumption\": \"include_str! frozen scene.agscene + world.at; the harness writes world.at to scripts/<sha>.at where <sha> = scene.behavior.source.sha256\",\n",
            );
            identity.push_str(
                "  \"load_verification\": \"the load path hash-verifies the behavior source closure against the scene-bound sha and rejects a damaged world copy at load (observed: 'behavior source closure identity: <scripts/<sha>.at>')\",\n",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"\n",
                lowered.source_sha256
            ));
            identity.push_str(" }\n}\n");
            std::fs::write(format!("{out}/identity.json"), identity).expect("identity.json");
            let closure_dir = format!("{out}/closure");
            std::fs::create_dir_all(&closure_dir).expect("closure dir");
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                std::fs::write(format!("{closure_dir}/{name}.at"), bytes).expect("closure file");
            }
            println!(
                "node2d-lower[components-2d]: wrote world.at ({} bytes, sha {}) + scene.agscene + closure/{}",
                lowered.source_bytes, lowered.source_sha256,
                names.len()
            );
        }
        Profile::Basic2D(basic) => {
            let lowered = autographics_node2d_pilot::lower_project_basic(&project, basic)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            if let Some(src) = &lowered.user_source {
                std::fs::write(format!("{out}/world.at"), src).expect("world.at");
            }
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
            identity.push_str(&format!(
                " \"generator\": \"autographics-node2d-pilot plan019 {}\",\n",
                basic.name()
            ));
            identity.push_str(&format!(
                " \"api\": {},\n",
                match basic.api_version() {
                    Some(api) => format!("\"{api}\""),
                    None => "null".to_string(),
                }
            ));
            identity.push_str(&format!(
                " \"user_source_sha256\": {},\n \"user_source_bytes\": {},\n",
                match &lowered.user_source {
                    Some(_) => format!("\"{}\"", lowered.source_sha256),
                    None => "null".to_string(),
                },
                lowered.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [\n");
            identity.push_str(closure.trim_end_matches(",\n"));
            identity.push_str("\n ],\n");
            identity.push_str(&format!(" \"roles\": {}\n", lowered.roles_json));
            identity.push_str("}\n");
            std::fs::write(format!("{out}/identity.json"), identity).expect("identity.json");
            let closure_dir = format!("{out}/closure");
            std::fs::create_dir_all(&closure_dir).expect("closure dir");
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                std::fs::write(format!("{closure_dir}/{name}.at"), bytes).expect("closure file");
            }
            let source_note = match &lowered.user_source {
                Some(_) => " + world.at",
                None => "",
            };
            println!(
                "node2d-lower[{}]: wrote scene.agscene{source_note} + closure/{} (source sha {})",
                basic.name(),
                names.len(),
                lowered.source_sha256
            );
        }
    }
}
