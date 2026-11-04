//! PLAN-044：driver 侧 Node 源映射受检消费。
//!
//! 与 pilot 侧旁件 schema 独立镜像（两 Cargo 图永不共享 crate）：结构
//! 校验 + world/closure 字节绑定 + user offset → 原模块行列诊断。映射只
//! 解释已有诊断；坏图/坏闭包一律拒用，调用方明确回退原
//! generated/unmapped 诊断，不部分采用、不读任意外部文件。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// 旁件格式版本（与 pilot source_map::MAP_FORMAT 同值，T-02 冻结）。
pub const MAP_FORMAT: &str = "node-source-map/v1";
/// metadata 上界（T-02 冻结）：旁件 JSON 本身 ≤64KiB。
pub const MAX_MAP_BYTES: usize = 64 * 1024;
/// 段数上界（T-02 冻结）：线性存储/查找预算；超出即整图拒用。
pub const MAX_SEGMENTS: usize = 4096;

/// 定位精度（与 pilot 口径一致）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Precision {
    Exact,
    Range,
    Context,
    Generated,
    Unmapped,
}

/// 生成用户源映射段：半开 byte 区间 [start, end)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub start: usize,
    pub end: usize,
    pub module: Option<String>,
    pub node: Option<String>,
    pub event: Option<String>,
    pub helper: Option<String>,
    pub precision: Precision,
    pub source_start: Option<usize>,
    pub source_end: Option<usize>,
}

/// 闭包模块身份（sha256 + byte_length，与 pilot identity.json 同口径）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleEntry {
    pub sha256: String,
    pub bytes: usize,
}

/// 旁件顶层结构（独立镜像；deny_unknown_fields 拒未来未知字段的旧消费）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceMap {
    pub format: String,
    pub generator: String,
    pub world_sha256: String,
    pub user_source_sha256: String,
    pub user_source_bytes: usize,
    pub modules: BTreeMap<String, ModuleEntry>,
    pub segments: Vec<Segment>,
}

/// 一个已解析的原作者位置（diagnose 的成功结果）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    pub module: String,
    pub node: Option<String>,
    pub event: Option<String>,
    pub helper: Option<String>,
    pub precision: Precision,
    /// 模块源内 byte offset。
    pub source_offset: usize,
    pub line: usize,
    pub column: usize,
}

fn valid_sha256(sha: &str) -> bool {
    sha.len() == 64 && sha.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_module_key(key: &str) -> bool {
    !key.is_empty()
        && !key.contains('\\')
        && !key.contains('/')
        && !key.contains(':')
        && !key.contains("..")
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// frozen mirror of the pilot's `lower_components::rename_module_calls`
/// (PLAN-044 T-02)：行内 `{name}(` → `{module}_{name}(`（prev_ok 防链式
/// 重写）。两侧 gates 钉死行为一致；改 pilot 语义必须同步此处并双侧红。
fn replay_module_rename(text: &str, names: &[String], module: &str) -> String {
    let mut out = text.to_string();
    for n in names {
        let mut res = String::new();
        let bytes = out.as_bytes();
        let mut i = 0usize;
        while i < out.len() {
            if out[i..].starts_with(n.as_str()) {
                let prev_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
                let next = i + n.len();
                let next_ok = next < out.len() && bytes[next] == b'(';
                if prev_ok && next_ok {
                    res.push_str(module);
                    res.push('_');
                    res.push_str(n);
                    i = next;
                    continue;
                }
            }
            let ch = out[i..].chars().next().unwrap();
            res.push(ch);
            i += ch.len_utf8();
        }
        out = res;
    }
    out
}

/// 顶层 fn 名重导出规则（与 pilot load.rs 的 AST 顶层 Fn 口径一致：
/// 模块源中列 0 起始的 `fn {name}(` 行）。sibling 表只用于重放重命名。
fn module_fn_names(module_source: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in module_source.lines() {
        if let Some(rest) = line.strip_prefix("fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out.sort();
    out
}

impl SourceMap {
    /// 解析 + 结构校验 + world 字节绑定。任何失败即整图拒用（Err）。
    pub fn parse(json: &str, world_source: &str) -> Result<Self, String> {
        if json.len() > MAX_MAP_BYTES {
            return Err("map exceeds 64KiB metadata budget".into());
        }
        let map: SourceMap =
            serde_json::from_str(json).map_err(|e| format!("map parse: {e}"))?;
        if map.format != MAP_FORMAT {
            return Err(format!("map format '{}' != {}", map.format, MAP_FORMAT));
        }
        if !valid_sha256(&map.world_sha256) {
            return Err("bad world_sha256".into());
        }
        if !valid_sha256(&map.user_source_sha256) {
            return Err("bad user_source_sha256".into());
        }
        if map.user_source_bytes != world_source.len() {
            return Err(format!(
                "map user_source_bytes {} != world {}",
                map.user_source_bytes,
                world_source.len()
            ));
        }
        if super::behavior::sha256_hex_for_tests(world_source.as_bytes()) != map.user_source_sha256 {
            return Err("map user_source_sha256 mismatch".into());
        }
        if super::behavior::sha256_hex_for_tests(world_source.as_bytes()) != map.world_sha256 {
            return Err("map world_sha256 mismatch".into());
        }
        if map.segments.len() > MAX_SEGMENTS {
            return Err("map exceeds segment budget".into());
        }
        for (key, entry) in &map.modules {
            if !valid_module_key(key) {
                return Err(format!("invalid module key '{key}'"));
            }
            if !valid_sha256(&entry.sha256) {
                return Err(format!("bad sha256 for module '{key}'"));
            }
        }
        let mut prev_end = 0usize;
        for seg in &map.segments {
            if seg.start > seg.end || seg.end > world_source.len() {
                return Err(format!("segment {}..{} out of bounds", seg.start, seg.end));
            }
            if seg.start < prev_end {
                return Err("segments overlap or unordered".into());
            }
            if !world_source.is_char_boundary(seg.start) || !world_source.is_char_boundary(seg.end)
            {
                return Err("segment not on char boundary".into());
            }
            match seg.precision {
                Precision::Exact | Precision::Range | Precision::Context => {
                    let key = seg.module.as_deref().ok_or("mapped segment missing module")?;
                    let entry = map
                        .modules
                        .get(key)
                        .ok_or_else(|| format!("segment module '{key}' not in closure"))?;
                    if seg.precision != Precision::Context {
                        let (ss, se) = (
                            seg.source_start.ok_or("mapped segment missing source_start")?,
                            seg.source_end.ok_or("mapped segment missing source_end")?,
                        );
                        if ss > se || se > entry.bytes {
                            return Err(format!("bad source range for segment {key}"));
                        }
                    }
                }
                Precision::Generated | Precision::Unmapped => {
                    if seg.module.is_some() || seg.source_start.is_some() || seg.source_end.is_some()
                    {
                        return Err("generated/unmapped segment must not carry module/source".into());
                    }
                }
            }
            prev_end = seg.end;
        }
        Ok(map)
    }

    /// 受检闭包装载：只读 `closure_dir/{key}.at` 且逐字节 sha 绑定；缺文件、
    /// hash 不符、闭包超申报长度一律拒用。不读任何其它路径。
    pub fn closure_sources(
        &self,
        closure_dir: &Path,
    ) -> Result<BTreeMap<String, String>, String> {
        let mut out = BTreeMap::new();
        for (key, entry) in &self.modules {
            let path = closure_dir.join(format!("{key}.at"));
            let bytes = std::fs::read(&path)
                .map_err(|e| format!("closure read {}: {e}", path.display()))?;
            if bytes.len() != entry.bytes {
                return Err(format!("closure bytes drift for '{key}'"));
            }
            if super::behavior::sha256_hex_for_tests(&bytes) != entry.sha256 {
                return Err(format!("closure sha mismatch for '{key}'"));
            }
            out.insert(key.clone(), String::from_utf8(bytes).map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    /// PLAN-044 T-03 loader 接受身份接线：世界目录（world.at + closure/ +
    /// 可选 source-map.json）→ 受检 (map, closure)。旁件缺失/坏图/坏闭包
    /// 一律 None（调用方走原默认诊断路径，不改变准入或运行语义）。
    /// 只读目录内三样受检物，不递归、不读任意外部文件。
    pub fn load_for_world(world_path: &Path) -> Option<(SourceMap, BTreeMap<String, String>)> {
        let world = std::fs::read(world_path).ok()?;
        let world_text = String::from_utf8(world).ok()?;
        let map_path = world_path.with_file_name("source-map.json");
        let map_text = std::fs::read_to_string(map_path).ok()?;
        let map = SourceMap::parse(&map_text, &world_text).ok()?;
        let closure_dir = world_path.parent()?.join("closure");
        let closure = map.closure_sources(&closure_dir).ok()?;
        Some((map, closure))
    }

    fn lookup(&self, offset: usize) -> Option<&Segment> {
        self.segments
            .binary_search_by(|s| {
                if offset < s.start {
                    std::cmp::Ordering::Greater
                } else if offset >= s.end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .ok()
            .map(|i| &self.segments[i])
    }

    /// user offset（受检 assembly user region 内）→ 原模块行列。
    /// 只解释、不猜测：generated/unmapped 段或任何不可证条件返回 None，
    /// 由调用方回退原诊断。
    ///
    /// PLAN-044 T-02 实证冻结：SDK681 的 undefined-symbol span 为
    /// "call 末尾 + 前置注释行数"口径（±1 字节级漂移，见 Plan §9），
    /// 不可字节回算。因此可证路径为双锚：
    /// 1. span 锚：offset 落入 range/exact 段且重放校验通过；若 message
    ///    携带符号，则要求 world 该处确为该符号（防偏移 span 误映）；
    /// 2. 符号锚恢复：span 锚失败且 message 有符号时，在全部 range 段内
    ///    搜该符号的唯一出现，唯一则映射该出现点。
    pub fn diagnose_user_offset(
        &self,
        offset: usize,
        world_source: &str,
        closure: &BTreeMap<String, String>,
        symbol: Option<&str>,
    ) -> Option<Origin> {
        if let Some(origin) = self.diagnose_span_anchored(offset, world_source, closure, symbol) {
            return Some(origin);
        }
        self.diagnose_symbol_anchored(world_source, closure, symbol)
    }

    fn diagnose_span_anchored(
        &self,
        offset: usize,
        world_source: &str,
        closure: &BTreeMap<String, String>,
        symbol: Option<&str>,
    ) -> Option<Origin> {
        let seg = self.lookup(offset)?;
        if !matches!(
            seg.precision,
            Precision::Exact | Precision::Range | Precision::Context
        ) {
            return None;
        }
        // span 锚 + 符号校验：SDK span 漂移时 world 处不是该符号 → 拒映。
        if let Some(sym) = symbol {
            let at = world_source.get(offset..offset + sym.len())?;
            if at != sym {
                return None;
            }
        }
        self.origin_in_segment(seg, offset, world_source, closure)
    }

    fn diagnose_symbol_anchored(
        &self,
        world_source: &str,
        closure: &BTreeMap<String, String>,
        symbol: Option<&str>,
    ) -> Option<Origin> {
        let sym = symbol?;
        let mut found: Vec<(usize, &Segment)> = Vec::new();
        for seg in &self.segments {
            if !matches!(seg.precision, Precision::Exact | Precision::Range) {
                continue;
            }
            let body = world_source.get(seg.start..seg.end)?;
            let mut cursor = 0usize;
            while let Some(rel) = body[cursor..].find(sym) {
                let pos = seg.start + cursor + rel;
                // 标识符边界：前后须非标识符字符（防子串误配）。
                let prev_ok = pos == 0
                    || !world_source.as_bytes()[pos - 1].is_ascii_alphanumeric()
                        && world_source.as_bytes()[pos - 1] != b'_';
                let next = pos + sym.len();
                let next_ok = next >= world_source.len()
                    || (!world_source.as_bytes()[next].is_ascii_alphanumeric()
                        && world_source.as_bytes()[next] != b'_');
                if prev_ok && next_ok {
                    found.push((pos, seg));
                }
                cursor += rel + sym.len();
            }
        }
        // 唯一出现才可证；多个候选不猜（回退上下文）。
        if found.len() != 1 {
            return None;
        }
        let (pos, seg) = found[0];
        self.origin_in_segment(seg, pos, world_source, closure)
    }

    fn origin_in_segment(
        &self,
        seg: &Segment,
        offset: usize,
        world_source: &str,
        closure: &BTreeMap<String, String>,
    ) -> Option<Origin> {
        let module = seg.module.clone()?;
        let source = closure.get(&module)?;
        let source_offset = match seg.precision {
            Precision::Exact => seg.source_start?.checked_add(offset - seg.start)?,
            Precision::Range => self.remap_range(seg, offset, world_source, source)?,
            Precision::Context => seg.source_start?,
            Precision::Generated | Precision::Unmapped => return None,
        };
        if source_offset > source.len() || !source.is_char_boundary(source_offset) {
            return None;
        }
        let before = &source[..source_offset];
        let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        Some(Origin {
            module,
            node: seg.node.clone(),
            event: seg.event.clone(),
            helper: seg.helper.clone(),
            precision: seg.precision,
            source_offset,
            line,
            column,
        })
    }

    /// range 段回算：重放重命名差分（只接受 `{module}_` 纯插入）。
    /// 先证条件：片段体与重放结果逐字节相等。
    fn remap_range(
        &self,
        seg: &Segment,
        offset: usize,
        world_source: &str,
        module_source: &str,
    ) -> Option<usize> {
        let helper = seg.helper.as_deref()?;
        let module = seg.module.as_deref()?;
        let marker = format!("// pooled fn {helper} from {module}\n");
        let body_start = seg.start.checked_add(marker.len())?;
        let source_start = seg.source_start?;
        let original =
            slice_top_fn(module_source, helper)?.trim_end();
        let siblings = module_fn_names(module_source);
        let renamed = replay_module_rename(original, &siblings, module);
        let frag = world_source.get(body_start..body_start + renamed.len())?;
        if frag != renamed {
            return None;
        }
        let ins = format!("{module}_").into_bytes();
        let ob = original.as_bytes();
        let rb = renamed.as_bytes();
        let mut oi = 0usize;
        let mut ri = 0usize;
        let mut insertions: Vec<(usize, usize)> = Vec::new();
        while oi < ob.len() && ri < rb.len() {
            if ob[oi] == rb[ri] {
                oi += 1;
                ri += 1;
                continue;
            }
            if rb[ri..].starts_with(&ins) {
                insertions.push((ri, oi));
                ri += ins.len();
                continue;
            }
            return None;
        }
        if ri < rb.len() {
            return None;
        }
        let rel = offset.checked_sub(body_start)?;
        let mut shift = 0usize;
        for &(gpos, opos) in &insertions {
            if (gpos..gpos + ins.len()).contains(&rel) {
                return source_start.checked_add(opos);
            }
            if rel >= gpos + ins.len() {
                shift += ins.len();
            } else {
                break;
            }
        }
        source_start.checked_add(rel.checked_sub(shift)?)
    }
}

/// 模块源内顶层 `fn {name}(` 的切片（花括号配平；与 pilot
/// lower::slice_fn_source 同规则）。
fn slice_top_fn<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("fn {name}(");
    let start = source.find(&marker)?;
    let open = source[start..].find('{')? + start;
    let mut depth = 0usize;
    for (i, c) in source[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(&source[start..open + i + 1]);
                }
            }
            _ => {}
        }
    }
    None
}
