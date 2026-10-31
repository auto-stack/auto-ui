// PLAN-814 T-04：随 ui/ 迁入的 UI 域测试树（原 auto-lang src/tests/ 中
// 引用 runtime 私有测试钩子的子集；挂载门与原 tests.rs 逐项一致，
// crate:: 路径经 runtime 根再导出面保持。
// PLAN-814 F814-MERGE-01 修复：app_registry 启动端到端测试随单源组合
// 归位本包（compiler 侧 ui-iced 门永假 → 选择面 5327 的缺口闭合）。
#[cfg(test)]
mod musk_vm_track_tests;
#[cfg(test)]
#[path = "ui_tests/p614_tree_vm_probe.rs"]
mod p614_tree_vm_probe;
#[cfg(test)]
#[path = "ui_tests/p618_filter_tree_probe.rs"]
mod p618_filter_tree_probe;
mod plan051_ext_widget_tests;
mod plan051_p2_tests;
mod plan051_timer_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan059_child_emit_probe_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan063_handler_float_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan066_filter_projection_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan066_native_widget_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan083_http_msg_bridge_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan089_mouse_area_press_tests;
#[cfg(test)]
mod plan339_tests;
#[cfg(test)]
mod plan340_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan370_015_behavior_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan370_store_vm_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan409_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan412_tests;
#[cfg(feature = "code-editor")]
mod plan413_code_editor_natives;
#[cfg(test)]
mod plan437_child_init_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_ext_link_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_musk_probe_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_sched_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_store_facade_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_webcompat_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan446_b1_tests;
#[cfg(test)]
mod plan446_batch3_tests;
#[cfg(test)]
mod plan446_batch4_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan446_batch5_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan446_c1_popover_tests;
#[cfg(test)]
mod plan446_j1_repro_tests;
#[cfg(test)]
mod plan484_chart_component_tests;
#[cfg(test)]
mod plan492_m4_tests;
#[cfg(test)]
mod plan492_m5_tests;
#[cfg(test)]
mod plan492_tests;
#[cfg(test)]
mod plan498_chart_interaction_tests;
#[cfg(test)]
mod plan499_pointer_stream_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan536_t12_send_chain_probe_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan536_t1_reactive_probe_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan606_gallery_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan622_store_facade_gap_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan624_cross_state_tests;
mod plan656_scroll_controller_pipeline;
#[cfg(test)]
mod plan657_bp_admin_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan670_code_editor_event_tests;
mod plan703_supply_probes;
mod plan710_supply_probes;
mod plan728_supply_probes;
#[cfg(all(test, feature = "ui-iced"))]
mod plan739_input_pin_tests;
mod plan765_active_session_stability_tests;
mod plan794_scroll_host_provider;
mod plan802_code_editor_provider;
mod plan807_text_core;
mod plan811_media_runtime;
#[cfg(all(test, feature = "ui-iced"))]
mod plan814_app_registry_launch_tests;

// PLAN-814 T-07：dev-deps 消费面随 owner 迁入（auto-lang 侧测试期
// UI 依赖清零）。
#[cfg(feature = "native-clipboard")]
mod plan485_clipboard_natives;
#[cfg(all(test, feature = "ui-iced"))]
mod plan732_wikilink_tests;
mod plan804_dialog_dnd_provider;
#[cfg(windows)]
mod plan804_os_e2e;

// PLAN-814 T-07 勘误补挂：T-04 批三模块（plan019/042/058）文件已迁但挂载
// 漏加（shell 短路事故），选择面验证器逐 ID 捕获——门与原 tests.rs 一致。
#[cfg(all(test, feature = "ui-iced"))]
mod plan019_vm_own_module_link_tests;
#[cfg(test)]
mod plan042_scan_probe_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan058_engine_gap_tests;

// ---- PLAN-900 T-05：auto-lang src/tests UI 域 32 文件随依赖反转迁入 ----
// （原 crates/auto-lang/src/tests/ 无条件 UI 符号文件；crate:: 路径已改写
// 为 auto_ui_compiler/auto_lang_core/crate:: 本位；cfg 门与原 tests.rs 声明
// 一致——plan814_v02_api_pin 原 ui-iced 门保留，其余默认编译（原 facade
// default=ui-iced 下同语义）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan814_v02_api_pin;
#[cfg(test)]
mod autodown_codegen_debts_tests;
#[cfg(test)]
mod back_proxy_tests;
#[cfg(test)]
mod error_spans_tests;
#[cfg(test)]
mod gallery_pages_compile_tests;
#[cfg(test)]
mod native_css_tests;
#[cfg(test)]
mod plan024_named_view_tests;
#[cfg(test)]
mod plan367_viewfn_tests;
#[cfg(test)]
mod plan408_tests;
#[cfg(test)]
mod plan492_m2_tests;
#[cfg(test)]
mod plan492_m3_tests;
#[cfg(test)]
mod plan502_diagram_tests;
#[cfg(test)]
mod plan503_tests;
#[cfg(test)]
mod plan627_qualified_api_tests;
#[cfg(test)]
mod plan634_block_tail_semi_tests;
#[cfg(test)]
mod plan639_bp_tests;
#[cfg(test)]
mod plan640_bp_tests;
#[cfg(test)]
mod plan643_chart_tag_tests;
#[cfg(test)]
mod plan645_bp_tests;
#[cfg(test)]
mod plan647_bp_version_tests;
#[cfg(test)]
mod plan649_bp_tests;
#[cfg(test)]
mod plan701_supply_probes;
#[cfg(test)]
mod plan711_cpu_slice_tests;
#[cfg(test)]
mod plan711_init_demand_tests;
#[cfg(test)]
mod plan716_supply_probes;
#[cfg(test)]
mod plan779_frame_provider;
#[cfg(test)]
mod plan785_clipboard_provider;
#[cfg(test)]
mod plan806_ui_declarations;
#[cfg(test)]
mod plan808_terminal_domain;
#[cfg(test)]
mod plan810_full_ui_grammar;
#[cfg(test)]
mod plan814_recipe_extraction_tests;
#[cfg(test)]
mod vm_json_float_read_tests;
