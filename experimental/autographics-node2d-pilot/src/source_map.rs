//! PLAN-044：Node 生成链可选源映射（旁件 `source-map.json`）。
//!
//! 只解释已有诊断：映射把 SDK compile error 的真实 span（经 driver 受检
//! assembly user region 换算出的 user offset）映回原模块/Node/事件/helper。
//! 不插桩、不改变 world/API 源字节；默认 CLI/旧档无旁件。
//!
//! 精度语义（T-02 冻结）：
//! - `exact`：生成片段与原作者源字节一致，可字节级回算行列；
//! - `range`：身份/行号可证（verbatim 片段经模块调用重命名等行内重写），
//!   列可能因行内改写偏移，只给源 byte 区间；
//! - `context`：只有 Node/事件/helper 上下文可证，不给位置；
//! - `generated`：合成代码（SoA/调度/派发），永不映到作者行；
//! - `unmapped`：映射缺失或被拒后的显式回落。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::load::Project;

/// 旁件格式版本（schema/字段/区间口径 T-02 冻结）。
pub const MAP_FORMAT: &str = "node-source-map/v1";

/// 定位精度（freeze 语义见模块文档）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Precision {
    Exact,
    Range,
    Context,
    Generated,
    Unmapped,
}

/// 生成用户源中的一个映射段：半开 byte 区间 [start, end)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub start: usize,
    pub end: usize,
    /// 原模块闭包 key（file stem，相对、无盘符/无 `..`）。
    pub module: Option<String>,
    /// 定义级 Node 名（可证时）。
    pub node: Option<String>,
    /// 事件/handler 名（可证时）。
    pub event: Option<String>,
    /// helper 纯函数名（可证时）。
    pub helper: Option<String>,
    pub precision: Precision,
    /// 原模块源内半开 byte 区间（仅 module 段携带）。
    pub source_start: Option<usize>,
    pub source_end: Option<usize>,
}

/// 闭包模块身份（与 pilot identity.json 的 module:sha 口径一致）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleEntry {
    pub sha256: String,
    pub bytes: usize,
}

/// 旁件顶层结构。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceMap {
    pub format: String,
    /// 生成器身份串（与 identity.json generator 同值）。
    pub generator: String,
    /// world.at 内容 SHA256（与 identity.json world_sha256 同值）。
    pub world_sha256: String,
    pub user_source_sha256: String,
    pub user_source_bytes: usize,
    pub modules: BTreeMap<String, ModuleEntry>,
    pub segments: Vec<Segment>,
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

impl SourceMap {
    /// 结构校验：拒坏 hash/错格式/逃逸 key/越界/重叠/非字符边界/精度与
    /// 字段不一致。任何一项失败即整图拒用（V-S03：明确 fallback，不部分采用）。
    pub fn validate(&self, user_source: &str) -> Result<(), String> {
        if self.format != MAP_FORMAT {
            return Err(format!("map format '{}' != {}", self.format, MAP_FORMAT));
        }
        if !valid_sha256(&self.world_sha256) {
            return Err("bad world_sha256".into());
        }
        if !valid_sha256(&self.user_source_sha256) {
            return Err("bad user_source_sha256".into());
        }
        if self.user_source_bytes != user_source.len() {
            return Err(format!(
                "map user_source_bytes {} != actual {}",
                self.user_source_bytes,
                user_source.len()
            ));
        }
        if crate::scene::sha256_hex(user_source.as_bytes()) != self.user_source_sha256 {
            return Err("map user_source_sha256 mismatch".into());
        }
        for (key, entry) in &self.modules {
            if !valid_module_key(key) {
                return Err(format!("invalid module key '{key}'"));
            }
            if !valid_sha256(&entry.sha256) {
                return Err(format!("bad sha256 for module '{key}'"));
            }
        }
        let mut prev_end = 0usize;
        for seg in &self.segments {
            if seg.start > seg.end || seg.end > user_source.len() {
                return Err(format!("segment {}..{} out of bounds", seg.start, seg.end));
            }
            if seg.start < prev_end {
                return Err(format!(
                    "segments overlap or unordered at {}..{} (prev_end {prev_end})",
                    seg.start, seg.end
                ));
            }
            if !user_source.is_char_boundary(seg.start) || !user_source.is_char_boundary(seg.end)
            {
                return Err(format!("segment {}..{} not on char boundary", seg.start, seg.end));
            }
            match seg.precision {
                Precision::Exact | Precision::Range | Precision::Context => {
                    let key = seg.module.as_deref().ok_or("mapped segment missing module")?;
                    if !self.modules.contains_key(key) {
                        return Err(format!("segment module '{key}' not in closure"));
                    }
                    if seg.precision != Precision::Context {
                        let (ss, se) = (
                            seg.source_start.ok_or("mapped segment missing source_start")?,
                            seg.source_end.ok_or("mapped segment missing source_end")?,
                        );
                        if ss > se {
                            return Err("source range inverted".into());
                        }
                        let module_bytes = self.modules[key].bytes;
                        if se > module_bytes {
                            return Err(format!(
                                "source range {ss}..{se} exceeds module '{key}' bytes {module_bytes}"
                            ));
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
        Ok(())
    }

    /// 二分查找覆盖 offset 的段（半开区间）；未覆盖返回 None。
    pub fn lookup(&self, offset: usize) -> Option<&Segment> {
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

    /// driver 载入侧绑定校验：旁件声明的 world/user 身份必须与实际载入的
    /// 字节一致（错 world/旧 Run/磁盘源被改 → 整图拒用，明确 fallback）。
    pub fn validate_against_world(&self, world_source: &str) -> Result<(), String> {
        if crate::scene::sha256_hex(world_source.as_bytes()) != self.world_sha256 {
            return Err("map world_sha256 does not match loaded world bytes".into());
        }
        if world_source.len() != self.user_source_bytes {
            return Err("map user_source_bytes does not match loaded world bytes".into());
        }
        self.validate(world_source)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("source map serialize")
    }
}

/// 模块源内 `fn {name}(` 起，按花括号配平找函数结束（含闭合 `}`）。
fn fn_end(source: &str, name: &str) -> Option<usize> {
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
                    return Some(open + i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// PLAN-044 T-02 冻结：pooled helper 后置锚定定位。
///
/// components-2d 发射器为每个 pooled helper 前置唯一锚定注释
/// `// pooled fn {name} from {module}`（lower_components.rs）。本函数按
/// 相同排序（module, fn）从游标顺序定位，片段终点按花括号配平。
/// 片段字节与 `lower::slice_fn_source` 原文一致 → `exact`；经模块调用
/// 重命名等行内改写 → `range`（行号可证，列不承诺）。
pub fn locate_pooled_fns(project: &Project, user_source: &str) -> Result<Vec<Segment>, String> {
    let mut fn_names: Vec<(String, String)> = Vec::new();
    for module in project.modules.values() {
        for (fname, _) in &module.fns {
            fn_names.push((module.name.clone(), fname.clone()));
        }
    }
    fn_names.sort();
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    for (module_name, fname) in &fn_names {
        let module = &project.modules[module_name];
        let marker = format!("// pooled fn {fname} from {module_name}\n");
        let rel = user_source[cursor..]
            .find(&marker)
            .ok_or_else(|| format!("pooled fn marker not found: {marker:?}"))?;
        let start = cursor + rel;
        let body_start = start + marker.len();
        let end = fn_end(user_source, fname)
            .filter(|e| *e > body_start && *e <= user_source.len())
            .ok_or_else(|| format!("pooled fn end not found: {fname}"))?;
        let body = &user_source[body_start..end];
        let verbatim = crate::lower::slice_fn_source(&module.source, fname)
            .ok_or_else(|| format!("fn source slice: {module_name}/{fname}"))?;
        let body_trimmed = body.trim_end();
        let verbatim_trimmed = verbatim.trim_end();
        let precision = if body_trimmed == verbatim_trimmed {
            Precision::Exact
        } else {
            Precision::Range
        };
        let source_range = module
            .fns
            .iter()
            .find(|(n, _)| n == fname)
            .map(|(_, r)| (r.start, r.end));
        let (source_start, source_end) =
            source_range.ok_or_else(|| format!("fn span missing: {module_name}/{fname}"))?;
        segments.push(Segment {
            start,
            end,
            module: Some(module_name.clone()),
            node: None,
            event: None,
            helper: Some(fname.clone()),
            precision,
            source_start: Some(source_start),
            source_end: Some(source_end),
        });
        cursor = end;
    }
    Ok(segments)
}

/// 用 generated 段补齐未被映射段覆盖的空隙（相邻空隙合并为一个段）。
pub fn fill_generated_gaps(mut segments: Vec<Segment>, user_source_len: usize) -> Vec<Segment> {
    segments.sort_by_key(|s| s.start);
    let mut out: Vec<Segment> = Vec::new();
    let mut cursor = 0usize;
    for seg in segments {
        if seg.start > cursor {
            out.push(Segment {
                start: cursor,
                end: seg.start,
                module: None,
                node: None,
                event: None,
                helper: None,
                precision: Precision::Generated,
                source_start: None,
                source_end: None,
            });
        }
        cursor = cursor.max(seg.end);
        out.push(seg);
    }
    if cursor < user_source_len {
        out.push(Segment {
            start: cursor,
            end: user_source_len,
            module: None,
            node: None,
            event: None,
            helper: None,
            precision: Precision::Generated,
            source_start: None,
            source_end: None,
        });
    }
    out
}

/// 组装完整旁件：闭包身份 + pooled helper 段 + generated 补齐。
pub fn build_source_map(
    project: &Project,
    user_source: &str,
    generator: &str,
    world_sha256: &str,
) -> Result<SourceMap, String> {
    let mut modules = BTreeMap::new();
    for (name, module) in &project.modules {
        modules.insert(
            name.clone(),
            ModuleEntry {
                sha256: crate::scene::sha256_hex(module.source.as_bytes()),
                bytes: module.source.len(),
            },
        );
    }
    let segments = fill_generated_gaps(locate_pooled_fns(project, user_source)?, user_source.len());
    let map = SourceMap {
        format: MAP_FORMAT.into(),
        generator: generator.into(),
        world_sha256: world_sha256.into(),
        user_source_sha256: crate::scene::sha256_hex(user_source.as_bytes()),
        user_source_bytes: user_source.len(),
        modules,
        segments,
    };
    map.validate(user_source)?;
    Ok(map)
}
