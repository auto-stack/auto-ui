#![cfg(feature = "ace-bridge")]

#[path = "support/ui980.rs"]
mod ui980;

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
fn checked_desktop_projection_fits_the_windows_main_stack() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::NodeDesktopMsg, iced::IntoIcedElement, View,
    };

    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let mut view = View::<NodeDesktopMsg>::text("directory entry");
            for _ in 0..6 {
                view = View::col()
                    .child(View::scrollable(View::row().child(view).build()).build())
                    .build();
            }
            let element = view.into_iced();
            assert_eq!(element.as_widget().children().len(), 1);
        })
        .expect("start projection with a Windows-sized stack")
        .join()
        .expect("project the nested desktop view");
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
    ui980::require_prepared();

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
fn checked_collection_events_decode_and_reject_stale_json() {
    use auto_ui_runtime::ui::{ace_node_desktop::StepOutcome, ace_state_bridge::AceStateValue};

    let original = include_str!("../../../examples/ui/027-file-manager/src/front/app.at");
    let start = original.find("        .CollectionMode(payload str)").unwrap();
    let end = original.find("        .PublishCollection ->").unwrap();
    let page = format!(r#"use std.json: json_object_text, json_object_i32
use official.collection_request: collection_request_token
use official.file_manager_model: FileManagerRow
use official.file_manager_rows: fm_path_index
widget CollectionHost {{
    use {{ package: official from "./components" }}
    msg {{ CollectionMode(str), CollectionSort(str), CollectionSelection(str), CollectionAction(str), CollectionItem(str, bool), SetViewMode(str), SortByName, SortBySize, SortByType, SortByDate, SelectItem(int), OpenItem(int), ItemCtx(int) }}
    model {{
        var current_path str = "D:/目录"
        var collection_revision int = 4
        var collection_token str = "mount-1"
        var view_mode str = "grid"
        var sort_col str = "name"
        var selected_id int = -1
        var opened_id int = -1
        var menu_id int = -1
        var files_view List<FileManagerRow> = [FileManagerRow {{ id: 0, name: "资料.txt", path: "D:/目录/资料.txt", is_dir: false, size: (1).to(u64), size_str: "1 B", file_ext: "txt", type_name: "文档", date: "", is_hidden: false, thumb_src: "" }}]
    }}
    on {{
{}
        .SetViewMode(mode str) -> {{ .view_mode = mode }}
        .SortByName -> {{ .sort_col = "name" }}
        .SortBySize -> {{ .sort_col = "size" }}
        .SortByType -> {{ .sort_col = "type" }}
        .SortByDate -> {{ .sort_col = "date" }}
        .SelectItem(index int) -> {{ .selected_id = index }}
        .OpenItem(index int) -> {{ .opened_id = index }}
        .ItemCtx(index int) -> {{ .menu_id = index }}
    }}
    view {{
        CollectionInput (onselectionRequested: .CollectionSelection($event)) {{}}
        text f"selected ${{.selected_id}}" {{}}
    }}
}}"#, &original[start..end]);
    let child = r#"widget CollectionInput {
    msg { "selectionRequested"(str) }
    view { button "Select" { onclick: .selectionRequested("{}") } }
}"#;
    let mut units = file_manager_row_units(&page).to_vec();
    units.push(unit("official", "collection", child));
    units.push(unit("official", "collection_request", include_str!("../../../examples/ui/027-file-manager/src/front/components/collection_request.at")));
    let entry = "app::app";
    let vue = generate_package_sfc(&units, entry).expect("A2Vue uses the package JSON codec");
    assert!(vue.contains("json_object_text") && vue.contains("selectionRequested"));

    ui980::require_prepared();
    let mut native = NodeDesktopBridge::load_package(units, entry, 2).unwrap();
    native.open().unwrap();
    let valid = r#"{"seq":1,"instanceToken":"mount-1","scopeId":"D:/目录","baseRevision":4,"id":"D:/目录/\u8d44\u6599.txt","itemRevision":"4"}"#;
    let invalid = [
        valid.trim_end_matches('}').into(),
        valid.replace("D:/目录", "D:/其他"),
        valid.replace("mount-1", "old-mount"),
        valid.replace(r#""baseRevision":4"#, r#""baseRevision":"4""#),
        valid.replace(r#""baseRevision":4"#, r#""baseRevision":3"#),
        valid.replace(r#""baseRevision":4"#, r#""baseRevision":4.0"#),
        valid.replace(r#""baseRevision":4"#, r#""baseRevision":4e0"#),
        valid.replace(r#""baseRevision":4"#, r#""baseRevision":4294967300"#),
        format!(r#"{},"baseRevision":3}}"#, valid.trim_end_matches('}')),
        valid.replace(r#""itemRevision":"4""#, r#""itemRevision":"3""#),
    ];
    for payload in &invalid {
        let reply = native.dispatch_typed(0, "CollectionSelection", &[AceStateValue::Str(payload.clone())]);
        assert!(matches!(&reply, StepOutcome::Dispatched { states, .. }
            if states.contains(&("selected_id".into(), "-1".into()))), "{payload}: {reply:?}");
    }
    for (handler, extra, field, expected) in [
        ("CollectionMode", r#", "viewMode":"list""#, "view_mode", "list"),
        ("CollectionSort", r#", "sortKey":"size""#, "sort_col", "size"),
        ("CollectionSelection", "", "selected_id", "0"),
        ("CollectionAction", r#", "actionId":"menu""#, "menu_id", "0"),
        ("CollectionAction", r#", "actionId":"open""#, "opened_id", "0"),
    ] {
        let payload = format!("{}{extra}}}", valid.trim_end_matches('}'));
        let reply = native.dispatch_typed(0, handler, &[AceStateValue::Str(payload)]);
        assert!(matches!(&reply, StepOutcome::Dispatched { states, .. }
            if states.contains(&(field.into(), expected.into()))), "{handler}: {reply:?}");
    }
    let untouched = native.dispatch_typed(1, "CollectionSelection", &[AceStateValue::Str(invalid[0].clone())]);
    assert!(matches!(&untouched, StepOutcome::Dispatched { states, .. }
        if states.contains(&("selected_id".into(), "-1".into()))
            && states.contains(&("view_mode".into(), "grid".into()))
            && states.contains(&("sort_col".into(), "name".into()))), "{untouched:?}");
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
    ui980::require_prepared();
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
    ui980::require_prepared();
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
fn checked_boolean_view_properties_follow_the_model() {
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
    model {
        visible bool = false
        path str = "/文件/📁.txt"
    }
    on {
        .Open -> {
            .visible = .path.starts_with("/文件/📁") && .path.starts_with("") && !.path.starts_with("/文件/📂") && !.path.starts_with("/文件/📁.txt/too-long")
        }
    }
    view {
        drawer (open: .visible) { Panel {} }
        alert-dialog (open: if .visible { true } else { false }) {}
        sidebar_menu_button (active: .visible) {}
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
    .expect("compile model-controlled boolean view properties");
    for (index, value) in ["false", "true"].into_iter().enumerate() {
        let items = evaluate_package_entry_view(&compiled, index, &ExecLimits::default())
            .expect("evaluate the checked properties");
        assert_eq!(items.len(), 3);
        for (item, (tag, property)) in items.iter().zip([
            ("drawer", "open"), ("alert-dialog", "open"), ("sidebar_menu_button", "active"),
        ]) {
            let WidgetViewItem::Element(element) = item else {
                panic!("expected {tag}, got {item:?}");
            };
            assert_eq!(element.tag, tag);
            assert_eq!(element.properties, [(property.into(), value.into())]);
        }
    }

    generate_package_sfc(&units, "gallery::app")
        .expect("lower the same prefix expression and boolean properties to Vue");
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
    ui980::require_prepared();
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
    ui980::require_prepared();

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
fn checked_widget_context_tracks_mounts_and_viewport() {
    use ace_av::age_package::WidgetViewItem;
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        style::theme, Component,
    };

    let page = r#"widget ContextPage {
    use { package: context_widgets from "./widgets" }
    msg { HostContext(str, int), Toggle }
    bind { "widget:context" -> .HostContext }
    model {
        var token str = ""
        var width int = 0
        var left bool = true
    }
    on {
        .HostContext(token str, width int) -> {
            .token = token
            .width = width
        }
        .Toggle -> { .left = !.left }
    }
    view {
        text f"ctx:root:${.token}:${.width}" {}
        if .left { ContextWrapper (label: "left") {} }
        ContextChild (name: "right") {}
        button "Toggle" { onclick: .Toggle }
    }
}
"#;
    let child = r#"widget ContextChild (name: str = "") {
    msg { HostContext(str, int), Init, Observe }
    bind { "widget:context" -> .HostContext }
    model {
        var token str = ""
        var width int = 0
        var seen int = 0
    }
    on {
        .HostContext(token str, width int) -> {
            .token = token
            .width = width
        }
        .Init -> { .seen = .width }
        .Observe -> { .seen = .width }
    }
    view {
        text f"ctx:${.name}:${.token}:${.seen}" {}
        button (text: f"Measure ${.name}", onclick: .Observe) {}
    }
}
"#;
    let wrapper = r#"use context_child: ContextChild

widget ContextWrapper (label: str) {
    view { ContextChild (name: .label) {} }
}
"#;
    let units = |child: &str| vec![
        unit("app", "app", page),
        unit("context_widgets", "context_child", child),
        unit("context_widgets", "context_wrapper", wrapper),
    ];
    let sfc = generate_package_sfc(&units(child), "app::app").unwrap();
    assert!(sfc.contains("contextHandler: \"HostContext\""));
    let altered = child.replace(".width = width", ".width = width + 1");
    let error = generate_package_sfc(&units(&altered), "app::app").unwrap_err();
    assert!(error.contains("source.age.widget-context"), "{error}");

    let mut bridge = NodeDesktopBridge::load_package(units(child), "app::app", 1).unwrap();
    ui980::require_prepared();

    struct RestoreWidth(f32);
    impl Drop for RestoreWidth {
        fn drop(&mut self) {
            theme::set_window_width(self.0);
        }
    }
    let _restore = RestoreWidth(theme::window_width());
    theme::set_window_width(700.0);
    bridge.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    fn readings(items: &[WidgetViewItem], found: &mut std::collections::BTreeMap<String, (String, i32)>) {
        for item in items {
            match item {
                WidgetViewItem::Element(element) => readings(&element.children, found),
                WidgetViewItem::Text(text) if text.starts_with("ctx:") => {
                    let parts = text.split(':').collect::<Vec<_>>();
                    assert_eq!(parts.len(), 4, "{text}");
                    found.insert(parts[1].to_owned(), (parts[2].to_owned(), parts[3].parse().unwrap()));
                }
                WidgetViewItem::Text(_) | WidgetViewItem::Timer(_) => {}
            }
        }
    }
    let snapshot = || current_session(|session| {
        let mut result = std::collections::BTreeMap::new();
        readings(&session.package_frame(0).unwrap().view, &mut result);
        result
    }).unwrap();
    let click = |message, ordinal| current_session(|session| {
        let frame = session.package_frame(0).unwrap();
        let target = frame.actions().iter().enumerate().filter(|(_, action)| action.source.message == message)
            .nth(ordinal).unwrap().0;
        NodeDesktopMsg::PackageClick { session: session.id, instance: 0, revision: frame.revision, target }
    }).unwrap();

    let initial = snapshot();
    assert_eq!(initial.len(), 3);
    for (token, width) in initial.values() {
        assert!(token.starts_with("av-") && *width == 700, "{initial:?}");
    }
    assert_ne!(initial["left"].0, initial["right"].0);
    assert_ne!(initial["root"].0, initial["right"].0);
    let refused = bridge.dispatch_typed(0, "HostContext", &[]);
    assert!(matches!(refused, StepOutcome::DispatchRefused(reason) if reason.contains("only the host")));

    theme::set_window_width(1150.0);
    window.on(click("Observe", 1));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert_eq!(snapshot()["right"], (initial["right"].0.clone(), 1150));
    assert_eq!(snapshot()["left"], initial["left"]);

    window.on(click("Toggle", 0));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert!(!snapshot().contains_key("left"));
    window.on(click("Toggle", 0));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    let remounted = snapshot();
    assert_ne!(remounted["left"].0, initial["left"].0);
    assert_eq!(remounted["left"].1, 1150);
    assert_eq!(remounted["right"].0, initial["right"].0);
    bridge.close_session().unwrap();
    bridge.reopen().unwrap();
    assert_ne!(snapshot()["root"].0, initial["root"].0);
    bridge.close_session().unwrap();
}

#[test]
fn checked_project_only_opens_legacy_vm_dependencies_on_the_legacy_track() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::AceNodeDesktopWindow,
        ace_package_sfc::generate_package_front_sfc,
        Component,
    };

    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("components");
    std::fs::create_dir(&package).unwrap();
    let app_path = root.path().join("app.at");
    std::fs::write(package.join("package.at"), "name: \"official\"\n").unwrap();
    std::fs::write(
        package.join("marker.at"),
        "widget Marker { view { text \"Marker\" {} } }",
    ).unwrap();
    std::fs::write(
        package.join("vm_provider.at"),
        "pub fn legacy_value() str {\n    try { return \"legacy\" } catch {}\n    return \"failed\"\n}",
    ).unwrap();
    let source = r#"use components.vm_provider: legacy_value

widget App {
    use { package: official from "./components" }
    msg { Init }
    model { var label str = "" }
    on {
        .Init -> {
            if Env.track() == "vm" { .label = legacy_value() }
            else { .label = "checked host" }
        }
    }
    view {
        Marker {}
        if Env.track() == "vm" {
            text "Legacy view only" {}
        } else {
            text "Checked view only" {}
        }
        text .label {}
    }
}
"#;
    std::fs::write(&app_path, source).unwrap();
    let (_, sfc) = generate_package_front_sfc(&app_path, source, &[]).unwrap();
    assert!(sfc.contains("checked host"), "{sfc}");
    assert!(!sfc.contains("legacy_value"), "{sfc}");
    assert!(sfc.contains("Checked view only"), "{sfc}");
    assert!(!sfc.contains("Legacy view only"), "{sfc}");

    let compatibility = source.replace(
        "Env.track() == \"vm\"",
        "Env.track() == \"vm\" || Env.track() == \"vue\"",
    );
    let (_, sfc) = generate_package_front_sfc(&app_path, &compatibility, &[]).unwrap();
    assert!(!sfc.contains("legacy_value"), "{sfc}");

    let active = source.replace("Env.track() ==", "Env.track() !=");
    let error = generate_package_front_sfc(&app_path, &active, &[]).unwrap_err();
    assert!(error.contains("vm_provider") && error.contains("try"), "{error}");

    let shadowed = source.replace(".Init -> {", ".Init -> {\n            let Env = 0");
    let error = generate_package_front_sfc(&app_path, &shadowed, &[]).unwrap_err();
    assert!(error.contains("vm_provider") && error.contains("try"), "{error}");

    let mut bridge = NodeDesktopBridge::load(source, app_path.to_str().unwrap(), 1).unwrap();
    ui980::require_prepared();
    bridge.open().unwrap();
    let view = format!("{:?}", AceNodeDesktopWindow::default().view());
    assert!(view.contains("checked host"), "{view}");
    bridge.close_session().unwrap();
}

#[test]
fn authored_source_paths_supply_only_the_referenced_package_modules() {
    let root = tempfile::tempdir().expect("project directory");
    let dir = root.path();
    std::fs::create_dir(dir.join("pages")).unwrap();
    std::fs::create_dir(dir.join("components")).unwrap();
    let page_path = dir.join("pages/tree.at");
    let page = r#"use components.schema: Column

widget TreePage {
    use { package: official from "../components" }
    model { columns List<Column> = [{ key: "name", sortable: true }] }
    view {
        col {
            FileTree {}
            text f"Columns: ${.columns.len()}" {}
        }
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
        dir.join("components/schema.at"),
        "pub type Column {\n    key str\n    sortable bool\n}\n",
    )
    .unwrap();
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
    ui980::require_prepared();
    use auto_ui_runtime::ui::{ace_node_desktop::AceNodeDesktopWindow, Component};
    native.open().expect("publish the checked entry and both widgets");
    let window = AceNodeDesktopWindow::default();
    let view = format!("{:?}", window.view());
    assert!(view.contains("Leaf"), "{view}");
    assert!(view.contains("Columns: 1"), "{view}");
    assert!(!view.contains("Other"), "{view}");
    native.close_session().expect("close the package");
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
    ui980::require_prepared();
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
    ui980::require_prepared();
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

#[test]
fn guarded_root_timer_follows_the_live_package_instance() {
    use auto_ui_runtime::ui::ace_node_desktop::StepOutcome;
    use std::time::Duration;

    let source = r#"widget App {
    msg { BootTick, Restart }
    timer { BootTick (every_ms: 250, when: .booting) }
    model {
        var booting bool = true
        var ticks int = 0
    }
    on {
        .BootTick -> { .ticks += 1; .booting = false }
        .Restart -> { .booting = true }
    }
    view {
        button "Restart" { onclick: .Restart }
        text f"Ticks: ${.ticks}" {}
    }
}
"#;
    let sfc = |source: &str| generate_package_sfc(&[unit("app", "app", source)], "app::app");
    let vue = sfc(source).expect("conditional timer emits a checked Vue widget");
    assert!(vue.contains("{ message: \"BootTick\", interval: 250, condition: a2x_fn_"));
    for (old, new, diagnostic) in [
        ("every_ms: 250", "every_ms: 0", "source.age.timer-interval"),
        ("timer { BootTick", "timer { Missing", "source.age.timer-message"),
        ("when: .booting", "when: .ticks", "source.type-mismatch"),
    ] {
        let error = sfc(&source.replace(old, new)).expect_err("invalid timer must be rejected");
        assert!(error.contains(diagnostic), "{error}");
    }
    ui980::require_prepared();

    let mut bridge = NodeDesktopBridge::load(source, "app.at", 2).unwrap();
    let session = bridge.open().unwrap();
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    std::thread::sleep(Duration::from_millis(270));
    for expected in 0..2 {
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { instance, msg, states }
            if instance == expected && msg == "BootTick" && states.contains(&("ticks".into(), "1".into()))));
    }
    assert!(matches!(bridge.dispatch_typed(1, "Restart", &[]), StepOutcome::Dispatched { .. }));
    std::thread::sleep(Duration::from_millis(270));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { instance: 1, msg, states }
        if msg == "BootTick" && states.contains(&("ticks".into(), "2".into()))));
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    bridge.close_session().unwrap();
    std::thread::sleep(Duration::from_millis(270));
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    assert_ne!(bridge.open().unwrap(), session);
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    std::thread::sleep(Duration::from_millis(270));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { instance: 0, msg, states }
        if msg == "BootTick" && states.contains(&("ticks".into(), "1".into()))));
    bridge.close_session().unwrap();
}

#[test]
fn checked_widget_handler_runs_mutable_locals_and_nested_loop_scope() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::StepOutcome, ace_state_bridge::AceStateValue,
    };

    let source = r#"type ScanRow {
    value int
    label str
}

widget App {
    use { package: official from "./components" }
    msg { Scan(int, bool) }
    model {
        var total int = 0
        var shadow int = 0
        var chosen str = ""
        var rows List<ScanRow> = [
            {value: 0, label: "a"}, {value: 1, label: "b"},
            {value: 2, label: "c"}, {value: 3, label: "d"}
        ]
    }
    on {
        .Scan(limit i32, enabled bool) -> {
            var index int = 0
            var total int = 0
            while enabled && index < limit && index < .rows.len() {
                if index == 1 {
                    var total int = 100
                    total += 1
                    .shadow = total
                } else {
                    let row = .rows[index]
                    total += row.value
                    .chosen = row.label
                }
                index = index + 1
            }
            .total = total
        }
    }
    view {
        Marker {}
        button "Scan" { onclick: .Scan(4, true) }
        text f"Total: ${.total} · Shadow: ${.shadow}" {}
        text f"Label: ${.chosen}" {}
    }
}
"#;
    let modules = |source: &str| {
        vec![
            unit("app", "app", source),
            unit("official", "marker", "widget Marker { view { text \"Ready\" {} } }"),
        ]
    };
    let entry = "app::app";
    let source_of = |source: &str| generate_package_sfc(&modules(source), entry);
    let sfc = source_of(source).expect("checked Vue loop product");
    assert!(sfc.contains("<AcePackageView />"));
    assert!(sfc.contains("while ("));
    let wrong_payload = source.replace("limit i32", "limit bool");
    let mismatch = source_of(&wrong_payload).expect_err("payload annotation must match");
    assert!(mismatch.contains("source.age.handler-parameter-type"), "{mismatch}");
    let immutable = source.replace("var index int", "let index int");
    let readonly = source_of(&immutable).expect_err("only mutable locals can be reassigned");
    assert!(readonly.contains("immutable"), "{readonly}");

    ui980::require_prepared();
    let mut bridge = NodeDesktopBridge::load_package(modules(source), entry, 2).unwrap();
    bridge.open().unwrap();
    let result = bridge.dispatch_typed(0, "Scan", &[AceStateValue::Int(4), AceStateValue::Bool(true)]);
    assert!(matches!(result, StepOutcome::Dispatched { instance: 0, states, .. }
        if states.contains(&("total".into(), "5".into()))
            && states.contains(&("shadow".into(), "101".into()))
            && states.contains(&("chosen".into(), "d".into()))));
    let result = bridge.dispatch_typed(1, "Scan", &[AceStateValue::Int(4), AceStateValue::Bool(false)]);
    assert!(matches!(result, StepOutcome::Dispatched { instance: 1, states, .. }
        if states.contains(&("total".into(), "0".into())) && states.contains(&("shadow".into(), "0".into()))));
    bridge.close_session().unwrap();
}

#[test]
fn original_history_handlers_call_typed_widget_messages_in_order() {
    use auto_ui_runtime::ui::ace_node_desktop::StepOutcome;
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;

    let original = include_str!("../../../examples/ui/027-file-manager/src/front/app.at");
    let start = original.find("        .CommitNavigation(path str").unwrap();
    let end = original.find("        .GoUp -> {").unwrap();
    let mut source = r#"use official.file_manager_model: extend_history
widget App {
    use { package: official from "./components" }
    msg { GoBack, GoForward, Navigate(str, int), CommitNavigation(str, int), Publish(str, int), SetAccess(bool) }
    model {
        var history List<str> = ["docs", "docs/计划"]
        var history_idx int = 1
        var can_back bool = true
        var can_forward bool = false
        var current_path str = "docs/计划"
        var trace str = ""
        var visits int = 0
        var can_access bool = false
    }
    on {
"#.to_owned();
    source.push_str(&original[start..end]);
    source.push_str(r#"        .Navigate(path str, index int) -> {
            if .can_access {
                .CommitNavigation(path, index)
                .current_path = path
                .Publish(path, .history_idx)
            }
        }
        .SetAccess(allowed bool) -> { .can_access = allowed }
        .Publish(path str, index int) -> {
            .trace = f"${path} @ ${index}"
            .visits += 1
        }
    }
    view {
        Marker {}
        button "Back" { onclick: .GoBack }
        button "Forward" { onclick: .GoForward }
        text f"Path: ${.current_path} | ${.trace} | Visits: ${.visits}" {}
    }
}
"#);
    let units = |src: &str| vec![
        unit("app", "app", src),
        unit("official", "marker", "widget Marker { view { text \"Ready\" {} } }"),
        unit("official", "file_manager_model", include_str!("../../../examples/ui/027-file-manager/src/front/components/file_manager_model.at")),
    ];
    let entry = "app::app";
    assert!(generate_package_sfc(&units(&source), entry).unwrap().contains("<AcePackageView />"));
    for (original, replacement, code) in [
        (".Navigate(.history[index], index)", ".Missing(.history[index], index)", "source.age.handler-message"),
        (".Navigate(.history[index], index)", ".Navigate(.history[index])", "source.age.handler-payload"),
        (".Navigate(.history[index], index)", ".Navigate(false, index)", "source.type-mismatch"),
        (".Publish(path, .history_idx)", ".GoBack()", "source.age.handler-cycle"),
        ("if .can_access {", "storage.set(\"path\", path)\n            if .can_access {", "source.age.handler-effect"),
    ] {
        let invalid = source.replacen(original, replacement, 1);
        let error = generate_package_sfc(&units(&invalid), entry).unwrap_err();
        assert!(error.contains(code), "{error}");
    }

    ui980::require_prepared();
    let mut bridge = NodeDesktopBridge::load_package(units(&source), entry, 2).unwrap();
    bridge.open().unwrap();
    let denied = bridge.dispatch_typed(0, "GoBack", &[]);
    assert!(matches!(denied, StepOutcome::Dispatched { states, .. }
        if states.contains(&("current_path".into(), "docs/计划".into()))
            && states.contains(&("history_idx".into(), "1".into()))
            && states.contains(&("visits".into(), "0".into()))));
    assert!(matches!(
        bridge.dispatch_typed(0, "SetAccess", &[AceStateValue::Bool(true)]),
        StepOutcome::Dispatched { .. }
    ));
    for (instance, message, path, index, visits) in [
        (0, "GoBack", "docs", "0", "1"),
        (0, "GoBack", "docs", "0", "1"),
        (0, "GoForward", "docs/计划", "1", "2"),
        (1, "GoForward", "docs/计划", "1", "0"),
    ] {
        let result = bridge.dispatch_typed(instance, message, &[]);
        assert!(matches!(result, StepOutcome::Dispatched { states, .. }
            if states.contains(&("current_path".into(), path.into()))
                && states.contains(&("history_idx".into(), index.into()))
                && states.contains(&("visits".into(), visits.into()))
                && (visits == "0" || states.contains(&("trace".into(), format!("{path} @ {index}"))))));
    }
    bridge.dispatch_typed(0, "GoBack", &[]);
    let next = [AceStateValue::Str("images".into()), AceStateValue::Int(-1)];
    let branched = bridge.dispatch_typed(0, "Navigate", &next);
    assert!(matches!(&branched, StepOutcome::Dispatched { states, .. }
        if states.contains(&("current_path".into(), "images".into()))
            && states.contains(&("history_idx".into(), "1".into()))
            && states.contains(&("can_forward".into(), "false".into()))), "{branched:?}");
    bridge.dispatch_typed(0, "GoBack", &[]);
    let replayed = bridge.dispatch_typed(0, "GoForward", &[]);
    assert!(matches!(&replayed, StepOutcome::Dispatched { states, .. }
        if states.contains(&("current_path".into(), "images".into()))), "{replayed:?}");
    bridge.close_session().unwrap();
}

fn file_manager_row_units(page: &str) -> [PackageModuleInput; 4] {
    [
        unit("app", "app", page),
        unit("official", "file_manager_model", include_str!("../../../examples/ui/027-file-manager/src/front/components/file_manager_model.at")),
        unit("official", "file_manager_rows", include_str!("../../../examples/ui/027-file-manager/src/front/components/file_manager_rows.at")),
        unit("official", "file_manager_metadata", include_str!("../../../examples/ui/027-file-manager/src/front/components/file_manager_metadata.at")),
    ]
}

#[test]
fn stateful_child_transitions_compile_checked_outbound_events() {
    use ace_core::source::age_package::WIDGET_OUTBOUND_FIELD;
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, host_session_stats, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component,
    };

    let parent = r#"use collection_view.preview_row: PreviewRow, preview_rows
widget CollectionParent {
    use { package: collection_view from "./collection" }
    msg { Record(str), Count(int), Flag(bool), Toggle }
    model {
        var trace str = ""
        var mark int = 0
        var show_left bool = true
        var previews List<PreviewRow> = preview_rows(false)
    }
    on {
        .Record(entry str) -> { .trace = .trace + "/view:" + entry }
        .Count(value int) -> {
            .mark = 1 / (value + 1)
            .trace = .trace + f"/count:${value}"
        }
        .Flag(value bool) -> {
            if value { .trace = .trace + "/enabled" }
            else { .trace = .trace + "/disabled" }
        }
        .Toggle -> {
            .show_left = !.show_left
            .previews = preview_rows(!.show_left)
        }
    }
    view {
        if .show_left {
            CollectionEmitter (label: "left", previews: .previews, onviewModeRequested: .Record($event), onorderedCount: .Count($event), onavailability: .Flag($event)) {}
        }
        CollectionEmitter (label: "right", previews: .previews, onviewModeRequested: .Record($event), onorderedCount: .Count($event), onavailability: .Flag($event)) {}
        button "Toggle left" { onclick: .Toggle }
        text .trace { id: "trace" }
    }
}
"#;
    let child = r#"use preview_row: PreviewRow
use caption: PreviewCaption
widget CollectionEmitter (label: str = "", previews: List<PreviewRow> = []) {
    msg { Choose, EmitValues, Fail, "viewModeRequested"(str), "orderedCount"(int), "availability"(bool), "ignored"(str) }
    model { var serial int = 0 }
    on {
        .Choose -> {
            .serial = .serial + 1
            .EmitValues()
        }
        .EmitValues -> {
            .viewModeRequested(f"${.label}:${.serial}:${.previews[0].label}")
            .orderedCount(.serial)
            .ignored("unbound")
            .availability(.serial != 2)
        }
        .Fail -> {
            .serial = .serial + 1
            .viewModeRequested("aborted")
            .orderedCount(-1)
        }
    }
    view {
        button (id: .label, text: f"${.label}:${.serial}", onclick: .Choose) {}
        button (id: f"fail-${.label}", text: "Fail", onclick: .Fail) {}
        for preview in .previews {
            PreviewCaption (key: preview.label, value: preview.label) {}
        }
    }
}
"#;
    let units = |source| {
        vec![
            unit("app", "app", parent),
            unit("collection_view", "emitter", source),
            unit("collection_view", "preview_row", r#"pub type PreviewRow { label str }
pub fn preview_rows(alternate bool) List<PreviewRow> {
    if alternate { return [PreviewRow { label: "second" }] }
    return [PreviewRow { label: "first" }]
}
"#),
            unit("collection_view", "caption", r#"widget PreviewCaption (value: str = "") {
    view { text f"Preview: ${.value}" {} }
}
"#),
        ]
    };
    let vue = generate_package_sfc(&units(child), "app::app").unwrap();
    assert!(vue.contains(WIDGET_OUTBOUND_FIELD));
    assert!(vue.contains(r#"["viewModeRequested"]: ["str"]"#));
    assert!(vue.contains(r#"["orderedCount"]: ["int"]"#));
    assert!(vue.contains(r#"["availability"]: ["bool"]"#));

    let wrong = child.replace(".availability(.serial != 2)", ".availability(\"true\")");
    let error = generate_package_sfc(&units(&wrong), "app::app").unwrap_err();
    assert!(error.contains("type"), "{error}");
    let mut native = NodeDesktopBridge::load_package(units(child), "app::app", 1).unwrap();
    ui980::require_prepared();
    let session = native.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let click = |message, ordinal| current_session(|session| {
        let frame = session.package_frame(0).unwrap();
        let (target, _) = frame.actions().iter().enumerate()
            .filter(|(_, action)| action.source.message == message).nth(ordinal).unwrap();
        NodeDesktopMsg::PackageClick { session: session.id, instance: 0, revision: frame.revision, target }
    }).unwrap();
    let saved = click("Choose", 0);
    let mut expected = String::new();
    let advance = |native: &mut NodeDesktopBridge, window: &mut AceNodeDesktopWindow, action, expected: &mut String, suffix| {
        window.on(action);
        expected.push_str(suffix);
        let result = native.step();
        assert!(matches!(&result, StepOutcome::Dispatched { states, .. }
            if states.iter().any(|(key, value)| key == "trace" && value == expected.as_str())), "{result:?}");
    };
    assert_eq!(format!("{:?}", window.view()).matches("Preview: first").count(), 2);
    advance(&mut native, &mut window, saved.clone(), &mut expected, "/view:left:1:first/count:1/enabled");
    advance(&mut native, &mut window, click("Choose", 1), &mut expected, "/view:right:1:first/count:1/enabled");
    advance(&mut native, &mut window, click("Choose", 0), &mut expected, "/view:left:2:first/count:2/disabled");
    let before = current_session(|session| session.package_frame(0).unwrap().revision).unwrap();
    window.on(click("Fail", 0));
    let refused = native.step();
    assert!(matches!(&refused, StepOutcome::DispatchRefused(error) if error.contains("DivByZero")), "{refused:?}");
    assert_eq!(current_session(|session| session.package_frame(0).unwrap().revision).unwrap(), before);
    advance(&mut native, &mut window, click("Choose", 0), &mut expected, "/view:left:3:first/count:3/enabled");
    window.on(click("Toggle", 0));
    assert!(matches!(native.step(), StepOutcome::Dispatched { .. }));
    window.on(saved);
    assert_eq!(native.step(), StepOutcome::Idle);
    assert_eq!(host_session_stats(session).unwrap().stale_rejected, 1);
    assert_eq!(format!("{:?}", window.view()).matches("Preview: second").count(), 1);
    advance(&mut native, &mut window, click("Choose", 0), &mut expected, "/view:right:2:second/count:2/disabled");
    window.on(click("Toggle", 0));
    assert!(matches!(native.step(), StepOutcome::Dispatched { .. }));
    advance(&mut native, &mut window, click("Choose", 0), &mut expected, "/view:left:1:first/count:1/enabled");
    native.close_session().unwrap();
}

#[test]
fn checked_view_fragments_preserve_tape_order_and_inputs() {
    use ace_av::age_package::WidgetViewItem;
    use auto_ui_runtime::ui::ace_node_desktop::{current_session, StepOutcome};

    let page = r#"widget FragmentPage {
    use { package: fragments from "./fragments" }
    msg { Toggle }
    model {
        var shown bool = true
        var rows List<int> = [8, 13]
    }
    on { .Toggle -> { .shown = !.shown } }
    view {
        text "before" {}
        LongPanel (caption: "邻接", active: .shown, rows: .rows) {}
        text "after" {}
    }
}
"#;
    let cells = (0..64).map(|index| format!("text f\"part-{index}:${{.caption}}\" {{}}\n"))
        .collect::<String>();
    let child = format!(r#"widget LongPanel (caption: str, active: bool, rows: List<int>) {{
    model {{ var stored_caption str = "" }}
    watch {{ .active -> {{ .stored_caption = .caption.to(Str) }} }}
    view {{
        col {{
            if .active {{
                {cells}
                for index, value in .rows {{ text f"row ${{index}}:${{value}}" {{}} }}
            }} else {{
                text f"closed:${{.stored_caption}}" {{}}
            }}
        }}
    }}
}}
"#);
    let units = vec![unit("app", "app", page), unit("fragments", "panel", &child)];
    let sfc = generate_package_sfc(&units, "app::app").expect("generate fragmented view");
    assert!(sfc.contains("runtime_view_1_fragment_"));

    let mut bridge = NodeDesktopBridge::load_package(units, "app::app", 1).unwrap();
    ui980::require_prepared();
    bridge.open().unwrap();
    fn record(items: &[WidgetViewItem], output: &mut Vec<String>) {
        for item in items {
            match item {
                WidgetViewItem::Text(text) => output.push(text.clone()),
                WidgetViewItem::Element(element) => record(&element.children, output),
                WidgetViewItem::Timer(_) => {}
            }
        }
    }
    let text = || current_session(|session| {
        let mut output = Vec::new();
        record(&session.package_frame(0).unwrap().view, &mut output);
        output
    }).unwrap();
    let mut expected = vec!["before".to_owned()];
    expected.extend((0..64).map(|index| format!("part-{index}:邻接")));
    expected.extend(["row 0:8".into(), "row 1:13".into(), "after".into()]);
    assert_eq!(text(), expected);
    assert!(matches!(bridge.dispatch_typed(0, "Toggle", &[]), StepOutcome::Dispatched { .. }));
    assert_eq!(text(), ["before", "closed:邻接", "after"]);
    assert!(matches!(bridge.dispatch_typed(0, "Toggle", &[]), StepOutcome::Dispatched { .. }));
    assert_eq!(text(), expected);
    bridge.close_session().unwrap();
}

#[test]
fn checked_watches_compare_published_values_per_mount() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component,
    };

    let page = r#"use watches.rows: WatchRow, watch_rows
widget WatchPage {
    use { package: watches from "./watches" }
    msg { ChangeBoth, Refresh, ChangeRows, Fail, Toggle }
    model {
        var rows List<WatchRow> = watch_rows(false)
        var mode str = "grid"
        var shown bool = true
        var pulse int = 0
        var pulses int = 0
    }
    watch { .pulse -> { .pulses += 1 } }
    on {
        .Refresh -> {
            .rows = watch_rows(.mode != "grid")
            .pulse += 1
        }
        .ChangeBoth -> {
            .mode = "rows"
            .rows = watch_rows(true)
        }
        .ChangeRows -> { .rows = watch_rows(false) }
        .Fail -> { .mode = "bad" }
        .Toggle -> { .shown = !.shown }
    }
    view {
        button "Refresh" { onclick: .Refresh }
        button "Change both" { onclick: .ChangeBoth }
        button "Change rows" { onclick: .ChangeRows }
        button "Fail" { onclick: .Fail }
        button "Toggle" { onclick: .Toggle }
        text f"Pulses: ${.pulses}" { id: "pulses" }
        if .shown {
            WatchChild (mode: .mode, rows: .rows) {}
        }
    }
}
"#;
    let child = r#"use rows: WatchRow
widget WatchChild (mode: str = "", rows: List<WatchRow> = []) {
    msg { Record, Bump }
    model {
        var changes int = 0
        var local int = 0
        var local_changes int = 0
        var zero int = 0
    }
    watch {
        .rows, .mode -> {
            if .mode == "bad" { .changes = .changes / .zero }
            .Record()
        }
        .local.immediate -> { .local_changes += 1 }
    }
    on {
        .Record -> { .changes += 1 }
        .Bump -> { .local += 1 }
    }
    view {
        text f"Watch: ${.mode}:${.changes}:${.local_changes}:${.rows[0].name}" { id: "watch-result" }
        button "Bump local" { onclick: .Bump }
    }
}
"#;
    let rows = r#"pub type WatchRow { name str }
pub fn watch_rows(alternate bool) List<WatchRow> {
    if alternate { return [WatchRow { name: "second" }] }
    return [WatchRow { name: "first" }]
}
"#;
    let units = |child: &str| vec![
        unit("app", "app", page),
        unit("watches", "watchchild", child),
        unit("watches", "rows", rows),
    ];
    let vue = generate_package_sfc(&units(child), "app::app").unwrap();
    assert!(vue.contains("sources: [1,0], immediate: false"));
    assert!(vue.contains(r#"sources: ["local"], immediate: true"#));
    let error = generate_package_sfc(&units(&child.replace(".rows, .mode", ".unknown, .mode")), "app::app")
        .unwrap_err();
    assert!(error.contains("source.age.member-field"), "{error}");
    let error = generate_package_sfc(&units(&child.replace(".Record()", ".Unknown()")), "app::app")
        .unwrap_err();
    assert!(error.contains("source.age.handler-message"), "{error}");

    let mut bridge = NodeDesktopBridge::load_package(units(child), "app::app", 1).unwrap();
    ui980::require_prepared();
    bridge.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let assert_view = |window: &AceNodeDesktopWindow, expected| {
        let view = format!("{:?}", window.view());
        assert!(view.contains(expected), "{view}");
    };
    let dispatch = |bridge: &mut NodeDesktopBridge, message| {
        let result = bridge.dispatch_typed(0, message, &[]);
        assert!(matches!(result, StepOutcome::Dispatched { .. }), "{result:?}");
    };
    assert_view(&window, "Watch: grid:0:1:first");
    dispatch(&mut bridge, "Refresh");
    assert_view(&window, "Watch: grid:0:1:first");
    assert_view(&window, "Pulses: 1");
    dispatch(&mut bridge, "ChangeBoth");
    assert_view(&window, "Watch: rows:1:1:second");
    dispatch(&mut bridge, "Refresh");
    assert_view(&window, "Watch: rows:1:1:second");

    let bump = current_session(|session| {
        let frame = session.package_frame(0).unwrap();
        let target = frame.actions().iter().position(|action| action.source.message == "Bump").unwrap();
        NodeDesktopMsg::PackageClick { session: session.id, instance: 0, revision: frame.revision, target }
    }).unwrap();
    window.on(bump);
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert_view(&window, "Watch: rows:1:2:second");
    let before = current_session(|session| session.package_frame(0).unwrap().revision).unwrap();
    let refused = bridge.dispatch_typed(0, "Fail", &[]);
    assert!(matches!(&refused, StepOutcome::DispatchRefused(error) if error.contains("DivByZero")), "{refused:?}");
    assert_eq!(current_session(|session| session.package_frame(0).unwrap().revision).unwrap(), before);
    assert_view(&window, "Watch: rows:1:2:second");
    dispatch(&mut bridge, "ChangeRows");
    assert_view(&window, "Watch: rows:2:2:first");
    dispatch(&mut bridge, "Toggle");
    assert!(!format!("{:?}", window.view()).contains("Watch:"));
    dispatch(&mut bridge, "Toggle");
    assert_view(&window, "Watch: rows:0:1:first");
    bridge.close_session().unwrap();
}

#[test]
fn checked_focus_requests_belong_to_committed_widget_instances() {
    use ace_core::source::age_package::WIDGET_FOCUS_FIELD;
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        Component,
    };

    let page = r#"widget FocusPage {
    use { package: focus_widgets from "./focus_widgets" }
    msg { Remove(str), Advance, Refresh }
    model {
        var visible bool = true
        var version int = 0
    }
    on {
        .Remove(name str) -> {
            if name == "left" { .visible = false }
        }
        .Advance -> { .version += 1 }
        .Refresh -> {}
    }
    view {
        if .visible {
            FocusChild (name: "left", version: .version, onremove: .Remove($event)) {}
        }
        FocusChild (name: "right", version: .version, onremove: .Remove($event)) {}
        button "Advance" { onclick: .Advance }
        button "Refresh" { onclick: .Refresh }
    }
}
"#;
    let child = r##"widget FocusChild (name: str = "", version: int = 0) {
    msg { Request, ApplyFocus, GroupRequest, Fail, RemoveMe, Select, Context, Open,
          Focused(str), Blurred, ChangeFocusPayload, "remove"(str) }
    model {
        var selected int = 0
        var denominator int = 0
        var focus_payload str = "initial"
        var focus_log str = ""
    }
    watch { .version -> { .ApplyFocus() } }
    on {
        .Request -> { .ApplyFocus() }
        .GroupRequest -> { ui.focus("#group-target") }
        .ApplyFocus -> {
            var selector str = "[id='path"
            selector = selector + "'s']"
            ui.focus(selector)
        }
        .Fail -> {
            .ApplyFocus()
            .selected = 1 / .denominator
        }
        .RemoveMe -> {
            .ApplyFocus()
            .remove(.name)
        }
        .Select -> { .selected += 1 }
        .Context -> { .selected += 10 }
        .Open -> { .selected += 100 }
        .Focused(payload str) -> { .focus_log = .focus_log + " F:" + payload }
        .Blurred -> { .focus_log = .focus_log + " B" }
        .ChangeFocusPayload -> { .focus_payload = "changed" }
    }
    view {
        button (text: f"Request ${.name}", onclick: .Request) {}
        button (id: f"fail-${.name}", text: f"Fail ${.name}", onclick: .Fail) {}
        button (text: f"Remove ${.name}", onclick: .RemoveMe) {}
        button (id: "path's", text: f"Focus: ${.name}", onclick: .Select,
                onkeydown.right.self.prevent: .Select, oncontextmenu.prevent: .Context,
                onfocus: .Focused(.focus_payload), onblur: .Blurred) {}
        button (text: f"Change ${.name}", onclick: .ChangeFocusPayload) {}
        text f"Focus log ${.name}:${.focus_log}" {}
        button (text: f"Group ${.name}", onclick: .GroupRequest) {}
        col (id: "group-target", tabindex: -1, onkeydown.left.self.prevent: .Select) {
            text f"Selected ${.name}: ${.selected}" {}
        }
        mouse-area (ondblclick: .Open, oncontextmenu.prevent: .Context) {
            button (text: f"Pointer ${.name}", onclick: .Select) {}
        }
    }
}
"##;
    let units = |child: &str| vec![
        unit("app", "app", page),
        unit("focus_widgets", "focuschild", child),
    ];
    let vue = generate_package_sfc(&units(child), "app::app").unwrap();
    assert!(vue.contains(WIDGET_FOCUS_FIELD));
    let invalid = child.replace("ui.focus(selector)", "ui.focus(13)");
    let error = generate_package_sfc(&units(&invalid), "app::app").unwrap_err();
    assert!(error.contains("type"), "{error}");

    let mut bridge = NodeDesktopBridge::load_package(units(child), "app::app", 1).unwrap();
    ui980::require_prepared();
    bridge.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let click = |message, ordinal| current_session(|session| {
        let frame = session.package_frame(0).unwrap();
        let target = frame.actions().iter().enumerate().filter(|(_, action)| action.source.message == message)
            .nth(ordinal).unwrap().0;
        NodeDesktopMsg::PackageClick { session: session.id, instance: 0, revision: frame.revision, target }
    }).unwrap();
    let focus = || current_session(|session| {
        session.package_frame(0).unwrap().focus_requests().iter()
            .map(|request| (request.component.clone(), request.target.clone())).collect::<Vec<_>>()
    }).unwrap();
    assert!(focus().is_empty());
    let target = "[id='path's']".to_owned();
    window.on(click("Request", 1));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert_eq!(focus(), vec![(vec![1], target.clone())]);
    #[cfg(feature = "ui-iced")]
    let mut native = {
        let mut native = PackageFocusHarness::new();
        let request = window.take_focus_request().unwrap();
        assert!(window.take_focus_request().is_none());
        let messages = native.press(&window, [request], iced::keyboard::key::Named::ArrowRight);
        assert_eq!(messages.len(), 1);
        window.on(messages.into_iter().next().unwrap());
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let view = format!("{:?}", window.view());
        assert!(view.contains("Selected right: 1"), "{view}");
        assert!(view.contains("Selected left: 0"), "{view}");
        native
    };
    window.on(click("Refresh", 0));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert!(focus().is_empty());
    let before = current_session(|session| session.package_frame(0).unwrap().revision).unwrap();
    window.on(click("Fail", 0));
    let refused = bridge.step();
    assert!(matches!(&refused, StepOutcome::DispatchRefused(error) if error.contains("DivByZero")), "{refused:?}");
    assert_eq!(current_session(|session| session.package_frame(0).unwrap().revision).unwrap(), before);
    assert!(focus().is_empty());
    window.on(click("Advance", 0));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert_eq!(focus(), vec![(vec![0], target.clone()), (vec![1], target)]);
    window.on(click("RemoveMe", 0));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert!(focus().is_empty());
    #[cfg(feature = "ui-iced")]
    {
        use iced::keyboard::key::Named::{ArrowLeft, ArrowRight};

        window.on(click("Request", 0));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let superseded = window.take_focus_request().unwrap();
        window.on(click("Refresh", 0));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        assert!(native.press(&window, [superseded], ArrowRight).is_empty());

        window.on(click("GroupRequest", 0));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let request = window.take_focus_request().unwrap();
        let messages = native.press(&window, [request], ArrowLeft);
        assert_eq!(messages.len(), 1);
        window.on(messages.into_iter().next().unwrap());
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let view = format!("{:?}", window.view());
        assert!(view.contains("Selected right: 2"), "{view}");

        window.on(click("Request", 0));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let closed = window.take_focus_request().unwrap();
        bridge.close_session().unwrap();
        bridge.reopen().unwrap();
        window.on(click("Request", 1));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let request = window.take_focus_request().unwrap();
        let messages = native.press(&window, [request, closed], ArrowRight);
        assert_eq!(messages.len(), 1);
        window.on(messages.into_iter().next().unwrap());
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let view = format!("{:?}", window.view());
        assert!(view.contains("Selected right: 1"), "{view}");
        assert!(view.contains("Selected left: 0"), "{view}");

        use auto_ui_runtime::ui::iced::ComponentIced;
        use iced::{mouse, Event};

        let mut ui = iced_test::simulator(window.view_iced());
        let target = ui.find("Pointer right").unwrap();
        ui.point_at(target.visible_bounds().unwrap().center());
        ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))]);
        let mut events = ui.into_messages();
        window.on(events.next().unwrap());
        assert!(events.next().is_none());
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let view = format!("{:?}", window.view());
        assert!(view.contains("Selected right: 11"), "{view}");

        let mut ui = iced_test::simulator(window.view_iced());
        ui.click("Pointer right").unwrap();
        ui.click("Pointer right").unwrap();
        let events = ui.into_messages().collect::<Vec<_>>();
        let authored = current_session(|session| {
            let frame = session.package_frame(0).unwrap();
            events.iter().map(|event| {
                let NodeDesktopMsg::PackageClick { target, .. } = event else {
                    panic!("unexpected event: {event:?}");
                };
                frame.actions()[*target].source.message.as_str()
            }).map(str::to_owned).collect::<Vec<_>>()
        }).unwrap();
        assert_eq!(authored, ["Select", "Open", "Select"]);
        window.on(events[1].clone());
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let view = format!("{:?}", window.view());
        assert!(view.contains("Selected right: 111"), "{view}");

        let mut cache = iced_runtime::user_interface::Cache::default();
        window.on(click("Request", 1));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let request = window.take_focus_request().unwrap();
        let notices = native.redraw(&window, &mut cache, Some(request));
        assert!(matches!(notices.as_slice(), [NodeDesktopMsg::PackageFocus { gained: true, .. }]));
        for notice in notices { window.on(notice); }
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Focused"));
        assert!(native.redraw(&window, &mut cache, None).is_empty());

        window.on(click("Request", 0));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let request = window.take_focus_request().unwrap();
        let notices = native.redraw(&window, &mut cache, Some(request));
        assert_eq!(notices.len(), 2);
        let stale = notices[0].clone();
        for notice in notices { window.on(notice); }
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Blurred"));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Focused"));
        let view = format!("{:?}", window.view());
        assert!(view.contains("Focus log left: F:initial"), "{view}");
        assert!(view.contains("Focus log right: F:initial B"), "{view}");
        window.on(stale);
        assert!(matches!(bridge.step(), StepOutcome::Idle));

        window.on(click("Request", 1));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let request = window.take_focus_request().unwrap();
        let notices = native.redraw(&window, &mut cache, Some(request));
        for notice in notices { window.on(notice); }
        window.on(click("ChangeFocusPayload", 1));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "ChangeFocusPayload"));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Blurred"));
        assert!(matches!(bridge.step(), StepOutcome::Idle));
        let view = format!("{:?}", window.view());
        assert!(view.contains("Focus log left: F:initial B"), "{view}");
        assert!(view.contains("Focus log right: F:initial B"), "{view}");
        assert!(!view.contains("F:changed"), "{view}");

        let mut ui = iced_test::simulator(window.view_iced());
        ui.click("Focus: right").unwrap();
        ui.simulate([PackageFocusHarness::redraw_event()]);
        let events = ui.into_messages().collect::<Vec<_>>();
        assert!(matches!(events.as_slice(), [
            NodeDesktopMsg::PackageClick { .. },
            NodeDesktopMsg::PackageFocus { gained: true, .. },
        ]));
        for event in events { window.on(event); }
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Select"));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Focused"));
        let view = format!("{:?}", window.view());
        assert!(view.contains("Selected right: 112"), "{view}");
        assert!(view.contains("Focus log right: F:initial B F:changed"), "{view}");

        window.on(click("Focused", 0));
        assert!(matches!(bridge.step(), StepOutcome::Idle));
        window.on(click("Request", 1));
        assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
        let request = window.take_focus_request().unwrap();
        let mut cache = iced_runtime::user_interface::Cache::default();
        let mut notices = native.redraw(&window, &mut cache, Some(request)).into_iter();
        let pending = notices.next().unwrap();
        assert!(notices.next().is_none());
        window.on(pending.clone());
        bridge.close_session().unwrap();
        bridge.reopen().unwrap();
        window.on(pending);
        assert!(matches!(bridge.step(), StepOutcome::Idle));
        let view = format!("{:?}", window.view());
        assert!(!view.contains("F:changed") && !view.contains("F:initial"), "{view}");
    }
    bridge.close_session().unwrap();
}

#[cfg(feature = "ui-iced")]
struct PackageFocusHarness(iced::Renderer);

#[cfg(feature = "ui-iced")]
impl PackageFocusHarness {
    fn new() -> Self {
        use iced::advanced::renderer::Headless;

        let renderer = iced_test::futures::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            12.0_f32.into(),
            None,
        ))
        .unwrap();
        Self(renderer)
    }

    fn press(
        &mut self,
        window: &auto_ui_runtime::ui::ace_node_desktop::AceNodeDesktopWindow,
        requests: impl IntoIterator<Item = auto_ui_runtime::ui::component::FocusRequest>,
        key: iced::keyboard::key::Named,
    ) -> Vec<auto_ui_runtime::ui::ace_node_desktop::NodeDesktopMsg> {
        let events = iced_test::simulator::tap_key(key, None).collect::<Vec<_>>();
        self.events(window, &mut Default::default(), requests, &events)
    }

    fn redraw_event() -> iced::Event {
        iced::Event::Window(iced::window::Event::RedrawRequested(iced::time::Instant::now()))
    }

    fn redraw(
        &mut self,
        window: &auto_ui_runtime::ui::ace_node_desktop::AceNodeDesktopWindow,
        cache: &mut iced_runtime::user_interface::Cache,
        request: Option<auto_ui_runtime::ui::component::FocusRequest>,
    ) -> Vec<auto_ui_runtime::ui::ace_node_desktop::NodeDesktopMsg> {
        self.events(window, cache, request, &[Self::redraw_event()])
    }

    fn events(
        &mut self,
        window: &auto_ui_runtime::ui::ace_node_desktop::AceNodeDesktopWindow,
        cache: &mut iced_runtime::user_interface::Cache,
        requests: impl IntoIterator<Item = auto_ui_runtime::ui::component::FocusRequest>,
        events: &[iced::Event],
    ) -> Vec<auto_ui_runtime::ui::ace_node_desktop::NodeDesktopMsg> {
        use auto_ui_runtime::ui::iced::ComponentIced;
        use iced_runtime::user_interface::UserInterface;

        let mut ui = UserInterface::build(
            window.view_iced(),
            iced::Size::new(800.0, 700.0),
            std::mem::take(cache),
            &mut self.0,
        );
        for request in requests {
            ui.operate(&mut self.0, &mut request.operation::<()>());
        }
        let mut messages = Vec::new();
        ui.update(
            events,
            iced::mouse::Cursor::Unavailable,
            &mut self.0,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        *cache = ui.into_cache();
        messages
    }
}

#[test]
fn checked_collection_requests_preserve_the_wire_contract() {
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;
    use serde_json::{json, Value};

    let page = r#"use collection_view.collection_request: cv_view_request, cv_sort_request, cv_item_request
widget CollectionRequests {
    use { package: collection_view from "./collection" }
    msg { Emit(str, str, str), Idle }
    timer { Idle (every_ms: 1000, when: false) }
    model {
        var view_event str = ""
        var sort_event str = ""
        var selection_event str = ""
        var action_event str = ""
    }
    on {
        .Idle -> {}
        .Emit(token str, root str, path str) -> {
            .view_event = cv_view_request(1, token, root, 42, "list")
            .sort_event = cv_sort_request(2, token, root, 42, "name")
            .selection_event = cv_item_request(3, token, root, 42, path, "rev-42", "")
            .action_event = cv_item_request(4, token, root, 42, path, "rev-42", "menu")
        }
    }
    view {
        button "Emit" { onclick: .Emit("token", "资料", "C:\\资料\\\"拍摄\".png") }
        text .view_event { id: "view-event" }
        text .sort_event { id: "sort-event" }
        text .selection_event { id: "selection-event" }
        text .action_event { id: "action-event" }
    }
}
"#;
    let request_source = include_str!("../../../packages/collection-view/collection_request.at");
    let units = vec![unit("app", "app", page), unit("collection_view", "collection_request", request_source)];
    let vue = generate_package_sfc(&units, "app::app").unwrap();
    assert!(vue.contains("<AcePackageView />"));
    let mut engine = PackageStateEngine::new(units, "app::app", LegacyWidgetAlias::Accept).unwrap();
    engine.mount("request", Vec::new()).unwrap();
    let token = "mount\x00\x01\x0b\x1f";
    let scope = "D:\\目录\n";
    let path = "D:\\目录\\\"拍摄\".png";
    engine.dispatch_with_args("request", "Emit", &[
        AceStateValue::Str(token.into()), AceStateValue::Str(scope.into()), AceStateValue::Str(path.into()),
    ]).unwrap();
    let state = engine.read_all("request").unwrap();
    let wire = |name: &str| -> Value {
        let (_, AceStateValue::Str(value)) = state.iter().find(|(key, _)| key == name).unwrap() else {
            panic!("request {name} must be text");
        };
        serde_json::from_str(value).unwrap()
    };
    assert_eq!(wire("view_event"), json!({"seq": 1, "instanceToken": token, "scopeId": scope, "baseRevision": 42, "viewMode": "list"}));
    assert_eq!(wire("sort_event"), json!({"seq": 2, "instanceToken": token, "scopeId": scope, "baseRevision": 42, "sortKey": "name"}));
    assert_eq!(wire("selection_event"), json!({"seq": 3, "instanceToken": token, "scopeId": scope, "baseRevision": 42, "id": path, "itemRevision": "rev-42"}));
    assert_eq!(wire("action_event"), json!({"seq": 4, "instanceToken": token, "scopeId": scope, "baseRevision": 42, "id": path, "itemRevision": "rev-42", "actionId": "menu"}));
}

#[test]
fn checked_collection_view_preserves_item_binding_validation() {
    use ace_av::age_package::{evaluate_package_entry_view, WidgetViewItem};
    use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
    use ace_core::source::{age_package::compile_age_package_with_scenario, packages::PACKAGES_CAPABILITY};

    let page = r#"use collection_view.collection_schema: CollectionAction, CollectionCell, CollectionColumn, CollectionItem
use collection_view.collection_view_util: cv_binding, cv_bound_index, cv_enabled, cv_header, cv_invalid, cv_move, cv_rows

fn row(label str, disabled bool) CollectionItem {
    let cells List<CollectionCell> = [CollectionCell {key: "kind", text: "文档"}]
    let actions List<CollectionAction> = [CollectionAction {id: "open", label: "打开", enabled: true}]
    return CollectionItem {
        id: "文档", itemRevision: "3", label: label.to(Str), kind: "file", icon: "file-text",
        thumbnailUri: "", thumbnailState: "", cells: cells, actions: actions,
        defaultActionId: "open", disabled: disabled
    }
}

fn verify_binding() Str {
    let items List<CollectionItem> = [row("原条目", false)]
    let replacement List<CollectionItem> = [row("新条目", false)]
    let disabled List<CollectionItem> = [row("原条目", true)]
    let duplicate List<CollectionItem> = [row("原条目", false), row("新条目", false)]
    let columns List<CollectionColumn> = []
    let binding = cv_binding(items, 0, "instance-a", "scope-a", 7)
    let accepted = cv_bound_index(items, binding.to(str), "instance-a", "scope-a", 7)
    let stale_content = cv_bound_index(replacement, binding.to(str), "instance-a", "scope-a", 7)
    let stale_scope = cv_bound_index(items, binding.to(str), "instance-a", "scope-b", 7)
    let stale_instance = cv_bound_index(items, binding.to(str), "instance-b", "scope-a", 7)
    let disabled_move = cv_move(disabled, "", 1)
    let validation = cv_invalid(duplicate, columns, "list")
    let shown_columns = cv_header(columns)
    let shown_rows = cv_rows(items, shown_columns, "", "", "scope-a", 7, "instance-a")
    let enabled = cv_enabled(items, 0, "open")
    if enabled {
        let column = shown_columns[0]
        let shown = shown_rows[0]
        return f"${accepted}/${stale_content}/${stale_scope}/${stale_instance}/${disabled_move}/${validation}/${column.key}/${shown.label}/${shown.imageState}"
    }
    return "enabled action refused"
}

widget CollectionBinding {
    msg { Verify }
    timer { Verify (every_ms: 1000, when: false) }
    model { var result str = "" }
    on { .Verify -> { .result = verify_binding() } }
    view { text .result {} }
}
"#;
    let units = vec![
        unit("app", "app", page),
        unit("collection_view", "collection_schema", include_str!("../../../packages/collection-view/collection_schema.at")),
        unit("collection_view", "collection_layout", include_str!("../../../packages/collection-view/collection_layout.at")),
        unit("collection_view", "collection_view_util", include_str!("../../../packages/collection-view/collection_view_util.at")),
    ];
    let compiled = compile_age_package_with_scenario(
        &units,
        "app::app",
        LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.into()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send { instance: 0, msg: "Verify".into() }],
        },
    ).unwrap_or_else(|errors| panic!("{errors:#?}"));
    let view = evaluate_package_entry_view(&compiled, 0, &Default::default()).unwrap();
    assert!(matches!(&view[0], WidgetViewItem::Text(text)
        if text == "0/-1/-1/-1/-1/CollectionView: duplicate_item_id/name/原条目/missing"), "{view:?}");
    let sfc = generate_package_sfc(&units, "app::app").unwrap();
    assert!(sfc.contains("CollectionBinding"));
}

#[test]
fn checked_collection_controls_keep_event_payloads_and_disabled_state() {
    use auto_ui_runtime::ui::ace_node_desktop::{
        current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome,
    };
    use auto_ui_runtime::ui::Component;

    let page = r#"use collection_view.collection_layout: cv_item_style, cv_scoped_focus, cv_focus_step
fn layout_preview() Str {
    let focus = cv_scoped_focus("before", "now", "stale")
    let retained = cv_scoped_focus("now", "now", "focus")
    let narrow = cv_focus_step("down", "grid", 639)
    let medium = cv_focus_step("down", "grid", 640)
    let wide = cv_focus_step("up", "grid", 1024)
    let list = cv_focus_step("right", "list", 1024)
    return f"${focus}/${retained}:${narrow},${medium},${wide},${list}"
}

widget CollectionControls {
    use { package: collection_view from "./collection" }
    msg { Input(str), Toggle }
    timer { Toggle (every_ms: 1000, when: false) }
    model {
        var trail str = ""
        var disabled bool = false
    }
    computed { button_style => cv_item_style(.disabled, true) }
    on {
        .Input(kind str) -> { .trail = .trail + "/" + kind }
        .Toggle -> { .disabled = !.disabled }
    }
    view {
        grid {
            mouse-area (id: "pointer", ondblclick: .Input("double"), oncontextmenu.prevent: .Input("menu")) {
                button (id: "primary", disabled: .disabled,
                    style: .button_style,
                    onclick: .Input("click"), onfocus: .Input("focus"), onblur: .Input("blur"),
                    onkeydown.left.self.prevent: .Input("left"), onkeydown.space.self.stop.prevent: .Input("space")) {
                    text "Primary" {}
                    span (id: "nested") { text "Nested" {} }
                }
            }
        }
        button "Toggle disabled" { onclick: .Toggle }
        text .trail { id: "trail" }
        text layout_preview() { id: "steps" }
    }
}
"#;
    let units = vec![
        unit("app", "app", page),
        unit("collection_view", "collection_layout", include_str!("../../../packages/collection-view/collection_layout.at")),
    ];
    let vue = generate_package_sfc(&units, "app::app").expect("checked Vue controls");
    assert!(vue.contains("checkedDomBinding"));
    let mut bridge = NodeDesktopBridge::load_package(units, "app::app", 1).unwrap();
    ui980::require_prepared();
    let session = bridge.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let click = |message: &str| {
        current_session(|session| {
            let frame = session.package_frame(0).unwrap();
            let target = frame.actions().iter().position(|action| {
                action.source.key == "onclick" && action.source.message == message
            }).unwrap();
            NodeDesktopMsg::PackageClick { session: session.id, instance: 0, revision: frame.revision, target }
        }).unwrap()
    };
    window.on(click("Toggle"));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Toggle"));
    window.on(click("Input"));
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    let audit = auto_ui_runtime::ui::ace_node_desktop::host_session_stats(session).unwrap();
    assert_eq!(audit.stale_rejected, 1);
    window.on(click("Toggle"));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Toggle"));
    window.on(click("Input"));
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, states, .. }
        if msg == "Input" && states.contains(&("trail".into(), "/click".into()))));
    bridge.close_session().unwrap();
}

#[test]
fn file_manager_collection_adapter_preserves_rows_and_actions() {
    use ace_av::age_component::{ComponentScenario, ScenarioOp};
    use ace_av::age_package::{evaluate_package_entry_view, WidgetViewItem};
    use ace_core::source::{age_package::compile_age_package_with_scenario, packages::PACKAGES_CAPABILITY};

    let page = r#"use collection_view.collection_schema: CollectionItem
use official.collection_items: fm_collection_items
use official.file_manager_model: FileManagerRow
use official.file_manager_rows: listed_file_row

fn input_files() List<FileManagerRow> {
    var rows List<FileManagerRow> = []
    rows.push(listed_file_row(0, "unused", "", false, (0).to(u64), (0).to(i64), ""))
    rows.push(listed_file_row(1, "照片.png", "/资料/照片.png", false, (12).to(u64), (0).to(i64), "png"))
    var folder = listed_file_row(2, "资料", "/资料", true, (0).to(u64), (0).to(i64), "png")
    folder.thumb_src = "/api/__auto/media/114"
    rows.push(folder.move)
    rows.push(listed_file_row(3, "README.txt", "/README.txt", false, (2).to(u64), (0).to(i64), "txt"))
    return rows
}

widget CollectionAdapter {
    msg { Load }
    timer { Load (every_ms: 50, when: !.loaded) }
    model {
        var loaded bool = false
        var items List<CollectionItem> = []
    }
    on {
        .Load -> {
            .items = fm_collection_items(input_files(), "42")
            .loaded = true
        }
    }
    view {
        for item in .items {
            text f"${item.id}|${item.label}|${item.itemRevision}|${item.kind}|${item.icon}|${item.thumbnailState}|${item.thumbnailUri}|${item.defaultActionId}" {}
            for cell in item.cells {
                text f"${cell.key}=${cell.text}" {}
            }
            for action in item.actions {
                if action.enabled { text f"${action.id}=${action.label}" {} }
            }
        }
    }
}
"#;
    let mut units = Vec::from(file_manager_row_units(page));
    units.extend([
        unit("official", "collection_items", include_str!("../../../examples/ui/027-file-manager/src/front/components/collection_items.at")),
        unit("official", "fs_util", include_str!("../../../examples/ui/027-file-manager/src/front/components/fs_util.at")),
        unit("collection_view", "collection_schema", include_str!("../../../packages/collection-view/collection_schema.at")),
    ]);
    let vue = generate_package_sfc(&units, "app::app").unwrap();
    assert!(vue.contains("<AcePackageView />"));
    let checked = compile_age_package_with_scenario(
        &units,
        "app::app",
        LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.into()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send { instance: 0, msg: "Load".into() }],
        },
    ).unwrap();
    let view = evaluate_package_entry_view(&checked, 0, &Default::default()).unwrap();
    let text: Vec<_> = view.into_iter().map(|child| match child {
        WidgetViewItem::Text(text) => text,
        other => panic!("unexpected collection adapter output: {other:?}"),
    }).collect();
    let rows: Vec<_> = text.chunks_exact(6).collect();
    assert_eq!(text.len(), 18);
    assert_eq!(rows[0][0], "/资料/照片.png|照片.png|42|image|image|missing||open");
    assert_eq!(rows[1][0], "/资料|资料|42|folder|folder|ready|/api/__auto/media/114|open");
    assert_eq!(rows[2][0], "/README.txt|README.txt|42|file|file|missing||open");
    assert_eq!(&rows[0][1..3], ["size=12 B", "type=图像"]);
    assert_eq!(rows[0][3], "date=--");
    for row in rows {
        assert_eq!(&row[4..], ["open=打开", "menu=菜单"]);
    }
}

#[test]
fn file_manager_rows_preserve_order_and_selection_identity() {
    use ace_av::age_component::{ComponentScenario, ScenarioOp};
    use ace_av::age_package::{evaluate_package_entry_view, WidgetViewItem};
    use ace_core::source::{age_package::compile_age_package_with_scenario, packages::PACKAGES_CAPABILITY};

    let original = include_str!("../../../examples/ui/027-file-manager/src/front/app.at");
    let handler = original.find("        .SelectItem(id) -> {").unwrap();
    let select_item = original[handler..].split_once("\n\n").unwrap().0;
    let page = r#"use official.file_manager_model: FileManagerRow
use official.file_manager_rows: matches_file, file_count_label, fm_path_index, sort_files
fn make_row(name str, path str, size u64, directory bool) FileManagerRow {
    return FileManagerRow {
        id: -1, name: name.to(Str), path: path.to(Str), is_dir: directory, size: size, size_str: "shown",
        file_ext: "", type_name: "", date: "", is_hidden: false, thumb_src: ""
    }
}
fn sample_rows() List<FileManagerRow> {
    var result List<FileManagerRow> = []
    let gib u64 = (1073741824).to(u64)
    let first = make_row("𐀀.txt", "/first", gib * gib * (8).to(u64), false)
    let second = make_row("龘.txt", "/second", gib * gib * (8).to(u64) - (1).to(u64), false)
    let directory = make_row("zzz", "/directory", (0).to(u64), true)
    let equal = make_row("龘.txt", "/equal", gib * gib * (8).to(u64) - (1).to(u64), false)
    result.push(first.move)
    result.push(second.move)
    result.push(directory.move)
    result.push(equal.move)
    return result
}
widget Rows {
    use { package: official from "./components" }
    msg { Reveal, SelectItem(int) }
    timer { Reveal (every_ms: 50, when: .initial) }
    model {
        var initial bool = true
        var size_order str = ""
        var name_order str = ""
        var status str = ""
        var selection str = ""
        var files_view List<FileManagerRow> = []
        var selected_id int = -1
        var selected_path str = ""
        var selected_name str = ""
        var selected_info str = ""
    }
    on {
        .Reveal -> {
            .initial = false
            .files_view = sort_files(sample_rows(), "size", "desc")
            let names = sort_files(sample_rows(), "name", "asc")
            let selected int = fm_path_index(.files_view, "/second")
            .size_order = "" + .files_view[0].path + .files_view[1].path + .files_view[2].path + .files_view[3].path
            .name_order = "" + names[0].path + names[1].path + names[2].path + names[3].path
            .SelectItem(selected)
            .selection = f"${selected}:${.files_view[selected].id}"
            if matches_file(".龘.txt", "龘", true) && !matches_file(".龘.txt", "龘", false) && !matches_file("龘.txt", "𐀀", true) {
                .status = file_count_label(500, true)
            }
        }
{{SELECT_ITEM}}
    }
    view {
        text .size_order {}
        text .name_order {}
        text .selection {}
        text .status {}
        text .selected_info {}
        text .selected_path {}
    }
}
"#.replace("{{SELECT_ITEM}}", select_item);
    let units = file_manager_row_units(&page);
    let vue = generate_package_sfc(&units, "app::app").unwrap();
    assert!(vue.contains("<AcePackageView />"));
    let checked = compile_age_package_with_scenario(
        &units,
        "app::app",
        LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.to_owned()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send { instance: 0, msg: "Reveal".into() }],
        },
    ).unwrap();
    let view = evaluate_package_entry_view(&checked, 0, &Default::default()).unwrap();
    assert_eq!(view, [
        WidgetViewItem::Text("/directory/first/second/equal".into()),
        WidgetViewItem::Text("/directory/second/equal/first".into()),
        WidgetViewItem::Text("2:2".into()),
        WidgetViewItem::Text("500+ 个项目（已截断）".into()),
        WidgetViewItem::Text("选定: 龘.txt (shown)".into()),
        WidgetViewItem::Text("/second".into()),
    ]);
}

#[test]
fn full_file_manager_listing_publishes_view_and_scalar_state() {
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;

    let page = r#"use official.file_manager_model: FileManagerRow
use official.file_manager_rows: listed_file_row, sort_files
fn sample_rows() List<FileManagerRow> {
    var result List<FileManagerRow> = []
    var i int = 500
    while i > 0 {
        let name = f"同一前缀-文件-𐀀-${i}.txt"
        let path = f"/folder/${i}"
        let row = listed_file_row(i, name.to(str), path.to(str), false, i.to(u64), (2147483647).to(i64) + (61505153).to(i64), "txt")
        result.push(row.move)
        i = i - 1
    }
    return result
}
widget Rows {
    use { package: official from "./components" }
    msg { Reveal }
    timer { Reveal (every_ms: 50, when: .initial) }
    model {
        var initial bool = true
        var display str = ""
    }
    on {
        .Reveal -> {
            .initial = false
            let entries = sort_files(sample_rows(), "name", "asc")
            .display = f"${entries.len()}" + entries[0].path + entries[499].path + " " + entries[499].date
        }
    }
    view {
        text .display {}
    }
}
"#;
    let units = file_manager_row_units(page);
    let mut desktop = PackageStateEngine::new(units.to_vec(), "app::app", LegacyWidgetAlias::Accept)
        .expect("checked 027 file row pipeline");
    desktop.mount("files", Vec::new()).expect("mount file list");
    desktop.dispatch("files", "Reveal").expect("publish 500 sorted file rows");
    assert_eq!(desktop.read_all("files"), Some(vec![
        ("initial".into(), AceStateValue::Bool(false)),
        ("display".into(), AceStateValue::Str("500/folder/1/folder/99 2040-01-01 00:00".into())),
    ]));
}

#[test]
fn file_manager_metadata_preserves_wide_size_and_time() {
    use ace_av::age_component::{ComponentScenario, ScenarioOp};
    use ace_av::age_package::{evaluate_package_entry_view, WidgetViewItem};
    use ace_core::source::{age_package::compile_age_package_with_scenario, packages::PACKAGES_CAPABILITY};

    let page = r#"use official.file_manager_model: FileManagerRow
use official.file_manager_metadata: format_bytes, format_modified, summarize_bytes
fn max_bytes() u64 {
    let gib u64 = (1073741824).to(u64)
    let half u64 = gib * gib * (8).to(u64) - (1).to(u64)
    return half * (2).to(u64) + (1).to(u64)
}
fn make_row(is_dir bool) FileManagerRow {
    return FileManagerRow {
        id: 0, name: "", path: "", is_dir: is_dir, size: max_bytes(), size_str: "",
        file_ext: "", type_name: "", date: "", is_hidden: false, thumb_src: ""
    }
}
fn sample_rows() List<FileManagerRow> {
    var result List<FileManagerRow> = []
    let first = make_row(false)
    let second = make_row(false)
    let directory = make_row(true)
    result.push(first.move)
    result.push(second.move)
    result.push(directory.move)
    return result
}
widget Metadata {
    use { package: official from "./components" }
    msg { Reveal }
    timer { Reveal (every_ms: 50, when: .initial) }
    model {
        var initial bool = true
        var size_text str = ""
        var total_text str = ""
        var date_text str = ""
        var missing_time str = ""
    }
    on {
        .Reveal -> {
            .initial = false
            .size_text = format_bytes(max_bytes())
            .total_text = summarize_bytes(sample_rows())
            .date_text = format_modified((2147483647).to(i64) + (61505153).to(i64))
            .missing_time = format_modified(-(1).to(i64))
        }
    }
    view {
        text .size_text {}
        text .total_text {}
        text .date_text {}
        text .missing_time {}
    }
}
"#;
    let units = [
        unit("app", "app", page),
        unit("official", "file_manager_model", include_str!("../../../examples/ui/027-file-manager/src/front/components/file_manager_model.at")),
        unit("official", "file_manager_metadata", include_str!("../../../examples/ui/027-file-manager/src/front/components/file_manager_metadata.at")),
    ];
    let vue = generate_package_sfc(&units, "app::app").unwrap();
    assert!(vue.contains("<AcePackageView />"));
    let checked = compile_age_package_with_scenario(
        &units,
        "app::app",
        LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.to_owned()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send { instance: 0, msg: "Reveal".into() }],
        },
    ).unwrap();
    let view = evaluate_package_entry_view(&checked, 0, &Default::default()).unwrap();
    assert_eq!(view, [
        WidgetViewItem::Text("17179869183.9 GB".into()),
        WidgetViewItem::Text("34359738367.9 GB".into()),
        WidgetViewItem::Text("2040-01-01 00:00".into()),
        WidgetViewItem::Text("--".into()),
    ]);
}

#[test]
fn file_manager_rows_and_path_are_published_only_after_an_authorized_read() {
    use ace_av::age_package::WidgetViewItem;
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, StepOutcome},
        ace_state_bridge::AceStateValue,
    };

    let original = include_str!("../../../examples/ui/027-file-manager/src/front/app.at");
    let start = original.find("        .NavigateInSession(target_path str").unwrap();
    let mutation = original.find("        .RequestMutation(operation str").unwrap();
    let listing = original.find("        .DirectoryListed(listing DirListing").unwrap();
    let end = original.find("        .GoBack -> {").unwrap();
    let advance = original.find("        .AdvanceBoot -> {").unwrap();
    let publish = original.find("        .PublishCollection -> {").unwrap();
    let select = original.find("        .SelectItem(id) -> {").unwrap();
    let handler = |start| original[start..].split_once("\n\n").unwrap().0;
    let contract = &original[original.find("type DirEntry {").unwrap()..original.find("widget App {").unwrap()];
    let page = format!(r#"use official.file_manager_model: FileManagerBreadcrumb, FileManagerRow, FileManagerShortcut, FileManagerDrive, PreparedListing, extend_history
use official.file_manager_paths: build_crumbs, slice_crumbs
use official.file_manager_rows: matches_file, listed_file_row, sort_files, file_count_label, fm_path_index
use official.file_manager_metadata: summarize_bytes
{contract}
widget App {{
    use {{ package: official from "./components" }}
    msg {{ AdvanceBoot, NavigateInSession(str, int), RequestDirectory(str, int, bool), DirectoryListed(DirListing), CommitDirectoryPath(str, int), CommitNavigation(str, int), PublishCollection, SelectItem(int) }}
    bind {{ "file:list" -> .DirectoryListed }}
    model {{
        var booting bool = true
        var tick_count int = 8
        var is_vm str = ""
        var file_request_pending bool = false
        var file_request_path str = ""
        var file_request_history_index int = -1
        var file_request_startup bool = false
        var file_request_revision int = 0
        var navigation_error str = ""
        var home str = ""
        var back_fs bool = false
        var quick List<FileManagerShortcut> = []
        var drives List<FileManagerDrive> = []
        var current_path str = ""
        var selected_path str = "old"
        var addr str = ""
        var selected_id int = -1
        var selected_name str = ""
        var selected_info str = ""
        var editing_id int = -1
        var ctx_id int = -1
        var crumbs_expanded bool = false
        var crumb_gap bool = false
        var crumbs List<FileManagerBreadcrumb> = []
        var crumbs_head List<FileManagerBreadcrumb> = []
        var crumbs_tail List<FileManagerBreadcrumb> = []
        var history List<str> = []
        var search_q str = ""
        var show_hidden bool = false
        var sort_col str = "name"
        var sort_dir str = "asc"
        var files_view List<FileManagerRow> = []
        var item_count_str str = ""
        var total_size_str str = ""
        var collection_revision int = 0
        var history_idx int = -1
        var can_back bool = false
        var can_forward bool = false
    }}
    on {{
{}
{}
{}
{publish_handler}
{select_handler}
    }}
    view {{
        for segment in .crumbs_head {{ text segment.name {{}} }}
        if .crumb_gap {{ text "…" {{}} }}
        for segment in .crumbs_tail {{ text segment.name {{}} }}
        for shortcut in .quick {{ text shortcut.label {{}} }}
        text f"盘位 ${{.drives.len()}}" {{}}
        text .item_count_str {{}}
        text .total_size_str {{}}
        for row in .files_view {{
            text row.name {{}}
            text row.type_name {{}}
        }}
    }}
}}"#, &original[start..mutation], &original[listing..end], handler(advance), publish_handler = handler(publish), select_handler = handler(select));
    let mut units = file_manager_row_units(&page).to_vec();
    units.push(
        unit("official", "file_manager_paths", include_str!("../../../examples/ui/027-file-manager/src/front/components/file_manager_paths.at")),
    );
    let vue = generate_package_sfc(&units, "app::app").unwrap();
    assert!(vue.contains("readFileDirectory") && vue.contains("includeExtension: true"));

    ui980::require_prepared();
    let fixture = tempfile::tempdir().unwrap();
    let denied = fixture.path().join("denied");
    let root = fixture.path().join("one/two/hidden-middle/four/父级/目录");
    std::fs::create_dir(&denied).unwrap();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("资料.É"), "hello").unwrap();
    std::fs::write(root.join(".profile"), "").unwrap();
    std::fs::create_dir(root.join("Documents")).unwrap();

    let mut native = NodeDesktopBridge::load_package(units, "app::app", 1).unwrap();
    native.open().unwrap();
    native.grant_file_read(0, &root).unwrap();
    let read = native.dispatch_typed(0, "AdvanceBoot", &[]);
    assert!(matches!(&read, StepOutcome::Dispatched { states, .. }
        if states.iter().any(|(name, value)| name == "current_path" && value.ends_with("目录"))
            && states.contains(&("history_idx".into(), "0".into()))
            && states.contains(&("booting".into(), "false".into()))
            && states.contains(&("selected_path".into(), String::new()))), "{read:?}");
    let labels = || current_session(|session| {
        session.package_frame(0).unwrap().view.iter().filter_map(|item| {
            if let WidgetViewItem::Text(text) = item { Some(text.clone()) } else { None }
        }).collect::<Vec<_>>()
    }).unwrap();
    let collapsed = labels();
    assert_eq!(&collapsed[1..], ["…", "父级", "目录", "主目录", "文档", "盘位 1", "2 个项目", "5 B", "Documents", "文件夹", "资料.É", "é 文件"]);

    let reply = native.dispatch_typed(0, "NavigateInSession", &[
        AceStateValue::Str(denied.to_str().unwrap().into()), AceStateValue::Int(-1),
    ]);
    assert!(matches!(&reply, StepOutcome::Dispatched { states, .. }
        if states.iter().any(|(name, value)| name == "current_path" && value.ends_with("目录"))
            && states.contains(&("history_idx".into(), "0".into()))
            && states.iter().any(|(name, value)| name == "navigation_error" && value.ends_with("denied"))), "{reply:?}");
    assert_eq!(labels(), collapsed);
    native.close_session().unwrap();
}

// The AV host resolves directory requests at the live commit boundary.
#[test]
fn desktop_file_binding_commits_one_scoped_read_and_retains_its_reply() {
    use auto_ui_runtime::Component;
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        ace_state_bridge::AceStateValue,
    };

    let page = r#"type DirEntry {
    name str
    path str
    is_dir bool
    size u64
    mtime i64
}
type DirListing {
    ok bool
    path str
    entries List<DirEntry>
}
widget Browser {
    msg { NavTo(str), DirectoryListed(DirListing), Remember }
    bind { "file:list" -> .DirectoryListed }
    model {
        var file_request_pending bool = false
        var file_request_path str = ""
        var current_path str = ""
        var first_name str = ""
        var entries_count int = 0
        var reads int = 0
        var touches int = 0
    }
    on {
        .NavTo(path str) -> {
            .file_request_path = path
            .file_request_pending = true
        }
        .DirectoryListed(listing DirListing) -> {
            .file_request_pending = false
            if listing.ok {
                .reads += 1
                .current_path = listing.path
                .entries_count = listing.entries.len()
                .first_name = ""
                if .entries_count > 0 {
                    .first_name = listing.entries[0].name
                }
            }
        }
        .Remember -> { .touches += 1 }
    }
    view {
        button "Refresh" { onclick: .NavTo(.current_path) }
        text f"${.current_path} | ${.first_name} | ${.entries_count}" {}
    }
}
"#;
    let wrong = page.replacen("size u64", "size int", 1);
    let product = generate_package_sfc(&[unit("app", "app", page)], "app::app").unwrap();
    assert!(product.contains("from '@/lib/file-browser'"));
    assert!(product.contains("reply: \"DirectoryListed\""));
    let error = generate_package_sfc(&[unit("app", "app", &wrong)], "app::app").unwrap_err();
    assert!(error.contains("source.age.sfc-file-binding"), "{error}");
    let rejected = PackageStateEngine::new(
        vec![unit("app", "app", &wrong)], "app::app", LegacyWidgetAlias::Accept,
    ).err().expect("validate the whole host reply shape before mounting");
    assert!(rejected.contains("type-mismatch"), "{rejected}");

    ui980::require_prepared();
    let root = tempfile::tempdir().unwrap();
    let allowed = root.path().join("allowed");
    let outside = root.path().join("outside");
    std::fs::create_dir(&allowed).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(allowed.join("计划.txt"), "visible").unwrap();
    std::fs::write(outside.join("outside.txt"), "outside").unwrap();

    let mut bridge = NodeDesktopBridge::load(page, "app.at", 2).unwrap();
    let session = bridge.open().unwrap();
    let path = AceStateValue::Str(allowed.to_str().unwrap().into());
    let outside_path = AceStateValue::Str(outside.to_str().unwrap().into());
    let assert_state = |outcome, field, expected| {
        assert!(matches!(&outcome, StepOutcome::Dispatched { states, .. }
            if states.iter().any(|(name, value)| name == field && value == expected)), "{outcome:?}");
    };

    let find_home = [AceStateValue::Str(String::new())];
    assert_state(bridge.dispatch_typed(0, "NavTo", &find_home), "reads", "0");
    bridge.grant_file_read(0, &allowed).unwrap();
    assert_state(bridge.dispatch_typed(0, "NavTo", &find_home), "first_name", "计划.txt");
    assert_state(bridge.dispatch_typed(0, "NavTo", std::slice::from_ref(&path)), "first_name", "计划.txt");
    assert_state(bridge.dispatch_typed(1, "NavTo", std::slice::from_ref(&path)), "reads", "0");
    assert_state(bridge.dispatch_typed(0, "NavTo", &[outside_path]), "first_name", "计划.txt");

    std::fs::remove_file(allowed.join("计划.txt")).unwrap();
    std::fs::write(allowed.join("new.txt"), "new contents").unwrap();
    assert_state(bridge.dispatch_typed(0, "Remember", &[]), "first_name", "计划.txt");
    bridge.revoke_file_read(0).unwrap();
    assert_state(bridge.dispatch_typed(0, "NavTo", std::slice::from_ref(&path)), "first_name", "计划.txt");
    assert_state(bridge.dispatch_typed(0, "NavTo", &find_home), "first_name", "计划.txt");
    bridge.grant_file_read(0, &allowed).unwrap();

    let revision = current_session(|s| s.package_frame(0).unwrap().revision).unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let _ = window.on(NodeDesktopMsg::PackageClick { session, instance: 0, revision, target: 0 });
    assert_state(bridge.step(), "first_name", "new.txt");

    bridge.close_session().unwrap();
    assert!(bridge.grant_file_read(0, &allowed).is_err());
    let reopened = bridge.reopen().unwrap();
    assert_ne!(reopened, session);
    let _ = window.on(NodeDesktopMsg::PackageClick { session, instance: 0, revision, target: 0 });
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    assert_state(bridge.dispatch_typed(0, "NavTo", std::slice::from_ref(&path)), "first_name", "");
    bridge.close_session().unwrap();

    let missing = root.path().join("missing");
    let error = bridge.open_with_file_access(&missing, Default::default()).unwrap_err();
    assert!(error.starts_with("ace.desktop.file-grant:"), "{error}");
    assert!(current_session(|_| ()).is_none());
    let fresh = bridge.open_with_file_access(&allowed, Default::default()).unwrap();
    assert_ne!(fresh, reopened);
    for instance in 0..2 {
        assert_state(bridge.dispatch_typed(instance, "NavTo", std::slice::from_ref(&path)), "first_name", "new.txt");
    }
    bridge.close_session().unwrap();
}

#[test]
fn checked_file_manager_preferences_restore_and_save_committed_state() {
    use auto_lang_core::vm::ffi::stdlib::{storage_host_publish, storage_host_read_fresh, storage_raw_remove};
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;

    let original = include_str!("../../../examples/ui/027-file-manager/src/front/app.at");
    let scratch = tempfile::tempdir().unwrap();
    let namespace = format!("plan114.{}", scratch.path().file_name().unwrap().to_str().unwrap());
    let bindings = original.lines().filter(|line| line.trim_start().starts_with("\"storage:fileman."))
        .map(|line| line.replace("fileman.", &format!("{namespace}."))).collect::<Vec<_>>().join("\n");
    let restore = original.split_once("        .RestoreViewMode(value str)").unwrap().1
        .split_once("        // 启动完成后").unwrap().0;
    let source = format!(r#"widget Preferences {{
    msg {{ RestoreViewMode(str), RestoreSortColumn(str), RestoreSortDirection(str), RestoreHidden(str), Change, Fail(int) }}
    bind {{
{bindings}
    }}
    model {{
        var view_mode str = "list"
        var sort_col str = "name"
        var sort_dir str = "asc"
        var show_hidden bool = false
        var attempts int = 0
    }}
    on {{
        .RestoreViewMode(value str){restore}
        .Change -> {{
            .view_mode = "list"
            .sort_col = "size"
            .sort_dir = "desc"
            .show_hidden = true
        }}
        .Fail(divisor int) -> {{
            .view_mode = "grid"
            .attempts = 1 / divisor
        }}
    }}
    view {{ text .view_mode {{}} }}
}}"#);
    let units = vec![unit("app", "app", &source)];
    let vue = generate_package_sfc(&units, "app::app").unwrap();
    assert!(vue.contains(&format!(r#"["{namespace}.view_mode","view_mode","RestoreViewMode"]"#)));
    let key = |field: &str| format!("{namespace}.{field}");
    storage_host_publish(&key("view_mode"), "grid".into());
    storage_host_publish(&key("sort_dir"), "invalid".into());
    let mount = || {
        let mut engine = PackageStateEngine::new(units.clone(), "app::app", LegacyWidgetAlias::Accept).unwrap();
        engine.mount("app", Vec::new()).unwrap();
        engine
    };
    let mut first = mount();
    let value = |name: &str, value: &str| (name.into(), AceStateValue::Str(value.into()));
    assert!(first.read_all("app").unwrap().contains(&value("view_mode", "grid")));
    assert!(first.read_all("app").unwrap().contains(&value("sort_dir", "asc")));
    assert_eq!(storage_host_read_fresh(&key("sort_dir")).as_deref(), Some("invalid"));
    assert!(first.dispatch_with_args("app", "RestoreViewMode", &[AceStateValue::Str("list".into())]).is_err());
    first.dispatch("app", "Change").unwrap();
    assert!(first.dispatch_with_args("app", "Fail", &[AceStateValue::Int(0)]).is_err());
    assert_eq!(storage_host_read_fresh(&key("view_mode")).as_deref(), Some("list"));
    drop(first);
    let reopened = mount();
    let state = reopened.read_all("app").unwrap();
    assert!(state.contains(&value("view_mode", "list")));
    assert!(state.contains(&value("sort_col", "size")));
    assert!(state.contains(&value("sort_dir", "desc")));
    assert!(state.contains(&("show_hidden".into(), AceStateValue::Bool(true))));
    for field in ["view_mode", "sort_col", "sort_dir", "show_hidden"] {
        storage_raw_remove(&key(field));
    }
}

#[test]
fn original_file_action_handlers_use_the_scoped_file_session() {
    use auto_lang_core::file_session::FileWritePermissions;
    use auto_ui_runtime::ui::{ace_node_desktop::StepOutcome, ace_state_bridge::AceStateValue};

    let original = include_str!("../../../examples/ui/027-file-manager/src/front/app.at");
    let block = |start: &str, end: &str| {
        let start = original.find(start).unwrap();
        let end = original[start..].find(end).unwrap() + start;
        &original[start..end]
    };
    let page = format!(r#"
{directory_types}
{child_path}
fn native_file_mutation(operation str, path str, target str) str {{
    return "unexpected native VM call"
}}
widget FileActions {{
    msg {{
        Folder(str), Edit(str, str), Clipboard(str, str, str), New(str, str), Remove(str, bool),
        CommitRename, CtxPaste, CommitNew, ExecuteDelete,
        RequestMutation(str, str, str, str, bool), MutationApplied(bool),
        RequestDirectory(str, int, bool), DirectoryListed(DirListing), Navigate(str, int)
    }}
    bind {{
        "file:list" -> .DirectoryListed
        "file:mutate" -> .MutationApplied
    }}
    model {{
        var is_vm str = "av"
        var back_fs bool = true
        var current_path str = ""
        var editing_id int = -1
        var edit_name str = ""
        var rename_path str = ""
        var rename_open bool = false
        var ctx_id int = -1
        var clipboard_op str = ""
        var clipboard_path str = ""
        var clipboard_name str = ""
        var clipboard_is_dir bool = false
        var clipboard_label str = ""
        var new_modal_open bool = false
        var new_modal_name str = ""
        var new_modal_type str = ""
        var confirm_del_open bool = false
        var confirm_del_id int = -1
        var confirm_del_path str = ""
        var confirm_del_name str = ""
        var confirm_del_is_dir bool = false
        var mutation_notice str = ""
        var mutation_error str = ""
        var navigation_error str = ""
        var history_idx int = 0
        var file_mutation_pending bool = false
        var file_mutation_operation str = ""
        var file_mutation_path str = ""
        var file_mutation_target str = ""
        var file_mutation_label str = ""
        var file_mutation_move bool = false
        var file_mutation_view_revision int = 0
        var file_request_revision int = 0
        var file_request_pending bool = false
        var file_request_path str = ""
        var file_request_history_index int = -1
        var file_request_startup bool = false
        var refreshes int = 0
    }}
    on {{
        .Folder(path str) -> {{ .current_path = path }}
        .Edit(path str, name str) -> {{
            .rename_path = path
            .edit_name = name
            .editing_id = 0
        }}
        .Clipboard(path str, name str, op str) -> {{
            .clipboard_path = path
            .clipboard_name = name
            .clipboard_op = op
        }}
        .New(name str, kind str) -> {{
            .new_modal_name = name
            .new_modal_type = kind
        }}
        .Remove(path str, directory bool) -> {{
            .confirm_del_path = path
            .confirm_del_name = path
            .confirm_del_is_dir = directory
        }}
        .Navigate(path str, index int) -> {{ .navigation_error = "unexpected native VM navigation" }}
        .DirectoryListed(listing DirListing) -> {{
            .file_request_pending = false
            if listing.ok {{ .refreshes += 1 }}
        }}
{requests}
{rename}
{paste}
{create}
{delete}
    }}
    view {{
        text .mutation_notice {{}}
        text .mutation_error {{}}
    }}
}}
"#,
        directory_types = block("type DirEntry {", "fn prepare_file_listing"),
        child_path = block("fn child_path(", "widget App {"),
        requests = block("        .RequestDirectory(requested_path", "        .DirectoryListed(listing"),
        rename = block("        .CommitRename ->", "        .CancelRename ->"),
        paste = block("        .CtxPaste ->", "        .CtxDelete ->"),
        create = block("        .CommitNew ->", "        // ---- 删除确认"),
        delete = block("        .ExecuteDelete ->", "        .CancelDelete ->"),
    );
    let vue = generate_package_sfc(&[unit("app", "app", &page)], "app::app").unwrap();
    assert!(vue.contains("applyFileMutation") && vue.contains("readFileDirectory"));

    ui980::require_prepared();
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("allowed");
    let outside = fixture.path().join("outside");
    let sub = root.join("remote");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::create_dir(&outside).unwrap();
    let protected = outside.join("protected.txt");
    std::fs::write(&protected, "outside").unwrap();
    let path = |path: &std::path::Path| AceStateValue::Str(path.to_str().unwrap().into());
    let text = |value: &str| AceStateValue::Str(value.into());
    let mut native = NodeDesktopBridge::load(&page, "app.at", 1).unwrap();
    native.open().unwrap();
    let assert_refreshes = |outcome, expected: &str| {
        assert!(matches!(&outcome, StepOutcome::Dispatched { states, .. }
            if states.contains(&("refreshes".into(), expected.into()))
                && states.contains(&("file_mutation_pending".into(), "false".into()))
                && states.contains(&("navigation_error".into(), String::new()))), "{outcome:?}");
    };
    native.grant_file_read(0, &root).unwrap();
    assert_refreshes(native.dispatch_typed(0, "Folder", &[path(&root)]), "0");
    assert_refreshes(native.dispatch_typed(0, "New", &[text("a.txt"), text("file")]), "0");
    assert_refreshes(native.dispatch_typed(0, "CommitNew", &[]), "0");
    let created = root.join("a.txt");
    assert!(!created.exists());
    native.grant_file_access(0, &root, FileWritePermissions {
        create: true, copy: true, rename: true, delete: true,
    }).unwrap();
    assert_refreshes(native.dispatch_typed(0, "CommitNew", &[]), "1");
    assert_eq!(std::fs::read(&created).unwrap(), b"");
    std::fs::write(&created, "copy oracle").unwrap();
    assert_refreshes(native.dispatch_typed(0, "Edit", &[path(&created), text("b.txt")]), "1");
    assert_refreshes(native.dispatch_typed(0, "CommitRename", &[]), "2");
    let renamed = root.join("b.txt");
    assert!(!created.exists());
    assert_refreshes(native.dispatch_typed(0, "Clipboard", &[path(&renamed), text("b.txt"), text("copy")]), "2");
    assert_refreshes(native.dispatch_typed(0, "Folder", &[path(&sub)]), "2");
    assert_refreshes(native.dispatch_typed(0, "CtxPaste", &[]), "3");
    let copied = sub.join("b.txt");
    assert_eq!(std::fs::read(&renamed).unwrap(), std::fs::read(&copied).unwrap());
    assert_refreshes(native.dispatch_typed(0, "Remove", &[path(&protected), AceStateValue::Bool(false)]), "3");
    assert_refreshes(native.dispatch_typed(0, "ExecuteDelete", &[]), "3");
    assert_eq!(std::fs::read(&protected).unwrap(), b"outside");
    assert_refreshes(native.dispatch_typed(0, "Remove", &[path(&copied), AceStateValue::Bool(false)]), "3");
    assert_refreshes(native.dispatch_typed(0, "ExecuteDelete", &[]), "4");
    assert!(!copied.exists());
    native.close_session().unwrap();
}

#[cfg(feature = "ui-iced")]
#[test]
fn checked_iced_viewport_shows_rows_grid_and_modal() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{host_session_stats, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        ace_state_bridge::AceStateValue,
        iced::ComponentIced,
        Component,
    };
    use iced::keyboard::{key::Named, Key};

    ui980::require_prepared();
    let source = r#"widget Viewport {
        msg { UseGrid, UseList, Pick, Open, Edit(str), Save, Cancel }
        timer { Open (every_ms: 1000, when: false) }
        model {
            var grid_mode bool = false
            var modal bool = false
            var selected str = "none"
            var input str = ""
            var saved str = ""
        }
        on {
            .UseGrid -> { .grid_mode = true }
            .UseList -> { .grid_mode = false }
            .Pick -> { .selected = "zeta.txt" }
            .Open -> { .input = ""; .modal = true }
            .Edit(value) -> { .input = value }
            .Save -> { .saved = .input; .modal = false }
            .Cancel -> { .modal = false }
        }
        view {
            col (style: "w-full h-full flex flex-col overflow-hidden") {
                row (style: "w-full h-10 shrink-0") {
                    button (text: "Grid", onclick: .UseGrid) {}
                    button (text: "List", onclick: .UseList) {}
                    button (text: "Edit name", onclick: .Open) {}
                }
                row (style: "flex-1 min-h-0 w-full") {
                    col (style: "flex-1 min-h-0 w-full bg-card overflow-y-auto") {
                        if .grid_mode {
                            grid (style: "w-full grid grid-cols-3 gap-2") {
                                button (text: "alpha.txt") {}
                                button (text: "zeta.txt", onclick: .Pick) {}
                            }
                        } else {
                            col (style: "w-full") {
                                button (text: "alpha.txt") {}
                                button (text: "zeta.txt", onclick: .Pick) {}
                            }
                        }
                    }
                }
                row (style: "w-full h-9 shrink-0") {
                    text (text: f"selected: ${.selected}") {}
                }
                alert-dialog (open: .modal) {
                    alert-dialog-content {
                        alert-dialog-title "Name" {}
                        input {
                            value: .input
                            placeholder: "New name"
                            oninput: .Edit
                            onenter: .Save
                        }
                        alert-dialog-footer {
                            alert-dialog-cancel "Cancel" { onclick: .Cancel }
                            alert-dialog-action "Save" { onclick: .Save }
                        }
                    }
                }
            }
        }
    }"#;
    let mut bridge =
        NodeDesktopBridge::load_package(vec![unit("app", "viewport", source)], "app::viewport", 1)
            .unwrap();
    let session = bridge.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    let click = |window: &mut AceNodeDesktopWindow, name: &str| {
        let mut ui = iced_test::simulator(window.view_iced());
        ui.click(name).unwrap();
        for message in ui.into_messages() {
            window.on(message);
        }
    };
    let positions = |window: &AceNodeDesktopWindow| {
        let mut ui = iced_test::simulator(window.view_iced());
        let alpha = ui.find("alpha.txt").unwrap().visible_bounds().unwrap();
        let zeta = ui.find("zeta.txt").unwrap().visible_bounds().unwrap();
        (alpha, zeta)
    };
    let assert_event = |bridge: &mut NodeDesktopBridge, expected: &str| {
        assert!(matches!(
            bridge.step(),
            StepOutcome::Dispatched { msg, .. } if msg == expected
        ));
    };

    let (alpha, zeta) = positions(&window);
    assert!(alpha.height > 0.0 && zeta.y > alpha.y);
    click(&mut window, "zeta.txt");
    assert_event(&mut bridge, "Pick");
    click(&mut window, "Grid");
    assert_event(&mut bridge, "UseGrid");
    let (alpha, zeta) = positions(&window);
    assert!(zeta.x > alpha.x);
    assert!(iced_test::simulator(window.view_iced())
        .find("selected: zeta.txt")
        .unwrap()
        .visible_bounds()
        .is_some());
    click(&mut window, "List");
    assert_event(&mut bridge, "UseList");
    let (alpha, zeta) = positions(&window);
    assert!(zeta.y > alpha.y);

    click(&mut window, "Edit name");
    assert_event(&mut bridge, "Open");
    let mut dialog = iced_test::simulator(window.view_iced());
    dialog.click("New name").unwrap();
    dialog.typewrite("验A");
    dialog.tap_key(Key::Named(Named::Enter));
    let events: Vec<_> = dialog.into_messages().collect();
    assert!(matches!(
        events.as_slice(),
        [NodeDesktopMsg::PackageText { text: first, .. }, NodeDesktopMsg::PackageText { text: second, .. }, NodeDesktopMsg::PackageClick { .. }]
            if first == "验" && second == "验A"
    ));
    let late = events[0].clone();
    for event in events {
        window.on(event);
    }
    assert_event(&mut bridge, "Edit");
    assert_event(&mut bridge, "Edit");
    assert_event(&mut bridge, "Save");
    assert!(bridge
        .states(0)
        .unwrap()
        .contains(&("saved".into(), AceStateValue::Str("验A".into()))));

    let stale = host_session_stats(session).unwrap().stale_rejected;
    window.on(late);
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    assert_eq!(
        host_session_stats(session).unwrap().stale_rejected,
        stale + 1
    );

    click(&mut window, "Edit name");
    assert_event(&mut bridge, "Open");
    let mut dialog = iced_test::simulator(window.view_iced());
    dialog.click("New name").unwrap();
    dialog.typewrite("Z");
    dialog.click("Cancel").unwrap();
    let mut events = dialog.into_messages();
    let edit = events.next().unwrap();
    let cancel = events.next().unwrap();
    assert!(events.next().is_none());
    window.on(cancel);
    window.on(edit);
    assert_event(&mut bridge, "Cancel");
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    assert!(bridge
        .states(0)
        .unwrap()
        .contains(&("saved".into(), AceStateValue::Str("验A".into()))));
    bridge.close_session().unwrap();
}

#[test]
fn checked_keyed_double_click_survives_selection_but_rejects_replaced_items() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{current_session, AceNodeDesktopWindow, NodeDesktopMsg, StepOutcome},
        iced::ComponentIced,
        Component,
    };
    use std::{cell::RefCell, collections::VecDeque, time::Duration};

    thread_local! {
        static PRESSES: RefCell<VecDeque<u32>> = const { RefCell::new(VecDeque::new()) };
    }

    fn next_press() -> Option<(u32, u32)> {
        PRESSES.with(|times| times.borrow_mut().pop_front().map(|at| (at, 500)))
    }

    ui980::require_prepared();
    let source = r#"widget FileRows {
    msg { Select, Open(str), Replace, Show, Cancel }
    timer { Select (every_ms: 1000, when: false) }
    model {
        var key str = "first"
        var selected bool = false
        var opened str = ""
        var dialog_open bool = false
    }
    on {
        .Select -> { .selected = true }
        .Open(item str) -> { .opened = item }
        .Replace -> { .key = "replacement" }
        .Show -> { .dialog_open = true }
        .Cancel -> { .dialog_open = false }
    }
    view {
        col (key: .key) {
            mouse-area (ondblclick: .Open(.key)) {
                button (text: "File row", onclick: .Select) {}
            }
        }
        button (text: "Replace", onclick: .Replace) {}
        alert-dialog (open: .dialog_open) {
            alert-dialog-content {
                alert-dialog-title "New file" {}
                alert-dialog-cancel "Cancel file dialog" { onclick: .Cancel }
            }
        }
    }
}"#;
    let mut bridge = NodeDesktopBridge::load_package(vec![unit("app", "app", source)], "app::app", 1).unwrap();
    let session = bridge.open().unwrap();
    let mut window = AceNodeDesktopWindow::default();
    assert!(matches!(bridge.dispatch_typed(0, "Show", &[]), StepOutcome::Dispatched { .. }));
    PRESSES.with(|times| *times.borrow_mut() = VecDeque::from([100]));
    let mut modal = iced_test::simulator(iced::widget::mouse_area(window.view_iced()).with_left_press_time(next_press));
    modal.click("Cancel file dialog").unwrap();
    for event in modal.into_messages() {
        window.on(event);
    }
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Cancel"));
    assert!(PRESSES.with(|times| times.borrow().is_empty()), "the modal owns its native press");

    let input = || iced::widget::mouse_area(window.view_iced()).with_left_press_time(next_press);
    PRESSES.with(|times| *times.borrow_mut() = VecDeque::from([1_000, 1_600]));
    let mut ui = iced_test::simulator(input());
    ui.click("File row").unwrap();
    ui.click("File row").unwrap();
    assert_eq!(ui.into_messages().count(), 2, "slow native clicks must remain separate");

    PRESSES.with(|times| *times.borrow_mut() = VecDeque::from([2_000, 2_188]));
    let mut ui = iced_test::simulator(input());
    ui.click("File row").unwrap();
    std::thread::sleep(Duration::from_millis(350));
    ui.click("File row").unwrap();
    let events = ui.into_messages().collect::<Vec<_>>();
    assert_eq!(events.len(), 3);
    for event in events {
        window.on(event);
    }
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Select"));
    let outcome = bridge.step();
    assert!(matches!(&outcome, StepOutcome::Dispatched { msg, states, .. }
        if msg == "Open" && states.contains(&("opened".into(), "first".into()))), "{outcome:?}");
    assert!(matches!(bridge.step(), StepOutcome::Idle));

    let (replace, open, rejected) = current_session(|front| {
        let frame = front.package_frame(0).unwrap();
        let event = |name| {
            let target = frame.actions().iter().position(|action| action.source.message == name).unwrap();
            NodeDesktopMsg::PackageClick { session, instance: 0, revision: frame.revision, target }
        };
        (event("Replace"), event("Open"), front.stale_rejected)
    }).unwrap();
    window.on(replace);
    window.on(open.clone());
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { msg, .. } if msg == "Replace"));
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    assert_eq!(current_session(|front| front.stale_rejected), Some(rejected + 1));
    bridge.close_session().unwrap();
    bridge.reopen().unwrap();
    window.on(open);
    assert!(matches!(bridge.step(), StepOutcome::Idle));
    bridge.close_session().unwrap();
}

#[test]
fn checked_file_open_rejects_forged_replies_and_released_mounts() {
    use auto_ui_runtime::ui::{ace_node_desktop::StepOutcome, ace_state_bridge::AceStateValue};

    let source = r#"widget OpenDocument {
    msg { TryOpen(str), Opened(int) }
    bind { "file:open" -> .Opened }
    model {
        var file_open_pending bool = false
        var file_open_path str = ""
        var status int = -1
    }
    on {
        .TryOpen(path str) -> {
            .file_open_path = path
            .file_open_pending = true
        }
        .Opened(result int) -> {
            .file_open_pending = false
            .status = result
        }
    }
    view { text f"${.status}" {} }
}"#;
    let vue = generate_package_sfc(&[unit("app", "app", source)], "app::app").unwrap();
    assert!(vue.contains(r#"{ reply: "Opened", unavailable: 0 }"#));
    let wrong = source.replace("-> .Opened }", "-> .TryOpen }");
    let rejected = generate_package_sfc(&[unit("app", "app", &wrong)], "app::app").unwrap_err();
    assert!(rejected.contains("requires an int reply handler"), "{rejected}");

    ui980::require_prepared();
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let path = root.join("missing.txt");
    let args = [AceStateValue::Str(path.to_str().unwrap().into())];
    let mut desktop = NodeDesktopBridge::load(source, "app.at", 1).unwrap();
    desktop.open().unwrap();
    desktop.grant_file_read(0, root).unwrap();
    let forged = desktop.dispatch_typed(0, "Opened", &[AceStateValue::Int(1)]);
    assert!(matches!(&forged, StepOutcome::DispatchRefused(message)
        if message.contains("only the file binding")), "{forged:?}");
    let expected = if cfg!(windows) { "2" } else { "0" };
    let outcome = desktop.dispatch_typed(0, "TryOpen", &args);
    assert!(matches!(&outcome, StepOutcome::Dispatched { states, .. }
        if states.contains(&("status".into(), expected.into()))
            && states.contains(&("file_open_pending".into(), "false".into()))), "{outcome:?}");
    desktop.close_session().unwrap();
    assert!(!matches!(desktop.dispatch_typed(0, "TryOpen", &args), StepOutcome::Dispatched { .. }));
}

#[test]
fn checked_file_mutation_commits_only_a_live_authorized_event() {
    use auto_lang_core::file_session::FileWritePermissions;
    use auto_ui_runtime::ui::{ace_node_desktop::StepOutcome, ace_state_bridge::AceStateValue};

    let source = r#"widget FileAction {
    msg { Remove(str), MutationApplied(bool), Pulse }
    bind { "file:mutate" -> .MutationApplied }
    model {
        var file_mutation_pending bool = false
        var file_mutation_operation str = "delete-file"
        var file_mutation_path str = ""
        var file_mutation_target str = ""
        var deleted bool = false
        var count int = 0
    }
    on {
        .Remove(path str) -> {
            .file_mutation_path = path
            .file_mutation_pending = true
        }
        .MutationApplied(ok bool) -> {
            .file_mutation_pending = false
            .deleted = ok
        }
        .Pulse -> { .count += 1 }
    }
    view {
        if .deleted { text "Deleted" {} } else { text "Kept" {} }
        text f"${.count}" {}
    }
}"#;
    let vue = generate_package_sfc(&[unit("app", "app", source)], "app::app").unwrap();
    assert!(vue.contains("apply: applyFileMutation") && vue.contains("reply: \"MutationApplied\""));
    let wrong = source.replace("-> .MutationApplied }", "-> .Pulse }");
    let rejected = generate_package_sfc(&[unit("app", "app", &wrong)], "app::app").unwrap_err();
    assert!(rejected.contains("requires a bool reply handler"), "{rejected}");

    ui980::require_prepared();
    let fixture = tempfile::tempdir().unwrap();
    let protected = fixture.path().join("fixture.txt");
    std::fs::write(&protected, "original").unwrap();
    let args = [AceStateValue::Str(protected.to_str().unwrap().into())];
    let assert_deleted = |outcome, expected: &str| {
        assert!(matches!(&outcome, StepOutcome::Dispatched { states, .. }
            if states.contains(&("deleted".into(), expected.into()))
                && states.contains(&("file_mutation_pending".into(), "false".into()))), "{outcome:?}");
    };
    let mut desktop = NodeDesktopBridge::load(source, "app.at", 1).unwrap();
    desktop.open().unwrap();
    desktop.grant_file_read(0, fixture.path()).unwrap();
    let forged = desktop.dispatch_typed(0, "MutationApplied", &[AceStateValue::Bool(true)]);
    assert!(matches!(&forged, StepOutcome::DispatchRefused(message)
        if message.contains("only the file binding")), "{forged:?}");
    assert_deleted(desktop.dispatch_typed(0, "Remove", &args), "false");
    assert_eq!(std::fs::read_to_string(&protected).unwrap(), "original");
    let grant = FileWritePermissions { delete: true, ..Default::default() };
    desktop.grant_file_access(0, fixture.path(), grant).unwrap();
    assert_deleted(desktop.dispatch_typed(0, "Remove", &args), "true");
    assert!(!protected.exists());

    // An unrelated event preserves the reply and the replacement file.
    std::fs::write(&protected, "replacement").unwrap();
    assert_deleted(desktop.dispatch_typed(0, "Pulse", &[]), "true");
    assert_eq!(std::fs::read_to_string(&protected).unwrap(), "replacement");
    desktop.close_session().unwrap();
    assert!(desktop.grant_file_access(0, fixture.path(), grant).is_err());

    let missing_clear = source.replace(".file_mutation_pending = false\n", "");
    let mut desktop = NodeDesktopBridge::load(&missing_clear, "app.at", 1).unwrap();
    desktop.open().unwrap();
    desktop.grant_file_access(0, fixture.path(), grant).unwrap();
    let refused = desktop.dispatch_typed(0, "Remove", &args);
    assert!(matches!(&refused, StepOutcome::DispatchRefused(message)
        if message.contains("mutation reply must clear")), "{refused:?}");
    assert_eq!(std::fs::read_to_string(&protected).unwrap(), "replacement");
    desktop.close_session().unwrap();

    let failed_refresh = format!(
        "type DirEntry {{\n    name str\n    path str\n    is_dir bool\n    size u64\n    mtime i64\n}}\n\
         type DirListing {{\n    ok bool\n    path str\n    entries List<DirEntry>\n}}\n{}",
        source
            .replace("MutationApplied(bool), Pulse", "MutationApplied(bool), DirectoryListed(DirListing), Pulse")
            .replace("-> .MutationApplied }", "-> .MutationApplied\n        \"file:list\" -> .DirectoryListed }")
            .replace("var deleted bool", "var file_request_pending bool = false\n        var file_request_path str = \"\"\n        var deleted bool")
            .replace(".deleted = ok", ".deleted = ok\n            .file_request_pending = ok")
            .replace(".Pulse -> { .count += 1 }", ".DirectoryListed(listing DirListing) -> { .count = listing.entries.len() }\n        .Pulse -> { .file_request_pending = false }"),
    );
    let mut desktop = NodeDesktopBridge::load(&failed_refresh, "app.at", 1).unwrap();
    desktop.open().unwrap();
    desktop.grant_file_access(0, fixture.path(), grant).unwrap();
    assert_deleted(desktop.dispatch_typed(0, "Remove", &args), "true");
    assert!(!protected.exists());
    std::fs::write(&protected, "after committed write").unwrap();
    assert_deleted(desktop.dispatch_typed(0, "Pulse", &[]), "true");
    assert_eq!(std::fs::read_to_string(&protected).unwrap(), "after committed write");
    desktop.close_session().unwrap();
}
