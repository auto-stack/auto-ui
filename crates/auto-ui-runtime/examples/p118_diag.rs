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
