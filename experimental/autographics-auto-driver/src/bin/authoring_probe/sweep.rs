//! Independent OS-action replay. Geometry comes from frozen actual all-part assets.
use autographics_auto_driver::authoring::{hierarchy, host::Host};
use serde_json::{json, Value};
pub fn trace(h: &Host, space: i32, short: bool, records: &mut Vec<Value>) -> Result<(), String> {
    let p = h.play.as_ref().ok_or("sweep missing world")?;
    let frame = p.last.as_ref().ok_or("sweep missing frame")?;
    let d = &h.editor.document;
    let actor = &d.objects[0];
    let (mlo, mhi) = super::native_spatial::Probe::offsets(h, actor)?;
    let mut origin = actor.translation;
    if let Some(id) = actor.parent {
        let group = d.groups.iter().find(|g| g.id == id).ok_or("sweep parent")?;
        for k in 0..3 {
            origin[k] += group.translation[k];
        }
    }
    let order = hierarchy::ordered(d)?;
    let speed = if space == 2 {
        if short {
            50.
        } else {
            100.
        }
    } else if short {
        5.
    } else {
        10.
    };
    let clearance = if space == 2 {
        if short {
            0.04
        } else {
            0.02
        }
    } else {
        if short {
            0.004
        } else {
            0.002
        }
    };
    let mut last = (-1, 1., origin, [0.; 3], false, false);
    let mut events = vec![];
    let mut any_input = false;
    for r in &p.reports {
        let dx = if r.space == space {
            r.x as f32 * speed
        } else {
            0.
        };
        let mut best: Option<(i32, f32, [f32; 3], bool)> = None;
        for id in if space == 2 { [2, 6] } else { [2, 5] } {
            let target = d
                .objects
                .iter()
                .find(|o| o.id == id)
                .ok_or("sweep target")?;
            let (lo, hi) = super::native_spatial::Probe::offsets(h, target)?;
            let l: [f32; 3] = std::array::from_fn(|k| (target.translation[k] + lo[k]) - mhi[k]);
            let u: [f32; 3] = std::array::from_fn(|k| (target.translation[k] + hi[k]) - mlo[k]);
            if (1..space as usize).any(|k| origin[k] < l[k] || origin[k] > u[k]) {
                continue;
            }
            let inside = origin[0] >= l[0] && origin[0] <= u[0];
            let (t, normal) = if inside {
                (0., [0.; 3])
            } else if dx == 0. {
                continue;
            } else {
                let near = if dx > 0. {
                    (l[0] - origin[0]) / dx
                } else {
                    (u[0] - origin[0]) / dx
                };
                let far = if dx > 0. {
                    (u[0] - origin[0]) / dx
                } else {
                    (l[0] - origin[0]) / dx
                };
                if near > 1. || far < 0. {
                    continue;
                }
                (near.max(0.), [if dx > 0. { -1. } else { 1. }, 0., 0.])
            };
            let ordinal = order
                .iter()
                .position(|n| n.id == id)
                .ok_or("sweep ordinal")? as i32;
            if best
                .as_ref()
                .is_none_or(|(n, f, _, _)| t < *f || t == *f && ordinal < *n)
            {
                best = Some((ordinal, t, normal, inside));
            }
        }
        let (node, t, normal, inside, hit) = best
            .map_or((-1, 1., [0.; 3], false, false), |(n, t, v, i)| {
                (n, t, v, i, true)
            });
        let mut query_origin = origin;
        query_origin[0] += t * dx;
        let movement = if hit {
            if inside {
                0.
            } else {
                (dx.abs() * t - clearance).max(0.) * dx.signum()
            }
        } else {
            dx
        };
        origin[0] += movement;
        if dx != 0. {
            any_input = true;
            last = (node, t, query_origin, normal, inside, hit);
        }
        events.push(json!({"tick":r.tick,"dx":dx,"expected_node":node,"expected_t":t,"query_origin":query_origin,"normal":normal,"start_overlap":inside,"movement":movement,"stopped_world":origin}));
    }
    for k in 0..3 {
        if (frame.objects[0].translation[k] - origin[k]).abs() > 1e-3 {
            return Err(format!(
                "sweep OS stop position axis{k} expected {} actual {}",
                origin[k], frame.objects[0].translation[k]
            ));
        }
    }
    let (node, t, q, n, inside, hit) = last;
    if p.tick() > 0 && any_input {
        let marker = frame
            .objects
            .iter()
            .find(|o| o.id == if space == 2 { 3 } else { 4 })
            .ok_or("sweep marker")?;
        let point = frame
            .objects
            .iter()
            .find(|o| o.id == if space == 2 { 5 } else { 6 })
            .ok_or("sweep point")?;
        let door = frame
            .objects
            .iter()
            .find(|o| o.id == if space == 2 { 4 } else { 3 })
            .ok_or("sweep door")?;
        if !marker.visible
            || door.visible != hit
            || (marker.translation[0] - (-100. + node as f32 * 5.)).abs() > 1e-3
            || (marker.translation[1] - (50. + t * 10.)).abs() > 1e-3
            || (point.translation[0] - (-150. + q[0])).abs() > 1e-3
            || (point.translation[1] - (100. + n[0] * 5.)).abs() > 1e-3
        {
            return Err(format!("sweep nearest/origin/normal/snapshot feedback node{node} t{t} origin{q:?} normal{n:?}"));
        }
    }
    records.push(json!({"case":"sweep-trace","tick":p.tick(),"space":space,"short":short,"expected":{"node":node,"t":t,"origin":q,"normal":n,"start_overlap":inside,"hit":hit,"stopped_world":origin},"events":events,"api":p.user_source.as_ref().map(|s|&s.version),"source":p.user_source.as_ref().map(|s|&s.user_hash),"actions":p.reports}));
    Ok(())
}
