//! PLAN-056 永久回归：多文件候选 Bundle 原子 Reload 与 Schema/生命周期收口
//! （V-01..V-05，AC-01..AC-05）。
//!
//! 合同：docs/plans/056-atomic-multifile-reload-and-schema-lifecycle.md §6/§7。
//! 真实入口：`build_dynamic_component`（真实装配）→ `set_source_path` →
//! `check_reload` / `try_reload`（真实 host 泵同款调用，host.rs __hot_reload
//! 泵臂现直调 `component.try_reload()`）。旧单文件 legacy `reload()` /
//! `check_file_changed` 由 042 探针维持。

#[cfg(test)]
mod plan056_atomic_reload_tests {
    use crate::ui::component::Component;
    use crate::ui::dynamic::DynamicComponent;
    use auto_val::Value;

    // ========================================================================
    // Fixtures（临时目录真实文件；所有语句独立成行，dialect 要求换行分隔）
    // ========================================================================

    /// 单文件 Counter V1：count/label，Bump → count+1。
    const SOLO_V1: &str = concat!(
        "widget Counter {\n",
        "    model {\n",
        "        var count int = 0\n",
        "        var label str = \"n\"\n",
        "    }\n",
        "    view { col { text .label button \"+\" { onclick: .Bump } } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
    );

    /// V2：加 step 字段 + 改视图 + Bump 用 step（新 view 新 handler 同现）。
    const SOLO_V2: &str = concat!(
        "widget Counter {\n",
        "    model {\n",
        "        var count int = 0\n",
        "        var label str = \"n\"\n",
        "        var step int = 5\n",
        "    }\n",
        "    view { col { text .label text \"step-v2\" button \"+\" { onclick: .Bump } } }\n",
        "    on { .Bump -> { .count = .count + .step } }\n",
        "}\n",
    );

    /// 同名类型冲突（int → str）。
    const SOLO_TYPECHANGE: &str = concat!(
        "widget Counter {\n",
        "    model {\n",
        "        var count str = \"0\"\n",
        "        var label str = \"n\"\n",
        "    }\n",
        "    view { col { text .label button \"+\" { onclick: .Bump } } }\n",
        "    on { .Bump -> { } }\n",
        "}\n",
    );

    /// V3：去掉 step 的独立合法变体（V-05 新路径收尾腿）。
    const SOLO_V3: &str = concat!(
        "widget Counter {\n",
        "    model {\n",
        "        var count int = 0\n",
        "        var label str = \"n\"\n",
        "    }\n",
        "    view { col { text .label text \"v3\" button \"+\" { onclick: .Bump } } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
    );

    /// 坏源（截断残句；940 EOF 推进守卫保证受控拒绝不 OOM）。
    const SOLO_BROKEN: &str = "widget Counter {\n    model { var count int = 0\n    view {\n";

    /// Init 计数 fixture（AC-04：准备期零 effect / Init 不重放）。
    const INIT_V1: &str = concat!(
        "widget InitCounter {\n",
        "    model {\n",
        "        var init_count int = 0\n",
        "        var n int = 0\n",
        "    }\n",
        "    view { text .init_count }\n",
        "    on { Init -> { .init_count = .init_count + 1 } }\n",
        "}\n",
    );
    const INIT_V2: &str = concat!(
        "widget InitCounter {\n",
        "    model {\n",
        "        var init_count int = 0\n",
        "        var n int = 0\n",
        "        var extra int = 9\n",
        "    }\n",
        "    view { col { text .init_count text \"v2\" } }\n",
        "    on { Init -> { .init_count = .init_count + 1 } }\n",
        "}\n",
    );

    /// Tick fixture（AC-04：Tick 订阅表整体替换）。
    const TICK_V1: &str = concat!(
        "widget Ticker {\n",
        "    model {\n",
        "        var interval int = 500\n",
        "        var n int = 0\n",
        "    }\n",
        "    view { text \"t1\" }\n",
        "    on { .Tick -> { .n = .n + 1 } }\n",
        "}\n",
    );
    const TICK_V2: &str = concat!(
        "widget Ticker {\n",
        "    model {\n",
        "        var n int = 0\n",
        "    }\n",
        "    view { text \"t2\" }\n",
        "    on { .Tick -> { .n = .n + 10 } }\n",
        "}\n",
    );

    /// 主题 scope fixture（AC-05：design 绑定后 scope 随提交保留）。
    const THEME_V1: &str = concat!(
        "widget Themed {\n",
        "    model {\n",
        "        var theme_name str = \"Auto\"\n",
        "        var dark_mode bool = true\n",
        "    }\n",
        "    design {\n",
        "        theme: .theme_name\n",
        "        mode: if .dark_mode { \"dark\" } else { \"light\" }\n",
        "    }\n",
        "    view { col { text \"t1\" } }\n",
        "}\n",
    );
    const THEME_V2: &str = concat!(
        "widget Themed {\n",
        "    model {\n",
        "        var theme_name str = \"Auto\"\n",
        "        var dark_mode bool = true\n",
        "    }\n",
        "    design {\n",
        "        theme: .theme_name\n",
        "        mode: if .dark_mode { \"dark\" } else { \"light\" }\n",
        "    }\n",
        "    view { col { text \"t1\" text \"t2\" } }\n",
        "}\n",
    );

    /// action_config fixture（AC-05：候选构建不得安装/劫持 action DSL）。
    const ACTIONS_V1: &str = concat!(
        "widget ActionApp {\n",
        "    model {\n",
        "        var count int = 0\n",
        "    }\n",
        "    actions {\n",
        "        action (id: \"test.bump\", handler: .Bump, title: \"Bump\")\n",
        "    }\n",
        "    view { col { text \"a1\" } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
    );
    const ACTIONS_V2: &str = concat!(
        "widget ActionApp {\n",
        "    model {\n",
        "        var count int = 0\n",
        "    }\n",
        "    actions {\n",
        "        action (id: \"test.bump\", handler: .Bump, title: \"Bump\")\n",
        "    }\n",
        "    view { col { text \"a2\" } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
    );

    /// 041 twins 多文件 fixture（app.at + p041_store.at 同构；根持 count 与
    /// .Bump，Inc 经实例落实例态 + 共享 store）。
    struct Twins {
        _dir: tempfile::TempDir,
        root: std::path::PathBuf,
        store: std::path::PathBuf,
    }

    const TWINS_ROOT_V1: &str = concat!(
        "use p041_store: P041Store\n",
        "widget P041Host {\n",
        "    model {\n",
        "        var host_marker str = \"host-shell\"\n",
        "        var show_b bool = true\n",
        "        var key_a str = \"a\"\n",
        "        var key_b str = \"b\"\n",
        "        var count int = 0\n",
        "    }\n",
        "    view {\n",
        "        col {\n",
        "            text \"host above counters\"\n",
        "            Counter (key: .key_a, start: 10) { }\n",
        "            if .show_b { Counter (key: .key_b, start: 20) { } }\n",
        "            text \"host below counters\"\n",
        "        }\n",
        "    }\n",
        "    on {\n",
        "        Init -> { }\n",
        "        .ToggleB -> { .show_b = !.show_b }\n",
        "        .SwapKeys -> {\n",
        "            let tmp = .key_a\n",
        "            .key_a = .key_b\n",
        "            .key_b = tmp\n",
        "        }\n",
        "        .Bump -> {\n",
        "            .count = .count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
        "widget Counter (start: int = 0) {\n",
        "    msg { Inc }\n",
        "    model { var count int = 0 }\n",
        "    view { col { text .start text .count text .total } }\n",
        "    on {\n",
        "        Inc -> {\n",
        "            count = count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
    );

    /// 根 V2：新增 host_note 字段 + 视图标记（合法迁移候选）。
    const TWINS_ROOT_V2: &str = concat!(
        "use p041_store: P041Store\n",
        "widget P041Host {\n",
        "    model {\n",
        "        var host_marker str = \"host-shell\"\n",
        "        var show_b bool = true\n",
        "        var key_a str = \"a\"\n",
        "        var key_b str = \"b\"\n",
        "        var count int = 0\n",
        "        var host_note str = \"n\"\n",
        "    }\n",
        "    view {\n",
        "        col {\n",
        "            text \"host above counters\"\n",
        "            text \"host-v2\"\n",
        "            Counter (key: .key_a, start: 10) { }\n",
        "            if .show_b { Counter (key: .key_b, start: 20) { } }\n",
        "            text \"host below counters\"\n",
        "        }\n",
        "    }\n",
        "    on {\n",
        "        Init -> { }\n",
        "        .ToggleB -> { .show_b = !.show_b }\n",
        "        .SwapKeys -> {\n",
        "            let tmp = .key_a\n",
        "            .key_a = .key_b\n",
        "            .key_b = tmp\n",
        "        }\n",
        "        .Bump -> {\n",
        "            .count = .count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
        "widget Counter (start: int = 0) {\n",
        "    msg { Inc }\n",
        "    model { var count int = 0 }\n",
        "    view { col { text .start text .count text .total } }\n",
        "    on {\n",
        "        Inc -> {\n",
        "            count = count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
    );

    /// 子件类型冲突：Counter.count int → str（整候选拒绝）。
    const TWINS_ROOT_TYPECHANGE: &str = concat!(
        "use p041_store: P041Store\n",
        "widget P041Host {\n",
        "    model {\n",
        "        var host_marker str = \"host-shell\"\n",
        "        var show_b bool = true\n",
        "        var key_a str = \"a\"\n",
        "        var key_b str = \"b\"\n",
        "        var count int = 0\n",
        "    }\n",
        "    view {\n",
        "        col {\n",
        "            text \"host above counters\"\n",
        "            Counter (key: .key_a, start: 10) { }\n",
        "            if .show_b { Counter (key: .key_b, start: 20) { } }\n",
        "            text \"host below counters\"\n",
        "        }\n",
        "    }\n",
        "    on {\n",
        "        Init -> { }\n",
        "        .ToggleB -> { .show_b = !.show_b }\n",
        "        .SwapKeys -> {\n",
        "            let tmp = .key_a\n",
        "            .key_a = .key_b\n",
        "            .key_b = tmp\n",
        "        }\n",
        "        .Bump -> {\n",
        "            .count = .count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
        "widget Counter (start: int = 0) {\n",
        "    msg { Inc }\n",
        "    model { var count str = \"0\" }\n",
        "    view { col { text .start text .count text .total } }\n",
        "    on {\n",
        "        Inc -> {\n",
        "            count = count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
    );

    /// 子件加字段：Counter 增 extra int = 7。
    const TWINS_ROOT_CHILD_FIELD: &str = concat!(
        "use p041_store: P041Store\n",
        "widget P041Host {\n",
        "    model {\n",
        "        var host_marker str = \"host-shell\"\n",
        "        var show_b bool = true\n",
        "        var key_a str = \"a\"\n",
        "        var key_b str = \"b\"\n",
        "        var count int = 0\n",
        "    }\n",
        "    view {\n",
        "        col {\n",
        "            text \"host above counters\"\n",
        "            Counter (key: .key_a, start: 10) { }\n",
        "            if .show_b { Counter (key: .key_b, start: 20) { } }\n",
        "            text \"host below counters\"\n",
        "        }\n",
        "    }\n",
        "    on {\n",
        "        Init -> { }\n",
        "        .ToggleB -> { .show_b = !.show_b }\n",
        "        .SwapKeys -> {\n",
        "            let tmp = .key_a\n",
        "            .key_a = .key_b\n",
        "            .key_b = tmp\n",
        "        }\n",
        "        .Bump -> {\n",
        "            .count = .count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
        "widget Counter (start: int = 0) {\n",
        "    msg { Inc }\n",
        "    model {\n",
        "        var count int = 0\n",
        "        var extra int = 7\n",
        "    }\n",
        "    view { col { text .start text .count text .total text .extra } }\n",
        "    on {\n",
        "        Inc -> {\n",
        "            count = count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
    );

    /// 子件删字段：Counter 回到只有 count（extra 删除具名）。
    const TWINS_ROOT_CHILD_DROP: &str = concat!(
        "use p041_store: P041Store\n",
        "widget P041Host {\n",
        "    model {\n",
        "        var host_marker str = \"host-shell\"\n",
        "        var show_b bool = true\n",
        "        var key_a str = \"a\"\n",
        "        var key_b str = \"b\"\n",
        "        var count int = 0\n",
        "    }\n",
        "    view {\n",
        "        col {\n",
        "            text \"host above counters\"\n",
        "            Counter (key: .key_a, start: 10) { }\n",
        "            if .show_b { Counter (key: .key_b, start: 20) { } }\n",
        "            text \"host below counters\"\n",
        "        }\n",
        "    }\n",
        "    on {\n",
        "        Init -> { }\n",
        "        .ToggleB -> { .show_b = !.show_b }\n",
        "        .SwapKeys -> {\n",
        "            let tmp = .key_a\n",
        "            .key_a = .key_b\n",
        "            .key_b = tmp\n",
        "        }\n",
        "        .Bump -> {\n",
        "            .count = .count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
        "widget Counter (start: int = 0) {\n",
        "    msg { Inc }\n",
        "    model { var count int = 0 }\n",
        "    view { col { text .start text .count text .total } }\n",
        "    on {\n",
        "        Inc -> {\n",
        "            count = count + 1\n",
        "            store.Bump()\n",
        "        }\n",
        "    }\n",
        "}\n",
    );

    /// 根去子件（删实例腿：只撤销 owner，迟到事件丢弃）。
    const TWINS_ROOT_NOCHILD: &str = concat!(
        "use p041_store: P041Store\n",
        "widget P041Host {\n",
        "    model {\n",
        "        var host_marker str = \"host-shell\"\n",
        "        var count int = 0\n",
        "    }\n",
        "    view { col { text \"host above counters\" text \"host below counters\" } }\n",
        "    on {\n",
        "        Init -> { }\n",
        "        .Bump -> { .count = .count + 1 }\n",
        "    }\n",
        "}\n",
    );

    const STORE_V1: &str = concat!(
        "store P041Store {\n",
        "    msg { Bump }\n",
        "    model { var total int = 0 }\n",
        "    on { .Bump -> { .total = .total + 1 } }\n",
        "}\n",
    );

    const STORE_V2: &str = concat!(
        "store P041Store {\n",
        "    msg { Bump }\n",
        "    model {\n",
        "        var total int = 0\n",
        "        var touched int = 0\n",
        "    }\n",
        "    on {\n",
        "        .Bump -> {\n",
        "            .total = .total + 1\n",
        "            .touched = 1\n",
        "        }\n",
        "    }\n",
        "}\n",
    );

    fn write_twins() -> Twins {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("app.at");
        let store = dir.path().join("p041_store.at");
        std::fs::write(&root, TWINS_ROOT_V1).unwrap();
        std::fs::write(&store, STORE_V1).unwrap();
        Twins {
            _dir: dir,
            root,
            store,
        }
    }

    fn build_at(code: &str, path: &std::path::Path) -> DynamicComponent {
        crate::dynamic_build::build_dynamic_component(code, Some(path.to_str().unwrap()))
            .expect("real multi-file build")
    }

    fn set_forward(path: &std::path::Path) {
        use std::time::{Duration, SystemTime};
        let t = SystemTime::now() + Duration::from_secs(2);
        let f = std::fs::File::options()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        f.set_modified(t).unwrap();
    }

    fn set_modified_back(path: &std::path::Path, t: std::time::SystemTime) {
        let f = std::fs::File::options()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        f.set_modified(t).unwrap();
    }

    fn baseline_mtime(path: &std::path::Path) -> std::time::SystemTime {
        std::fs::metadata(path).unwrap().modified().unwrap()
    }

    fn rendered(comp: &mut DynamicComponent) -> String {
        format!("{:?}", comp.view_with_debug().0)
    }

    // ========================================================================
    // V-01 / AC-01：内容快照与失效（同 mtime 改内容、缺源恢复、坏候选）
    // ========================================================================

    /// 同 mtime 改内容必须检出（内容级失效，不再依赖 mtime guard 误报/漏检）。
    #[test]
    fn v01_same_mtime_content_change_detected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("counter.at");
        std::fs::write(&path, SOLO_V1).unwrap();
        let mut comp = build_at(SOLO_V1, &path);
        comp.write_state("count", Value::Int(42)).unwrap();

        let baseline = baseline_mtime(&path);
        std::fs::write(&path, SOLO_V2).unwrap();
        set_modified_back(&path, baseline);

        assert!(
            comp.check_reload().unwrap(),
            "同 mtime 改内容必须被内容 hash 检出"
        );
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(42), "检测不提交");

        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(report) => {
                assert_eq!(report.preserved, 2, "count/label 保值");
                assert_eq!(report.added, 1, "step 新增默认");
            }
            other => panic!("同 mtime 编辑应原子提交，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(42), "提交后保值");
        assert_eq!(comp.read_state("step").unwrap(), Value::Int(5), "新字段取默认");
        assert!(rendered(&mut comp).contains("step-v2"), "新视图生效");
        comp.on_with_input("Bump", None);
        assert_eq!(
            comp.read_state("count").unwrap(),
            Value::Int(47),
            "新 handler（.step）生效"
        );
    }

    /// 缺源：Err 保旧；恢复后下一次合法编辑成功。
    #[test]
    fn v01_missing_import_keeps_old_and_recovers() {
        let fx = write_twins();
        let mut comp = build_at(TWINS_ROOT_V1, &fx.root);
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(1), "业务可运行");
        assert_eq!(comp.read_state("total").unwrap(), Value::Int(1), "store 联动");

        std::fs::remove_file(&fx.store).unwrap();
        // 缺源：候选构建具名拒绝保旧（旧 bundle 继续服役）。check_reload
        // 侧：快照差异为真（模块消失），提交腿失败保旧。
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Rejected(_)
            | crate::ui::dynamic::ReloadOutcome::Error(_) => {}
            other => panic!("缺源必须拒绝/报错保旧，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(1), "缺源后旧业务继续");
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(2), "旧桥 handler 仍可用");

        // 恢复 = 内容变化（store 加字段）→ 检出并原子提交。
        std::fs::write(&fx.store, STORE_V2).unwrap();
        set_forward(&fx.store);
        assert!(comp.check_reload().unwrap(), "恢复后检出");
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("恢复后应原子提交，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(2), "提交后保值");
        assert_eq!(
            comp.read_state("touched").unwrap(),
            Value::Int(0),
            "新 store 字段默认"
        );
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("touched").unwrap(), Value::Int(1), "新 store handler 生效");
    }

    /// 坏候选不发布；合法下一编辑整包成功（不出现部分替换）。
    #[test]
    fn v01_bad_candidate_keeps_old_next_legal_wins() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("counter.at");
        std::fs::write(&path, SOLO_V1).unwrap();
        let mut comp = build_at(SOLO_V1, &path);
        comp.write_state("count", Value::Int(9)).unwrap();

        std::fs::write(&path, SOLO_BROKEN).unwrap();
        set_forward(&path);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Rejected(_) => {}
            other => panic!("坏源必须具名拒绝保旧，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(9), "坏源后旧值原样");
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(10), "坏源后旧 handler 仍可用");

        std::fs::write(&path, SOLO_V2).unwrap();
        set_forward(&path);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("下一次合法编辑应成功，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(10), "合法编辑保值");
        assert!(rendered(&mut comp).contains("step-v2"), "新视图生效");
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(15), "新 handler 生效");
    }

    // ========================================================================
    // V-02 / AC-02：实例/schema 迁移（keyed 保值、共享保持、类型冲突拒绝）
    // ========================================================================

    /// 041 twins：keyed 实例 + store 共享值跨 reload 准确保持；新字段默认；
    /// 提交后同身份实例集合不变。
    #[test]
    fn v02_twins_keyed_instances_and_store_migrate() {
        let fx = write_twins();
        let mut comp = build_at(TWINS_ROOT_V1, &fx.root);

        let _ = comp.view();
        let ids = comp.bridge().child_instance_identities();
        assert_eq!(ids.len(), 2, "两个 keyed Counter 实例");
        let id_a = ids.iter().find(|i| i.ends_with("#a")).cloned().expect("key a");
        let id_b = ids.iter().find(|i| i.ends_with("#b")).cloned().expect("key b");
        comp.bridge_mut()
            .write_instance_state(&id_a, "count", Value::Int(11))
            .expect("A=11");
        comp.bridge_mut()
            .write_instance_state(&id_b, "count", Value::Int(22))
            .expect("B=22");
        comp.write_state("total", Value::Int(7)).expect("shared store total");

        std::fs::write(&fx.root, TWINS_ROOT_V2).unwrap();
        set_forward(&fx.root);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(report) => {
                assert!(report.preserved >= 2, "count/key 字段保值: {report:?}");
            }
            other => panic!("合法候选应原子提交，得到 {other:?}"),
        }

        let _ = comp.view();
        let ids2 = comp.bridge().child_instance_identities();
        assert_eq!(ids2.len(), 2, "提交后同身份 keyed 实例集合不变");
        let id_a2 = ids2.iter().find(|i| i.ends_with("#a")).cloned().expect("key a 仍存");
        let id_b2 = ids2.iter().find(|i| i.ends_with("#b")).cloned().expect("key b 仍存");
        assert_eq!(
            comp.bridge().read_instance_state(&id_a2, "count").unwrap(),
            Value::Int(11),
            "keyed 实例 A=11 跨 reload 保持（不重置）"
        );
        assert_eq!(
            comp.bridge().read_instance_state(&id_b2, "count").unwrap(),
            Value::Int(22),
            "keyed 实例 B=22 跨 reload 保持"
        );
        assert_eq!(comp.read_state("total").unwrap(), Value::Int(7), "store 共享值保持");
        assert_eq!(comp.read_state("host_note").unwrap(), Value::str("n"), "新根字段默认");
        assert!(rendered(&mut comp).contains("host-v2"), "新视图生效");

        // store 仍共享：Inc 经实例落到根 total。
        comp.on_with_input_for_instance("Counter", "Inc", None, Some(&id_a2));
        assert_eq!(
            comp.read_state("total").unwrap(),
            Value::Int(8),
            "实例 handler + 共享 store 仍联动"
        );
    }

    /// 子件加字段：实例兼容保值 + 新字段默认；删字段后实例面不残留。
    #[test]
    fn v02_child_field_add_default_and_drop_named() {
        let fx = write_twins();
        let mut comp = build_at(TWINS_ROOT_V1, &fx.root);
        let _ = comp.view();
        let id = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a");
        comp.bridge_mut()
            .write_instance_state(&id, "count", Value::Int(33))
            .expect("A=33");

        std::fs::write(&fx.root, TWINS_ROOT_CHILD_FIELD).unwrap();
        set_forward(&fx.root);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(report) => {
                assert!(report.preserved >= 1, "count 保值: {report:?}");
            }
            other => panic!("加字段候选应提交，得到 {other:?}"),
        }
        let _ = comp.view();
        let id2 = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a 仍存");
        assert_eq!(
            comp.bridge().read_instance_state(&id2, "count").unwrap(),
            Value::Int(33),
            "实例 model 兼容值保值"
        );
        assert_eq!(
            comp.bridge().read_instance_state(&id2, "extra").unwrap(),
            Value::Int(7),
            "实例新字段默认"
        );

        std::fs::write(&fx.root, TWINS_ROOT_CHILD_DROP).unwrap();
        set_forward(&fx.root);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(report) => {
                assert!(
                    report.dropped_names.iter().any(|n| n == "extra"),
                    "extra 删除必须具名: {:?}",
                    report.dropped_names
                );
            }
            other => panic!("删字段候选应提交，得到 {other:?}"),
        }
        let _ = comp.view();
        let id3 = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a 仍存");
        // extra 已从新 schema 删除：实例迁移按新定义丢弃（删除具名已验）。
        assert!(comp.bridge().read_instance_state(&id3, "extra").is_err());
        assert_eq!(
            comp.bridge().read_instance_state(&id3, "count").unwrap(),
            Value::Int(33),
            "删字段后兼容值仍保值"
        );
    }

    /// 同名类型冲突 → 整候选拒绝保旧；旧值 + handler 仍可用；修复后成功。
    #[test]
    fn v02_type_conflict_rejects_candidate_keeps_old() {
        let fx = write_twins();
        let mut comp = build_at(TWINS_ROOT_V1, &fx.root);
        let _ = comp.view();
        let id_a = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a");
        comp.bridge_mut()
            .write_instance_state(&id_a, "count", Value::Int(11))
            .expect("A=11");
        comp.write_state("total", Value::Int(5)).expect("shared");

        std::fs::write(&fx.root, TWINS_ROOT_TYPECHANGE).unwrap();
        set_forward(&fx.root);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Rejected(why) => {
                assert!(
                    why.contains("schema conflict"),
                    "拒绝必须具名 schema 冲突: {why}"
                );
            }
            other => panic!("类型冲突必须整候选拒绝，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("total").unwrap(), Value::Int(5), "拒绝后 store 共享值原样");
        comp.on_with_input_for_instance("Counter", "Inc", None, Some(&id_a));
        assert_eq!(comp.read_state("total").unwrap(), Value::Int(6), "旧 handler 仍可用");
        // 旧实例值仍可读（拒绝未丢数据；Inc 已落 +1）。
        assert_eq!(
            comp.bridge().read_instance_state(&id_a, "count").unwrap(),
            Value::Int(12),
            "拒绝后实例旧值原样（Inc 后 12）"
        );

        std::fs::write(&fx.root, TWINS_ROOT_V2).unwrap();
        set_forward(&fx.root);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("修复后应成功，得到 {other:?}"),
        }
        let _ = comp.view();
        let id_a2 = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a 仍存");
        assert_eq!(
            comp.bridge().read_instance_state(&id_a2, "count").unwrap(),
            Value::Int(12),
            "修复后实例保值（此前拒绝未丢数据）"
        );
    }

    // ========================================================================
    // V-03 / AC-03：失败原子性（失败后旧视图/handler/store/实例/Tick 原版）
    // ========================================================================

    /// 失败注入后整旧 bundle 原样；下一合法候选整包变新版（无新view旧handler）。
    #[test]
    fn v03_failure_atomicity_old_bundle_intact() {
        let fx = write_twins();
        let mut comp = build_at(TWINS_ROOT_V1, &fx.root);
        let _ = comp.view();
        let id_a = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a");
        comp.bridge_mut()
            .write_instance_state(&id_a, "count", Value::Int(11))
            .expect("A=11");
        comp.write_state("total", Value::Int(7)).expect("shared");
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(1));

        // 失败 1：坏源（截断）。
        std::fs::write(&fx.root, SOLO_BROKEN).unwrap();
        set_forward(&fx.root);
        assert!(matches!(comp.try_reload(), crate::ui::dynamic::ReloadOutcome::Rejected(_)));
        assert!(rendered(&mut comp).contains("host above counters"), "旧视图原版");
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(1), "旧值原版");
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(2));
        assert_eq!(
            comp.read_state("total").unwrap(),
            Value::Int(9),
            "store 原版（7 + 两次 Bump 各 +1）"
        );

        // 失败 2：类型冲突。
        std::fs::write(&fx.root, TWINS_ROOT_TYPECHANGE).unwrap();
        set_forward(&fx.root);
        assert!(matches!(comp.try_reload(), crate::ui::dynamic::ReloadOutcome::Rejected(_)));
        assert!(rendered(&mut comp).contains("host above counters"), "旧视图仍原版");
        comp.on_with_input_for_instance("Counter", "Inc", None, Some(&id_a));
        assert_eq!(
            comp.read_state("total").unwrap(),
            Value::Int(10),
            "实例 handler 原版（9 + Inc 的 store.Bump）"
        );

        // 合法下一编辑：整包变新版（新视图 + 新 handler 同现，无混合）。
        std::fs::write(&fx.root, TWINS_ROOT_V2).unwrap();
        set_forward(&fx.root);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("合法编辑应成功，得到 {other:?}"),
        }
        assert!(rendered(&mut comp).contains("host-v2"), "新视图生效");
        assert_eq!(comp.read_state("host_note").unwrap(), Value::str("n"));
        let _ = comp.view();
        let id_a2 = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a 仍存");
        assert_eq!(
            comp.bridge().read_instance_state(&id_a2, "count").unwrap(),
            Value::Int(12),
            "实例保值（两次失败均未丢数据；Inc 后 12）"
        );
    }

    // ========================================================================
    // V-04 / AC-04：effect/代次/退出（准备期零 effect、繁忙拒绝、迟到事件）
    // ========================================================================

    /// 准备期零 effect + Init 不重放：提交后 init_count 不变（build 装配时
    /// Init 已按挂载语义派发一次）。
    #[test]
    fn v04_prep_zero_effect_init_not_replayed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("init.at");
        std::fs::write(&path, INIT_V1).unwrap();
        let mut comp = build_at(INIT_V1, &path);
        assert_eq!(comp.read_state("init_count").unwrap(), Value::Int(1), "装配 Init 一次");

        std::fs::write(&path, INIT_V2).unwrap();
        set_forward(&path);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(report) => {
                assert_eq!(report.added, 1, "extra 新增默认");
            }
            other => panic!("合法候选应提交，得到 {other:?}"),
        }
        assert_eq!(
            comp.read_state("init_count").unwrap(),
            Value::Int(1),
            "提交不重放 Init（准备期零 effect；迁移值不被 Init 覆盖）"
        );
        let _ = comp.view();
        assert_eq!(
            comp.read_state("init_count").unwrap(),
            Value::Int(1),
            "首帧后 Init 仍不重放"
        );
        assert_eq!(comp.read_state("extra").unwrap(), Value::Int(9));
    }

    /// 繁忙（parked 任务）→ Busy 保旧；静止后重试成功。
    #[test]
    fn v04_busy_rejects_keeps_old_then_retry_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("counter.at");
        std::fs::write(&path, SOLO_V1).unwrap();
        let mut comp = build_at(SOLO_V1, &path);
        comp.write_state("count", Value::Int(5)).unwrap();

        std::fs::write(&path, SOLO_V2).unwrap();
        set_forward(&path);
        comp.bridge().inject_parked_task_for_test();
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Busy => {}
            other => panic!("繁忙必须保旧 Busy，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(5), "繁忙期间旧值原样");
        assert!(comp.reload_pending(), "繁忙标记待重试");

        comp.bridge().clear_parked_for_test();
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("静止后重试应成功，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(5), "重试提交保值");
    }

    /// 删实例只撤销 owner：tombstone 后迟到事件丢弃、不串新实例。
    #[test]
    fn v04_deleted_instance_tombstoned_late_event_dropped() {
        let fx = write_twins();
        let mut comp = build_at(TWINS_ROOT_V1, &fx.root);
        let _ = comp.view();
        let ids = comp.bridge().child_instance_identities();
        assert_eq!(ids.len(), 2);
        let id_a = ids.iter().find(|i| i.ends_with("#a")).cloned().expect("key a");

        std::fs::write(&fx.root, TWINS_ROOT_NOCHILD).unwrap();
        set_forward(&fx.root);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("去子件应提交，得到 {other:?}"),
        }
        let _ = comp.view();
        assert!(
            comp.bridge().child_instance_identities().is_empty(),
            "删实例后新桥无实例"
        );
        // 迟到事件（旧身份）：丢弃 + 诊断，不重定向、不复活。
        comp.on_with_input_for_instance("Counter", "Inc", None, Some(&id_a));
        assert!(
            comp.bridge().child_instance_identities().is_empty(),
            "迟到事件不得复活/重定向实例"
        );
        assert!(rendered(&mut comp).contains("host above counters"), "新视图正常渲染");
    }

    /// Tick 订阅表整体替换：新 tick_interval + 新 Tick handler 即刻生效。
    #[test]
    fn v04_tick_table_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tick.at");
        std::fs::write(&path, TICK_V1).unwrap();
        let mut comp = build_at(TICK_V1, &path);
        assert_eq!(comp.tick_interval(), Some(500), "interval model 变量驱动");

        std::fs::write(&path, TICK_V2).unwrap();
        set_forward(&path);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("合法候选应提交，得到 {other:?}"),
        }
        assert_eq!(
            comp.tick_interval(),
            Some(1000),
            "新定义 tick_interval 整体替换旧订阅表"
        );
        comp.on_with_input("Tick", None);
        assert_eq!(comp.read_state("n").unwrap(), Value::Int(10), "新 Tick handler 即刻生效");
    }

    // ========================================================================
    // V-05 / AC-05：真实 host 兼容（旧单文件 API、主题 scope、action_config）
    // ========================================================================

    /// 单文件旧 API（check_file_changed + reload）无回归；同一工程走新
    /// try_reload 同规则（单文件沿同一规则）。
    #[test]
    fn v05_single_file_legacy_and_new_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("counter.at");
        std::fs::write(&path, SOLO_V1).unwrap();
        let mut comp = build_at(SOLO_V1, &path);
        comp.write_state("count", Value::Int(4)).unwrap();

        // legacy 单文件泵面：check_file_changed + reload。
        std::fs::write(&path, SOLO_V2).unwrap();
        set_forward(&path);
        assert!(
            comp.check_file_changed().unwrap().is_some(),
            "legacy mtime 检测仍可用"
        );
        let report = {
            let session = auto_lang_core::session::CompilerSession::ui();
            let mut parser = auto_lang_core::parser::Parser::from(SOLO_V2).with_session(session);
            let ast = parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .map(auto_ui_dialect::UiCode)
                .unwrap();
            let mut found = None;
            for stmt in ast.items() {
                if let auto_parser_api::DomainStmt::Domain(
                    auto_ui_dialect::UiStmt::WidgetDecl(decl),
                ) = stmt
                {
                    let w = auto_ui_compiler::aura::extract_widget_from_decl(decl).unwrap();
                    found = Some(comp.reload(&w).unwrap());
                    break;
                }
            }
            found.expect("widget")
        };
        assert_eq!(report.preserved, 2, "legacy reload 保值");
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(4));

        // 新路径同规则：写独立合法 V3 走 try_reload（内容级）。
        comp.write_state("count", Value::Int(6)).unwrap();
        std::fs::write(&path, SOLO_V3).unwrap();
        set_forward(&path);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(report) => {
                assert_eq!(report.preserved, 2, "count/label 保值");
                assert!(report.dropped_names.iter().any(|n| n == "step"), "step 删除具名");
            }
            other => panic!("单文件同规则应提交，得到 {other:?}"),
        }
        assert_eq!(comp.read_state("count").unwrap(), Value::Int(6), "单文件保值");
        assert!(rendered(&mut comp).contains("v3"), "V3 视图生效");
    }

    /// action_config 不被劫持：候选构建不安装 DSL action，独立泵不受扰。
    #[test]
    fn v05_action_config_not_hijacked() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("action.at");
        std::fs::write(&path, ACTIONS_V1).unwrap();
        let mut comp = build_at(ACTIONS_V1, &path);
        let gen_before = crate::ui::action_config::config_generation();

        std::fs::write(&path, ACTIONS_V2).unwrap();
        set_forward(&path);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("action app 合法候选应提交，得到 {other:?}"),
        }
        assert_eq!(
            crate::ui::action_config::config_generation(),
            gen_before,
            "候选构建/提交不得安装 action DSL（action_config 独立 mtime 泵）"
        );
        assert!(rendered(&mut comp).contains("a2"), "组件视图本身正常切换");
    }

    /// 主题 scope：design 绑定 App 提交后 scope 保留（旧支持无回归）。
    #[test]
    fn v05_theme_scope_survives_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("themed.at");
        std::fs::write(&path, THEME_V1).unwrap();
        let mut comp = build_at(THEME_V1, &path);
        assert!(comp.theme_scope().is_some(), "design 绑定建立 scope");

        std::fs::write(&path, THEME_V2).unwrap();
        set_forward(&path);
        match comp.try_reload() {
            crate::ui::dynamic::ReloadOutcome::Reloaded(_) => {}
            other => panic!("主题 App 合法候选应提交，得到 {other:?}"),
        }
        assert!(comp.theme_scope().is_some(), "提交后主题 scope 保留（candidate 自带 scope）");
        assert!(rendered(&mut comp).contains("t2"), "新视图在 scope 下正常渲染");
    }

    /// root / import / child 各改真实生效（AC-05 主机面）。
    #[test]
    fn v05_root_and_import_child_changes_take_effect() {
        let fx = write_twins();
        let mut comp = build_at(TWINS_ROOT_V1, &fx.root);
        let _ = comp.view();
        let id_a = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a");
        comp.bridge_mut()
            .write_instance_state(&id_a, "count", Value::Int(11))
            .expect("A=11");

        // root 改 → 真实生效。
        std::fs::write(&fx.root, TWINS_ROOT_V2).unwrap();
        set_forward(&fx.root);
        assert!(matches!(comp.try_reload(), crate::ui::dynamic::ReloadOutcome::Reloaded(_)));
        assert!(rendered(&mut comp).contains("host-v2"), "root 改动真实生效");

        // import/store 改 → 真实生效。
        std::fs::write(&fx.store, STORE_V2).unwrap();
        set_forward(&fx.store);
        assert!(matches!(comp.try_reload(), crate::ui::dynamic::ReloadOutcome::Reloaded(_)));
        comp.on_with_input("Bump", None);
        assert_eq!(comp.read_state("touched").unwrap(), Value::Int(1), "store 改动真实生效");

        // child 改 → 真实生效（实例保值仍成立）。
        std::fs::write(&fx.root, TWINS_ROOT_CHILD_FIELD).unwrap();
        set_forward(&fx.root);
        assert!(matches!(comp.try_reload(), crate::ui::dynamic::ReloadOutcome::Reloaded(_)));
        let _ = comp.view();
        let id_a2 = comp
            .bridge()
            .child_instance_identities()
            .iter()
            .find(|i| i.ends_with("#a"))
            .cloned()
            .expect("key a 仍存");
        assert_eq!(
            comp.bridge().read_instance_state(&id_a2, "count").unwrap(),
            Value::Int(11),
            "child 改动后 keyed 实例保值"
        );
        assert_eq!(
            comp.bridge().read_instance_state(&id_a2, "extra").unwrap(),
            Value::Int(7),
            "child 新字段默认"
        );
    }
}
