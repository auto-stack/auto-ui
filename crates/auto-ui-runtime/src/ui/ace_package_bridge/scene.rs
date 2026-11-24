//! PLAN-118（T-03）：包面画布场景解析 —— 五平行文本列表（`<scene>_pts/_meta/
//! _nodes/_edges/_labels`）→ `CanvasScene`。
//!
//! 与 VM 轨（`aura_view_builder` 的 Plan 563/661 解析）同一契约与宽容策略，
//! 但输入是包面 live 会话读回的文本元素（`&[String]`），不引入 VM `Value` 依赖。

use crate::ui::view::{CanvasEdge, CanvasLabel, CanvasNode, CanvasScene, CanvasStroke};

fn split_item(item: &str) -> Option<Vec<&str>> {
    if item.is_empty() {
        None
    } else {
        Some(item.split(',').map(str::trim).collect())
    }
}

fn parse_strokes(pts: &[String], meta: &[String]) -> Vec<CanvasStroke> {
    let mut strokes = Vec::with_capacity(pts.len());
    for (i, item) in pts.iter().enumerate() {
        if item.is_empty() {
            continue;
        }
        let points: Vec<(f32, f32)> = item
            .split('|')
            .filter_map(|pair| {
                let (x, y) = pair.split_once(',')?;
                Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
            })
            .collect();
        if points.is_empty() {
            continue;
        }
        let meta_s = meta.get(i).map(String::as_str).unwrap_or_default();
        let mut parts = meta_s.split(',');
        let color = parts
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "#111827".to_string());
        let width = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .filter(|w| *w > 0.0)
            .unwrap_or(3.0);
        let eraser = parts
            .next()
            .map(|s| s.trim() == "1" || s.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        strokes.push(CanvasStroke {
            points,
            color,
            width,
            eraser,
        });
    }
    strokes
}

fn parse_nodes(items: &[String]) -> Vec<CanvasNode> {
    let mut nodes = Vec::with_capacity(items.len());
    for item in items {
        let Some(parts) = split_item(item) else { continue };
        if parts.len() < 5 || parts[0].is_empty() {
            continue;
        }
        let (Ok(x), Ok(y)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>()) else {
            continue;
        };
        let shape = parts[3].to_ascii_lowercase();
        let shape = if shape == "rect" { "rect" } else { "circle" }.to_string();
        let color = if parts[4].is_empty() {
            "#3b82f6"
        } else {
            parts[4]
        }
        .to_string();
        let (r, w, h) = if shape == "rect" {
            let w = parts
                .get(5)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(40.0);
            let h = parts
                .get(6)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(w);
            (None, Some(w), Some(h))
        } else {
            (
                Some(
                    parts
                        .get(5)
                        .and_then(|s| s.parse::<f32>().ok())
                        .unwrap_or(16.0),
                ),
                None,
                None,
            )
        };
        nodes.push(CanvasNode {
            id: parts[0].to_string(),
            x,
            y,
            shape,
            color,
            r,
            w,
            h,
        });
    }
    nodes
}

fn parse_edges(items: &[String]) -> Vec<CanvasEdge> {
    let mut edges = Vec::with_capacity(items.len());
    for item in items {
        let Some(parts) = split_item(item) else { continue };
        if parts.len() < 4 {
            continue;
        }
        let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
            parts[0].parse::<f32>(),
            parts[1].parse::<f32>(),
            parts[2].parse::<f32>(),
            parts[3].parse::<f32>(),
        ) else {
            continue;
        };
        let color = parts
            .get(4)
            .filter(|s| !s.is_empty())
            .unwrap_or(&"#9ca3af")
            .to_string();
        let width = parts
            .get(5)
            .and_then(|s| s.parse::<f32>().ok())
            .filter(|w| *w > 0.0)
            .unwrap_or(2.0);
        edges.push(CanvasEdge {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
        });
    }
    edges
}

fn parse_labels(items: &[String]) -> Vec<CanvasLabel> {
    let mut labels = Vec::with_capacity(items.len());
    for item in items {
        if item.is_empty() {
            continue;
        }
        let parts: Vec<&str> = item.splitn(3, ',').map(str::trim).collect();
        if parts.len() < 3 {
            continue;
        }
        let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) else {
            continue;
        };
        // 文本段可能自带逗号：恰 3 段 = 无 color/size；否则尾段再拆。
        let (text, color, size) = if parts.len() == 3 {
            let tail = parts[2];
            match tail.splitn(3, ',').map(str::trim).collect::<Vec<_>>().as_slice() {
                [text, color, size] => (
                    (*text).to_string(),
                    if color.is_empty() { "#111827".to_string() } else { (*color).to_string() },
                    size.parse::<f32>().ok().filter(|s| *s > 0.0).unwrap_or(12.0),
                ),
                _ => (tail.to_string(), "#111827".to_string(), 12.0),
            }
        } else {
            (parts[2].to_string(), "#111827".to_string(), 12.0)
        };
        labels.push(CanvasLabel {
            x,
            y,
            text,
            color,
            size,
        });
    }
    labels
}

/// 五平行列表 → 场景（缺表 = 空；畸形项宽容跳过，与 VM 轨同律）。
pub(super) fn parse_scene(
    pts: &[String],
    meta: &[String],
    nodes: &[String],
    edges: &[String],
    labels: &[String],
) -> CanvasScene {
    CanvasScene {
        strokes: parse_strokes(pts, meta),
        nodes: parse_nodes(nodes),
        edges: parse_edges(edges),
        labels: parse_labels(labels),
    }
}
