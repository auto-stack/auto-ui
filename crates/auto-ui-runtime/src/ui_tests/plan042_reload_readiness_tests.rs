//! PLAN-042 有界研究探针：多文件 Reload 与状态 Schema 及 Effect 生命周期。
//!
//! 研究合同：docs/plans/042-reload-schema-and-effect-readiness.md（T-02/T-03）。
//! 探针走真实生产入口形态（Parser+UiDialect → extract_widget_from_decl →
//! DynamicComponent::reload / check_file_changed），不手造字段 Map 充数。
//! 结论折 Plan 与 docs/specs/runtime/reload-readiness-and-lifecycle.md（SD-01）。

#[cfg(test)]
mod plan042_reload_readiness_tests {
    use crate::ui::component::Component;
    use crate::ui::dynamic::DynamicComponent;
    use auto_lang_core::parser::Parser;
    use auto_lang_core::session::CompilerSession;
    use auto_val::Value;

    // ========================================================================
    // Helpers（镜像 iced/renderer/host.rs:2822-2873 泵内 reload 链）
    // ========================================================================

    /// 真实解析：source → (decls, root AuraWidget, registry)。
    fn parse_widgets(
        src: &str,
    ) -> (
        Vec<auto_ui_dialect::WidgetDecl>,
        auto_ui_compiler::aura::AuraWidget,
        crate::ui::widget_registry::WidgetRegistry,
    ) {
        let session = CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = auto_ui_dialect::UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("plan042 corpus parses"),
        );
        let mut decls: Vec<auto_ui_dialect::WidgetDecl> = vec![];
        for s in &ast.0.items {
            if let auto_parser_api::DomainStmt::Domain(auto_ui_dialect::UiStmt::WidgetDecl(d)) = s
            {
                decls.push(d.clone());
            }
        }
        assert!(!decls.is_empty(), "corpus needs a widget decl");
        let root =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(&decls[0]).expect("root");
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        for d in &decls[1..] {
            let w = auto_ui_compiler::aura::extract::extract_widget_from_decl(d).expect("child");
            registry.register(w);
        }
        (decls, root, registry)
    }

    /// 真实装配（生产 from_decl 路径；兄弟 decl 全量传入）。
    fn build_comp(src: &str) -> DynamicComponent {
        let (decls, root, registry) = parse_widgets(src);
        DynamicComponent::with_registry_and_imports_from_decls(
            &decls[0],
            &decls[1..],
            &root,
            registry,
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component builds")
    }

    /// 泵内 reload 链（host.rs 形态）：重解析源码，取第一个 WidgetDecl，
    /// 调 `reload`。返回 None = 解析/提取失败（泵静默跳过）。
    fn pump_try_reload(
        comp: &mut DynamicComponent,
        code: &str,
    ) -> Result<Option<crate::ui::state_migration::MigrationReport>, String> {
        let session = CompilerSession::ui();
        let mut parser = Parser::from(code).with_session(session);
        let ast = parser
            .parse_domain(&auto_ui_dialect::UiDialect)
            .map(auto_ui_dialect::UiCode)
            .map_err(|e| format!("parse: {:?}", e))?;
        for stmt in ast.items() {
            if let auto_parser_api::DomainStmt::Domain(auto_ui_dialect::UiStmt::WidgetDecl(decl)) =
                stmt
            {
                let widget = auto_ui_compiler::aura::extract_widget_from_decl(decl)
                    .map_err(|e| format!("extract: {:?}", e))?;
                return comp.reload(&widget).map(Some);
            }
        }
        Ok(None)
    }

    const COUNTER_V1: &str = concat!(
        "widget Counter {\n",
        "    model {\n",
        "        var count int = 0\n",
        "        var label str = \"n\"\n",
        "    }\n",
        "    view { col { text .label button \"+\" { onclick: .Bump } } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
    );

    /// V1 → 增字段/改视图（合法迁移候选）。
    const COUNTER_V2: &str = concat!(
        "widget Counter {\n",
        "    model {\n",
        "        var count int = 0\n",
        "        var label str = \"n\"\n",
        "        var step int = 5\n",
        "    }\n",
        "    view { col { text .label text \"step\" button \"+\" { onclick: .Bump } } }\n",
        "    on { .Bump -> { .count = .count + .step } }\n",
        "}\n",
    );

    /// V1 → 删字段。
    const COUNTER_V3_DROP: &str = concat!(
        "widget Counter {\n",
        "    model { var count int = 0 }\n",
        "    view { col { text \"n\" button \"+\" { onclick: .Bump } } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
    );

    /// V1 → 同名 field 类型变化（int → str）。
    const COUNTER_V4_TYPECHANGE: &str = concat!(
        "widget Counter {\n",
        "    model {\n",
        "        var count str = \"0\"\n",
        "        var label str = \"n\"\n",
        "    }\n",
        "    view { col { text .label button \"+\" { onclick: .Bump } } }\n",
        "    on { .Bump -> { } }\n",
        "}\n",
    );

    // ========================================================================
    // T-02 单文件：schema/迁移/失败恢复（V-01/V-02/V-04）
    // ========================================================================

    /// 正控：真实解析→装配→泵链 reload，兼容字段保值、新字段取默认。
    #[test]
    fn t02_real_entry_reload_preserves_and_defaults() {
        let mut comp = build_comp(COUNTER_V1);
        comp.write_state("count", Value::Int(42)).unwrap();

        let report = pump_try_reload(&mut comp, COUNTER_V2)
            .expect("pump reload")
            .expect("widget extracted");

        assert_eq!(comp.read_state("count").unwrap(), Value::Int(42), "保值");
        assert_eq!(comp.read_state("step").unwrap(), Value::Int(5), "新字段默认");
        assert_eq!(report.preserved, 2, "count+label preserved");
        assert_eq!(report.added, 1, "step added");
        assert!(
            !report.dropped_names.iter().any(|n| n == "count" || n == "label"),
            "用户字段不得出现在 dropped：{:?}",
            report.dropped_names
        );
    }

    /// 删除字段：旧值丢弃、报告点名。
    #[test]
    fn t02_removed_field_dropped() {
        let mut comp = build_comp(COUNTER_V1);
        comp.write_state("label", Value::str("hello")).unwrap();

        let report = pump_try_reload(&mut comp, COUNTER_V3_DROP)
            .expect("pump reload")
            .expect("widget extracted");

        assert!(comp.read_state("label").is_err(), "label dropped");
        assert!(
            report.dropped_names.contains(&"label".to_string()),
            "dropped 点名 label：{:?}",
            report.dropped_names
        );
        assert!(
            !report.dropped_names.contains(&"count".to_string()),
            "count 不得 dropped：{:?}",
            report.dropped_names
        );
    }

    /// 负控（具名缺口）：同名 field 类型 int→str。reload 传空 old_field_defs，
    /// migrate_state 按名乐观保值——类型变化不被检测。本探针实测 write_state
    /// 对新桥的真实落点（写失败静默 → 可能回退默认），把"同名≠同型"缺口钉实。
    #[test]
    fn t02_same_name_type_change_optimistically_preserved() {
        let mut comp = build_comp(COUNTER_V1);
        comp.write_state("count", Value::Int(42)).unwrap();

        let report = pump_try_reload(&mut comp, COUNTER_V4_TYPECHANGE)
            .expect("pump reload")
            .expect("widget extracted");

        // 关键观察：report 把 count 记为 preserved（无旧型可判）。
        assert_eq!(
            report.preserved, 2,
            "同名保值：migrate 无旧 schema 可判型，Int(42) 被当作兼容"
        );
        // 新桥 write_state 对 Int 写入 str 字段的实测（2026-11-04 探针）：
        // 不判型、原样落盘——新桥持有违反自身 schema 的值。
        let observed = comp.read_state("count").unwrap();
        assert_eq!(
            observed,
            Value::Int(42),
            "同名类型变化：旧 Int 值原样写入 str 字段（具名缺口钉死）"
        );
    }

    /// 负控：检测=mtime 非内容。同 mtime 改内容 → 漏检（Ok(None)）。
    #[test]
    fn t02_same_mtime_content_change_missed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("counter.at");
        std::fs::write(&path, COUNTER_V1).unwrap();
        let baseline = std::fs::metadata(&path).unwrap().modified().unwrap();

        let mut comp = build_comp(COUNTER_V1);
        comp.set_source_path(&path);
        assert!(comp.check_file_changed().unwrap().is_none(), "基线无变化");

        // 内容修改但 mtime 拨回基线。
        std::fs::write(&path, COUNTER_V2).unwrap();
        let f = std::fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        f.set_modified(baseline).unwrap();
        drop(f);

        assert!(
            comp.check_file_changed().unwrap().is_none(),
            "同 mtime 内容变化被漏检（检测仅比较 mtime）"
        );
        assert_eq!(
            comp.read_state("count").unwrap_or(Value::Int(-1)),
            Value::Int(0),
            "旧 bridge 仍在服役"
        );

        // mtime 前进 → 检出。
        f_set_forward(&path);
        assert!(
            comp.check_file_changed().unwrap().is_some(),
            "mtime 前进后检出"
        );
    }

    fn f_set_forward(path: &std::path::Path) {
        use std::time::{Duration, SystemTime};
        let t = SystemTime::now() + Duration::from_secs(2);
        let f = std::fs::File::options()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        f.set_modified(t).unwrap();
    }

    /// 缺源：Err；旧组件完整继续（失败保旧）。
    #[test]
    fn t02_missing_source_errors_but_old_component_survives() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("counter.at");
        std::fs::write(&path, COUNTER_V1).unwrap();

        let mut comp = build_comp(COUNTER_V1);
        comp.set_source_path(&path);
        comp.write_state("count", Value::Int(7)).unwrap();
        assert!(comp.check_file_changed().unwrap().is_none());

        std::fs::remove_file(&path).unwrap();
        assert!(
            comp.check_file_changed().is_err(),
            "缺源必须报错（泵内被 if let Ok(Some(_)) 吞掉）"
        );
        assert_eq!(
            comp.read_state("count").unwrap(),
            Value::Int(7),
            "缺源后旧业务继续"
        );

        // 恢复：文件重建、mtime 前进 → 下一次合法编辑成功。
        std::fs::write(&path, COUNTER_V2).unwrap();
        f_set_forward(&path);
        assert!(comp.check_file_changed().unwrap().is_some());
        pump_try_reload(&mut comp, COUNTER_V2)
            .expect("reload ok")
            .expect("extracted");
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(7), "恢复后保值");
    }

    /// 泵链失败保旧 + 合法恢复：非法编辑该拍被跳过（且 last_modified 已推进），
    /// 下一次合法编辑成功。
    #[test]
    fn t02_parse_failure_keeps_old_and_next_legal_edit_wins() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("counter.at");
        std::fs::write(&path, COUNTER_V1).unwrap();

        let mut comp = build_comp(COUNTER_V1);
        comp.set_source_path(&path);
        comp.write_state("count", Value::Int(11)).unwrap();

        // 非法编辑：可解析但无 widget 声明（泵找不到候选，跳过该拍）。
        // 注：截断的 widget 残句曾触发 parser 51GB 病理分配（进程 OOM 中止），
        // 作为具名缺口记入 §11，探针不采用该类输入。
        std::fs::write(&path, "fn forty_two() -> int { 42 }\n").unwrap();
        f_set_forward(&path);
        assert!(
            comp.check_file_changed().unwrap().is_some(),
            "泵第一拍：检出变化（last_modified 在此推进）"
        );
        let code = std::fs::read_to_string(&path).unwrap();
        assert!(
            pump_try_reload(&mut comp, &code)
                .map(|r| r.is_none())
                .unwrap_or(true),
            "无 widget 候选不得产出 reload"
        );
        assert_eq!(
            comp.read_state("count").unwrap(),
            Value::Int(11),
            "失败拍旧组件完整继续"
        );

        // 修复（mtime 再次前进）→ 下一次合法编辑成功。
        std::fs::write(&path, COUNTER_V2).unwrap();
        f_set_forward(&path);
        assert!(comp.check_file_changed().unwrap().is_some());
        let code = std::fs::read_to_string(&path).unwrap();
        pump_try_reload(&mut comp, &code)
            .expect("reload ok")
            .expect("extracted");
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(11), "恢复后保值");
        assert_eq!(comp.read_state("step").unwrap(), Value::Int(5));
    }

    /// reload 内部失败（新桥 VmBridge 构造 Err）：旧 bridge 不得被替换。
    /// 失败注入：handler 体内调用未声明函数（AuraWidget 路径无
    /// strip_callback_calls → 链接期 Undefined symbol）。
    #[test]
    fn t02_reload_bridge_failure_keeps_old_bridge() {
        let mut comp = build_comp(COUNTER_V1);
        comp.write_state("count", Value::Int(9)).unwrap();

        const BAD: &str = concat!(
            "widget Counter {\n",
            "    model { var count int = 0 }\n",
            "    view { col { text \"n\" } }\n",
            "    on { .Bump -> { nope_undefined_fn() } }\n",
            "}\n",
        );

        let parse_first = {
            let session = CompilerSession::ui();
            let mut parser = Parser::from(BAD).with_session(session);
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .map(auto_ui_dialect::UiCode)
        };

        match parse_first {
            Ok(ast) => {
                let mut extracted = None;
                for stmt in ast.items() {
                    if let auto_parser_api::DomainStmt::Domain(
                        auto_ui_dialect::UiStmt::WidgetDecl(decl),
                    ) = stmt
                    {
                        extracted =
                            Some(auto_ui_compiler::aura::extract_widget_from_decl(decl));
                        break;
                    }
                }
                match extracted {
                    Some(Ok(bad_widget)) => {
                        let r = comp.reload(&bad_widget);
                        assert!(
                            r.is_err(),
                            "链接失败的 widget 必须让 reload 返回 Err（实际 {:?}）",
                            r.map(|rep| (rep.preserved, rep.added, rep.dropped))
                        );
                        assert_eq!(
                            comp.read_state("count").unwrap(),
                            Value::Int(9),
                            "reload Err 后旧 bridge 完整继续"
                        );
                    }
                    _ => {
                        // 提取期即失败：等价于泵链解析失败腿（t02_parse_failure_*）。
                        assert_eq!(
                            comp.read_state("count").unwrap(),
                            Value::Int(9),
                            "候选未产出，旧 bridge 完整继续"
                        );
                    }
                }
            }
            Err(_) => {
                assert_eq!(
                    comp.read_state("count").unwrap(),
                    Value::Int(9),
                    "候选解析失败，旧 bridge 完整继续"
                );
            }
        }

        // 下一次合法 reload 成功。
        pump_try_reload(&mut comp, COUNTER_V2)
            .expect("reload ok")
            .expect("extracted");
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(9));
    }

    // ========================================================================
    // T-03 多文件：root/import/store/child 整链（V-03）
    // ========================================================================

    /// 多文件 fixture（真实磁盘模块装载，build_dynamic_component 同管线）。
    struct MultiFileFixture {
        _dir: tempfile::TempDir,
        root: std::path::PathBuf,
        store: std::path::PathBuf,
        child: std::path::PathBuf,
    }

    fn write_multi_file_fixture() -> MultiFileFixture {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("app.at");
        let store = dir.path().join("count_store.at");
        let child = dir.path().join("widgets.at");
        std::fs::write(
            &root,
            concat!(
                "use store: CountStore\n",
                "use widgets: Badge\n",
                "widget App {\n",
                "    model { var count int = 0 }\n",
                "    view { col { text \"app\" button \"+\" { onclick: .Bump } Badge() } }\n",
                "    on {\n",
                "        .Bump -> {\n",
                "            .count = .count + 1\n",
                "            store.Add()\n",
                "        }\n",
                "    }\n",
                "}\n",
            ),
        )
        .unwrap();
        std::fs::write(
            &store,
            concat!(
                "store CountStore {\n",
                "    model { var total int = 0 }\n",
                "    on { .Add -> { .total = .total + 1 } }\n",
                "}\n",
            ),
        )
        .unwrap();
        std::fs::write(
            &child,
            concat!(
                "widget Badge {\n",
                "    model { var label str = \"b\" }\n",
                "    view { text .label }\n",
                "}\n",
            ),
        )
        .unwrap();
        MultiFileFixture {
            _dir: dir,
            root,
            store,
            child,
        }
    }

    /// 整源热重载路径对多文件工程（use imports / child registry 非空）
    /// 恒报"无变化"——根与依赖都不 reload（Plan 451 guard 实测）。
    #[test]
    fn t03_multi_file_guard_skips_root_and_deps() {
        let fx = write_multi_file_fixture();
        let code = std::fs::read_to_string(&fx.root).unwrap();
        let mut comp = crate::dynamic_build::build_dynamic_component(&code, Some(
            fx.root.to_str().unwrap(),
        ))
        .expect("multi-file build");

        // 业务可运行： Bump + store.Add 派发成功。
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(1));

        // 依赖文件改内容 + mtime 前进 → guard 仍报无变化。
        std::fs::write(
            &fx.store,
            concat!(
                "store CountStore {\n",
                "    model { var total int = 0 var extra int = 1 }\n",
                "    on { .Add -> { .total = .total + 1 } }\n",
                "}\n",
            ),
        )
        .unwrap();
        f_set_forward(&fx.store);
        assert!(
            comp.check_file_changed().unwrap().is_none(),
            "store 改动不得触发整源 reload（状态/handler 会丢）"
        );

        std::fs::write(
            &fx.child,
            concat!(
                "widget Badge {\n",
                "    model { var label str = \"b2\" }\n",
                "    view { col { text .label text \"!\" } }\n",
                "}\n",
            ),
        )
        .unwrap();
        f_set_forward(&fx.child);
        assert!(
            comp.check_file_changed().unwrap().is_none(),
            "child 改动不得触发整源 reload"
        );

        // 根文件改动同样被 guard 跳过（import_stmts/registry 非空）。
        std::fs::write(
            &fx.root,
            concat!(
                "use store: CountStore\n",
                "use widgets: Badge\n",
                "widget App {\n",
                "    model { var count int = 0 var hint str = \"h\" }\n",
                "    view { col { text \"app2\" button \"+\" { onclick: .Bump } Badge() } }\n",
                "    on { .Bump -> { .count = .count + 1 CountStore.Add() } }\n",
                "}\n",
            ),
        )
        .unwrap();
        f_set_forward(&fx.root);
        assert!(
            comp.check_file_changed().unwrap().is_none(),
            "多文件形态下根改动也跳过（不是只 reload 根）"
        );
        assert_eq!(
            comp.read_state("count").unwrap(),
            Value::Int(1),
            "旧根状态原样保留"
        );
    }

    /// 正控：同形态单文件工程（无 use/store/child）检测可用，
    /// 与 t03 guard 形成正负对照。
    #[test]
    fn t03_single_file_positive_control_detected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("solo.at");
        std::fs::write(&path, COUNTER_V1).unwrap();

        let mut comp = build_comp(COUNTER_V1);
        comp.set_source_path(&path);
        std::fs::write(&path, COUNTER_V2).unwrap();
        f_set_forward(&path);
        assert!(
            comp.check_file_changed().unwrap().is_some(),
            "单文件形态检出"
        );
    }

    // ========================================================================
    // T-04 owner/effect：实例、store、Tick、迟到回调（V-05）
    // 生产 guard 保证这些形态不会进入 reload；以下均为**强制 reload**（绕过
    // guard）的灾害边界实验，为后继候选 bundle 原子发布合同提供反例。
    // ========================================================================

    /// App + keyed child（同文件兄弟 → registry 非空，即多文件 guard 形态）。
    const APP_CHILD_V1: &str = concat!(
        "widget App {\n",
        "    model { var count int = 0 }\n",
        "    view { col { text \"app\" Badge(key: \"b1\") } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
        "widget Badge(key: str) {\n",
        "    model { var label str = \"b\" }\n",
        "    view { text .label }\n",
        "    on { .Flip -> { .label = \"flipped\" } }\n",
        "}\n",
    );

    /// V2 只改根视图（生产泵只提取第一个 WidgetDecl——兄弟 decl 不进候选）。
    const APP_CHILD_V2: &str = concat!(
        "widget App {\n",
        "    model { var count int = 0 }\n",
        "    view { col { text \"app v2\" Badge(key: \"b1\") } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
        "widget Badge(key: str) {\n",
        "    model { var label str = \"b\" }\n",
        "    view { text .label }\n",
        "    on { .Flip -> { .label = \"flipped\" } }\n",
        "}\n",
    );

    /// 强制 reload 后：根状态迁移保值，但子件实例数据随旧 bridge 消亡、
    /// 子件 handler 在新桥缺席、迟到实例事件不得重定向。
    #[test]
    fn t04_forced_reload_drops_child_instances_and_handlers() {
        let mut comp = build_comp(APP_CHILD_V1);

        // 帧构建创建 key 实例；写入实例态。
        let _ = comp.view();
        let ids = comp.bridge().child_instance_identities();
        assert_eq!(ids.len(), 1, "一次 view 建立一个 Badge 实例");
        let id = ids[0].clone();
        assert!(id.contains("Badge"), "identity 点名 widget：{}", id);
        comp.bridge_mut()
            .write_instance_state(&id, "label", Value::str("custom"))
            .expect("instance write");
        assert_eq!(
            comp.bridge().read_instance_state(&id, "label").unwrap(),
            Value::str("custom")
        );

        // 子件 handler 派发可用（旧桥）。
        comp.on_with_input_for("Badge", "Flip", None);
        assert_eq!(
            comp.bridge().read_instance_state(&id, "label").unwrap(),
            Value::str("flipped"),
            "旧桥实例 handler 按 receiver 写入实例态"
        );

        comp.write_state("count", Value::Int(3)).unwrap();

        // 强制 reload（生产 guard 本不允许到达这里）。
        pump_try_reload(&mut comp, APP_CHILD_V2)
            .expect("reload ok")
            .expect("extracted");

        // 根态保值（migrate 腿正常）。
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(3));

        // 灾害边界 1：新 bridge 实例表为空——实例数据（含用户输入）全灭。
        assert_eq!(
            comp.bridge().child_instance_identities().len(),
            0,
            "新桥 child_instances 全新，旧实例态不随 migrate 迁移"
        );

        // 灾害边界 2：子件 handler 在新桥缺席（reload 只编译根 widget）。
        comp.on_with_input_for("Badge", "Flip", None);
        assert!(
            comp.bridge().child_instance_identities().is_empty(),
            "Flip 派发不得在新桥凭空建立实例"
        );

        // 灾害边界 3：迟到实例事件（旧 identity）按 041 合同丢弃、不重定向。
        comp.on_with_input_for_instance("Badge", "Flip", None, Some(&id));
        assert!(
            comp.bridge().child_instance_identities().is_empty(),
            "迟到事件不得重定向/复活实例"
        );

        // 下一次帧重建实例：从 model 默认重新开始（数据丢失可观察）。
        let _ = comp.view();
        let ids2 = comp.bridge().child_instance_identities();
        assert_eq!(ids2.len(), 1, "下一帧重建同身份实例");
        assert_eq!(
            comp.bridge().read_instance_state(&ids2[0], "label").unwrap(),
            Value::str("b"),
            "重建实例回 model 默认，custom/flipped 均丢失"
        );
    }

    /// 多文件 store（store-as-child 与根同桥）：强制 reload 只带根 widget，
    /// store handler 缺席、store 数据虽随根态快照迁入但不可再变更——
    /// Plan 449/451 实机 failure mode 的单测级永久对照。
    #[test]
    fn t04_forced_reload_on_store_app_loses_store_handlers() {
        let fx = write_multi_file_fixture();
        let code = std::fs::read_to_string(&fx.root).unwrap();
        let mut comp = crate::dynamic_build::build_dynamic_component(&code, Some(
            fx.root.to_str().unwrap(),
        ))
        .expect("multi-file build");

        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(1));
        assert_eq!(
            comp.read_state("total").unwrap(),
            Value::Int(1),
            "store 字段平铺根态（Bump 体内 CountStore.Add()）"
        );

        // 强制 reload：V2 根（同 Bump 语义）。
        std::fs::write(
            &fx.root,
            concat!(
                "use store: CountStore\n",
                "use widgets: Badge\n",
                "widget App {\n",
                "    model { var count int = 0 }\n",
                "    view { col { text \"app v2\" button \"+\" { onclick: .Bump } Badge() } }\n",
                "    on {\n",
                "        .Bump -> {\n",
                "            .count = .count + 1\n",
                "            store.Add()\n",
                "        }\n",
                "    }\n",
                "}\n",
            ),
        )
        .unwrap();
        let code = std::fs::read_to_string(&fx.root).unwrap();
        pump_try_reload(&mut comp, &code)
            .expect("reload ok")
            .expect("extracted");

        // 根态快照迁入（count 在根 model，保值）。
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(1));
        // 具名发现：store 数据面**整体丢失**——新桥只有根 widget 的字段，
        // migrate 无 `total` 可写；这正是 Plan 449/451 实机 "field not found"
        // 的单测级复现（比 handler 缺席更基础的失效层）。
        assert!(
            comp.read_state("total").is_err(),
            "store 字段不进候选 → 数据面随强制 reload 丢失"
        );

        // store handler 不在新桥：Add 派发空转（total 保持缺席）。
        comp.on_with_input_for("CountStore", "Add", None);
        assert!(
            comp.read_state("total").is_err(),
            "store handler 缺席：派发不改态"
        );
        // 更深一层具名发现：根 handler Bump 的**体内引用了 store.Add()**，
        // 候选桥（无 store）编译该 handler 失败 → Bump 本身也不在新桥。
        // 这正是 449/451 实机 'field not found + 派发失效' 的完整单测级复现。
        comp.on_with_input("Bump", None);
        assert_eq!(
            comp.read_state("count").unwrap(),
            Value::Int(1),
            "根 Bump handler 随候选编译失败灭失：count 不再增长"
        );
        assert!(
            comp.read_state("total").is_err(),
            "体内 store 调用随 handler 一并失效"
        );
    }

    /// Tick（effect 资源腿）：reload 用新 widget 的 tick_interval 替换旧值。
    #[test]
    fn t04_tick_interval_replaced_by_reload() {
        const TICK_V1: &str = concat!(
            "widget Ticker {\n",
            "    model {\n",
            "        var interval int = 500\n",
            "        var n int = 0\n",
            "    }\n",
            "    view { text \"t\" }\n",
            "    on { .Tick -> { .n = .n + 1 } }\n",
            "}\n",
        );
        const TICK_V2: &str = concat!(
            "widget Ticker {\n",
            "    model { var n int = 0 }\n",
            "    view { col { text \"t\" text \"v2\" } }\n",
            "    on { .Tick -> { .n = .n + 10 } }\n",
            "}\n",
        );

        let mut comp = build_comp(TICK_V1);
        assert_eq!(comp.tick_interval(), Some(500), "interval model 变量驱动");

        pump_try_reload(&mut comp, TICK_V2)
            .expect("reload ok")
            .expect("extracted");
        assert_eq!(
            comp.tick_interval(),
            Some(1000),
            "reload 用新定义 tick_interval（无 interval 变量 → 默认 1000）"
        );
        // 旧 Tick 语义已被替换：新 handler 体生效。
        comp.on_with_input("Tick", None);
        assert_eq!(
            comp.read_state("n").unwrap(),
            Value::Int(10),
            "新 Tick handler 生效"
        );
    }
}
