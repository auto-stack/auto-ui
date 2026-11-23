//! PLAN-118 T-02 诊断探针：组件面事件程序完整装载，逐条打印诊断
//! （code + span + 源片段），替代宿主的单条 hint 摘要。
//!
//! 用法：p118_diag.exe <app.at> [module_path]

use ace_av::age_component::LegacyWidgetAlias;
use ace_core::source::age_component::compile_component_event_program;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: p118_diag <app.at> [module_path]");
    let module_path = args.next().unwrap_or_else(|| "app::main".to_string());
    let src = std::fs::read_to_string(&path).expect("read source");
    // 包面模式：忠实复刻宿主路径——prep（pac.at 祖先根）+ `$event`→`nevent`
    // 归一化 + color_picker 包单元，再走宿主包面装载。
    if std::env::var("P118_PKG").ok().as_deref() == Some("1") {
        let root = std::path::Path::new(&path)
            .ancestors()
            .find(|p| p.join("pac.at").is_file())
            .unwrap_or_else(|| std::path::Path::new(&path).parent().unwrap())
            .to_path_buf();
        let prepped = auto_ui_runtime::ui::ace_source_prep::prepare_component_source(&root, &src)
            .map(|p| p.source)
            .unwrap_or_else(|_| src.clone())
            .replace("$event", "nevent");
        let mut units: Vec<ace_core::source::packages::PackageModuleInput> =
            vec![ace_core::source::packages::PackageModuleInput {
                package: "app".into(),
                version: "1.0.0".into(),
                id: "app".into(),
                source: prepped,
                provider: None,
            }];
        let pkg_dir = root.join("../../../../../packages/color-picker");
        let which = std::env::var("P118_PKG_SET")
            .unwrap_or_else(|_| "color_picker,color_model,color_dialog".into());
        for name in which.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let f = pkg_dir.join(format!("{name}.at"));
            if let Ok(s) = std::fs::read_to_string(&f) {
                units.push(ace_core::source::packages::PackageModuleInput {
                    package: "color_picker".into(),
                    version: "0.1.0".into(),
                    id: name.into(),
                    source: s,
                    provider: None,
                });
            }
        }
        match auto_ui_runtime::ui::ace_node_desktop::NodeDesktopBridge::load_package(
            units, "app::app", 1,
        ) {
            Ok(bridge) => {
                let ev = bridge.evidence();
                println!(
                    "PACKAGE LOAD OK: buttons={} fields={} dispatch_errors={}",
                    ev["buttons"].as_array().map(|a| a.len()).unwrap_or(0),
                    ev["field_names"].as_array().map(|a| a.len()).unwrap_or(0),
                    ev["errors"].as_array().map(|a| a.len()).unwrap_or(0),
                );
            }
            Err(e) => {
                println!("PACKAGE LOAD FAILED: {e}");
                std::process::exit(2);
            }
        }
        return;
    }
    // 包面帧转储：直连 PackageStateEngine，打印画布元素的 properties/events。
    if std::env::var("P118_PKG_DUMP").ok().as_deref() == Some("1") {
        use auto_ui_runtime::ui::ace_package_bridge::PackageStateEngine;
        let units = vec![ace_core::source::packages::PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: src.into(),
            provider: None,
        }];
        let mut engine = PackageStateEngine::new(
            units,
            "app::app",
            ace_av::age_component::LegacyWidgetAlias::Accept,
        )
        .map_err(|e| e.to_string())
        .unwrap_or_else(|e| {
            println!("ENGINE NEW FAILED: {e}");
            std::process::exit(2);
        });
        engine.mount("i0", Vec::new()).unwrap_or_else(|e| {
            println!("MOUNT FAILED: {e}");
            std::process::exit(2);
        });
        let frame = engine.package_frame("i0").unwrap_or_else(|| {
            println!("NO FRAME");
            std::process::exit(2);
        });
        fn walk(items: &[ace_av::age_package::WidgetViewItem], depth: usize, out: &mut String) {
            for item in items {
                match item {
                    ace_av::age_package::WidgetViewItem::Text(t) => {
                        println!("{}text {:?}", "  ".repeat(depth), &t[..t.len().min(60)]);
                    }
                    ace_av::age_package::WidgetViewItem::Timer(t) => {
                        println!("{}timer {} {}ms", "  ".repeat(depth), t.message, t.every_ms);
                    }
                    ace_av::age_package::WidgetViewItem::Element(el) => {
                        println!(
                            "{}<{}> site={} label={:?} props={:?} events={:?}",
                            "  ".repeat(depth),
                            el.tag,
                            el.site,
                            el.label,
                            el.properties,
                            el.events.iter().map(|e| (&e.key, &e.message, &e.args)).collect::<Vec<_>>()
                        );
                        walk(&el.children, depth + 1, out);
                    }
                }
            }
        }
        walk(&frame.view, 0, &mut String::new());
        return;
    }
    // 与宿主一致：有 pac.at 祖先时先过组件面源准备。
    let prepared = std::path::Path::new(&path)
        .ancestors()
        .find(|p| p.join("pac.at").is_file())
        .map(|root| {
            auto_ui_runtime::ui::ace_source_prep::prepare_component_source(root, &src)
                .map(|p| p.source)
                .unwrap_or_else(|_| src.clone())
        })
        .unwrap_or_else(|| src.clone());
    if std::env::var("P118_DUMP_PREP").ok().as_deref() == Some("1") {
        println!("=== PREPARED SOURCE ===\n{prepared}\n=== END ===");
    }
    // 原始单单元包编译（不过 prep）：对准备源做构造级二分的入口。
    if std::env::var("P118_RAW_FULL").ok().as_deref() == Some("1") {
        let mut units = vec![ace_core::source::packages::PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: src.clone(),
            provider: None,
        }];
        if let Ok(extra) = std::env::var("P118_RAW_EXTRA") {
            for (pkg, id, path) in extra.split(';').filter_map(|item| {
                let mut it = item.splitn(3, '=');
                Some((it.next()?.to_string(), it.next()?.to_string(), it.next()?.to_string()))
            }) {
                if let Ok(s) = std::fs::read_to_string(&path) {
                    units.push(ace_core::source::packages::PackageModuleInput {
                        package: pkg, version: "1.0.0".into(), id, source: s, provider: None,
                    });
                }
            }
        }
        let caps = [ace_core::source::packages::PACKAGES_CAPABILITY.to_string()];
        let scenario = ace_av::age_component::ComponentScenario { instances: 1, ops: Vec::new() };
        match ace_core::source::age_package::compile_age_package_with_scenario(
            &units, "app::app", LegacyWidgetAlias::Accept, &caps, &scenario,
        ) {
            Ok(_) => println!("RAW PACKAGE COMPILE OK"),
            Err(diags) => {
                println!("RAW PACKAGE COMPILE FAILED: {} diagnostics", diags.len());
                for d in &diags {
                    println!("- [{}] {} @ {}..{}", d.code, d.message, d.span.start, d.span.end);
                }
                std::process::exit(2);
            }
        }
        return;
    }
    let caps = ["ac.runtime.text.v1".to_string()];
    match compile_component_event_program(
        &prepared,
        &module_path,
        LegacyWidgetAlias::Accept,
        &caps,
        None,
        true,
    ) {
        Ok(program) => {
            println!("COMPILE OK: handlers={} fields={}", program.handlers.len(), program.fields.len());
        }
        Err(diags) => {
            println!("COMPILE FAILED: {} diagnostics", diags.len());
            for d in &diags {
                let (s, e) = (d.span.start.min(prepared.len()), d.span.end.min(prepared.len()));
                let snippet: String = prepared[s..e].chars().take(80).collect();
                println!(
                    "- [{}] {} @ {}..{}: {:?}",
                    d.code, d.message, d.span.start, d.span.end, snippet
                );
            }
        }
    }
    // Lane compile（A2Vue 严格道同一 producer）+ 逐 handler 缺席归因。
    let msgs: Vec<&str> = {
        let mut v: Vec<&str> = Vec::new();
        for line in prepared.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix(".") {
                if let Some(end) = rest.find(" ->") {
                    let head = &rest[..end];
                    let name = head.split('(').next().unwrap_or(head).trim();
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        v.push(Box::leak(name.to_string().into_boxed_str()));
                    }
                }
            }
        }
        v
    };
    match auto_ui_runtime::ui::ace_ts_provider::ComponentFragmentLane::compile_at(
        &prepared,
        &module_path,
        &msgs,
        0,
    ) {
        Ok(lane) => {
            println!("LANE COMPILE OK; per-handler facts:");
            for m in &msgs {
                println!("- .{} foreign={}", m, lane.handler_body_foreign(m));
            }
        }
        Err(e) => {
            println!("LANE COMPILE FAILED: {e}");
            std::process::exit(2);
        }
    }
}
