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
        "usage: node2d-lower [--profile legacy|g11-2d|scene-2d|character-2d|camera-2d|components-2d|components-2d-game] --entry <dir>/main.at --out <dir>"
    );
    std::process::exit(2);
}


/// PLAN-024/027 共享 components-3d 产品落盘（world.at/scene.agscene/assets/
/// identity.json/closure；身份串与消费者由 profile 注入，024 字节不变）。
#[allow(clippy::too_many_arguments)]
fn write_components3d_products(
    out: &str,
    entry: &str,
    project: &autographics_node2d_pilot::load::Project,
    lowered: autographics_node2d_pilot::LowerOutputBasic2D,
    generator: &str,
    behavior_api: &str,
    consumer_for: fn(&str) -> &'static str,
    presentation: Option<&autographics_node2d_pilot::lower_node3d_presentation::PresentationMeta3>,
) {
            if let Some(src) = &lowered.user_source {
                std::fs::write(format!("{out}/world.at"), src).expect("world.at");
            }
            std::fs::write(format!("{out}/scene.agscene"), &lowered.scene_json)
                .expect("scene.agscene");
            // GLB 资产字节（完整闭包：身份 = 原始字节；字节等值复制）。
            std::fs::create_dir_all(format!("{out}/assets")).expect("assets dir");
            let mut copied: Vec<(String, String, String)> = Vec::new();
            for module in project.modules.values() {
                for r in &module.resources {
                    let is_model = matches!(
                        &r.kind,
                        autographics_node2d_pilot::load::ResourceKind::Model3D { .. }
                            | autographics_node2d_pilot::load::ResourceKind::Model3DTextured { .. }
                            | autographics_node2d_pilot::load::ResourceKind::Model3DAnimated { .. }
                    );
                    if !is_model {
                        continue;
                    }
                    let (file, sha256, byte_length) = match &r.kind {
                        autographics_node2d_pilot::load::ResourceKind::Model3D {
                            file,
                            sha256,
                            byte_length,
                        }
                        | autographics_node2d_pilot::load::ResourceKind::Model3DTextured {
                            file,
                            sha256,
                            byte_length,
                        }
                        | autographics_node2d_pilot::load::ResourceKind::Model3DAnimated {
                            file,
                            sha256,
                            byte_length,
                        } => (file, sha256, byte_length),
                        _ => unreachable!("is_model gate"),
                    };
                    let bytes = std::fs::read(
                        std::path::Path::new(entry)
                            .parent()
                            .expect("entry parent")
                            .join(file),
                    )
                    .unwrap_or_else(|e| panic!("read model {file}: {e}"));
                    assert_eq!(bytes.len(), *byte_length, "model byte drift: {file}");
                    assert_eq!(
                        autographics_node2d_pilot::scene::sha256_hex(&bytes),
                        *sha256,
                        "model sha drift: {file}"
                    );
                    let dest = format!("{out}/assets/{sha256}.glb");
                    std::fs::write(&dest, &bytes).expect("asset copy");
                    copied.push((sha256.clone(), format!("{sha256}.glb"), file.clone()));
                    // PLAN-034：外部图像边随闭包复制（原 URI/字节不重写）。
                    if let Some(meta) = presentation {
                        if let Some(entry_meta) = meta.resources.get(&r.name) {
                            for image in &entry_meta.images {
                                let src = std::path::Path::new(entry)
                                    .parent()
                                    .expect("entry parent")
                                    .join(&image.path);
                                let ibytes = std::fs::read(&src)
                                    .unwrap_or_else(|e| panic!("read image {}: {e}", src.display()));
                                assert_eq!(
                                    autographics_node2d_pilot::scene::sha256_hex(&ibytes),
                                    image.sha256,
                                    "image sha drift: {}",
                                    src.display()
                                );
                                let dest = format!("{out}/assets/{}.png", image.sha256);
                                std::fs::write(&dest, &ibytes).expect("image copy");
                                copied.push((
                                    image.sha256.clone(),
                                    format!("{}.png", image.sha256),
                                    image.path.clone(),
                                ));
                            }
                        }
                    }
                }
            }
            copied.sort();
            copied.dedup();
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
            identity.push_str(&format!(" \"generator\": \"{generator}\",\n"));
            identity.push_str(&format!(" \"api\": \"{behavior_api}\",\n"));
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
            identity.push_str(&format!(" \"roles\": {},\n", lowered.roles_json));
            identity.push_str(" \"assets\": [\n");
            let asset_rows = copied
                .iter()
                .map(|(sha, file, declared)| {
                    format!(
                        "   {{\"sha256\": \"{sha}\", \"file\": \"{file}\", \"declared\": \"{declared}\"}}"
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n");
            identity.push_str(&asset_rows);
            identity.push_str("\n ],\n");
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = consumer_for(&product);
            identity.push_str(" \"provenance\": {\n");
            identity.push_str(&format!("  \"product\": \"{product}\",\n"));
            identity.push_str(&format!("  \"consumer\": \"{consumer}\",\n"));
            identity.push_str(
                "  \"consumption\": \"include_str! frozen scene.agscene + world.at + assets/<sha>.glb; the harness writes the project (scene + assets) under target and loads via Storage::load_project; variants feed scripts/<sha>.at where <sha> = scene.behavior.source.sha256\",\n",
            );
            identity.push_str(
                "  \"load_verification\": \"the load path hash-verifies the behavior source and every assets/<sha>.glb against the scene manifest; GLB geometry admission runs in the driver preflight decoder before allocation\",\n",
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
            let source_note = match &lowered.user_source {
                Some(_) => " + world.at",
                None => "",
            };
            println!(
                "node2d-lower[components-3d]: wrote scene.agscene{source_note} + closure/{} + assets/{} (source sha {})",
                names.len(),
                copied.len(),
                lowered.source_sha256
            );
}

fn components3d_consumer(product: &str) -> &'static str {
    match product {
        "g02" | "g03" | "g03-parent" | "g03-hidden" | "twin" | "envelope8" | "rows129"
        | "rows129-ok" | "fault-nine" | "budget-full" | "lifecycle" | "model-param" => {
            "experimental/autographics-auto-driver/tests/node3d_basic_flow.rs"
        }
        "neg-bad-props" | "neg-wrong-dim" | "neg-cycle" | "neg-param-write" => {
            "experimental/autographics-node2d-pilot/tests/node3d_basic_gates.rs (fixture-level; lowering rejection)"
        }
        _ => "unrecorded (this out dir has no registered flow consumer)",
    }
}

fn runtime_behavior_api(project: &autographics_node2d_pilot::load::Project) -> &'static str {
    // 与降层同规则：camera3d -> runtime_camera3d/1；character3d ->
    // character3d/1；其它行为场景 -> behavior/1（无行为场景产品不读该串）。
    let has_camera = project
        .modules
        .values()
        .any(|m| m.nodes.iter().any(|n| n.camera3d.is_some()));
    let has_character = project
        .modules
        .values()
        .any(|m| m.nodes.iter().any(|n| n.character3d.is_some()));
    if has_camera {
        "autographics.runtime_camera3d/1"
    } else if has_character {
        "autographics.character3d/1"
    } else {
        "autographics.behavior/1"
    }
}

fn components3d_events_consumer(product: &str) -> &'static str {
    match product {
        "g11-main" | "g11-alt" | "g11-triple" | "g11-geometry" | "g11-oldgen" | "g11-stale"
        | "sensor" => {
            "experimental/autographics-auto-driver/tests/node3d_events_flow.rs"
        }
        "neg-slot-nine" | "neg-pair-nine" | "neg-template-instanced" | "neg-template-params"
        | "neg-template-model" | "neg-template-two-leaves" | "neg-template-solid"
        | "neg-template-builtin-dyn" | "neg-pool-nonroot" | "neg-unknown-slot"
        | "neg-unknown-template" | "neg-unknown-pair" | "neg-bind-non-const-subject"
        | "neg-bind-shape" | "neg-bind-radius" => {
            "experimental/autographics-node2d-pilot/tests/node3d_events_gates.rs (fixture-level; lowering rejection)"
        }
        _ => "unrecorded (this out dir has no registered flow consumer)",
    }
}

fn components3d_presentation_consumer(product: &str) -> &'static str {
    match product {
        "g10-textures" | "g10-textures-alt" | "g10-animation" | "g10-animation-alt" => {
            "experimental/autographics-auto-driver/tests/node3d_presentation_flow.rs"
        }
        "g10-textures-frozen" | "g10-textures-missvis" | "g10-textures-altimage" => {
            "experimental/autographics-auto-driver/tests/node3d_presentation_oracles.rs"
        }
        "g10-animation-frozen" | "g10-animation-extra-read" | "g10-animation-neutral" => {
            "experimental/autographics-auto-driver/tests/node3d_presentation_oracles_anim.rs"
        }
        _ => "unrecorded (this out dir has no registered flow consumer)",
    }
}

fn components3d_bindings_consumer(product: &str) -> &'static str {
    match product {
        "bg11-main" | "bg11-alt" | "bg11-geometry" | "bg11-oldgen" | "bg11-stale"
        | "sensor2" => {
            "experimental/autographics-auto-driver/tests/node3d_bindings_flow.rs"
        }
        "bneg-kind-mix" | "bneg-actor-bare" | "bneg-actor-unknown" | "bneg-actor-elem"
        | "bneg-slot-unknown" | "bneg-pair-unknown" | "bneg-template-unknown"
        | "bneg-ref-write" | "bneg-msg-payload" | "bneg-forward-kind"
        | "bneg-pool-nonroot" | "bneg-actor-group" => {
            "experimental/autographics-node2d-pilot/tests/node3d_bindings_gates.rs (fixture-level; lowering rejection)"
        }
        _ => "unrecorded (this out dir has no registered flow consumer)",
    }
}

fn components3d_runtime_consumer(product: &str) -> &'static str {
    match product {
        "g07a" | "g07a-fast" | "g07b" | "g08" | "g08-alt" | "static-actors" | "events3d"
        | "wide128" | "deep127" | "two-moves" | "stale-token" => {
            "experimental/autographics-auto-driver/tests/node3d_runtime_flow.rs"
        }
        "neg-two-characters" | "neg-camera-without-character" | "neg-camera-non-root"
        | "neg-solid-dynamic" | "neg-solid-on-character" | "neg-character-shape"
        | "neg-character-dyn" | "neg-solid-builtin" | "neg-camera-domain" => {
            "experimental/autographics-node2d-pilot/tests/node3d_runtime_gates.rs (fixture-level; lowering rejection)"
        }
        _ => "unrecorded (this out dir has no registered flow consumer)",
    }
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
                Some("queries-2d") => Profile::Queries2D,
                Some("sprites-2d") => Profile::Sprites2D,
                Some("components-2d-game") => Profile::Components2DGame,
                Some("components-3d") => Profile::Components3D,
                Some("components-3d-runtime") => Profile::Components3DRuntime,
                Some("components-3d-events") => Profile::Components3DEvents,
                Some("components-3d-bindings") => Profile::Components3DBindings,
                Some("components-3d-presentation") => Profile::Components3DPresentation,
                Some("presentation-2d") => Profile::Presentation2D,
                Some("components-2d-runtime") => Profile::Components2DRuntime,
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
    // PLAN-034：受检资源元数据（resource-meta 产出）。缺省 = 声明面
    // （resources-request.json）；提供 = 最终产品面。
    let resources_arg = args
        .iter()
        .position(|a| a == "--resources")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());
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
        Profile::Sprites2D => {
            let lowered = autographics_node2d_pilot::lower_project_sprites(&project)
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
            // 图像资产字节（完整闭包：身份 = 原始字节；字节等值复制）。
            std::fs::create_dir_all(format!("{out}/assets")).expect("assets dir");
            let mut copied: Vec<(String, String)> = Vec::new();
            for module in project.modules.values() {
                for r in &module.resources {
                    if let autographics_node2d_pilot::load::ResourceKind::Image {
                        file,
                        sha256,
                        byte_length,
                        ..
                    } = &r.kind
                    {
                        let bytes = std::fs::read(
                            std::path::Path::new(entry)
                                .parent()
                                .expect("entry parent")
                                .join(file),
                        )
                        .unwrap_or_else(|e| panic!("read image {file}: {e}"));
                        assert_eq!(bytes.len(), *byte_length, "image byte drift: {file}");
                        assert_eq!(
                            autographics_node2d_pilot::scene::sha256_hex(&bytes),
                            *sha256,
                            "image sha drift: {file}"
                        );
                        let dest = format!("{out}/assets/{sha256}.png");
                        std::fs::write(&dest, &bytes).expect("asset copy");
                        copied.push((sha256.clone(), file.clone()));
                    }
                }
            }
            copied.sort();
            copied.dedup();
            let mut names: Vec<_> = project.modules.keys().collect();
            names.sort();
            let mut modules = String::new();
            let mut closure = String::new();
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                modules.push_str(&format!(
                    "  \"module:{}\": \"{}\",
",
                    name,
                    autographics_node2d_pilot::scene::sha256_hex(bytes)
                ));
                closure.push_str(&format!(
                    "  {{\"name\": \"{name}\", \"sha256\": \"{}\", \"bytes\": {}}},
",
                    autographics_node2d_pilot::scene::sha256_hex(bytes),
                    bytes.len()
                ));
            }
            let mut identity = String::from(
                "{
",
            );
            identity.push_str(
                " \"generator\": \"autographics-node2d-pilot plan022 sprites-2d\",
",
            );
            identity.push_str(
                " \"api\": \"autographics.sprites2d/1\",
",
            );
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",
 \"user_source_bytes\": {},
",
                lowered.source_sha256, lowered.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(
                " \"closure\": [
",
            );
            identity.push_str(closure.trim_end_matches(
                ",
",
            ));
            identity.push_str(
                "
 ],
",
            );
            identity.push_str(&format!(
                " \"roles\": {},
",
                lowered.roles_json
            ));
            identity.push_str(
                " \"assets\": [
",
            );
            let asset_rows = copied
                .iter()
                .map(|(sha, declared)| {
                    format!(
                        "   {{\"sha256\": \"{sha}\", \"file\": \"{sha}.png\", \"declared\": \"{declared}\"}}"
                    )
                })
                .collect::<Vec<_>>()
                .join(
                    "\n",
                );
            identity.push_str(&asset_rows);
            identity.push_str(
                "
 ],
",
            );
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = match product.as_str() {
                "atlas-g09" | "atlas-alt" | "atlas-flip-control" | "atlas-alpha-control" | "runtime-oob"
                | "atlas-slot" | "atlas-slot-control" | "character-g09" | "character-alt"
                | "character-wrong-air" | "character-budget-plus" | "pair-isolated" => {
                    "experimental/autographics-auto-driver/tests/node_sprites_flow.rs"
                }
                "slot-9" | "neg-unknown-spawn" | "neg-image-frame-oob" | "neg-image-type" => {
                    "experimental/autographics-node2d-pilot/tests/node_sprites_gates.rs (fixture-level; lowering rejection)"
                }
                _ => "unrecorded (this out dir has no registered flow consumer)",
            };
            identity.push_str(
                " \"provenance\": {
",
            );
            identity.push_str(&format!(
                "  \"product\": \"{product}\",
"
            ));
            identity.push_str(&format!(
                "  \"consumer\": \"{consumer}\",
"
            ));
            identity.push_str(
                "  \"consumption\": \"include_str! frozen scene.agscene + world.at; the harness writes the project (scene + assets/<sha>.png) under target and loads via Storage::load_project; variants feed scripts/<sha>.at\",
",
            );
            identity.push_str(
                "  \"load_verification\": \"the load path hash-verifies behavior source and every assets/<sha>.png against the scene manifest; damaged copies are rejected at load\",
",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"
",
                lowered.source_sha256
            ));
            identity.push_str(
                " }
}
",
            );
            std::fs::write(format!("{out}/identity.json"), identity).expect("identity.json");
            let closure_dir = format!("{out}/closure");
            std::fs::create_dir_all(&closure_dir).expect("closure dir");
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                std::fs::write(format!("{closure_dir}/{name}.at"), bytes).expect("closure file");
            }
            println!(
                "node2d-lower[sprites-2d]: wrote world.at ({} bytes, sha {}) + scene.agscene + closure/{} + assets/{}",
                lowered.source_bytes,
                lowered.source_sha256,
                names.len(),
                copied.len()
            );
        }
        Profile::Components2D | Profile::Queries2D => {
            let queries = matches!(profile, Profile::Queries2D);
            let lowered = if queries {
                autographics_node2d_pilot::lower_project_queries(&project).unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                })
            } else {
                autographics_node2d_pilot::lower_project_components(&project).unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                })
            };
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
            if queries {
                identity.push_str(
                    " \"generator\": \"autographics-node2d-pilot plan021 queries-2d\",\n",
                );
            } else {
                identity.push_str(
                    " \"generator\": \"autographics-node2d-pilot plan020 components-2d\",\n",
                );
            }
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
                "q-segment" | "q-sweep" | "q-cross" | "q-cross-plus" | "q-changed" | "q-g05seg"
                | "q-g05sweep" | "q-g05changed" => {
                    "experimental/autographics-auto-driver/tests/node_queries_flow.rs"
                }
                "q-helpers" => {
                    "experimental/autographics-node2d-pilot/tests/node_queries_gates.rs (fixture-level; no driver flow consumer)"
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
                "node2d-lower[{}]: wrote world.at ({} bytes, sha {}) + scene.agscene + closure/{}",
                if queries {
                    "queries-2d"
                } else {
                    "components-2d"
                },
                lowered.source_bytes,
                lowered.source_sha256,
                names.len()
            );
        }
        Profile::Components2DGame => {
            let entry_path = std::path::Path::new(entry);
            let entry_dir = entry_path.parent().unwrap_or(entry_path);
            let lowered =
                autographics_node2d_pilot::lower_project_components_game(&project, entry_dir)
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e}");
                        std::process::exit(3);
                    });
            std::fs::write(
                format!("{out}/world.at"),
                lowered.base.user_source.expect("source"),
            )
            .expect("world.at");
            std::fs::write(format!("{out}/scene.agscene"), &lowered.base.scene_json)
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
            // game 资产进 closure 清单与 closure/assets/（A→B 冷再生）。
            let assets_dir = format!("{out}/closure/assets");
            std::fs::create_dir_all(&assets_dir).expect("closure assets dir");
            let mut asset_entries = String::new();
            for a in &lowered.assets {
                let bytes = std::fs::read(&a.src).expect("asset bytes");
                std::fs::write(format!("{assets_dir}/{}", a.name), &bytes).expect("asset copy");
                asset_entries.push_str(&format!(
                    "  {{\"name\": \"asset:{}\", \"sha256\": \"{}\", \"bytes\": {}}},\n",
                    a.name, a.sha256, a.bytes
                ));
            }
            let mut identity = String::from("{\n");
            identity.push_str(
                " \"generator\": \"autographics-node2d-pilot plan023 components-2d-game\",\n",
            );
            identity.push_str(" \"api\": \"autographics.hud2d/1\",\n");
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",\n \"user_source_bytes\": {},\n",
                lowered.base.source_sha256, lowered.base.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [\n");
            let combined = format!("{closure}{asset_entries}");
            identity.push_str(combined.trim_end_matches(",\n"));
            identity.push_str("\n ],\n");
            identity.push_str(&format!(" \"roles\": {},\n", lowered.base.roles_json));
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = match product.as_str() {
                "game" => {
                    "experimental/autographics-auto-driver/tests/t03_game_gates.rs (V-01 byte pin + V-02 Hud2V1 admission)"
                }
                _ => "unrecorded (this out dir has no registered flow consumer)",
            };
            identity.push_str(" \"provenance\": {\n");
            identity.push_str(&format!("  \"product\": \"{product}\",\n"));
            identity.push_str(&format!("  \"consumer\": \"{consumer}\",\n"));
            identity.push_str(
                "  \"consumption\": \"include_str! frozen scene.agscene + world.at; the harness writes world.at to scripts/<sha>.at where <sha> = scene.behavior.source.sha256\",\n",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"\n",
                lowered.base.source_sha256
            ));
            identity.push_str(" }\n}\n");
            std::fs::write(format!("{out}/identity.json"), identity).expect("identity.json");
            let closure_dir = format!("{out}/closure");
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                std::fs::write(format!("{closure_dir}/{name}.at"), bytes).expect("closure file");
            }
            println!(
                "node2d-lower[components-2d-game]: wrote world.at ({} bytes, sha {}) + scene.agscene + closure/{}+assets/{}",
                lowered.base.source_bytes,
                lowered.base.source_sha256,
                names.len(),
                lowered.assets.len()
            );
        }
        Profile::Presentation2D => {
            let entry_path = std::path::Path::new(entry);
            let entry_dir = entry_path.parent().unwrap_or(entry_path);
            let lowered =
                autographics_node2d_pilot::lower_project_presentation(&project, entry_dir)
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e}");
                        std::process::exit(3);
                    });
            std::fs::write(
                format!("{out}/world.at"),
                lowered.base.user_source.expect("source"),
            )
            .expect("world.at");
            std::fs::write(format!("{out}/scene.agscene"), &lowered.base.scene_json)
                .expect("scene.agscene");
            let scene_api = if lowered.base.scene_json.contains("\"autographics.hud2d/1\"") {
                "autographics.hud2d/1"
            } else {
                "autographics.presentation2d/1"
            };
            let mut names: Vec<_> = project.modules.keys().collect();
            names.sort();
            let mut modules = String::new();
            let mut closure = String::new();
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                modules.push_str(&format!(
                    "  \"module:{}\": \"{}\",
",
                    name,
                    autographics_node2d_pilot::scene::sha256_hex(bytes)
                ));
                closure.push_str(&format!(
                    "  {{\"name\": \"{name}\", \"sha256\": \"{}\", \"bytes\": {}}},
",
                    autographics_node2d_pilot::scene::sha256_hex(bytes),
                    bytes.len()
                ));
            }
            // 资产进 closure 清单与 closure/assets/（A→B 冷再生）。
            let assets_dir = format!("{out}/closure/assets");
            std::fs::create_dir_all(&assets_dir).expect("closure assets dir");
            let mut asset_entries = String::new();
            for a in &lowered.assets {
                let bytes = std::fs::read(&a.src).expect("asset bytes");
                std::fs::write(format!("{assets_dir}/{}", a.name), &bytes).expect("asset copy");
                asset_entries.push_str(&format!(
                    "  {{\"name\": \"asset:{}\", \"sha256\": \"{}\", \"bytes\": {}}},
",
                    a.name, a.sha256, a.bytes
                ));
            }
            let mut identity = String::from("{
");
            identity.push_str(
                " \"generator\": \"autographics-node2d-pilot plan025 presentation-2d\",
",
            );
            identity.push_str(&format!(" \"api\": \"{scene_api}\",
"));
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",
 \"user_source_bytes\": {},
",
                lowered.base.source_sha256, lowered.base.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [
");
            let combined = format!("{closure}{asset_entries}");
            identity.push_str(combined.trim_end_matches(",
"));
            identity.push_str("
 ],
");
            identity.push_str(&format!(" \"roles\": {},
", lowered.base.roles_json));
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = match product.as_str() {
                "notify" => {
                    "experimental/autographics-auto-driver/tests/node_presentation_flow.rs (V-02 actor-free Auto/GPU/PCM)"
                }
                "game" => {
                    "experimental/autographics-auto-driver/tests/node_presentation_flow.rs (V-03 author G12-equivalent branches)"
                }
                "four" => {
                    "experimental/autographics-auto-driver/tests/node_presentation_flow.rs (V-04 four-request bound + fifth named rejection)"
                }
                _ => "unrecorded (this out dir has no registered flow consumer)",
            };
            identity.push_str(" \"provenance\": {
");
            identity.push_str(&format!("  \"product\": \"{product}\",
"));
            identity.push_str(&format!("  \"consumer\": \"{consumer}\",
"));
            identity.push_str(
                "  \"consumption\": \"include_str! frozen scene.agscene + world.at; the harness writes world.at to scripts/<sha>.at where <sha> = scene.behavior.source.sha256\",
",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"
",
                lowered.base.source_sha256
            ));
            identity.push_str(" }
}
");
            std::fs::write(format!("{out}/identity.json"), identity).expect("identity.json");
            let closure_dir = format!("{out}/closure");
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                std::fs::write(format!("{closure_dir}/{name}.at"), bytes).expect("closure file");
            }
            println!(
                "node2d-lower[presentation-2d]: wrote world.at ({} bytes, sha {}) + scene.agscene + closure/{}+assets/{}",
                lowered.base.source_bytes,
                lowered.base.source_sha256,
                names.len(),
                lowered.assets.len()
            );
        }
        Profile::Components2DRuntime => {
            let entry_path = std::path::Path::new(entry);
            let entry_dir = entry_path.parent().unwrap_or(entry_path);
            let lowered = autographics_node2d_pilot::lower_project_components2d_runtime(
                &project,
                entry_dir,
            )
            .unwrap_or_else(|e| {
                eprintln!("node2d-lower: {e}");
                std::process::exit(3);
            });
            std::fs::write(
                format!("{out}/world.at"),
                lowered.base.user_source.expect("source"),
            )
            .expect("world.at");
            std::fs::write(format!("{out}/scene.agscene"), &lowered.base.scene_json)
                .expect("scene.agscene");
            let mut names: Vec<_> = project.modules.keys().collect();
            names.sort();
            let mut modules = String::new();
            let mut closure = String::new();
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                modules.push_str(&format!(
                    "  \"module:{}\": \"{}\",
",
                    name,
                    autographics_node2d_pilot::scene::sha256_hex(bytes)
                ));
                closure.push_str(&format!(
                    "  {{\"name\": \"{name}\", \"sha256\": \"{}\", \"bytes\": {}}},
",
                    autographics_node2d_pilot::scene::sha256_hex(bytes),
                    bytes.len()
                ));
            }
            let assets_dir = format!("{out}/closure/assets");
            std::fs::create_dir_all(&assets_dir).expect("closure assets dir");
            let mut asset_entries = String::new();
            for a in &lowered.assets {
                let bytes = std::fs::read(&a.src).expect("asset bytes");
                std::fs::write(format!("{assets_dir}/{}", a.name), &bytes).expect("asset copy");
                asset_entries.push_str(&format!(
                    "  {{\"name\": \"asset:{}\", \"sha256\": \"{}\", \"bytes\": {}}},
",
                    a.name, a.sha256, a.bytes
                ));
            }
            let mut identity = String::from("{
");
            identity.push_str(
                " \"generator\": \"autographics-node2d-pilot plan032 components-2d-runtime\",
",
            );
            identity.push_str(" \"api\": \"autographics.components2d/1\",
");
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",
 \"user_source_bytes\": {},
",
                lowered.base.source_sha256, lowered.base.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [
");
            let combined = format!("{closure}{asset_entries}");
            identity.push_str(combined.trim_end_matches(",
"));
            identity.push_str("
 ],
");
            identity.push_str(&format!(" \"roles\": {},
", lowered.base.roles_json));
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = match product.as_str() {
                "g2" => {
                    "experimental/autographics-auto-driver/tests/node2d_full_game_flow.rs + node2d_full_game_media.rs (frozen G2 route/budget/GPU/PCM)"
                }
                _ => "unrecorded (this out dir has no registered flow consumer)",
            };
            identity.push_str(" \"provenance\": {
");
            identity.push_str(&format!("  \"product\": \"{product}\",
"));
            identity.push_str(&format!("  \"consumer\": \"{consumer}\",
"));
            identity.push_str(
                "  \"consumption\": \"include_str! frozen scene.agscene + world.at; the harness writes world.at to scripts/<sha>.at where <sha> = scene.behavior.source.sha256\",
",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"
",
                lowered.base.source_sha256
            ));
            identity.push_str(" }
}
");
            std::fs::write(format!("{out}/identity.json"), identity).expect("identity.json");
            let closure_dir = format!("{out}/closure");
            for name in &names {
                let bytes = project.modules[*name].source.as_bytes();
                std::fs::write(format!("{closure_dir}/{name}.at"), bytes).expect("closure file");
            }
            println!(
                "node2d-lower[components-2d-runtime]: wrote world.at ({} bytes, sha {}) + scene.agscene + closure/{}+assets/{}",
                lowered.base.source_bytes,
                lowered.base.source_sha256,
                names.len(),
                lowered.assets.len()
            );
        }
        Profile::Components3D => {
            let lowered = autographics_node2d_pilot::lower_project_components3d(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            write_components3d_products(
                &out,
                &entry,
                &project,
                lowered,
                "autographics-node2d-pilot plan024 components-3d",
                "autographics.behavior/1",
                components3d_consumer,
                None,
            );
        }
        Profile::Components3DRuntime => {
            let lowered = autographics_node2d_pilot::lower_project_components3d_runtime(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            write_components3d_products(
                &out,
                &entry,
                &project,
                lowered,
                "autographics-node2d-pilot plan027 components-3d-runtime",
                runtime_behavior_api(&project),
                components3d_runtime_consumer,
                None,
            );
        }
        Profile::Components3DEvents => {
            let lowered = autographics_node2d_pilot::lower_project_components3d_events(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            write_components3d_products(
                &out,
                &entry,
                &project,
                lowered,
                "autographics-node2d-pilot plan028 components-3d-events",
                "autographics.triggers3d/1",
                components3d_events_consumer,
                None,
            );
        }
        Profile::Components3DBindings => {
            let lowered = autographics_node2d_pilot::lower_project_components3d_bindings(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            write_components3d_products(
                &out,
                &entry,
                &project,
                lowered,
                "autographics-node2d-pilot plan030 components-3d-bindings",
                "autographics.triggers3d/1",
                components3d_bindings_consumer,
                None,
            );
        }
        Profile::Components3DPresentation => {
            match resources_arg {
                None => {
                    // pass A（声明面）：声明齐全性与文件身份校验已在
                    // load_project 内完成；本面只产出受检元数据请求。
                    let request =
                        autographics_node2d_pilot::resources_request_json(&project);
                    std::fs::write(
                        format!("{out}/resources-request.json"),
                        &request,
                    )
                    .expect("resources-request.json");
                    let n = request.matches("\"animated\"").count();
                    println!(
                        "node2d-lower[components-3d-presentation]: declared {n} texture/animation resources -> resources-request.json; run resource-meta, then rerun with --resources <resources.json>"
                    );
                }
                Some(meta_path) => {
                    // pass B（产品面）：受检元数据对账 + 全量产品。
                    let meta = autographics_node2d_pilot::load_resources_meta(
                        std::path::Path::new(meta_path),
                        &project,
                    )
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e}");
                        std::process::exit(3);
                    });
                    let (lowered, api) = autographics_node2d_pilot::
                        lower_project_components3d_presentation(&project, &meta)
                        .unwrap_or_else(|e| {
                            eprintln!("node2d-lower: {e}");
                            std::process::exit(3);
                        });
                    write_components3d_products(
                        &out,
                        &entry,
                        &project,
                        lowered,
                        "autographics-node2d-pilot plan034 components-3d-presentation",
                        &api,
                        components3d_presentation_consumer,
                        Some(&meta),
                    );
                }
            }
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
