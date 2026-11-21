#![cfg(feature = "ace-bridge")]

use ace_av::age_component::LegacyWidgetAlias;
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::{
    ace_node_desktop::NodeDesktopBridge,
    ace_package_bridge::PackageStateEngine,
    ace_package_sfc::{generate_package_sfc, generate_routed_front_sfc},
};

fn unit(package: &str, id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: package.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

#[test]
fn checked_package_format_text_reads_current_state_and_generates_vue_bindings() {
    use ace_av::age_component::{ComponentScenario, ScenarioOp};
    use ace_core::source::{
        age_package::compile_age_package_with_scenario,
        packages::PACKAGES_CAPABILITY,
    };

    let src = r#"widget TreeStatus {
    msg { Expand }
    model {
        expanded int = 1
        selected str = "src & docs"
    }
    on {
        .Expand -> {
            .expanded = .expanded + 1
        }
    }
    view {
        row (class: f"tree ${.selected}") {
            text (text: f"expanded: ${.expanded} · selected: ${.selected}") { style: "text-sm" }
        }
    }
}
"#;
    let units = [unit("gallery", "status", src)];
    let entry = "gallery::status";
    let compilation = compile_age_package_with_scenario(
        &units,
        entry,
        LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send { instance: 0, msg: "Expand".into() }],
        },
    ).expect("check the package");
    let observed = ace_av::age_package::observe_age_package(&compilation, 1)
        .expect("execute formatted text through AV");
    assert_eq!(observed.components[0].instance_texts, [vec!["expanded: 2 · selected: src & docs"]]);

    let vue = generate_package_sfc(&units, entry).expect("checked Vue SFC");
    assert!(vue.contains("const selected = ref(\"src & docs\")"), "{vue}");
    assert!(vue.contains(":class=\"[('' + 'tree ' + selected)]\""), "{vue}");
    assert!(
        vue.contains("v-text=\"('' + 'expanded: ' + expanded + ' · selected: ' + selected)\""),
        "{vue}"
    );
}

#[test]
fn p107_checked_child_class_tracks_its_parent_property() {
    use ace_av::age_package::WidgetViewItem;

    let root = r#"widget ChartApp {
    use { package: official from "components" }
    msg { Toggle }
    model { var active bool = true }
    on { .Toggle -> { .active = !.active } }
    view { chart-legend (active: .active) {} }
}
"#;
    let child = r#"widget ChartLegend (active: bool = true) {
    view {
        row (style: ["items-center", if .active { "" } else { "opacity-40" }]) {}
    }
}
"#;
    let units = vec![unit("app", "main", root), unit("official", "legend", child)];
    generate_package_sfc(&units, "app::main").expect("the child view compiles for Vue");
    let mut engine = PackageStateEngine::new(units, "app::main", LegacyWidgetAlias::Accept)
        .expect("the child view compiles for AV");
    engine.mount("chart", vec![]).unwrap();

    fn classes(items: &[WidgetViewItem]) -> Option<&str> {
        for item in items {
            if let WidgetViewItem::Element(element) = item {
                if let Some((_, style)) = element.properties.iter().find(|(name, _)| name == "style") {
                    return Some(style);
                }
                if let Some(style) = classes(&element.children) {
                    return Some(style);
                }
            }
        }
        None
    }
    let initial = engine.package_frame("chart").unwrap();
    assert_eq!(classes(&initial.view).unwrap().split_whitespace().collect::<Vec<_>>(), ["items-center"]);
    engine.dispatch("chart", "Toggle").unwrap();
    let next = engine.package_frame("chart").unwrap();
    assert_eq!(classes(&next.view).unwrap().split_whitespace().collect::<Vec<_>>(), ["items-center", "opacity-40"]);
}

#[test]
fn p107_024_area_accepts_an_empty_input() {
    assert_empty_024_chart(
        "area-chart",
        "area_chart",
        include_str!("../../../examples/ui/024-charts/src/front/components/area_chart.at"),
        r#", curve: "monotone""#,
    );
}

#[test]
fn p107_024_bar_accepts_an_empty_input() {
    assert_empty_024_chart(
        "bar-chart",
        "bar_chart",
        include_str!("../../../examples/ui/024-charts/src/front/components/bar_chart.at"),
        "",
    );
}

fn assert_empty_024_chart(tag: &str, module: &str, component: &str, extra_props: &str) {
    use ace_av::{
        age_component::{ComponentScenario, ScenarioOp},
        age_package::{evaluate_package_view, WidgetViewItem},
    };
    use ace_core::source::{
        age_package::compile_age_package_with_scenario, packages::PACKAGES_CAPABILITY,
    };

    let source = format!(r#"type Month {{
        m Str
        desktop int
    }}
    widget ChartData {{
        use {{ package: official from "components" }}
        model {{ var monthly List<Month> = [] }}
        view {{ {tag} (data: .monthly, index: "m", fields: ["desktop"]{extra_props}) {{}} }}
    }}"#);
    let units = vec![
        unit("app", "main", &source),
        unit("official", module, component),
        unit("stylekit", "styles", include_str!("../../../examples/ui/stylekit/src/front/styles.at")),
    ];
    let ops = vec![ScenarioOp::ForNode {
        node: 1,
        op: Box::new(ScenarioOp::Send { instance: 0, msg: "Init".into() }),
    }];
    let checked = compile_age_package_with_scenario(
        &units, "app::main", LegacyWidgetAlias::Accept, &[PACKAGES_CAPABILITY.into()],
        &ComponentScenario { instances: 1, ops },
    ).expect("the original chart component admits an empty typed list");

    fn filled_paths(items: &[WidgetViewItem], paths: &mut Vec<String>) {
        for item in items {
            let WidgetViewItem::Element(element) = item else { continue };
            if element.tag == "path" && element.properties.iter().any(|(key, value)|
                key == "fill" && matches!(value.as_str(), "#2563eb" | "#16a34a"))
            {
                paths.extend(element.properties.iter().filter_map(|(key, d)|
                    (key == "d" && !d.is_empty()).then(|| d.clone())));
            }
            filled_paths(&element.children, paths);
        }
    }

    let view = evaluate_package_view(&checked, 0, &Default::default())
        .expect("empty chart view without a division trap");
    let mut paths = Vec::new();
    filled_paths(&view, &mut paths);
    assert!(paths.is_empty(), "{paths:?}");
}

#[test]
fn p107_chart_child_uses_the_parent_record_list_type() {
    use ace_av::age_package::WidgetViewItem;

    let root = r#"widget ChartApp {
    use { package: official from "components" }
    msg { Toggle }
    model {
        var first bool = true
        monthly = [{ m: "Jan", v: 7 }]
        alternate = [{ m: "Feb", v: 13 }]
    }
    on { .Toggle -> { .first = !.first } }
    view {
        if .first { chart-label (data: .monthly) {} }
        else { chart-label (data: .alternate) {} }
    }
}
"#;
    let child = r#"widget ChartLabel (data: List) {
    view { text f"${.data[0].v}" {} }
}
"#;
    let units = vec![unit("app", "main", root), unit("official", "chart_label", child)];
    generate_package_sfc(&units, "app::main").expect("Vue checks the same record type as AV");
    let mut engine = PackageStateEngine::new(units, "app::main", LegacyWidgetAlias::Accept)
        .expect("AV checks the list at both call sites");
    engine.mount("chart", vec![]).unwrap();

    let label = |engine: &PackageStateEngine| {
        let frame = engine.package_frame("chart").unwrap();
        let [WidgetViewItem::Element(widget)] = frame.view.as_slice() else {
            panic!("expected the evaluated child widget, got {:?}", frame.view);
        };
        assert_eq!(widget.tag, "ChartLabel");
        let [WidgetViewItem::Text(label)] = widget.children.as_slice() else {
            panic!("expected the child label, got {:?}", widget.children);
        };
        label.clone()
    };
    assert_eq!(label(&engine), "7");
    engine.dispatch("chart", "Toggle").unwrap();
    assert_eq!(label(&engine), "13");

    let conflicting = root.replace("v: 13", "v: true");
    let incompatible = [unit("app", "main", &conflicting), unit("official", "chart_label", child)];
    let error = generate_package_sfc(&incompatible, "app::main")
        .expect_err("the chart label cannot interpolate a Boolean field");
    assert!(
        error.contains("source.fstr-arg") && error.contains("found `bool`"),
        "{error}"
    );
}

#[test]
fn p107_bound_chart_record_keys_use_the_authored_selector_values() {
    let root = r#"widget ChartPage {
    use { package: official from "components" }
    model { monthly = [{ m: "Jan", desktop: 7, mobile: 13 }] }
    view { chart-series (data: .monthly, index: "m", field: "desktop", fields: ["mobile", "desktop"]) {} }
}
"#;
    let child = r#"widget ChartSeries (data: List, index: str = "m", field: str = "v", fields: List<str> = ["v"]) {
    msg { Init }
    model { var total int = 0 }
    on {
        .Init -> {
            var iname = .index
            for row in .data {
                if row[iname] == "Jan" { .total += row[.field] }
                for field in .fields {
                    var coordinate float = row[field]
                    if coordinate > 0.0 { .total = .total * 10 + row[field] }
                }
            }
        }
    }
    view { text f"${.total}" {} }
}
"#;
    let package = |root, child| vec![unit("app", "main", root), unit("official", "chart_series", child)];
    let units = package(root, child);
    let vue = generate_package_sfc(&units, "app::main").expect("the Vue body uses checked record keys");
    assert!(vue.contains("<AcePackageView />"));
    let mut engine = PackageStateEngine::new(units, "app::main", LegacyWidgetAlias::Accept)
        .expect("AV checks the binding at the same view site");
    engine.mount("chart", vec![]).unwrap();
    let frame = engine.package_frame("chart").unwrap();
    let [ace_av::age_package::WidgetViewItem::Element(series)] = frame.view.as_slice() else {
        panic!("expected the checked chart series");
    };
    assert!(matches!(series.children.as_slice(),
        [ace_av::age_package::WidgetViewItem::Text(result)] if result == "837"));

    let unknown = root.replace("field: \"desktop\"", "field: \"missing\"");
    let error = generate_package_sfc(&package(&unknown, child), "app::main").unwrap_err();
    assert!(error.contains("source.undefined") && error.contains("missing"), "{error}");
    let reassigned = child.replace("var iname = .index", "var iname = .index\n            iname = \"m\"");
    let error = generate_package_sfc(&package(root, &reassigned), "app::main").unwrap_err();
    assert!(error.contains("source.index-base"), "{error}");
    let mutated = child.replace("var iname = .index", "var iname = .index\n            .fields.push(\"outside\")");
    let error = generate_package_sfc(&package(root, &mutated), "app::main").unwrap_err();
    assert!(error.contains("source.index-base"), "{error}");
}

#[test]
fn p107_chart_builder_uses_types_of_selected_bound_record_fields() {
    let root = r#"widget ChartPage {
    use { package: official from "components" }
    model { monthly = [{ m: "Jan", desktop: 7.5, mobile: 13.0 }] }
    view { chart-series (data: .monthly, fields: ["mobile", "desktop"]) {} }
}
"#;
    let child = r#"widget ChartSeries (data: List, fields: List<str> = ["v"]) {
    msg { Init }
    model { var coordinates List = [] }
    on {
        .Init -> {
            var selected List = []
            for row in .data {
                for field in .fields { selected.push(row[field]) }
            }
            .coordinates = selected
        }
    }
    view { text f"${.coordinates[0]}/${.coordinates[1]}" {} }
}
"#;
    // A caller offset falls inside a UTF-8 codepoint in this module. The
    // generated widget signature must be anchored at its own List parameter.
    let child = format!("//{}\n{child}", "原始记录类型 ".repeat(28));
    let units = vec![unit("app", "main", root), unit("official", "chart_series", &child)];
    let vue = generate_package_sfc(&units, "app::main").expect("record values seed an f32 list");
    assert!(vue.contains("<AcePackageView />"));
    let mut engine = PackageStateEngine::new(units, "app::main", LegacyWidgetAlias::Accept)
        .expect("the chart compiles to the same typed AV frame");
    engine.mount("chart", vec![]).unwrap();
    let frame = engine.package_frame("chart").unwrap();
    let [ace_av::age_package::WidgetViewItem::Element(series)] = frame.view.as_slice() else {
        panic!("expected the checked chart series");
    };
    assert!(matches!(series.children.as_slice(),
        [ace_av::age_package::WidgetViewItem::Text(value)] if value == "13/7.5"));
}

#[test]
fn p107_checked_child_svg_updates_the_native_document() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component, View,
    };

    let root = r#"widget ChartPage {
    use { package: official from "components" }
    view { ChartGeometry {} }
}
"#;
    let child = r##"widget ChartGeometry {
    msg { Reset }
    model { var x float = 1.2 }
    on { .Reset -> { .x = -0.0 } }
    view {
        svg (viewBox: "0 0 8 8") {
            path (d: f"M 0 0 L ${.x} 4", stroke: "#114477") {}
            text "A < B & C" { x: 1, y: 5 }
        }
        button "Reset geometry" { onclick: .Reset }
    }
}
"##;
    let units = vec![unit("app", "main", root), unit("official", "chart_geometry", child)];
    let vue = generate_package_sfc(&units, "app::main").expect("Vue accepts the checked SVG");
    assert!(vue.contains("return checkedSvgNode(element)"));
    assert!(vue.contains("a2x_str_display_f32"));
    let mut native = NodeDesktopBridge::load_package(units, "app::main", 1).unwrap();
    if std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_none() {
        return;
    }

    fn svg(view: &View<NodeDesktopMsg>) -> Option<&str> {
        match view {
            View::Image { src, .. } => src.strip_prefix("svgdoc:"),
            View::Column { children, .. } | View::Row { children, .. } => {
                children.iter().find_map(svg)
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => svg(child),
            _ => None,
        }
    }
    let check_svg = |view: &View<NodeDesktopMsg>, expected: &str| {
        let document = svg(view).unwrap_or_else(|| {
            panic!("the checked frame must send an SVG document to Iced, got {view:?}")
        });
        assert!(document.contains(expected), "{document}");
        assert!(document.contains("A &lt; B &amp; C"), "{document}");
        resvg::usvg::Tree::from_str(document, &resvg::usvg::Options::default())
            .expect("the native renderer can parse the updated document");
    };
    native.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    check_svg(&window.view(), r#"d="M 0 0 L 1.2 4""#);
    let click = current_session(|session| {
        let frame = session.package_frame(0).unwrap();
        let target = frame.actions().iter().position(|action| action.source.message == "Reset").unwrap();
        NodeDesktopMsg::PackageClick {
            session: session.id, instance: 0, revision: frame.revision, target,
        }
    }).unwrap();
    window.on(click);
    assert!(matches!(native.step(), StepOutcome::Dispatched { msg, .. } if msg == "Reset"));
    check_svg(&window.view(), r#"d="M 0 0 L -0 4""#);
    native.close_session().unwrap();
}

#[test]
fn p107_checked_chart_pointer_dispatches_native_coordinates_and_rejects_stale_events() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component, View,
    };

    let root = r#"widget ChartPage {
    use { package: official from "components" }
    view { ChartMarker {} }
}
"#;
    let child = r#"widget ChartMarker {
    msg { PointerMove(float, float), PointerOut }
    model { var sample str = "outside" }
    on {
        .PointerMove(x, y) -> { .sample = f"${x}/${y}" }
        .PointerOut -> { .sample = "outside" }
    }
    view {
        mouse-area (coords: "260x260", onmousemove: .PointerMove, onmouseleave: .PointerOut) {
            text .sample {}
        }
    }
}
"#;
    let units = vec![unit("app", "main", root), unit("official", "chart_marker", child)];
    let vue = generate_package_sfc(&units, "app::main").expect("checked pointer frame");
    assert!(vue.contains("acePointerSample(domEvent, pointerSamples"));
    let mut native = NodeDesktopBridge::load_package(units, "app::main", 1).unwrap();
    if std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_none() {
        return;
    }
    fn area(view: &View<NodeDesktopMsg>) -> Option<(NodeDesktopMsg, NodeDesktopMsg)> {
        match view {
            View::MouseArea { on_move: Some(sample), on_exit: Some(exit), logical_extent, .. } => {
                assert_eq!(*logical_extent, Some((260.0, 260.0)));
                Some((sample.call(1.5, 2.25), exit.clone()))
            }
            View::Column { children, .. } | View::Row { children, .. } => {
                children.iter().find_map(area)
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => area(child),
            _ => None,
        }
    }
    native.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let view = window.view();
    let (motion, stale_exit) = area(&view)
        .unwrap_or_else(|| panic!("checked Iced mouse area missing from {view:?}"));
    window.on(motion);
    assert!(matches!(native.step(), StepOutcome::Dispatched { msg, .. } if msg == "PointerMove"));
    assert!(format!("{:?}", window.view()).contains("1.5/2.25"));
    window.on(stale_exit);
    assert!(matches!(native.step(), StepOutcome::Idle));
    assert!(format!("{:?}", window.view()).contains("1.5/2.25"));
    let (_, exit) = area(&window.view()).expect("current frame mouse area");
    window.on(exit);
    assert!(matches!(native.step(), StepOutcome::Dispatched { msg, .. } if msg == "PointerOut"));
    assert!(format!("{:?}", window.view()).contains("outside"));
    native.close_session().unwrap();
}

#[test]
fn p107_checked_chart_checkbox_tracks_the_routed_message() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component, View,
    };

    let root = r#"widget ChartApp {
    use { package: official from "components" }
    msg { Toggle }
    model {
        var visible bool = true
        rows List<int> = [3, 4]
    }
    on { .Toggle -> { .visible = !.visible } }
    view {
        checkbox "Include series" { checked: .visible, onclick: .Toggle }
        chart-legend (rows: .rows) {}
    }
}
"#;
    let child = r#"widget ChartLegend (rows: List<int>) {
    msg { Init }
    model { var total int = 0 }
    on {
        .Init -> {
            var sum int = 0
            for value in .rows { sum = sum + value }
            .total = sum
        }
    }
    view { text f"Series total: ${.total}" {} }
}
"#;
    let units = |root| vec![unit("app", "main", root), unit("official", "chart_legend", child)];
    let vue = generate_package_sfc(&units(root), "app::main").expect("checked checkbox and child handler");
    assert!(vue.contains("<AcePackageView />"));
    let invalid = root.replace("checked: .visible", "checked: 1");
    let error = generate_package_sfc(&units(&invalid), "app::main").expect_err("checked requires bool");
    assert!(error.contains("source.type-mismatch"), "{error}");

    let mut native = NodeDesktopBridge::load_package(units(root), "app::main", 1).unwrap();
    if std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_none() {
        return;
    }
    fn checkbox(view: &View<NodeDesktopMsg>) -> Option<(bool, NodeDesktopMsg)> {
        match view {
            View::Checkbox { is_checked, label, on_toggle: Some(on_toggle), .. } => {
                assert_eq!(label, "Include series");
                Some((*is_checked, on_toggle.clone()))
            }
            View::Column { children, .. } | View::Row { children, .. } => children.iter().find_map(checkbox),
            View::Scrollable { child, .. } => checkbox(child),
            _ => None,
        }
    }
    native.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let view = window.view();
    assert!(format!("{view:?}").contains("Series total: 7"));
    let (initial, toggle) = checkbox(&view).expect("checked native checkbox");
    assert!(initial);
    window.on(toggle);
    assert!(matches!(native.step(), StepOutcome::Dispatched { msg, .. } if msg == "Toggle"));
    assert!(!checkbox(&window.view()).expect("same native checkbox after click").0);
    native.close_session().unwrap();
}

#[test]
fn checked_drawer_open_property_follows_the_model() {
    use ace_av::{
        age_component::{ComponentScenario, ScenarioOp},
        age_package::{evaluate_package_entry_view, ExecLimits, WidgetViewItem},
    };
    use ace_core::source::{
        age_package::compile_age_package_with_scenario, packages::PACKAGES_CAPABILITY,
    };

    let page = r#"widget GalleryPage {
    use { package: official from "./components" }
    msg { Open }
    model { visible bool = false }
    on { .Open -> { .visible = true } }
    view {
        drawer (open: .visible) { Panel {} }
    }
}
"#;
    let units = [
        unit("gallery", "app", page),
        unit("official", "panel", "widget Panel { view { text \"Settings\" {} } }"),
    ];
    let compiled = compile_age_package_with_scenario(
        &units,
        "gallery::app",
        LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.into()],
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::Send { instance: 1, msg: "Open".into() }],
        },
    )
    .expect("compile a drawer with a model-controlled open property");
    for (index, value) in ["false", "true"].into_iter().enumerate() {
        let items = evaluate_package_entry_view(&compiled, index, &ExecLimits::default())
            .expect("evaluate the checked drawer view");
        let [WidgetViewItem::Element(drawer)] = items.as_slice() else {
            panic!("expected one drawer, got {items:?}");
        };
        assert_eq!(drawer.tag, "drawer");
        assert_eq!(drawer.properties, [("open".into(), value.into())]);
    }
}

#[test]
fn stateless_widget_forwards_each_site_to_its_authored_channel() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component,
    };

    let page = r#"widget GalleryPage {
    use { package: official from "./components" }
    msg { ChooseLeft, ChooseRight, WithPayload(str) }
    model { chosen str = "" }
    on {
        .ChooseLeft -> { .chosen = "left" }
        .ChooseRight -> { .chosen = "right" }
    }
    view {
        Panel (on_pick: .ChooseLeft, on_accept: .ChooseRight) {}
        Panel (on_pick: .ChooseRight, on_accept: .ChooseLeft) {}
    }
}
"#;
    let panel = r#"widget Panel (on_pick: msg, on_accept: msg) {
    msg { Accept }
    on { .Accept -> { on_pick() } }
    view { button "Accept" { onclick: .Accept } }
}
"#;
    let units = |page| [unit("gallery", "app", page), unit("official", "panel", panel)];
    let entry = "gallery::app";
    let bad = page.replacen("on_pick: .ChooseLeft", "on_pick: .WithPayload", 1);
    let error = generate_package_sfc(&units(&bad), entry).unwrap_err();
    assert!(error.contains("widget channel `on_pick` forwards without a payload"), "{error}");

    let sfc = generate_package_sfc(&units(page), entry).expect("generate the checked callbacks");
    assert!(sfc.contains("[\"Accept\"]: \"ChooseLeft\""));
    assert!(sfc.contains("[\"Accept\"]: \"ChooseRight\""));

    let mut bridge = NodeDesktopBridge::load_package(units(page).to_vec(), entry, 1).unwrap();
    assert!(std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_some());
    let session = bridge.open().unwrap();
    assert_eq!(current_session(|session| session.package_frame(0).unwrap().actions().len()), Some(2));
    let mut window = AceNodeDesktopWindow::default();
    for (target, expected) in ["left", "right"].into_iter().enumerate() {
        let revision = current_session(|session| session.package_frame(0).unwrap().revision).unwrap();
        let _ = window.on(NodeDesktopMsg::PackageClick { session, instance: 0, revision, target });
        let outcome = bridge.step();
        assert!(matches!(outcome, StepOutcome::Dispatched { .. }), "{outcome:?}");
        assert!(bridge.dispatch_log[target].2.contains(&("chosen".into(), expected.into())));
    }
    bridge.close_session().unwrap();
}

#[test]
fn desktop_package_routes_typed_node_ids_to_the_selected_instance() {
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;

    let page = r#"widget TreePage {
    msg { Select(str) }
    model {
        selected str = ""
        changes int = 0
    }
    on {
        .Select(id) -> {
            .selected = id
            .changes = .changes + 1
        }
    }
    view {
        text (text: f"selected: ${.selected}") {}
        text (text: f"changes: ${.changes}") {}
    }
}
"#;
    let units = vec![
        unit("gallery", "auxiliary", "widget Unrelated {\n view { text \"unrelated\" {} }\n}\n"),
        unit("gallery", "tree", page),
    ];
    let mut engine = PackageStateEngine::new(units, "gallery::tree", LegacyWidgetAlias::Accept)
        .expect("load the checked tree page");
    engine.mount("left", Vec::new()).unwrap();
    engine.mount("right", Vec::new()).unwrap();
    let refused = engine.dispatch_with_args("right", "Select", &[AceStateValue::Int(7)])
        .expect_err("a numeric value cannot act as a node ID");
    assert!(refused.contains("source."), "{refused}");
    assert_eq!(engine.texts("right").unwrap(), ["selected: ", "changes: 0"]);

    engine.dispatch_with_args("right", "Select", &[AceStateValue::Str("docs/计划".into())])
        .expect("deliver the checked node ID");
    assert_eq!(engine.texts("right").unwrap(), ["selected: docs/计划", "changes: 1"]);
    assert_eq!(engine.texts("left").unwrap(), ["selected: ", "changes: 0"]);
}

#[test]
fn vue_package_renders_checked_list_models_and_keeps_the_runtime_lifecycle() {
    let source = r#"widget TreePage {
    msg { Expand, Collapse }
    model {
        expanded List = ["docs", "src"]
        count int = 2
    }
    on {
        .Expand -> {
            .expanded = ["docs", "docs/计划", "src"]
            .count = .expanded.len()
        }
        .Collapse -> {
            .expanded = []
            .count = .expanded.len()
        }
    }
    view {
        text f"expanded: ${.count}" {}
        button "Expand" { onclick: .Expand }
        button "Collapse" { onclick: .Collapse }
    }
}
"#;
    let units = [unit("gallery", "tree", source)];
    let vue = generate_package_sfc(&units, "gallery::tree").expect("checked list SFC");
    assert!(vue.contains("const expanded = ref(__ageCreateTextList([\"docs\", \"src\"]))"));
    assert!(vue.contains("__ageReplaceTextList(expanded, __ageCreateTextList([\"docs\", \"docs/计划\", \"src\"]))"));
    assert!(vue.contains("__ageReplaceTextList(expanded, __ageCreateTextList([]))"));
    assert!(vue.contains("a2x_list_len(expanded.value)"));
    assert!(vue.contains("__ageReleaseTextList(expanded.value)"));
}

#[test]
fn desktop_package_button_delivers_the_authored_id_from_current_instance_state() {
    use auto_ui_runtime::ui::ace_node_desktop::{push_session_event, StepOutcome};

    let source = r#"widget TreeActions {
    msg { Select(str), Next }
    model {
        candidate str = "docs"
        selected str = ""
    }
    on {
        .Select(id) -> { .selected = id }
        .Next -> { .candidate = "docs/计划" }
    }
    view {
        button "Next" { onclick: .Next }
        button "Select" { onclick: .Select(.candidate) }
        text f"selected: ${.selected}" {}
    }
}
"#;
    let mut bridge = NodeDesktopBridge::load_package(
        vec![unit("gallery", "actions", source)], "gallery::actions", 2,
    ).expect("desktop loads the checked typed button");
    assert_eq!(bridge.buttons()[1].msg, "Select");
    if std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_none() {
        return;
    }

    let session = bridge.open().expect("open prepared UI host");
    push_session_event(session, 1, 0);
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { instance: 1, .. }));
    push_session_event(session, 1, 1);
    let outcome = bridge.step();
    assert!(matches!(outcome, StepOutcome::Dispatched { instance: 1, .. }), "{outcome:?}");
    let log = &bridge.dispatch_log;
    assert_eq!(log.len(), 2);
    assert_eq!(log[1].1, "Select");
    assert!(log[1].2.contains(&("selected".into(), "docs/计划".into())));
    push_session_event(session, 0, 1);
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { instance: 0, .. }));
    assert!(bridge.dispatch_log[2].2.contains(&("selected".into(), "docs".into())));
    bridge.close_session().expect("close UI host");
}
#[test]
fn authored_source_paths_supply_only_the_referenced_package_modules() {
    let root = tempfile::tempdir().expect("project directory");
    let dir = root.path();
    std::fs::create_dir(dir.join("pages")).unwrap();
    std::fs::create_dir(dir.join("components")).unwrap();
    let page_path = dir.join("pages/tree.at");
    let page = r#"widget TreePage {
    use { package: official from "../components" }
    view {
        col { file-tree {} }
    }
}
"#;
    let shell = r#"widget App {
    routes {
        "/" -> use tree
    }
    view { outlet }
}
"#;
    let manifest = dir.join("components/package.at");
    std::fs::write(&page_path, page).unwrap();
    std::fs::write(&manifest, "name: \"official\"\nversion: \"0.1.0\"\n").unwrap();
    std::fs::write(
        dir.join("components/tree.at"),
        "widget FileTree {\n    view { TreeIcon {} }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("components/tree_icon.at"),
        "widget TreeIcon {\n    view { text \"Leaf\" {} }\n}\n",
    )
    .unwrap();
    // Legal in the legacy widget registry but outside the checked lane.
    std::fs::write(
        dir.join("components/unrelated.at"),
        "widget Other {\n    model { selected List = [] }\n    view { text \"Other\" }\n}\n",
    )
    .unwrap();

    let desktop_error = || match NodeDesktopBridge::load(page, page_path.to_str().unwrap(), 1) {
        Err(error) => error,
        Ok(_) => panic!("a checked widget must not be silently omitted from the window"),
    };
    let vue_product = || {
        generate_routed_front_sfc(&dir.join("app.at"), shell, &[page_path.clone()])
            .expect("routed entry")
            .expect("route table")
    };
    let mut native = NodeDesktopBridge::load(page, page_path.to_str().unwrap(), 1)
        .expect("load the authored project closure");
    if std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_some() {
        use auto_ui_runtime::ui::{ace_node_desktop::AceNodeDesktopWindow, Component};
        native.open().expect("publish the checked entry and both widgets");
        let window = AceNodeDesktopWindow::default();
        let view = format!("{:?}", window.view());
        assert!(view.contains("Leaf"), "{view}");
        assert!(!view.contains("Other"), "{view}");
        native.close_session().expect("close the package");
    }
    let product = vue_product();
    let vue = &product.pages[&product.paths[0].1];
    assert!(vue.contains("<AcePackageView />"));
    assert!(vue.contains("name: \"FileTree\""));
    assert!(vue.contains("name: \"TreeIcon\""));
    assert!(!vue.contains("name: \"Other\""));
    assert_eq!(product.embedded_widget_sources, ["components/tree.at", "components/tree_icon.at"]
        .map(|path| dir.join(path).canonicalize().unwrap()).into_iter().collect());

    std::fs::write(&manifest, "name: \"different\"\nversion: \"0.1.0\"\n").unwrap();
    let native = desktop_error();
    let product = vue_product();
    assert!(product.embedded_widget_sources.is_empty());
    for error in [&native, &product.pages[&product.paths[0].1]] {
        assert!(error.contains("ace.widget.package-identity"), "{error}");
        assert!(
            error.contains("official") && error.contains("different"),
            "{error}"
        );
    }
}

#[test]
fn p107_standalone_widget_app_links_its_declared_style_source() {
    use auto_ui_runtime::ui::ace_package_sfc::generate_linked_front_sfc;

    let root = tempfile::tempdir().unwrap();
    let chart = root.path().join("chart");
    let front = chart.join("src/front");
    let components = front.join("components");
    let stylekit = root.path().join("stylekit");
    let style_front = stylekit.join("src/front");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::create_dir_all(&style_front).unwrap();
    std::fs::write(
        chart.join("pac.at"),
        "name: \"chart\"\nversion: \"1.0.0\"\ndep stylekit {\n    path: \"../stylekit\"\n}\n",
    ).unwrap();
    std::fs::write(stylekit.join("pac.at"), "name: \"stylekit\"\nversion: \"0.1.0\"\n").unwrap();
    std::fs::write(
        style_front.join("styles.at"),
        include_str!("../../../examples/ui/stylekit/src/front/styles.at"),
    ).unwrap();
    std::fs::write(components.join("package.at"), "name: \"official\"\nversion: \"0.1.0\"\n").unwrap();
    let child = components.join("chart_caption.at");
    std::fs::write(&child, r#"
use stylekit.styles: caption_text
widget ChartCaption { view { text "Series 1" { style: caption_text } } }
"#).unwrap();
    let app = front.join("app.at");
    let source = r#"
widget App {
    use { package: official from "components" }
    view { ChartCaption {} }
}
"#;
    std::fs::write(&app, source).unwrap();
    let result = generate_linked_front_sfc(&app, source, &[]).unwrap().expect("checked root product");
    assert!(result.paths.is_empty());
    assert!(result.app.contains("<AcePackageView />"));
    assert!(result.app.contains("text-xs text-muted-foreground"));
    assert!(result.embedded_widget_sources.contains(&child.canonicalize().unwrap()));
    NodeDesktopBridge::load(source, app.to_str().unwrap(), 1).expect("native uses the same closure");
}

#[test]
fn bound_component_callbacks_route_in_current_native_view_revision() {
    use ace_av::age_component::ComponentFieldValue;
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, host_session_stats, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component,
    };

    let page = unit(
        "gallery",
        "page",
        r#"widget GalleryPage {
    use { package: official from "../components" }
    msg { Picked(str) }
    model { selected str = "" }
    on { .Picked(id) -> { .selected = id } }
    view {
        FileTree (on_pick: .Picked) {}
        text f"picked: ${.selected}" {}
    }
}
"#,
    );
    let tree = unit(
        "official",
        "filetree",
        r#"widget FileTree (on_pick: msg) {
    msg { Pick(str), Toggle, Init }
    model { expanded bool = false }
    on {
        .Init -> { .expanded = true }
        .Toggle -> { .expanded = false }
    }
    view {
        button "Toggle" { onclick: .Toggle }
        if .expanded {
            button "Tree" { onclick: .Pick("docs/计划") }
        }
    }
}
"#,
    );
    let units = vec![page, tree];
    let entry = "gallery::page";

    let vue = generate_package_sfc(&units, entry).expect("Vue consumes the checked package");
    assert!(vue.contains("<AcePackageView />"));
    assert!(vue.contains("[\"Pick\"]: \"Picked\""));
    let mut native = NodeDesktopBridge::load_package(units, entry, 2).unwrap();
    if std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_none() {
        return;
    }
    let session = native.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let click = |instance, message| {
        current_session(|session| {
            let frame = session.package_frame(instance).unwrap();
            let target = frame.actions().iter().position(|action| action.source.message == message).unwrap();
            NodeDesktopMsg::PackageClick { session: session.id, instance, revision: frame.revision, target }
        }).unwrap()
    };
    let second = click(1, "Pick");
    let target = if let NodeDesktopMsg::PackageClick { target, .. } = &second { *target } else { unreachable!() };
    assert_eq!(
        current_session(|session| session.package_frame(1).unwrap().actions()[target].source.args.clone()).unwrap(),
        [ComponentFieldValue::Str("docs/计划".into())]
    );
    window.on(second.clone());
    assert!(matches!(native.step(), StepOutcome::Dispatched { instance: 1, msg, .. } if msg == "Picked"));
    assert_eq!(native.dispatch_log[0].2, [("selected".into(), "docs/计划".into())]);
    let first = click(0, "Toggle");
    window.on(first);
    assert!(matches!(native.step(), StepOutcome::Dispatched { instance: 0, msg, .. } if msg == "Toggle"));
    assert!(current_session(|session| session.package_frame(0).unwrap().actions().iter().all(|action| action.source.message != "Pick")).unwrap());
    assert!(current_session(|session| session.package_frame(1).unwrap().actions().iter().any(|action| action.source.message == "Pick")).unwrap());
    window.on(second.clone());
    assert!(matches!(native.step(), StepOutcome::Idle));
    assert_eq!(host_session_stats(session).unwrap().stale_rejected, 1);
    native.close_session().unwrap();
    window.on(second);
    assert_eq!(host_session_stats(session).unwrap().late_dropped, 1);
    native.reopen().unwrap();
    current_session(|session| {
        for instance in 0..2 {
            assert!(session.package_frame(instance).unwrap().actions().iter().any(|action| action.source.message == "Pick"));
        }
    }).unwrap();
    assert_eq!(format!("{:?}", window.view()).matches("picked: ").count(), 2);
    native.close_session().unwrap();
}

#[test]
fn declared_child_event_passes_the_authored_payload_to_its_parent() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component,
    };

    let parent = r#"widget FilePage {
    use { package: official from "./components" }
    msg { Selected(str) }
    model { selected str = "" }
    on { .Selected(payload) -> { .selected = payload } }
    view {
        Collection (onselectionRequested: .Selected($event)) {}
        text f"Selected: ${.selected}" {}
    }
}"#;
    let child = r#"widget Collection {
    msg { "selectionRequested"(str) }
    view { button "Select" { onclick: .selectionRequested("docs/计划") } }
}"#;
    let units = |parent: &str, child: &str| {
        vec![unit("gallery", "page", parent), unit("official", "collection", child)]
    };
    let entry = "gallery::page";
    let vue = generate_package_sfc(&units(parent, child), entry).unwrap();
    assert!(vue.contains("[\"selectionRequested\"]: \"Selected\""));
    let alias_parent = parent.replace("Collection (", "collection-picker (");
    let alias_child = child.replace("widget Collection {", "widget CollectionPicker {");
    let alias_vue = generate_package_sfc(&units(&alias_parent, &alias_child), entry).unwrap();
    assert!(alias_vue.contains("[\"selectionRequested\"]: \"Selected\""));

    for (old, new, code) in [
        ("Selected(str)", "Selected(bool)", "source.age.child-event-payload"),
        (".Selected($event)", ".Missing($event)", "source.age.child-event-target"),
        ("onselectionRequested", "onmissing", "source.age.child-event-unknown"),
        ("Collection (", "button (", "source.age.child-event-site"),
        ("Collection (", "scroll-area (", "source.age.child-event-site"),
        (
            "onselectionRequested: .Selected($event)",
            "onselectionRequested: .Selected($event), onselectionRequested: .Selected($event)",
            "source.age.child-event-duplicate",
        ),
    ] {
        let units = units(&parent.replace(old, new), child);
        let error = generate_package_sfc(&units, entry).expect_err("invalid child event");
        assert!(error.contains(code), "{error}");
        let error = match NodeDesktopBridge::load_package(units, entry, 1) {
            Ok(_) => panic!("native host accepted an invalid child event"),
            Err(error) => error,
        };
        assert!(error.contains(code), "{error}");
    }

    let mut native =
        NodeDesktopBridge::load_package(units(&alias_parent, &alias_child), entry, 1).unwrap();
    if std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_none() {
        return;
    }
    native.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let click = current_session(|session| {
        let frame = session.package_frame(0).unwrap();
        let target = frame.actions().iter()
            .position(|action| action.source.message == "selectionRequested").unwrap();
        NodeDesktopMsg::PackageClick {
            session: session.id, instance: 0, revision: frame.revision, target,
        }
    }).unwrap();
    window.on(click);
    assert!(matches!(native.step(), StepOutcome::Dispatched { msg, .. } if msg == "Selected"));
    assert_eq!(native.dispatch_log[0].2, [("selected".into(), "docs/计划".into())]);
    native.close_session().unwrap();
}

#[test]
fn vue_names_the_unavailable_reactive_computed_view_binding() {
    let source = r#"widget Card {
    model { count int = 2 }
    computed { total => .count + 1 }
    view { text f"${.total}" {} }
}
"#;
    let err = generate_package_sfc(&[unit("gallery", "card", source)], "gallery::card")
        .expect_err("a checked computed view reference needs runtime support");
    assert!(err.contains("source.age.sfc-computed"), "{err}");
    let unused = source.replace("${.total}", "${.count}");
    generate_package_sfc(&[unit("gallery", "card", &unused)], "gallery::card")
        .expect("an unused computed declaration does not affect the view");
}
