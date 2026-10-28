//! Independent sample oracle, sharing only the pre-existing all-mesh reference.
use autographics_auto_driver::authoring::{host::Host, play::DT};
use serde_json::{json, Value};
pub fn trace(h: &Host, space: i32, short: bool, records: &mut Vec<Value>) -> Result<(), String> {
    let p = h.play.as_ref().ok_or("segment missing world")?;
    let frame = p.last.as_ref().ok_or("segment missing frame")?;
    let d = &h.editor.document;
    let accepted = &d.objects[0];
    let mut origin = accepted.translation;
    if let Some(parent) = accepted.parent {
        let g = d
            .groups
            .iter()
            .find(|g| g.id == parent)
            .ok_or("segment parent")?;
        for k in 0..3 {
            origin[k] += g.translation[k];
        }
    }
    for r in &p.reports {
        if r.space == space {
            let speed = if space == 2 { 60. } else { 3. };
            origin[0] += r.x as f32 * speed * DT;
            if space == 2 {
                origin[1] += r.y as f32 * speed * DT;
            } else {
                origin[2] += 0. - r.y as f32 * speed * DT;
            }
        }
    }
    let actual = &frame.objects[0];
    for k in 0..3 {
        if (actual.translation[k] - origin[k]).abs() > 1e-3 {
            return Err("segment actual OS origin trace".into());
        }
    }
    let reach = if space == 2 {
        if short {
            10.
        } else {
            20.
        }
    } else if short {
        2.
    } else {
        4.
    };
    let end = [origin[0] + reach, origin[1], origin[2]];
    let mut best: Option<(usize, f32, [f32; 3], [f32; 3])> = None;
    for (id, ordinal) in if space == 2 {
        [(2, 1), (6, 5)]
    } else {
        [(2, 2), (5, 5)]
    } {
        let o = d
            .objects
            .iter()
            .find(|o| o.id == id)
            .ok_or("segment reference target")?;
        let (offset_min, offset_max) = super::native_spatial::Probe::offsets(h, o)?;
        let mut position = o.translation;
        if space == 3 && id == 2 && p.tick() > 0 {
            position[0] = 0.5;
        }
        let lo = std::array::from_fn::<_, 3, _>(|k| position[k] + offset_min[k]);
        let hi = std::array::from_fn::<_, 3, _>(|k| position[k] + offset_max[k]);
        let mut enter = 0.0f32;
        let mut leave = 1.0f32;
        let mut normal = [0.; 3];
        let mut inside = true;
        for k in 0..space as usize {
            if origin[k] < lo[k] || origin[k] > hi[k] {
                inside = false;
            }
            let direction = end[k] - origin[k];
            if direction == 0. {
                if origin[k] < lo[k] || origin[k] > hi[k] {
                    leave = -1.;
                    break;
                }
            } else {
                let a = (lo[k] - origin[k]) / direction;
                let b = (hi[k] - origin[k]) / direction;
                let near = a.min(b);
                let far = a.max(b);
                if near > enter {
                    enter = near;
                    normal = [0.; 3];
                    normal[k] = if direction > 0. { -1. } else { 1. };
                }
                leave = leave.min(far);
            }
        }
        if enter <= leave
            && best
                .as_ref()
                .is_none_or(|(n, t, _, _)| enter < *t || (enter == *t && ordinal < *n))
        {
            if inside {
                normal = [0.; 3];
            }
            let point = std::array::from_fn(|k| origin[k] + enter * (end[k] - origin[k]));
            best = Some((ordinal, enter, point, normal));
        }
    }
    let (node, t, point, normal, hit) = match best {
        Some((n, t, p, v)) => (n as i32, t, p, v, true),
        None => (-1, 1., end, [0.; 3], false),
    };
    if p.tick() > 0 {
        let marker = frame
            .objects
            .iter()
            .find(|o| o.id == if space == 2 { 3 } else { 4 })
            .ok_or("segment marker")?;
        let point_marker = frame
            .objects
            .iter()
            .find(|o| o.id == if space == 2 { 5 } else { 6 })
            .ok_or("segment point marker")?;
        let door = frame
            .objects
            .iter()
            .find(|o| o.id == if space == 2 { 4 } else { 3 })
            .ok_or("segment door")?;
        if !marker.visible
            || door.visible != hit
            || (marker.translation[0] - (-100. + node as f32 * 5.)).abs() > 1e-3
            || (marker.translation[1] - (50. + t * 10.)).abs() > 1e-3
            || (point_marker.translation[0] - (-150. + point[0])).abs() > 1e-3
            || (point_marker.translation[1] - (100. + normal[0] * 5.)).abs() > 1e-3
        {
            return Err(format!("segment nearest/snapshot/point/normal feedback expected node={node} t={t} point={point:?} normal={normal:?}"));
        }
    }
    records.push(json!({"case":"segment-trace","tick":p.tick(),"space":space,"short":short,"expected":{"node":node,"t":t,"point":point,"normal":normal,"hit":hit},"api":p.user_source.as_ref().map(|s|&s.version),"source":p.user_source.as_ref().map(|s|&s.user_hash),"actions":p.reports}));
    Ok(())
}
