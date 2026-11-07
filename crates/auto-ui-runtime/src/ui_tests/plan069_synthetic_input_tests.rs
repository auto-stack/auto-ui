//! PLAN-069 T-01（round-19 家族 F1）：嵌套 store 路径 value 绑定的注册与值写。
//!
//! 缺陷形态（修前）：`scan_node_for_inputs` 的 value prop 解析只接受
//! `Expr::Ident` / `Dot(Ident(_), field)`；Reader 书库搜索框的
//! `value: .store.query` 解析为 `Dot(Dot(Ident("."), "store"), "query")` →
//! `_ => None` 静默丢弃 → `input_state_map` 不注册该事件 → 合成键入
//! （`autoui_type`/真实输入）的值不写绑定状态（READER-009 round-19 实证：
//! 搜索框键入 OK 但 store.query 恒空、prefs 落盘 query:""）。
//!
//! 修复口径（SD-01）：store 字段合并进根态（视图读通道
//! `aura_view_builder::eval::read_state` 剥 store 别名读裸字段），故绑定的
//! 注册键同样取**裸字段名**；非 store 别名的真嵌套对象不展平。写通道
//! 与读通道同序：实例对象优先、缺字段回退根态（页模型字段仍落实例，
//! 展平 store 字段落根态）。
//!
//! 本组为永久反例：修前（注册缺失）红，修后绿；直 field 对照格不回退。

use crate::build_dynamic_component;

/// 最小 Reader 同型语料：store + 宿主 widget。搜索框绑定嵌套 store 路径，
/// handler **只读绑定字段**（不接收 typed 文本实参）——于是绑定写回是值的
/// 唯一落点，注册缺口直接表现为 handler 读到空值。
fn write_fixture(dir: &std::path::Path) -> std::path::PathBuf {
    std::fs::write(
        dir.join("s069_store.at"),
        r#"
store SearchStore {
    model {
        var q str = ""
    }
}
"#,
    )
    .expect("write s069_store.at");

    let host = dir.join("host.at");
    std::fs::write(
        &host,
        r#"
use s069_store: SearchStore

widget App {
    model {
        var seen str = ""
        var seen_direct str = ""
        var draft str = ""
        var note = { title: "" }
    }

    view {
        col {
            input {
                value: .store.q
                oninput: .Apply
            }
            input {
                value: .draft
                oninput: .ApplyDirect
            }
            input {
                value: .note.title
                oninput: .ApplyNote
            }
            text .seen {}
        }
    }

    on {
        .Apply -> {
            .seen = .store.q
        }
        .ApplyDirect -> {
            .seen_direct = .draft
        }
    }
}
"#,
    )
    .expect("write host.at");
    host
}

fn build(dir: &std::path::Path) -> crate::ui::dynamic::DynamicComponent {
    let host = write_fixture(dir);
    let code = std::fs::read_to_string(&host).expect("read host.at");
    build_dynamic_component(&code, Some(&host.to_string_lossy()))
        .expect("plan-069 fixture must compile")
}

fn read_str(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
    match dc.read_state(field) {
        Ok(auto_val::Value::Str(s)) => s.as_str().to_string(),
        other => format!("<{field} not str: {other:?}>"),
    }
}

/// V01(a)：嵌套 `.store.query` 同型绑定注册为裸字段名 `q`（与读通道同口径），
/// 直 field 对照 `draft` 注册不变。
#[test]
fn f1_nested_store_path_binding_registers_with_flattened_field() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dc = build(dir.path());

    let map = dc.input_state_map();
    assert_eq!(
        map.get("Apply"),
        Some(&"q".to_string()),
        "嵌套 `.store.q` 绑定必须注册为展平根字段 q（修前整条缺失 = F1 缺口）；实际 map={map:?}"
    );
    assert_eq!(
        map.get("ApplyDirect"),
        Some(&"draft".to_string()),
        "直 field 绑定注册不得回退（首匹配/裸字段语义保持）；实际 map={map:?}"
    );
}

/// V01(b)：嵌套 store 绑定的合成键入真实写入 store 状态，且 handler 读到的
/// 就是写入值（注册 + 写落点同链）。
#[test]
fn f1_nested_store_binding_write_reaches_store_state() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut dc = build(dir.path());
    dc.fire_init();
    let _ = dc.view_with_debug_gated(true);

    dc.on_with_input_for("App", "Apply", Some("reader-query".to_string()));
    let _ = dc.view_with_debug_gated(true);

    assert_eq!(
        read_str(&dc, "q"),
        "reader-query",
        "type_text 同路径的输入必须写入 store 合并字段 q（修前恒空）"
    );
    assert_eq!(
        read_str(&dc, "seen"),
        "reader-query",
        "handler 读绑定字段必须看到本次键入值（所见=所用）"
    );

    // 二次键入 = 整体替换（不是追加），绑定写回语义与真实输入一致。
    dc.on_with_input_for("App", "Apply", Some("second".to_string()));
    let _ = dc.view_with_debug_gated(true);
    assert_eq!(read_str(&dc, "q"), "second", "二次键入必须整体替换");
    assert_eq!(read_str(&dc, "seen"), "second");
}

/// V01(c) 对照格：直 field 绑定的写回与 handler 读值不变。
#[test]
fn f1_direct_field_binding_write_is_unchanged() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut dc = build(dir.path());
    dc.fire_init();
    let _ = dc.view_with_debug_gated(true);

    dc.on_with_input_for("App", "ApplyDirect", Some("draft-text".to_string()));
    let _ = dc.view_with_debug_gated(true);

    assert_eq!(read_str(&dc, "draft"), "draft-text", "直 field 写回不回退");
    assert_eq!(
        read_str(&dc, "seen_direct"),
        "draft-text",
        "直 field handler 读值不回退"
    );
}

/// V01(d) 负例（终审 F1 备注）：**非 store 别名**的真嵌套路径（`.note.title`）
/// 不展平、不注册——展平只对 store 合并字段成立，否则会把任意嵌套对象的
/// 叶子名误绑定到根态同名字段。
#[test]
fn f1_non_store_nested_path_is_not_flattened() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dc = build(dir.path());
    let map = dc.input_state_map();
    assert!(
        map.get("ApplyNote").is_none(),
        "非 store 嵌套路径不得注册（展平仅限 store 别名）；实际 map={map:?}"
    );
    assert_eq!(
        map.get("Apply"),
        Some(&"q".to_string()),
        "store 展平对照仍须在案"
    );
}
