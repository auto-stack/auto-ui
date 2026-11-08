//! PLAN-071 只读原生取证记录（schema `autographics.m1-native-evidence/1`）。
//!
//! 一个 run_id 绑定：宿主种类/模式、项目身份（路径＋sha256）、候选身份、
//! 窗口实例（实际 scale/尺寸、真实 ScaleFactorChanged 事件）、逐检查点
//! 实际读数（前后计数）、OS 事件计数、N1..N6 人工逐项观察与观察人字段。
//! 记录逻辑零业务访问：只读宿主公开面（texts/callbacks/scene_state/epochs/
//! widget_bounds），不改 Tick/派发/路由。
//!
//! 分层规则（C5，validator 强制）：`offscreen`/`posted-window`/`state-record`
//! 只是控制/绑定层，永不 native-pass；`native-manual` 记录在 N1..N6 任一
//! 结论缺失（观察人未填）、观察人缺失、单一 scale 档、项目/候选身份缺失
//! 时保持 `pending`（具名 blocked 原因）；`auto_pass` 恒为 false，报告里
//! 出现 true 视为伪记录具名拒绝。事件计数多少不等于语义正确——结论字段
//! 才是判据。

/// 产物 schema 身份（AG026/027 消费面；SD-02 冻结）。
pub const M1_EVIDENCE_SCHEMA: &str = "autographics.m1-native-evidence/1";

/// 混合区域宿主的 N1..N6 手工检查表（`mixed_regions_window --manual`；
/// PLAN-066 T-02 检查表逐项具名化，第二真实 scale 为 N6）。
pub const MIXED_MANUAL_CHECKLIST: [(&str, &str); 6] = [
    ("N1", "Tab + Enter activate the next button exactly once"),
    ("N2", "a click outside every control changes nothing"),
    (
        "N3",
        "Alt-Tab away then back: the stale focused button does not activate on Enter",
    ),
    ("N4", "mouse leave cancels the press (no region click)"),
    ("N5", "resize; the next click lands on the widget's new position"),
    ("N6", "repeat on a second DPI scale if the machine has one"),
];

static RUN_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// 一个证据记录（构建期 JSON 对象；写入/校验都走显式路径）。
pub struct EvidenceRecord {
    data: serde_json::Value,
}

impl EvidenceRecord {
    /// 新记录：run_id 由宿主/时刻/单调计数生成；auto_pass 恒 false。
    pub fn new(host: &str, mode: &str) -> Self {
        let n = RUN_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let run_id = format!(
            "m1-{host}-{}-{n}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        );
        let mut data = serde_json::json!({
            "schema": M1_EVIDENCE_SCHEMA,
            "run_id": run_id,
            "host": host,
            "mode": mode,
            "plan_scope": "auto-ui/PLAN-071",
            "project": { "kind": serde_json::Value::Null, "bindings": {} },
            "candidate": {},
            "window": {
                "scale": serde_json::Value::Null,
                "logical": serde_json::Value::Null,
                "physical": serde_json::Value::Null,
                "scale_factor_changed": [],
            },
            "checkpoints": [],
            "os_events": {},
            "checklist_items": {},
            "observer": { "name": serde_json::Value::Null, "notes": serde_json::Value::Null },
            "tier": { "channel": serde_json::Value::Null, "scales_observed": [] },
            "auto_pass": false,
            "failure": serde_json::Value::Null,
        });
        for (id, item) in MIXED_MANUAL_CHECKLIST {
            data["checklist_items"][id] = serde_json::json!({
                "item": item,
                "observed": {},
                "conclusion": serde_json::Value::Null,
            });
        }
        EvidenceRecord { data }
    }

    /// 项目身份绑定（label → sha256；内容身份由调用方实际读文件计算）。
    pub fn bind_project(&mut self, kind: &str, bindings: serde_json::Value) {
        self.data["project"]["kind"] = serde_json::json!(kind);
        self.data["project"]["bindings"] = bindings;
    }

    /// 候选身份绑定（engine/runner/window/lang/sdk 等，值由调用方实测）。
    pub fn bind_candidate(&mut self, candidate: serde_json::Value) {
        self.data["candidate"] = candidate;
    }

    /// 实际窗口观察（winit 读回；不做推断）。
    pub fn observe_window(&mut self, scale: f64, logical: [f64; 2], physical: [u32; 2]) {
        self.data["window"]["scale"] = serde_json::json!(scale);
        self.data["window"]["logical"] = serde_json::json!(logical);
        self.data["window"]["physical"] = serde_json::json!(physical);
        self.data["tier"]["scales_observed"] = serde_json::json!([scale]);
    }

    /// 真实 ScaleFactorChanged（或平台等价）事件记录（前后实际读数）。
    pub fn record_scale_change(&mut self, before: f64, after: f64, at: &str) {
        self.data["window"]["scale_factor_changed"]
            .as_array_mut()
            .expect("scale array")
            .push(serde_json::json!({ "before": before, "after": after, "at": at }));
        let mut scales = self.data["tier"]["scales_observed"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if !scales.iter().any(|v| v == &serde_json::json!(after)) {
            scales.push(serde_json::json!(after));
        }
        self.data["tier"]["scales_observed"] = serde_json::Value::Array(scales);
    }

    /// 一个检查点的实际读数（label；前后计数由调用方语义命名）。
    pub fn checkpoint(&mut self, label: &str, counters: serde_json::Value) {
        let mut entry = serde_json::json!({ "label": label });
        if let Some(obj) = counters.as_object() {
            for (k, v) in obj {
                entry[k.clone()] = v.clone();
            }
        }
        self.data["checkpoints"]
            .as_array_mut()
            .expect("checkpoints")
            .push(entry);
    }

    /// OS 事件计数（只增；kinds 由调用方具名）。
    pub fn count_os_event(&mut self, kind: &str) {
        let cur = self.data["os_events"][kind].as_u64().unwrap_or(0);
        self.data["os_events"][kind] = serde_json::json!(cur + 1);
    }

    /// 逐项观察值（不设结论——结论只由观察人字段写入）。
    pub fn observe_checklist(&mut self, id: &str, observed: serde_json::Value) {
        if self.data["checklist_items"][id].is_null() {
            self.data["checklist_items"][id] =
                serde_json::json!({ "item": id, "observed": {}, "conclusion": serde_json::Value::Null });
        }
        self.data["checklist_items"][id]["observed"] = observed;
    }

    /// 观察人结论（手工面专用；自动链不得调用）。
    pub fn conclude_checklist(&mut self, id: &str, conclusion: &str) {
        self.data["checklist_items"][id]["conclusion"] = serde_json::json!(conclusion);
    }

    pub fn set_observer(&mut self, name: Option<&str>, notes: Option<&str>) {
        self.data["observer"]["name"] = name.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null);
        self.data["observer"]["notes"] = notes.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null);
    }

    pub fn set_tier_channel(&mut self, channel: &str) {
        self.data["tier"]["channel"] = serde_json::json!(channel);
    }

    pub fn fail(&mut self, reason: &str) {
        self.data["failure"] = serde_json::json!(reason);
    }

    pub fn to_value(&self) -> serde_json::Value {
        self.data.clone()
    }

    /// 写入显式路径（父目录自动创建；返回字节数）。禁默认路径——调用方
    /// 必须给出显式输出文件。
    pub fn write_json(&self, path: &std::path::Path) -> Result<u64, String> {
        let bytes = serde_json::to_vec_pretty(&self.data).map_err(|e| e.to_string())?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, &bytes).map_err(|e| e.to_string())?;
        Ok(bytes.len() as u64)
    }
}

/// 严格校验结论：`status` ＋ 具名 blocked 原因（升级路径被什么挡住）。
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub status: String,
    pub blocked: Vec<String>,
}

impl Verdict {
    fn pending(blocked: Vec<String>) -> Self {
        Verdict { status: "pending".into(), blocked }
    }
}

fn is_sha256(v: &serde_json::Value) -> bool {
    v.as_str()
        .map(|s| s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or(false)
}

/// 严格校验（AG026/027 checker 的引擎侧同面；C4/C5 全规则）。
pub fn validate(record: &serde_json::Value) -> Verdict {
    let mut blocked: Vec<String> = Vec::new();
    if record["schema"] != serde_json::json!(M1_EVIDENCE_SCHEMA) {
        return Verdict {
            status: "invalid".into(),
            blocked: vec![format!(
                "schema must be {M1_EVIDENCE_SCHEMA} (got {})",
                record["schema"]
            )],
        };
    }
    if record["auto_pass"] == serde_json::json!(true) {
        return Verdict {
            status: "invalid".into(),
            blocked: vec!["auto_pass=true is a fabricated record (recorders always write false)".into()],
        };
    }
    if record["failure"].is_string() {
        return Verdict {
            status: "fail".into(),
            blocked: vec![format!("recorded failure: {}", record["failure"])],
        };
    }
    let mode = record["mode"].as_str().unwrap_or("");
    // 项目身份：至少一个 64-hex sha 绑定。
    let bindings = &record["project"]["bindings"];
    let bound = bindings
        .as_object()
        .map(|m| m.values().any(is_sha256))
        .unwrap_or(false);
    if !bound {
        blocked.push("project identity has no sha256 binding".into());
    }
    if record["candidate"].as_object().map(|m| m.is_empty()).unwrap_or(true) {
        blocked.push("candidate identity is unbound".into());
    }
    // 分层：非 native-manual 永远只是控制/绑定层。
    if mode != "native-manual" {
        blocked.push(format!(
            "mode '{mode}' is a control/binding tier and can never close the native gate"
        ));
        return Verdict::pending(blocked);
    }
    // N1..N6 逐项人工结论。
    let items = match record["checklist_items"].as_object() {
        Some(m) => m,
        None => {
            blocked.push("checklist_items missing".into());
            return Verdict::pending(blocked);
        }
    };
    for (id, item) in MIXED_MANUAL_CHECKLIST {
        match items.get(id) {
            None => blocked.push(format!("checklist {id} missing")),
            Some(v) => match v["conclusion"].as_str() {
                None | Some("") => blocked.push(format!(
                    "checklist {id} has no human conclusion ({item})"
                )),
                Some("pass") => {}
                Some(other) => blocked.push(format!("checklist {id} conclusion '{other}'")),
            },
        }
    }
    // 观察人。
    if record["observer"]["name"].as_str().unwrap_or("").is_empty() {
        blocked.push("observer name missing (human evidence requires an observer)".into());
    }
    // 真实 scale 档：单一档不闭双 scale 门。
    let scales: Vec<f64> = record["tier"]["scales_observed"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();
    if scales.len() < 2 {
        blocked.push(format!(
            "single scale tier {scales:?}: the second real DPI scale stays an external gate"
        ));
    }
    if blocked.is_empty() {
        Verdict { status: "native-pass".into(), blocked }
    } else {
        Verdict::pending(blocked)
    }
}

/// 读文件并校验（消费入口；坏 JSON/坏 schema 具名拒绝）。
pub fn validate_file(path: &std::path::Path) -> Result<Verdict, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let record: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("{}: not JSON: {e}", path.display()))?;
    Ok(validate(&record))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bound_record() -> EvidenceRecord {
        let mut r = EvidenceRecord::new("mixed-regions", "native-manual");
        r.bind_project(
            "mixed-artifact",
            serde_json::json!({"ui.json": "a".repeat(64)}),
        );
        r.bind_candidate(serde_json::json!({"window": "b".repeat(64)}));
        r
    }

    #[test]
    fn single_scale_without_observer_stays_pending_with_named_blocks() {
        let mut r = bound_record();
        r.observe_window(2.0, [480.0, 560.0], [960, 1120]);
        r.set_tier_channel("native");
        let v = validate(&r.to_value());
        assert_eq!(v.status, "pending");
        assert!(v.blocked.iter().any(|b| b.contains("second real DPI")));
        assert!(v.blocked.iter().any(|b| b.contains("human conclusion")));
        assert!(v.blocked.iter().any(|b| b.contains("observer")));
    }

    #[test]
    fn completed_dual_scale_human_record_closes() {
        let mut r = bound_record();
        r.observe_window(2.0, [480.0, 560.0], [960, 1120]);
        r.record_scale_change(2.0, 1.25, "N6");
        for (id, _) in MIXED_MANUAL_CHECKLIST {
            r.conclude_checklist(id, "pass");
        }
        r.set_observer(Some("human"), Some("done"));
        let v = validate(&r.to_value());
        assert_eq!(v.status, "native-pass", "{:?}", v.blocked);
    }

    #[test]
    fn fabricated_auto_pass_is_named_invalid() {
        let mut v = bound_record().to_value();
        v["auto_pass"] = serde_json::json!(true);
        let out = validate(&v);
        assert_eq!(out.status, "invalid");
        assert!(out.blocked[0].contains("fabricated"));
    }

    #[test]
    fn unbound_project_and_control_tiers_never_pass() {
        let mut r = EvidenceRecord::new("mixed-regions", "offscreen");
        let v = validate(&r.to_value());
        assert_eq!(v.status, "pending");
        assert!(v.blocked.iter().any(|b| b.contains("control/binding tier")));
        assert!(v.blocked.iter().any(|b| b.contains("no sha256 binding")));
    }

    #[test]
    fn recorded_failure_is_fail_not_pending() {
        let mut r = bound_record();
        r.fail("timeout");
        let v = validate(&r.to_value());
        assert_eq!(v.status, "fail");
    }

    #[test]
    fn scale_changes_extend_observed_tiers() {
        let mut r = bound_record();
        r.observe_window(2.0, [480.0, 560.0], [960, 1120]);
        assert_eq!(
            r.to_value()["tier"]["scales_observed"],
            serde_json::json!([2.0])
        );
        r.record_scale_change(2.0, 1.25, "N6");
        assert_eq!(
            r.to_value()["tier"]["scales_observed"],
            serde_json::json!([2.0, 1.25])
        );
    }
}
