//! PLAN-119 T-01/T-02 诊断探针：038-minesweeper 在 AV 组件面的逐构造归因。
//!
//! 用法：
//!   p119_diag.exe <app.at>                    # prep + 组件面事件程序（含逐条诊断+snippet）
//!   P119_DUMP_PREP=1 p119_diag.exe <app.at>   # 转储准备源（判断 store 是否内联）
//!   P119_STORE=1 p119_diag.exe <app.at>       # store 片断探针（prep 内联门的同一判据）
//!   P119_BISECT=1 p119_diag.exe <app.at>      # 逐 handler 的 store 探针（首错所在臂）
//!
//! 说明：prep 只在 `project_component_stores` + `project_component_candidates`
//! 同时接受 store 片断时才整块内联（ace_source_prep.rs 的 `__StoreProbe` 判据）；
//! 本探针复刻该判据并打印其诊断，避免 prep 的「静默不内联」把真实墙藏成
//! `.store.* 需要同源 store 声明` 的边界诊断。

use ace_core::atom_text::Diagnostic;
use ace_core::source::age_component::{project_component_candidates, project_component_stores};

fn diags(label: &str, src: &str, listing: &[Diagnostic]) {
    println!("{label}: {} diagnostics", listing.len());
    for d in listing {
        let snippet = safe_slice(src, d.span.start, d.span.end);
        println!("- [{}] {} @ {}..{}: {:?}", d.code, d.message, d.span.start, d.span.end, snippet);
    }
}

/// 诊断 span 可能落在多字节字符内部——按字符边界收敛。
fn safe_slice(src: &str, start: usize, end: usize) -> String {
    let mut s = start.min(src.len());
    while s > 0 && !src.is_char_boundary(s) {
        s -= 1;
    }
    let mut e = end.min(src.len());
    while e > s && !src.is_char_boundary(e) {
        e -= 1;
    }
    src[s..e].chars().take(120).collect()
}

/// 从源码找 `store <Type> { … }` 整块（花括号配平；minesweeper 语料无花括号字面量）。
fn extract_store_block(src: &str, type_name: &str) -> Option<String> {
    let needle = format!("store {type_name}");
    let start = src.find(&needle)?;
    let open = src[start..].find('{')? + start;
    let bytes = src.as_bytes();
    let mut depth = 0usize;
    for (i, b) in bytes.iter().enumerate().skip(open) {
        match *b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(src[start..=i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// 顶层 `<kw> { … }` 区块的**内部**文本。
fn section_inner(block: &str, kw: &str) -> Option<String> {
    let mut from = 0usize;
    while let Some(rel) = block[from..].find(kw) {
        let at = from + rel;
        let line_head = block[block[..at].rfind('\n').map(|p| p + 1).unwrap_or(0)..at]
            .trim()
            .to_string();
        if line_head.is_empty() {
            let open = block[at..].find('{')? + at;
            let bytes = block.as_bytes();
            let mut depth = 0usize;
            for (i, b) in bytes.iter().enumerate().skip(open) {
                match *b {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(block[open + 1..i].to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        from = at + kw.len();
    }
    None
}

/// `on { … }` 内逐 `.Msg(..) -> { … }` 臂（含臂头）。
fn handler_arms(on_inner: &str) -> Vec<(String, String)> {
    let bytes = on_inner.as_bytes();
    let mut arms = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if !on_inner.is_char_boundary(i) {
            i += 1;
            continue;
        }
        // 找行首（仅空白前缀）以 `.` 起头的臂头。
        let line_start = on_inner[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let at_line_start = i == line_start || on_inner[line_start..i].trim().is_empty();
        if at_line_start && bytes[i] == b'.' {
            let Some(open) = on_inner[i..].find('{').map(|o| o + i) else { break };
            let mut depth = 0usize;
            let mut end = None;
            for (j, b) in bytes.iter().enumerate().skip(open) {
                match *b {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(j);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let Some(end) = end else { break };
            let arm = on_inner[i..=end].to_string();
            let head = arm.split("->").next().unwrap_or("").trim().to_string();
            arms.push((head, arm));
            i = end + 1;
            continue;
        }
        i += 1;
    }
    arms
}

const PROBE_WIDGET: &str = "\nwidget __StoreProbe {\n    model { var __n int = 0 }\n    view { text \"x\" {} }\n}\n";

/// 从一条 handler 臂（`.Msg(..) -> { … }`）取体内文本。
fn arm_body(arm: &str) -> String {
    let Some(arrow) = arm.find("->") else { return String::new() };
    let Some(open) = arm[arrow..].find('{').map(|o| o + arrow) else { return String::new() };
    let bytes = arm.as_bytes();
    let mut depth = 0usize;
    for (i, b) in bytes.iter().enumerate().skip(open) {
        match *b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return arm[open + 1..i].to_string();
                }
            }
            _ => {}
        }
    }
    String::new()
}

/// 顶层语句切分（花括号/圆括号深度 0 处按换行断句）。
fn top_level_stmts(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut parens = 0i32;
    for (i, b) in bytes.iter().enumerate() {
        if !body.is_char_boundary(i) {
            continue;
        }
        let ch = *b as char;
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            '(' => parens += 1,
            ')' => parens -= 1,
            _ => {}
        }
        cur.push(ch);
        if ch == '\n' && depth == 0 && parens == 0 && !cur.trim().is_empty() {
            out.push(cur.trim().to_string());
            cur.clear();
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn store_probe(store_src: &str) -> (bool, String) {
    let probe = format!("{store_src}{PROBE_WIDGET}");
    let stores_ok = project_component_stores(&probe)
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    match project_component_candidates(&probe) {
        Ok(_) => {
            let verdict = stores_ok && true;
            (verdict, String::new())
        }
        Err(ds) => {
            let mut out = String::new();
            for d in &ds {
                let snippet = safe_slice(&probe, d.span.start, d.span.end);
                out.push_str(&format!(
                    "[{}] {} @ {}..{}: {:?}\n",
                    d.code, d.message, d.span.start, d.span.end, snippet
                ));
            }
            (false, out)
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: p119_diag <app.at>");
    let src = std::fs::read_to_string(&path).expect("read source");
    let root = std::path::Path::new(&path)
        .ancestors()
        .find(|p| p.join("pac.at").is_file())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::Path::new(&path).parent().unwrap().to_path_buf());
    let prepared = auto_ui_runtime::ui::ace_source_prep::prepare_component_source(&root, &src)
        .map(|p| p.source)
        .unwrap_or_else(|_| src.clone());

    if std::env::var("P119_DUMP_PREP").ok().as_deref() == Some("1") {
        println!("=== PREPARED SOURCE ===\n{prepared}\n=== END ===");
    }
    let store_inlined = prepared.contains("store MinesweeperStore");
    println!(
        "PREP: store_inlined={store_inlined} prepared_len={} (src_len={})",
        prepared.len(),
        src.len()
    );

    let store_src = std::fs::read_to_string(root.join("src/front/minesweeper_store.at"))
        .expect("read minesweeper_store.at");
    let block = extract_store_block(&store_src, "MinesweeperStore").expect("store block");

    if std::env::var("P119_STORE").ok().as_deref() == Some("1") {
        let (ok, report) = store_probe(&block);
        println!("STORE PROBE: ok={ok}");
        if !report.is_empty() {
            println!("{report}");
        }
        // 分区分证：model / msg / on 各自的受理面。
        let model = section_inner(&block, "model").unwrap_or_default();
        let msgs = section_inner(&block, "msg").unwrap_or_default();
        let on = section_inner(&block, "on").unwrap_or_default();
        for (label, body) in [
            ("model-only", format!("store __P {{\n model {{{model}}}\n msg {{ Tick }}\n on {{ .Tick -> {{ }} }}\n}}")),
            ("model+msg", format!("store __P {{\n model {{{model}}}\n msg {{{msgs}}}\n on {{ .Tick -> {{ }} }}\n}}")),
        ] {
            let (ok, report) = store_probe(&body);
            println!("PROBE[{label}]: ok={ok}");
            if !report.is_empty() {
                println!("{report}");
            }
        }
        let _ = on;
    }

    if std::env::var("P119_BISECT").ok().as_deref() == Some("1") {
        let model = section_inner(&block, "model").unwrap_or_default();
        let msgs = section_inner(&block, "msg").unwrap_or_default();
        let on = section_inner(&block, "on").unwrap_or_default();
        let arms = handler_arms(&on);
        println!("BISECT: {} handler arms", arms.len());
        for (head, arm) in &arms {
            let body = format!(
                "store __P {{\n model {{{model}}}\n msg {{{msgs}}}\n on {{\n {arm}\n }}\n}}"
            );
            let (ok, report) = store_probe(&body);
            let first = report.lines().next().unwrap_or("").to_string();
            println!("- {head}: ok={ok} {first}");
        }
    }

    // 包面（AV 生产道 --package）装载：units = 入口 + store 单元，过
    // prepare_store_units 后编译；打印全部诊断，并把准备后的单元写到 outdir。
    if let Ok(outdir) = std::env::var("P119_PKG") {
        let app_src = src.clone();
        let store_unit = std::fs::read_to_string(root.join("src/front/minesweeper_store.at"))
            .expect("read store unit");
        let units = auto_ui_runtime::ui::ace_source_prep::prepare_store_units(vec![
            ace_core::source::packages::PackageModuleInput {
                package: "app".into(),
                version: "1.0.0".into(),
                id: "app".into(),
                source: app_src,
                provider: None,
            },
            ace_core::source::packages::PackageModuleInput {
                package: "app".into(),
                version: "1.0.0".into(),
                id: "minesweeper_store".into(),
                source: store_unit,
                provider: None,
            },
        ]);
        if !outdir.is_empty() {
            std::fs::create_dir_all(&outdir).ok();
            for u in &units {
                std::fs::write(
                    std::path::Path::new(&outdir).join(format!("prepared-{}.at", u.id)),
                    &u.source,
                )
                .ok();
            }
            println!("PKG: wrote prepared units to {outdir}");
        }
        for u in &units {
            println!("PKG unit app::{}: {} bytes", u.id, u.source.len());
        }
        if std::env::var("P119_LIVE").ok().as_deref() == Some("1") {
            use ace_core::source::packages::PACKAGES_CAPABILITY as CAP;
            match ace_core::source::age_package::compile_age_package_widget_runtime(
                &units,
                "app::app",
                ace_av::age_component::LegacyWidgetAlias::Accept,
                &[CAP.to_string()],
            ) {
                Ok(_) => println!("LIVE RUNTIME COMPILE OK"),
                Err(ds) => {
                    println!("LIVE RUNTIME COMPILE FAILED: {} diagnostics", ds.len());
                    for d in &ds {
                        println!("- [{}] {} @ {}..{}", d.code, d.message, d.span.start, d.span.end);
                        for u in &units {
                            if d.span.end <= u.source.len() {
                                println!("    ({}) {:?}", u.id, safe_slice(&u.source, d.span.start, d.span.end));
                                break;
                            }
                        }
                    }
                }
            }
            return;
        }
        let caps = [ace_core::source::packages::PACKAGES_CAPABILITY.to_string()];
        let inst_count: usize = std::env::var("P119_PKG_INST").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
        let mut units = units;
        if std::env::var("P119_PKG_UNITS").ok().as_deref() == Some("entry") {
            units.retain(|u| u.id == "app");
            println!("PKG: entry-unit-only inspection");
        }
        let scenario = ace_av::age_component::ComponentScenario {
            instances: inst_count,
            ops: Vec::new(),
        };
        match ace_core::source::age_package::compile_age_package_with_scenario(
            &units,
            "app::app",
            ace_av::age_component::LegacyWidgetAlias::Accept,
            &caps,
            &scenario,
        ) {
            Ok(_) => println!("PACKAGE COMPILE OK"),
            Err(ds) => {
                println!("PACKAGE COMPILE FAILED: {} diagnostics", ds.len());
                for d in &ds {
                    println!("- [{}] {} @ {}..{}", d.code, d.message, d.span.start, d.span.end);
                    for u in &units {
                        if d.span.end <= u.source.len() {
                            let snip = safe_slice(&u.source, d.span.start, d.span.end);
                            println!("    ({}) {:?}", u.id, snip);
                            break;
                        }
                    }
                }
            }
        }
        return;
    }

    // 单文件探针：直接喂一份完整 store+widget 源，报 store 探针判据结果。
    if let Ok(probe_path) = std::env::var("P119_PROBE") {
        let probe_src = std::fs::read_to_string(&probe_path).expect("read probe");
        let (ok, report) = store_probe(&probe_src);
        println!("PROBE[{probe_path}]: ok={ok}");
        if !report.is_empty() {
            println!("{report}");
        }
        return;
    }

    // 语句级归因：在指定 handler 臂内逐条语句构造最小 store 探针——报语法/profile
    // 面拒的语句（`source.syntax`），语义面拒（undefined/no such field）算语法已受理。
    if let Ok(arm_name) = std::env::var("P119_STMT") {
        let model = section_inner(&block, "model").unwrap_or_default();
        let msgs = section_inner(&block, "msg").unwrap_or_default();
        let on = section_inner(&block, "on").unwrap_or_default();
        let arms = handler_arms(&on);
        let Some((head, arm)) = arms
            .iter()
            .find(|(h, _)| h.contains(arm_name.trim_start_matches('.')))
            .or_else(|| arms.first())
        else {
            println!("STMT: no arms");
            return;
        };
        let body = arm_body(arm);
        let stmts = top_level_stmts(&body);
        println!("STMT[{}]: {} statements", head, stmts.len());
        for (i, st) in stmts.iter().enumerate() {
            let probe = format!(
                "store __P {{\n model {{{model}}}\n msg {{{msgs}}}\n on {{\n .Tick -> {{\n {st}\n }}\n }}\n}}"
            );
            let (ok, report) = store_probe(&probe);
            let first = report.lines().next().unwrap_or("");
            let syntax = first.contains("source.syntax");
            println!(
                "- #{i} syntax_gate={syntax} ok={ok} :: {} :: {}",
                st.lines().next().unwrap_or("").chars().take(90).collect::<String>(),
                first
            );
        }
    }

    if std::env::var("P119_COMPONENT").ok().as_deref() == Some("1") || std::env::var("P119_STORE").is_err() {
        match auto_ui_runtime::ui::ace_node_desktop::NodeDesktopBridge::load(&prepared, &path, 1) {
            Ok(_) => println!("COMPONENT LOAD OK"),
            Err(e) => println!("COMPONENT LOAD FAILED: {e}"),
        }
        match ace_core::source::age_component::compile_component_event_program(
            &prepared,
            "app::main",
            ace_av::age_component::LegacyWidgetAlias::Accept,
            &["ac.runtime.text.v1".to_string()],
            None,
            true,
        ) {
            Ok(program) => println!(
                "EVENT PROGRAM OK: handlers={} fields={}",
                program.handlers.len(),
                program.fields.len()
            ),
            Err(ds) => diags("EVENT PROGRAM FAILED", &prepared, &ds),
        }
    }
}
