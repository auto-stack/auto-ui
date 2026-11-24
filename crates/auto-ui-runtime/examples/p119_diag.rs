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

fn scan_balanced(src: &str, open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = open;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if b == b'\\' { i += 2; continue; }
            if b == b'"' { in_str = false; }
            i += 1;
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => { depth -= 1; if depth == 0 { return Some(i); } }
            _ => {}
        }
        i += 1;
    }
    None
}

fn split_stmts(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut parens = 0i32;
    let mut in_str = false;
    for (i, b) in bytes.iter().enumerate() {
        if !body.is_char_boundary(i) { continue; }
        if in_str {
            let ch = *b as char;
            cur.push(ch);
            if ch == '\\' { continue; }
            if ch == '"' { in_str = false; }
            continue;
        }
        let ch = *b as char;
        match ch {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => depth -= 1,
            '(' => parens += 1,
            ')' => parens -= 1,
            _ => {}
        }
        cur.push(ch);
        if ch == '\n' && depth == 0 && parens == 0 {
            let piece = cur.trim_end_matches(|c| c == '\r' || c == '\n');
            if !piece.trim().is_empty() { out.push(piece.to_string()); }
            cur.clear();
        }
    }
    let piece = cur.trim_end_matches(|c| c == '\r' || c == '\n');
    if !piece.trim().is_empty() { out.push(piece.to_string()); }
    out
}
// ─── PLAN-119 live 道二分：文件喂入 units → 直连 widget-runtime 编译 ───────
//
// 用法：P119_LIVE_FILE="<app.at>|<store.at>" p119_diag.exe <任意 app.at>
// 前缀/留一扫描：P119_LIVE_SWEEP=prefix|leaveone|arm
fn live_file_mode() -> bool {
    let Ok(spec) = std::env::var("P119_LIVE_FILE") else { return false };
    let mut parts = spec.split('|');
    let (Some(app_path), Some(store_path)) = (parts.next(), parts.next()) else {
        println!("P119_LIVE_FILE needs <app.at>|<store.at>");
        return true;
    };
    let app_src = std::fs::read_to_string(app_path).expect("read app unit");
    let store_src = std::fs::read_to_string(store_path).expect("read store unit");
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
            source: store_src,
            provider: None,
        },
    ]);
    let compile = |units: &[ace_core::source::packages::PackageModuleInput]| -> String {
        use ace_core::source::packages::PACKAGES_CAPABILITY as CAP;
        match ace_core::source::age_package::compile_age_package_widget_runtime(
            units,
            "app::app",
            ace_av::age_component::LegacyWidgetAlias::Accept,
            &[CAP.to_string()],
        ) {
            Ok(_) => "OK".to_string(),
            Err(ds) => ds
                .iter()
                .map(|d| format!("[{}] {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join(" | "),
        }
    };
    println!("LIVE(baseline): {}", compile(&units));
    let sweep = std::env::var("P119_LIVE_SWEEP").unwrap_or_default();
    if sweep.is_empty() {
        return true;
    }
    // 目标 arm：store 单元里 `.Reveal(x, y) -> { … }`
    let store = units[1].source.clone();
    let arm = std::env::var("P119_LIVE_ARM").unwrap_or_else(|_| ".Reveal".to_string());
    let Some(arm_at) = store.find(&format!("{arm}(")).or_else(|| store.find(&format!("{arm} "))) else {
        println!("arm `{arm}` not found");
        return true;
    };
    let Some(open) = store[arm_at..].find('{').map(|o| o + arm_at) else { return true };
    let Some(close) = scan_balanced(&store, open) else { return true };
    let body = store[open + 1..close].to_string();
    let stmts = split_stmts(&body);
    println!("arm {arm}: {} top-level statements", stmts.len());
    let mutate = |new_body: String| -> Vec<ace_core::source::packages::PackageModuleInput> {
        let mut next = units.clone();
        next[1].source = format!("{}{}{}", &store[..=open], new_body, &store[close..]);
        next
    };
    // leave-one-out：在指定块的字节区间内逐条摘除（保上下文），找出摘掉后转绿的语句。
    if sweep == "leaveone-block" || sweep == "leaveone-ready" {
        let mut isolated = units.clone();
        let src = isolated[1].source.clone();
        let Some(arm_at) = src
            .find(&format!("{arm}("))
            .or_else(|| src.find(&format!("{arm} ")))
        else {
            return true;
        };
        let Some(arm_open) = src[arm_at..].find('{').map(|o| o + arm_at) else { return true };
        let Some(arm_close) = scan_balanced(&src, arm_open) else { return true };
        // 目标块：默认 `<arm> 体里的 if .game_state == "ready" {`。
        let marker = std::env::var("P119_LIVE_BLOCK")
            .unwrap_or_else(|_| "if .game_state == \"ready\" {".to_string());
        let Some(rel) = src[arm_open + 1..arm_close].find(&marker) else {
            println!("block marker `{marker}` not found");
            return true;
        };
        let block_open = src[..arm_open + 1 + rel].rfind('{').map(|p| p).unwrap_or(0);
        let block_open = arm_open + 1 + rel + marker.rfind('{').unwrap_or(marker.len() - 1);
        let Some(block_close) = scan_balanced(&src, block_open) else { return true };
        let range = (block_open + 1, block_close);
        let level = split_stmts(&src[range.0..range.1]);
        println!("block has {} top-level statements", level.len());
        for (i, st) in level.iter().enumerate() {
            println!(
                "  #{i}: {}",
                st.lines().next().unwrap_or("").chars().take(90).collect::<String>()
            );
        }
        let mut kept: Vec<String> = Vec::new();
        // subset 掩码：P119_LIVE_KEEP="0,1,2" 只保留这些下标（覆盖 leave-out 扫描）。
        if let Ok(keep) = std::env::var("P119_LIVE_KEEP") {
            let want: Vec<usize> = keep
                .split(',')
                .filter_map(|v| v.trim().parse().ok())
                .collect();
            for (j, st) in level.iter().enumerate() {
                if want.contains(&j) {
                    kept.push(st.clone());
                }
            }
            let body = format!("\n{}\n", kept.join("\n"));
            let mut next = isolated.clone();
            let mut s = src.clone();
            s.replace_range(range.0..range.1, &body);
            next[1].source = s;
            println!("subset keep={keep:?}: {}", compile(&next));
            return true;
        }
        for i in 0..level.len() {
            kept.clear();
            for (j, st) in level.iter().enumerate() {
                if i != j {
                    kept.push(st.clone());
                }
            }
            let body = format!("\n{}\n", kept.join("\n"));
            let mut next = isolated.clone();
            let mut s = src.clone();
            s.replace_range(range.0..range.1, &body);
            next[1].source = s;
            let verdict = compile(&next);
            println!(
                "leave-out #{i}: ok={} :: {}",
                verdict == "OK",
                level[i].lines().next().unwrap_or("").chars().take(70).collect::<String>()
            );
        }
        return true;
    }
    // 隔离扫描：其余 arm 体换成单条标量赋值（已知可发射），再对目标 arm 做前缀扫描。
    if sweep == "isolate-prefix" || sweep == "isolate" {
        let mut isolated = units.clone();
        let mut src = isolated[1].source.clone();
        let mut from = 0usize;
        let mut arms_replaced = 0;
        while let Some(rel) = src[from..].find(" -> {") {
            let head_at = src[..from + rel].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let head = src[head_at..from + rel].trim().to_string();
            let open = from + rel + 4;
            let Some(close) = scan_balanced(&src, open) else { break };
            if !head.starts_with(&arm) {
                src.replace_range(open + 1..close, "\n.game_state = .game_state\n");
                arms_replaced += 1;
            }
            let next_close = scan_balanced(&src, open).unwrap_or(open);
            from = next_close + 1;
        }
        isolated[1].source = src;
        println!("isolated {arms_replaced} other arms");
        println!("LIVE(isolated): {}", compile(&isolated));
        if sweep == "isolate-prefix" {
            let src = isolated[1].source.clone();
            let Some(arm_at) = src
                .find(&format!("{arm}("))
                .or_else(|| src.find(&format!("{arm} ")))
            else {
                return true;
            };
            let Some(open) = src[arm_at..].find('{').map(|o| o + arm_at) else { return true };
            let Some(close) = scan_balanced(&src, open) else { return true };
            let stmts = split_stmts(&src[open + 1..close]);
            println!("arm {arm}: {} top-level statements (isolated)", stmts.len());
            for (i, st) in stmts.iter().enumerate() {
                println!(
                    "  #{i}: {}",
                    st.lines().next().unwrap_or("").chars().take(100).collect::<String>()
                );
            }
            for k in 1..=stmts.len() {
                let body = format!("\n{}\n", stmts[..k].join("\n"));
                let mut next = isolated.clone();
                let mut s = src.clone();
                s.replace_range(open + 1..close, &body);
                next[1].source = s;
                let verdict = compile(&next);
                let ok = verdict == "OK";
                println!("prefix {k}: ok={ok} {}", if ok { String::new() } else { verdict });
                if !ok {
                    break;
                }
            }
            // 递归下降：始终在**当前块的字节区间**内做前缀替换，保持外层上下文
            // （守卫/声明）不变——最小失败前缀的最后一条即嫌疑语句；它是块时进入
            // 其体内继续。
            let mut range = (open + 1, close);
            for _depth in 0..12 {
                let base = isolated[1].source.clone();
                let level = split_stmts(&base[range.0..range.1]);
                let mut fail_at = None;
                for k in 1..=level.len() {
                    let body = format!("\n{}\n", level[..k].join("\n"));
                    let mut next = isolated.clone();
                    let mut s = base.clone();
                    s.replace_range(range.0..range.1, &body);
                    next[1].source = s;
                    if compile(&next) != "OK" {
                        fail_at = Some(k);
                        break;
                    }
                }
                let Some(k) = fail_at else {
                    println!("DESCEND: level passes (depth {_depth})");
                    break;
                };
                let culprit = level[k - 1].clone();
                println!(
                    "DESCEND depth={_depth} k={k}/{}: {}",
                    level.len(),
                    culprit.lines().next().unwrap_or("").chars().take(90).collect::<String>()
                );
                let Some(brace) = culprit.find('{') else {
                    println!("MINIMAL FAILING STATEMENT:\n{culprit}");
                    break;
                };
                let Some(end) = scan_balanced(&culprit, brace) else {
                    println!("MINIMAL FAILING STATEMENT (unbalanced):\n{culprit}");
                    break;
                };
                // culprit 在重建体里的起点：前置一个 "\n"，外加前面每条语句 + "\n"。
                let mut culprit_start = 1usize;
                for prev in &level[..k - 1] {
                    culprit_start += prev.len() + 1;
                }
                let new_open = range.0 + culprit_start + brace;
                let new_close = range.0 + culprit_start + end;
                range = (new_open + 1, new_close);
            }
        }
        return true;
    }
    match sweep.as_str() {
        "prefix" => {
            for k in 1..=stmts.len() {
                let body = format!("\n{}\n", stmts[..k].join("\n"));
                let verdict = compile(&mutate(body));
                let ok = verdict == "OK";
                println!("prefix {k}: ok={ok} {}", if ok { String::new() } else { verdict });
                if !ok {
                    break;
                }
            }
        }
        "leaveone" => {
            for i in 0..stmts.len() {
                let mut kept: Vec<String> = Vec::new();
                for (j, st) in stmts.iter().enumerate() {
                    if i != j {
                        kept.push(st.clone());
                    }
                }
                let body = format!("\n{}\n", kept.join("\n"));
                let verdict = compile(&mutate(body));
                println!(
                    "leave-out #{i}: ok={} :: {}",
                    verdict == "OK",
                    stmts[i].lines().next().unwrap_or("").chars().take(80).collect::<String>()
                );
            }
        }
        _ => println!("unknown sweep `{sweep}`"),
    }
    true
}
fn main() {
    if live_file_mode() { return; }
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
            ops: match std::env::var("P119_PKG_OP").ok().as_deref() {
                Some(msg) => vec![ace_av::age_component::ScenarioOp::Send {
                    instance: 0,
                    msg: msg.to_string(),
                }],
                None => Vec::new(),
            },
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
