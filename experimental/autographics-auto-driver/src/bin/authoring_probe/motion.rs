use autographics_auto_driver::authoring::{hierarchy, host::Host};
use serde_json::{json, Value};
pub fn trace(h: &Host, space: i32, short: bool, records: &mut Vec<Value>) -> Result<(), String> {
    let p = h.play.as_ref().ok_or("motion world")?;
    let frame = p.last.as_ref().ok_or("motion frame")?;
    let d = &h.editor.document;
    let actor = &d.objects[0];
    let (ml, mh) = super::native_spatial::Probe::offsets(h, actor)?;
    let order = hierarchy::ordered(d)?;
    let mut parent = [0.; 3];
    if let Some(id) = actor.parent {
        parent = d
            .groups
            .iter()
            .find(|g| g.id == id)
            .ok_or("motion parent")?
            .translation;
    }
    let mut local = actor.translation;
    let mut origin: [f32; 3] = std::array::from_fn(|a| local[a] + parent[a]);
    let speed = if space == 2 {
        if short {
            4.
        } else {
            8.
        }
    } else if short {
        0.1
    } else {
        0.2
    };
    let clearance = if space == 2 {
        if short {
            0.04
        } else {
            0.02
        }
    } else if short {
        0.004
    } else {
        0.002
    };
    let tangent = if space == 2 { 1 } else { 2 };
    let mut events = vec![];
    let mut last = None;
    for r in &p.reports {
        let mut delta = [0.; 3];
        if r.space == space {
            delta[0] = r.x as f32 * speed;
            delta[tangent] = (r.y as f32 + r.x as f32 * 0.4) * speed;
        }
        let start = origin;
        let start_local = local;
        let mut requested_advance = false;
        let mut remaining = delta;
        let mut status = 0;
        let mut contacts = 0;
        let mut node = -1;
        let mut normal = [0.; 3];
        for _ in 0..2 {
            let mut best: Option<(f32, i32, [f32; 3])> = None;
            for id in [2, if space == 2 { 6 } else { 5 }] {
                let target = d
                    .objects
                    .iter()
                    .find(|o| o.id == id)
                    .ok_or("motion target")?;
                let (lo, hi) = super::native_spatial::Probe::offsets(h, target)?;
                let l: [f32; 3] = std::array::from_fn(|a| (target.translation[a] + lo[a]) - mh[a]);
                let u: [f32; 3] = std::array::from_fn(|a| (target.translation[a] + hi[a]) - ml[a]);
                let ordinal = order.iter().position(|o| o.id == id).unwrap() as i32;
                if (0..space as usize).all(|a| origin[a] > l[a] && origin[a] < u[a]) {
                    status = 3;
                    node = ordinal;
                    break;
                }
                if (0..space as usize).all(|a| origin[a] >= l[a] && origin[a] <= u[a])
                    && (0..space as usize).any(|a| {
                        origin[a] == l[a] && remaining[a] <= 0.
                            || origin[a] == u[a] && remaining[a] >= 0.
                    })
                {
                    continue;
                }
                let mut enter = 0f32;
                let mut exit = 1f32;
                let mut n = [0.; 3];
                let mut valid = true;
                for a in 0..space as usize {
                    let v = remaining[a];
                    if v == 0. {
                        if origin[a] < l[a] || origin[a] > u[a] {
                            valid = false;
                            break;
                        };
                        continue;
                    }
                    let (near, far, sign) = if v > 0. {
                        ((l[a] - origin[a]) / v, (u[a] - origin[a]) / v, -1.)
                    } else {
                        ((u[a] - origin[a]) / v, (l[a] - origin[a]) / v, 1.)
                    };
                    if near > enter || near == enter && n == [0.; 3] {
                        enter = near;
                        n = [0.; 3];
                        n[a] = sign;
                    }
                    exit = exit.min(far);
                    if enter > exit {
                        valid = false;
                        break;
                    }
                }
                if valid
                    && best
                        .as_ref()
                        .is_none_or(|(t, o, _)| enter < *t || enter == *t && ordinal < *o)
                {
                    best = Some((enter, ordinal, n));
                }
            }
            if status == 3 {
                break;
            }
            let safe = if let Some((t, id, n)) = best {
                contacts += 1;
                status = if contacts == 2 { 2 } else { 1 };
                node = id;
                normal = n;
                let axis = n
                    .iter()
                    .position(|x| *x != 0.)
                    .ok_or("reference entering face")?;
                let speed = remaining[axis].abs();
                if clearance >= t * speed {
                    0.
                } else {
                    t - clearance / speed
                }
            } else {
                1.
            };
            for a in 0..3 {
                if remaining[a] * safe != 0. {
                    requested_advance = true;
                }
                local[a] += remaining[a] * safe;
                origin[a] = local[a] + parent[a];
                remaining[a] *= 1. - safe;
                if normal[a] != 0. {
                    remaining[a] = 0.;
                }
            }
            if best.is_none() || remaining == [0.; 3] {
                break;
            }
        }
        if status != 3 && requested_advance && origin == start {
            status = 4;
            contacts = 0;
            node = -1;
            normal = [0.; 3];
            origin = start;
            local = start_local;
        }
        if delta != [0.; 3] {
            last = Some((node, status, contacts, normal, origin));
        }
        events.push(json!({"tick":r.tick,"delta":delta,"start":start,"world":origin,"status":status,"contacts":contacts,"node":node,"normal":normal}));
    }
    for a in 0..3 {
        if (frame.objects[0].translation[a] - origin[a]).abs() > 0.002 {
            return Err(format!(
                "motion world axis{a}: expected {} actual {}",
                origin[a], frame.objects[0].translation[a]
            ));
        }
    }
    if let Some((node, status, contacts, n, q)) = last {
        let marker = frame
            .objects
            .iter()
            .find(|o| o.id == if space == 2 { 3 } else { 4 })
            .ok_or("motion marker")?;
        let point = frame
            .objects
            .iter()
            .find(|o| o.id == if space == 2 { 5 } else { 6 })
            .ok_or("motion point")?;
        if !marker.visible
            || (marker.translation[0] - (-100. + node as f32 * 5.)).abs() > 0.002
            || (marker.translation[1] - (50. + status as f32 * 10.)).abs() > 0.002
            || (point.translation[0] - (-150. + q[0])).abs() > 0.002
            || (point.translation[1] - (100. + n[0] * 5. + n[tangent])).abs() > 0.002
        {
            return Err(format!(
                "motion status/node/normal snapshot {node}/{status}/{contacts}/{n:?}/{q:?}"
            ));
        }
    }
    records.push(json!({"case":"motion-trace","tick":p.tick(),"space":space,"expected":origin,"events":events,"api":p.user_source.as_ref().map(|s|&s.version),"source":p.user_source.as_ref().map(|s|&s.user_hash)}));
    Ok(())
}
