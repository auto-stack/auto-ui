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
        "usage: node2d-lower [--profile legacy|node-ui|g11-2d|scene-2d|character-2d|camera-2d|components-2d|components-2d-game|queries-2d|queries-3d|editor-projects|sprites-2d|components-3d|components-3d-runtime|components-3d-events|components-3d-bindings|components-3d-presentation|components-3d-audio-hud|presentation-2d|components-2d-runtime] [--diagnostics-map] --entry <dir>/main.at --out <dir>"
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
    std::fs::write(format!("{out}/scene.agscene"), &lowered.scene_json).expect("scene.agscene");
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

fn queries3d_consumer(product: &str) -> &'static str {
    match product {
        "q3-seg" | "q3-sweep" | "q3-changed" | "q3-cross" | "q3-snapshot" | "q3-budget" => {
            "experimental/autographics-auto-driver/tests/node3d_queries_flow.rs"
        }
        "q3-gates" => {
            "experimental/autographics-node2d-pilot/tests/node3d_queries_gates.rs (fixture-level; no driver flow consumer)"
        }
        _ => "unrecorded (this out dir has no registered flow consumer)",
    }
}

/// PLAN-040 editor-products consumer registration（受检交接身份）。
fn editor_projects_consumer(product: &str) -> &'static str {
    match product {
        "g04-editor-2d" | "g04-editor-3d" => {
            "experimental/autographics-auto-driver/tests/node_editor_projects.rs + pilot tests/node_editor_gates.rs"
        }
        _ => "unrecorded (this out dir has no registered flow consumer)",
    }
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

fn components3d_model_objects_consumer(product: &str) -> &'static str {
    match product {
        "g12-model" => "experimental/autographics-auto-driver/tests/node3d_model_objects_flow.rs",
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

fn components3d_audio_hud_consumer(product: &str) -> &'static str {
    match product {
        "actorfree" | "batch-tick" | "g3-full-loop" => {
            "experimental/autographics-auto-driver/tests/node3d_audio_hud_flow.rs"
        }
        "neg-*" => {
            "experimental/autographics-node2d-pilot/tests/node3d_audio_hud_gates.rs (fixture-level; lowering rejection)"
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
                Some("queries-3d") => Profile::Queries3D,
                Some("editor-projects") => Profile::EditorProjects,
                Some("sprites-2d") => Profile::Sprites2D,
                Some("components-2d-game") => Profile::Components2DGame,
                Some("components-3d") => Profile::Components3D,
                Some("components-3d-runtime") => Profile::Components3DRuntime,
                Some("components-3d-events") => Profile::Components3DEvents,
                Some("components-3d-bindings") => Profile::Components3DBindings,
                Some("components-3d-presentation") => Profile::Components3DPresentation,
                Some("model-objects-3d") => Profile::ModelObjects3D,
                Some("components-3d-audio-hud") => Profile::Components3DAudioHud,
                Some("presentation-2d") => Profile::Presentation2D,
                Some("components-2d-runtime") => Profile::Components2DRuntime,
                Some("node-ui") => Profile::NodeUi,
                Some("mixed-ui-2d-3d") => Profile::MixedUi2d3d,
                Some("node-ui-mixed-regions") => Profile::MixedRegions,
                _ => usage(),
            }
        }
    };
    // PLAN-044：可选源映射旁件（T-02 冻结工作名）。仅 components-2d /
    // queries-2d 发射路径支持；默认 CLI 无此旗标 = 无旁件，world/API
    // 源字节与旧档行为不变。
    let diagnostics_map = args.iter().any(|a| a == "--diagnostics-map");
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
    // PLAN-039：queries-3d 在自己的 arm 内以双 view 解析载入（全局
    // 预载走默认 dialect，会先拒双 view）。PLAN-040 editor-projects 沿同一
    // 双出口准入，在自己的 arm 内载入。PLAN-049 mixed-ui-2d-3d 同理以混合
    // dialect 载入（view 内嵌入区域；node 级双出口守卫不放宽）。
    let project = if matches!(
        profile,
        Profile::Queries3D | Profile::EditorProjects | Profile::MixedUi2d3d | Profile::MixedRegions
    ) {
        autographics_node2d_pilot::Project::default()
    } else {
        autographics_node2d_pilot::load_project(std::path::Path::new(entry)).unwrap_or_else(|e| {
            eprintln!("node2d-lower: {e}");
            std::process::exit(2);
        })
    };
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
        Profile::Queries3D => {
            let project =
                autographics_node2d_pilot::load_project_dual(std::path::Path::new(entry), true)
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e:?}");
                        std::process::exit(2);
                    });
            let lowered = autographics_node2d_pilot::lower_project_queries3d(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            // 身份段 API 从派生结果读回（segment-only → behavior/3；含
            // sweep → behavior/4）。
            let behavior_api = if lowered.scene_json.contains("autographics.behavior/4") {
                autographics_node2d_pilot::BEHAVIOR_API_V4
            } else {
                autographics_node2d_pilot::BEHAVIOR_API_V3
            };
            write_components3d_products(
                out,
                entry,
                &project,
                lowered,
                "autographics-node2d-pilot plan039 queries-3d",
                behavior_api,
                queries3d_consumer,
                None,
            );
            println!(
                "node2d-lower[queries-3d]: wrote world.at + scene.agscene (api {behavior_api}) + identity.json + closure/ + assets/"
            );
        }
        Profile::EditorProjects => {
            // PLAN-040：与 queries-3d 同一机制/准入/产物规则，仅身份串不同
            // （plan040-editor-projects）；G04 纯声明双维编辑工程入口。
            let project =
                autographics_node2d_pilot::load_project_dual(std::path::Path::new(entry), true)
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e:?}");
                        std::process::exit(2);
                    });
            let lowered = autographics_node2d_pilot::lower_project_editor_projects(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            let behavior_api = if lowered.scene_json.contains("autographics.behavior/4") {
                autographics_node2d_pilot::BEHAVIOR_API_V4
            } else {
                autographics_node2d_pilot::BEHAVIOR_API_V3
            };
            write_components3d_products(
                out,
                entry,
                &project,
                lowered,
                "autographics-node2d-pilot plan040 editor-projects",
                behavior_api,
                editor_projects_consumer,
                None,
            );
            println!(
                "node2d-lower[editor-projects]: wrote scene.agscene + identity.json + closure/ + assets/ (behavior:null when the source is purely declarative)"
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
            if diagnostics_map {
                let world = std::fs::read_to_string(format!("{out}/world.at")).expect("world.at");
                let generator = if queries {
                    "autographics-node2d-pilot plan021 queries-2d"
                } else {
                    "autographics-node2d-pilot plan020 components-2d"
                };
                let map = autographics_node2d_pilot::source_map::build_source_map(
                    &project,
                    &world,
                    generator,
                    &lowered.source_sha256,
                )
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: diagnostics map: {e}");
                    std::process::exit(4);
                });
                std::fs::write(format!("{out}/source-map.json"), map.to_json())
                    .expect("source-map.json");
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
            let mut identity = String::from(
                "{
",
            );
            identity.push_str(
                " \"generator\": \"autographics-node2d-pilot plan025 presentation-2d\",
",
            );
            identity.push_str(&format!(
                " \"api\": \"{scene_api}\",
"
            ));
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",
 \"user_source_bytes\": {},
",
                lowered.base.source_sha256, lowered.base.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(
                " \"closure\": [
",
            );
            let combined = format!("{closure}{asset_entries}");
            identity.push_str(combined.trim_end_matches(
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
                lowered.base.roles_json
            ));
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
                "  \"consumption\": \"include_str! frozen scene.agscene + world.at; the harness writes world.at to scripts/<sha>.at where <sha> = scene.behavior.source.sha256\",
",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"
",
                lowered.base.source_sha256
            ));
            identity.push_str(
                " }
}
",
            );
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
            let lowered =
                autographics_node2d_pilot::lower_project_components2d_runtime(&project, entry_dir)
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
            let mut identity = String::from(
                "{
",
            );
            identity.push_str(
                " \"generator\": \"autographics-node2d-pilot plan032 components-2d-runtime\",
",
            );
            identity.push_str(
                " \"api\": \"autographics.components2d/1\",
",
            );
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",
 \"user_source_bytes\": {},
",
                lowered.base.source_sha256, lowered.base.source_bytes
            ));
            identity.push_str(&modules);
            identity.push_str(
                " \"closure\": [
",
            );
            let combined = format!("{closure}{asset_entries}");
            identity.push_str(combined.trim_end_matches(
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
                lowered.base.roles_json
            ));
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
                "  \"consumption\": \"include_str! frozen scene.agscene + world.at; the harness writes world.at to scripts/<sha>.at where <sha> = scene.behavior.source.sha256\",
",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"
",
                lowered.base.source_sha256
            ));
            identity.push_str(
                " }
}
",
            );
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
        Profile::ModelObjects3D => {
            match resources_arg {
                None => {
                    // pass A（声明面）：GLB 闭包元数据请求（034 同一公开面）。
                    let request = autographics_node2d_pilot::resources_request_json(&project);
                    std::fs::write(format!("{out}/resources-request.json"), &request)
                        .expect("resources-request.json");
                    let n = request.matches("\"animated\"").count();
                    println!(
                        "node2d-lower[model-objects-3d]: declared {n} texture/animation resources -> resources-request.json; run resource-meta, then rerun with --resources <resources.json>"
                    );
                }
                Some(meta_path) => {
                    // pass B（产品面）：受检元数据对账 + 模板闭包键 + 全量产品。
                    let meta = autographics_node2d_pilot::load_resources_meta(
                        std::path::Path::new(meta_path),
                        &project,
                    )
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e}");
                        std::process::exit(3);
                    });
                    let lowered =
                        autographics_node2d_pilot::lower_project_components3d_model_objects(
                            &project, &meta,
                        )
                        .unwrap_or_else(|e| {
                            eprintln!("node2d-lower: {e}");
                            std::process::exit(3);
                        });
                    write_components3d_products(
                        &out,
                        &entry,
                        &project,
                        lowered,
                        "autographics-node2d-pilot plan043 model-objects-3d",
                        "autographics.triggers3d/1",
                        components3d_model_objects_consumer,
                        Some(&meta),
                    );
                }
            }
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
                    let request = autographics_node2d_pilot::resources_request_json(&project);
                    std::fs::write(format!("{out}/resources-request.json"), &request)
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
                    let (lowered, api) =
                        autographics_node2d_pilot::lower_project_components3d_presentation(
                            &project, &meta,
                        )
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
        Profile::Components3DAudioHud => {
            match resources_arg {
                None => {
                    // pass A（声明面）：GLB 闭包元数据请求（034 同一面）。
                    let request = autographics_node2d_pilot::resources_request_json(&project);
                    std::fs::write(format!("{out}/resources-request.json"), &request)
                        .expect("resources-request.json");
                    let n = request.matches("\"animated\"").count();
                    println!(
                        "node2d-lower[components-3d-audio-hud]: declared {n} texture/animation resources -> resources-request.json; run resource-meta, then rerun with --resources <resources.json>"
                    );
                }
                Some(meta_path) => {
                    // pass B（产品面）：受检元数据对账 + 组合块 + 全量产品。
                    let meta = autographics_node2d_pilot::load_resources_meta(
                        std::path::Path::new(meta_path),
                        &project,
                    )
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e}");
                        std::process::exit(3);
                    });
                    let entry_dir = std::path::Path::new(&entry)
                        .parent()
                        .expect("entry parent")
                        .to_path_buf();
                    let lowered = autographics_node2d_pilot::lower_project_components3d_audio_hud(
                        &project, &entry_dir, &meta,
                    )
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e}");
                        std::process::exit(3);
                    });
                    write_components3d_products(
                        &out,
                        &entry,
                        &project,
                        lowered.base,
                        "autographics-node2d-pilot plan035 components-3d-audio-hud",
                        &lowered.api,
                        components3d_audio_hud_consumer,
                        Some(&meta),
                    );
                    for a in &lowered.audio {
                        if !a.in_scene {
                            continue;
                        }
                        let bytes = autographics_node2d_pilot::
                            lower_components3d_audio_hud::verify_audio_asset(a);
                        std::fs::write(format!("{out}/assets/{}", a.name), &bytes)
                            .expect("audio asset copy");
                    }
                }
            }
        }
        Profile::NodeUi => {
            // PLAN-048 基础 UI：ui.json（autographics.node-ui/1）+ world.at
            // （真实 Auto 业务源）+ identity.json + closure/（A→B 冷再生）。
            let lowered = autographics_node2d_pilot::lower_project_node_ui(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            std::fs::write(format!("{out}/world.at"), &lowered.user_source).expect("world.at");
            std::fs::write(format!("{out}/ui.json"), &lowered.ui_json).expect("ui.json");
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
                " \"generator\": \"{}\",\n",
                autographics_node2d_pilot::NODE_UI_GENERATOR
            ));
            identity.push_str(" \"api\": \"autographics.node-ui/1\",\n");
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",\n \"user_source_bytes\": {},\n \"ui_schema\": \"{}\",\n",
                lowered.source_sha256,
                lowered.source_bytes,
                autographics_node2d_pilot::NODE_UI_SCHEMA
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [\n");
            identity.push_str(closure.trim_end_matches(",\n"));
            identity.push_str("\n ],\n");
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = match product.as_str() {
                "ui-ab" | "ui-ab-alt" => {
                    "experimental/autographics-auto-driver/tests/node_ui_flow.rs + node_ui_storage.rs"
                }
                _ => "unrecorded (this out dir has no registered flow consumer)",
            };
            identity.push_str(" \"provenance\": {\n");
            identity.push_str(&format!("  \"product\": \"{product}\",\n"));
            identity.push_str(&format!("  \"consumer\": \"{consumer}\",\n"));
            identity.push_str(
                "  \"consumption\": \"include_str! frozen ui.json + world.at; the harness writes world.at to scripts/<sha>.at where <sha> = ui.world_sha256 and loads the behavior via SourceSnapshot (Api::NodeUi); the load path hash-verifies the behavior source; ui.json element/instance tables drive the real Iced adapter\",\n",
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
                "node2d-lower[node-ui]: wrote ui.json (schema {}) + world.at ({} bytes, sha {}) + closure/{}",
                autographics_node2d_pilot::NODE_UI_SCHEMA,
                lowered.source_bytes,
                lowered.source_sha256,
                names.len()
            );
        }
        Profile::MixedUi2d3d => {
            // PLAN-049 混合组合：ui.json（autographics.node-ui-mixed/1）+
            // world.at（与 048 字节同面）+ identity.json + closure/。混合
            // dialect 载入（view 内嵌入区域放行；旧档守卫字节不变）。
            let project =
                autographics_node2d_pilot::load_project_mixed(std::path::Path::new(entry))
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e:?}");
                        std::process::exit(2);
                    });
            let lowered = autographics_node2d_pilot::lower_project_node_mixed(&project)
                .unwrap_or_else(|e| {
                    eprintln!("node2d-lower: {e}");
                    std::process::exit(3);
                });
            std::fs::write(format!("{out}/world.at"), &lowered.user_source).expect("world.at");
            std::fs::write(format!("{out}/ui.json"), &lowered.ui_json).expect("ui.json");
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
                " \"generator\": \"{}\",\n",
                autographics_node2d_pilot::NODE_UI_MIXED_GENERATOR
            ));
            identity.push_str(" \"api\": \"autographics.node-ui-mixed/1\",\n");
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",\n \"user_source_bytes\": {},\n \"ui_schema\": \"{}\",\n",
                lowered.world_sha256,
                lowered.source_bytes,
                autographics_node2d_pilot::NODE_UI_MIXED_SCHEMA
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [\n");
            identity.push_str(closure.trim_end_matches(",\n"));
            identity.push_str("\n ],\n");
            identity.push_str(" \"provenance\": {\n");
            // 048 同一约定：product 只记目录名（byte-identical 再生成控制
            // 要求 temp 路径不进产物身份）。
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = match product.as_str() {
                "ui-ab-mixed" => {
                    "experimental/autographics-auto-driver/tests/node_mixed_views_{flow,storage,host}.rs + bin/mixed_ui_window.rs (AG021 candidate recipe)"
                }
                _ => "unrecorded (this out dir has no registered flow consumer)",
            };
            identity.push_str(&format!("  \"product\": \"{product}\",\n"));
            identity.push_str(&format!("  \"consumer\": \"{consumer}\",\n"));
            identity.push_str(
                "  \"consumption\": \"ui.json regions drive per-dimension child surfaces (Game2d/Scene3d) mounted Flat2d into the UI view; scene bindings read the SAME int getter ABI as text/condition/layout; region clicks dispatch through the shared _ui_click_ ABI; one Session owner, one app_start/app_update per explicit step\",\n",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"\n",
                lowered.world_sha256
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
                "node2d-lower[mixed-ui-2d-3d]: wrote ui.json (schema {}) + world.at ({} bytes, sha {}) + closure/{}",
                autographics_node2d_pilot::NODE_UI_MIXED_SCHEMA,
                lowered.source_bytes,
                lowered.world_sha256,
                names.len()
            );
        }
        Profile::MixedRegions => {
            // PLAN-054 组件/实例活区域：ui.json（node-ui-mixed-regions/2）+
            // world.at（与 048/049 字节同面）+ identity.json + closure/。
            // 混合 dialect 载入（view 内嵌入区域放行到任意 def；node 级
            // 双出口守卫不放宽；v1 身份/诊断逐字不变）。
            let project =
                autographics_node2d_pilot::load_project_mixed(std::path::Path::new(entry))
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e:?}");
                        std::process::exit(2);
                    });
            let lowered =
                autographics_node2d_pilot::lower_project_node_mixed_regions(&project)
                    .unwrap_or_else(|e| {
                        eprintln!("node2d-lower: {e}");
                        std::process::exit(3);
                    });
            std::fs::write(format!("{out}/world.at"), &lowered.user_source).expect("world.at");
            std::fs::write(format!("{out}/ui.json"), &lowered.ui_json).expect("ui.json");
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
                " \"generator\": \"{}\",\n",
                autographics_node2d_pilot::NODE_UI_MIXED_REGIONS_GENERATOR
            ));
            identity.push_str(" \"api\": \"autographics.node-ui-mixed-regions/2\",\n");
            identity.push_str(&format!(
                " \"user_source_sha256\": \"{}\",\n \"user_source_bytes\": {},\n \"ui_schema\": \"{}\",\n",
                lowered.world_sha256,
                lowered.source_bytes,
                autographics_node2d_pilot::NODE_UI_MIXED_REGIONS_SCHEMA
            ));
            identity.push_str(&modules);
            identity.push_str(" \"closure\": [\n");
            identity.push_str(closure.trim_end_matches(",\n"));
            identity.push_str("\n ],\n");
            identity.push_str(" \"provenance\": {\n");
            let product = std::path::Path::new(out)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unrecorded")
                .to_string();
            let consumer = match product.as_str() {
                "ui-dash-regions" => {
                    "experimental/autographics-auto-driver/tests/node_mixed_regions_*.rs (PLAN-054 owner evidence)"
                }
                _ => "unrecorded (this out dir has no registered flow consumer)",
            };
            identity.push_str(&format!("  \"product\": \"{product}\",\n"));
            identity.push_str(&format!("  \"consumer\": \"{consumer}\",\n"));
            identity.push_str(
                "  \"consumption\": \"ui.json regions in ANY def drive per-instance child surfaces (Game2d/Scene3d) mounted Flat2d into the UI view with identity {run generation, instance path, region local name, mount epoch}; active regions app-total <= 4; conditional unmount revokes portal+surface (last-owner release) and rejects queued region events by epoch; scene bindings read the SAME int getter ABI; region clicks dispatch through the shared _ui_click_ ABI with the instance index; one Session owner, one app_start/app_update per explicit step\",\n",
            );
            identity.push_str(&format!(
                "  \"world_sha256\": \"{}\"\n",
                lowered.world_sha256
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
                "node2d-lower[node-ui-mixed-regions]: wrote ui.json (schema {}) + world.at ({} bytes, sha {}) + closure/{}",
                autographics_node2d_pilot::NODE_UI_MIXED_REGIONS_SCHEMA,
                lowered.source_bytes,
                lowered.world_sha256,
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
