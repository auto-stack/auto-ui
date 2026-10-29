//! One driver-local trusted assembler. No user input, IO, admission or caching.
use super::{
    Api, API, CAMERA2_API, CAMERA3_API, CHARACTER3_API, CHARACTER_API, MOTION_API, MOTION_SCAN_API,
    OBJECTS_API, SEGMENT_API, SOURCE, SPATIAL_API, SPRITES_API, SWEEP_API, TRIGGERS_API,
};

pub(super) struct Units {
    pub transport: String,
    pub api: String,
    pub entry: String,
    pub api_region: &'static str,
}
#[derive(Clone, Copy)]
struct Inputs<'a> {
    transport: &'a str,
    base: &'a str,
    spatial: &'a str,
    segment: &'a str,
    sweep: &'a str,
    motion_scan: &'a str,
    motion: &'a str,
    character: &'a str,
    character3: &'a str,
    camera2: &'a str,
    camera3: &'a str,
    objects: &'a str,
    triggers: &'a str,
    sprite: &'a str,
    entry: &'a str,
}
const INPUTS: Inputs<'static> = Inputs {
    transport: SOURCE,
    base: API,
    spatial: SPATIAL_API,
    segment: SEGMENT_API,
    sweep: SWEEP_API,
    motion_scan: MOTION_SCAN_API,
    motion: MOTION_API,
    character: CHARACTER_API,
    character3: CHARACTER3_API,
    camera2: CAMERA2_API,
    camera3: CAMERA3_API,
    objects: OBJECTS_API,
    triggers: TRIGGERS_API,
    sprite: SPRITES_API,
    entry: ENTRY,
};
pub(super) fn assemble(api: Api) -> Result<Units, String> {
    assemble_from(api, INPUTS)
}
fn assemble_from(api: Api, input: Inputs<'_>) -> Result<Units, String> {
    if api.audio() {
        let three = api == Api::Audio3V1;
        let mut units = assemble_from(
            if three {
                Api::NodeAnimation3V1
            } else {
                Api::Sprites2V1
            },
            input,
        )?;
        units.entry = replace(
            &units.entry,
            if three {
                "fn profile() int {return 21}"
            } else {
                "fn profile() int {return 20}"
            },
            if three {
                "fn profile() int {return 23}"
            } else {
                "fn profile() int {return 22}"
            },
            1,
        )?;
        units.entry = replace(
            &units.entry,
            if three {
                "fn _ag_version() int {return 110}"
            } else {
                "fn _ag_version() int {return 109}"
            },
            if three {
                "fn _ag_version() int {return 112}"
            } else {
                "fn _ag_version() int {return 111}"
            },
            1,
        )?;
        units.entry = replace(
            &units.entry,
            "app_start();",
            "_ag_audio_freeze(); app_start();",
            1,
        )?;
        units.entry = replace(
            &units.entry,
            "app_update(dt);",
            "_ag_audio_tick_begin(); app_update(dt);",
            1,
        )?;
        // Internal visual setters are reachable only inside this trusted unit.
        // Legacy profile units are never modified on disk or through this branch.
        let names: &[(&str, &str)] = if three {
            &[("ag_visual_pose", "_ag_audio_pose")]
        } else {
            &[
                ("ag_sprite_frame", "_ag_audio_sprite"),
                ("ag_sprite_slot", "_ag_audio_slot"),
            ]
        };
        for (public, internal) in names {
            units.api = replace(
                &units.api,
                &format!("fn {public}("),
                &format!("fn {internal}("),
                1,
            )?;
        }
        let visual_charge = if three {
            "fn _ag_audio_pose("
        } else {
            "fn _ag_spr_charge("
        };
        let start = units
            .api
            .find(visual_charge)
            .ok_or("audio visual boundary")?;
        let end = units.api[start + 3..]
            .find("\nfn ")
            .map_or(units.api.len(), |i| start + 3 + i);
        let body = replace(
            &units.api[start..end],
            "_ag_cast_charge(1)",
            "_ag_audio_visual_charge()",
            1,
        )?;
        units.api.replace_range(start..end, &body);
        let extension = if three {
            include_str!("../../../api/public_audio3d_v1.at")
        } else {
            include_str!("../../../api/public_audio2d_v1.at")
        };
        units.api = format!("{}\n{}\n{}", units.api, super::AUDIO_API, extension);
        units.api_region = if three {
            "<public_audio3d_v1.at>"
        } else {
            "<public_audio2d_v1.at>"
        };
        return Ok(units);
    }
    if api == Api::NodeAnimation3V1 {
        // Derive from the unchanged delivered triggers3d assembly. This branch
        // adds only a visual unit and hooks; every earlier profile keeps bytes.
        let mut units = assemble_from(Api::Triggers3V1, input)?;
        let visual = super::NODE_ANIMATION_API;
        require(visual, "fn ag_visual_pose(", 1)?;
        require(visual, "fn _ag_vis_freeze(", 1)?;
        units.api = format!("{}\n{}", units.api, visual);
        units.api_region = "<node_animation3d_v1.at>";
        units.entry = replace(
            &units.entry,
            "fn profile() int {return 19}",
            "fn profile() int {return 21}",
            1,
        )?;
        units.entry = replace(
            &units.entry,
            "fn _ag_version() int {return 108}",
            "fn _ag_version() int {return 110}",
            1,
        )?;
        units.entry = replace(
            &units.entry,
            "_ag_trg_freeze(); app_start();",
            "_ag_trg_freeze(); _ag_vis_freeze(); app_start();",
            1,
        )?;
        units.entry = replace(
            &units.entry,
            "_ag_trg_tick_begin(); app_update(dt);",
            "_ag_trg_tick_begin(); _ag_vis_tick_begin(); app_update(dt);",
            1,
        )?;
        return Ok(units);
    }
    require(input.base, "fn ag_node_count(", 1)?;
    if api != Api::V1 {
        require(input.spatial, "fn _ag_spatial_begin(", 1)?;
    }
    if matches!(
        api,
        Api::SegmentV3
            | Api::SweepV4
            | Api::MotionV5
            | Api::CharacterV1
            | Api::Character3V1
            | Api::Camera2V1
            | Api::Camera3V1
            | Api::Objects2V1
            | Api::Objects3V1
            | Api::Triggers2V1
            | Api::Triggers3V1
            | Api::Sprites2V1
    ) {
        require(input.segment, "fn ag_cast_segment(", 1)?;
    }
    if matches!(
        api,
        Api::SweepV4
            | Api::MotionV5
            | Api::CharacterV1
            | Api::Character3V1
            | Api::Camera2V1
            | Api::Camera3V1
            | Api::Objects2V1
            | Api::Objects3V1
            | Api::Triggers2V1
            | Api::Triggers3V1
            | Api::Sprites2V1
    ) {
        require(input.sweep, "fn ag_sweep_aabb(", 1)?;
    }
    if matches!(
        api,
        Api::MotionV5
            | Api::CharacterV1
            | Api::Character3V1
            | Api::Camera2V1
            | Api::Camera3V1
            | Api::Objects2V1
            | Api::Objects3V1
            | Api::Triggers2V1
            | Api::Triggers3V1
            | Api::Sprites2V1
    ) {
        require(input.motion_scan, "fn _ag_motion_two(", 1)?;
        require(input.motion_scan, "fn _ag_motion_three(", 1)?;
        require(input.motion, "fn ag_move_and_slide(", 1)?;
        require(input.motion, "fn _ag_motion_set_leaf(", 1)?;
    }
    if matches!(
        api,
        Api::CharacterV1 | Api::Camera2V1 | Api::Objects2V1 | Api::Triggers2V1 | Api::Sprites2V1
    ) {
        require(input.character, "fn ag_char_move(", 1)?;
        require(input.character, "fn _ag_char_freeze(", 1)?;
        require(input.character, "fn _ag_char_tick_begin(", 1)?;
        require(input.character, "fn _ag_char_input(", 1)?;
    }
    if matches!(
        api,
        Api::Character3V1 | Api::Camera3V1 | Api::Objects3V1 | Api::Triggers3V1
    ) {
        require(input.character3, "fn ag_char3_move(", 1)?;
        require(input.character3, "fn _ag_char3_freeze(", 1)?;
        require(input.character3, "fn _ag_char3_tick_begin(", 1)?;
        require(input.character3, "fn _ag_char3_input(", 1)?;
    }
    if matches!(
        api,
        Api::Camera2V1 | Api::Objects2V1 | Api::Triggers2V1 | Api::Sprites2V1
    ) {
        require(input.camera2, "fn ag_camera_follow_2d(", 1)?;
        require(input.camera2, "fn _ag_cam2_freeze(", 1)?;
        require(input.camera2, "fn _ag_cam2_input(", 0)?;
    }
    if matches!(api, Api::Camera3V1 | Api::Objects3V1 | Api::Triggers3V1) {
        require(input.camera3, "fn ag_camera_orbit_3d(", 1)?;
        require(input.camera3, "fn _ag_cam3_freeze(", 1)?;
        require(input.camera3, "fn _ag_cam3_basis(", 1)?;
        require(input.camera3, "fn _ag_cam3_orbit(", 1)?;
    }
    if matches!(
        api,
        Api::Objects2V1 | Api::Objects3V1 | Api::Triggers2V1 | Api::Triggers3V1 | Api::Sprites2V1
    ) {
        require(input.objects, "fn ag_object_spawn(", 1)?;
        require(input.objects, "fn _ag_obj_freeze(", 1)?;
        require(input.objects, "fn _ag_obj_tick_begin(", 1)?;
        require(input.objects, "var _ag_obj_dim=0", 1)?;
    }
    if matches!(api, Api::Triggers2V1 | Api::Triggers3V1 | Api::Sprites2V1) {
        require(input.triggers, "fn ag_trigger_bind(", 1)?;
        require(input.triggers, "fn ag_round_request(", 1)?;
        require(input.triggers, "fn _ag_trg_freeze(", 1)?;
        require(input.triggers, "fn _ag_trg_tick_begin(", 1)?;
        require(input.triggers, "fn _ag_trg_evaluate(", 1)?;
        require(input.triggers, "var _ag_trg_dim=0", 1)?;
    }
    // PLAN-004 sprite unit: only the explicit sprites2d profile composes it.
    if api.sprites() {
        require(input.sprite, "fn _ag_spr_freeze(", 1)?;
        require(input.sprite, "fn _ag_spr_tick_begin(", 1)?;
        require(input.sprite, "fn ag_sprite_frame(", 1)?;
        require(input.sprite, "fn ag_sprite_slot(", 1)?;
        require(input.sprite, "fn _ag_spr_pv(", 1)?;
        require(input.sprite, "fn _ag_spr_qv(", 1)?;
    }

    let transport = match api {
        Api::V1 | Api::SpatialV2 => bridge(input.transport)?,
        Api::SegmentV3
        | Api::SweepV4
        | Api::MotionV5
        | Api::CharacterV1
        | Api::Character3V1
        | Api::Camera2V1
        | Api::Camera3V1
        | Api::Objects2V1
        | Api::Objects3V1
        | Api::Triggers2V1
        | Api::Triggers3V1
        | Api::Sprites2V1
        | Api::NodeAnimation3V1
        | Api::Audio2V1
        | Api::Audio3V1 => segment_transport(input.transport)?,
    };
    // The former v5 pre-namespace `_ag_derive` replacement never matched.
    // Preserve those emitted bytes; activating that guard would change behavior.
    let (api_text, api_region) = match api {
        Api::NodeAnimation3V1 | Api::Audio2V1 | Api::Audio3V1 => unreachable!("derived above"),
        Api::V1 => (input.base.to_owned(), "<behavior_v1.at>"),
        Api::SpatialV2 => (
            format!("{}\n{}", input.base, input.spatial),
            "<behavior_spatial_v2.at>",
        ),
        Api::SegmentV3 => (query_api(input, false)?, "<behavior_segment_v3.at>"),
        Api::SweepV4 => (query_api(input, true)?, "<behavior_sweep_v4.at>"),
        Api::MotionV5 => (motion_api(input)?, "<behavior_motion_v5.at>"),
        Api::CharacterV1 => (
            segment_compact(&format!("{}\n{}", motion_api(input)?, input.character))?,
            "<character2d_v1.at>",
        ),
        Api::Character3V1 => (
            segment_compact(&format!("{}\n{}", motion_api(input)?, input.character3))?,
            "<character3d_v1.at>",
        ),
        Api::Camera2V1 => (
            segment_compact(&format!(
                "{}\n{}\n{}",
                motion_api(input)?,
                input.character,
                input.camera2
            ))?,
            "<runtime_camera2d_v1.at>",
        ),
        Api::Camera3V1 => (
            segment_compact(&format!(
                "{}\n{}\n{}",
                motion_api(input)?,
                input.character3,
                input.camera3
            ))?,
            "<runtime_camera3d_v1.at>",
        ),
        Api::Objects2V1 => (
            segment_compact(&format!(
                "{}\n{}\n{}\n{}",
                motion_api(input)?,
                input.character,
                input.camera2,
                objects_unit(input, 2)?
            ))?,
            "<runtime_objects_v1.at>",
        ),
        Api::Objects3V1 => (
            segment_compact(&format!(
                "{}\n{}\n{}\n{}",
                motion_api(input)?,
                input.character3,
                input.camera3,
                objects_unit(input, 3)?
            ))?,
            "<runtime_objects_v1.at>",
        ),
        Api::Triggers2V1 => (
            segment_compact(&format!(
                "{}\n{}\n{}\n{}\n{}",
                motion_api(input)?,
                locked_character(input.character, "fn ag_char_move(dx float, dy float, clearance float) int {\n if _ag_char_frozen==0 {_ag_fail(26,-1,0);return 0}\n")?,
                input.camera2,
                locked_objects(input, 2)?,
                triggers_unit(input, 2)?
            ))?,
            "<runtime_triggers_v1.at>",
        ),
        Api::Sprites2V1 => (
            segment_compact(&format!(
                "{}\n{}\n{}\n{}\n{}\n{}",
                motion_api(input)?,
                locked_character(input.character, "fn ag_char_move(dx float, dy float, clearance float) int {\n if _ag_char_frozen==0 {_ag_fail(26,-1,0);return 0}\n")?,
                input.camera2,
                locked_objects(input, 2)?,
                triggers_unit(input, 2)?,
                input.sprite
            ))?,
            "<runtime_sprites_v1.at>",
        ),
        Api::Triggers3V1 => (
            segment_compact(&format!(
                "{}\n{}\n{}\n{}\n{}",
                motion_api(input)?,
                locked_character(input.character3, "fn ag_char3_move(dx float, dy float, dz float, clearance float) int {\n if _ag_char3_frozen==0 {_ag_fail(26,-1,0);return 0}\n")?,
                input.camera3,
                locked_objects(input, 3)?,
                triggers_unit(input, 3)?
            ))?,
            "<runtime_triggers_v1.at>",
        ),
    };
    Ok(Units {
        transport,
        api: api_text,
        entry: entry(api, input.entry)?,
        api_region,
    })
}
/// The objects unit is one file for both dimensions; the assembly freezes the
/// Run dimension by replacing the declared literal (guarded by the require
/// marker above), so no user or host call can flip it at runtime.
fn objects_unit(input: Inputs<'_>, dim: i32) -> Result<String, String> {
    Ok(input
        .objects
        .replace("var _ag_obj_dim=0", &format!("var _ag_obj_dim={dim}")))
}
/// The post-evaluate geometry lock (PLAN-002 contract section 1): a derived
/// variant of the shared units, never a change to the delivered bytes. After
/// `_ag_trg_evaluate` froze this tick's snapshot, object spawn/move and the
/// character move are named rejections while despawn/visibility stay allowed.
const LOCK_GUARD: &str = " if _ag_obj_locked==1 {_ag_fail(40,-1,0);return 0}\n";
fn inject_lock(source: &str, marker: &str) -> Result<String, String> {
    let normalized = source.replace("\r\n", "\n");
    require(&normalized, marker, 1)?;
    Ok(normalized.replace(marker, &format!("{marker}{LOCK_GUARD}")))
}
fn locked_character(unit: &str, marker: &str) -> Result<String, String> {
    // The lock global is declared here because the character unit is the
    // earliest text in the combined unit that references it: the compiler
    // requires declaration before use across the managed source.
    let guarded = inject_lock(unit, marker)?;
    Ok(format!("var _ag_obj_locked=0\n{guarded}"))
}
fn locked_objects(input: Inputs<'_>, dim: i32) -> Result<String, String> {
    let move_marker = "fn ag_object_move(token int, x float, y float, z float) int {\n if _ag_error!=0 {return 0}\n if _ag_obj_frozen==0 {_ag_fail(38,-1,0);return 0}\n";
    let spawn_marker = "fn ag_object_spawn(slot int, template int, x float, y float, z float) int {\n if _ag_error!=0 {return 0}\n if _ag_obj_frozen==0 {_ag_fail(38,-1,0);return 0}\n";
    let guarded = inject_lock(input.objects, move_marker)?;
    let guarded = inject_lock(&guarded, spawn_marker)?;
    Ok(guarded.replace("var _ag_obj_dim=0", &format!("var _ag_obj_dim={dim}")))
}
fn triggers_unit(input: Inputs<'_>, dim: i32) -> Result<String, String> {
    Ok(input
        .triggers
        .replace("var _ag_trg_dim=0", &format!("var _ag_trg_dim={dim}")))
}
fn motion_api(input: Inputs<'_>) -> Result<String, String> {
    let base = query_api(input, true)?;
    let marker = segment_compact("fields[node*12+axis]=value; _ag_dirty=1")?;
    let replacement = segment_compact("if _ag_motion_set_leaf(node,axis,value)==1 {return};fields[node*12+axis]=value; _ag_dirty=1")?;
    let base = replace(&base, &marker, &replacement, 1)?;
    segment_compact(&format!("{base}\n{}\n{}", input.motion_scan, input.motion))
}
fn query_api(input: Inputs<'_>, sweep: bool) -> Result<String, String> {
    let base = segment_base_api(input.base)?;
    if !sweep {
        return segment_compact(&format!("{base}\n{}\n{}", input.spatial, input.segment));
    }
    let segment = format!("var _ag_query_kind=0\n{}", input.segment);
    let segment = replace(
        &segment,
        "_ag_cast_token=0",
        "_ag_cast_token=0; _ag_query_kind=1",
        3,
    )?;
    let segment = replace(
        &segment,
        "if token<=0",
        "if _ag_query_kind!=1 {_ag_fail(13,-1,0); return 0}; if token<=0",
        1,
    )?;
    segment_compact(&format!(
        "{base}\n{}\n{segment}\n{}",
        input.spatial, input.sweep
    ))
}
fn entry(api: Api, source: &str) -> Result<String, String> {
    // Check even v1 boundaries so a malformed trusted entry cannot slip through.
    for marker in [
        "fn main() {}",
        "fn profile() int {return 7}",
        "fn _ag_version() int {return 1}",
        "app_start();",
        "app_update(dt);",
    ] {
        require(source, marker, 1)?;
    }
    require(source, "; derive(); _ag_dirty=0", 2)?;
    if api == Api::V1 {
        return Ok(source.to_owned());
    }
    let source = replace(
        source,
        "fn profile() int {return 7}",
        &format!("fn profile() int {{return {}}}", api.profile()),
        1,
    )?;
    let source = replace(
        &source,
        "fn _ag_version() int {return 1}",
        &format!("fn _ag_version() int {{return {}}}", api.number()),
        1,
    )?;
    let source = replace(
        &source,
        "app_start();",
        "_ag_spatial_begin(); if _ag_error==0 { app_start() };",
        1,
    )?;
    let source = replace(
        &source,
        "app_update(dt);",
        "_ag_spatial_begin(); if _ag_error==0 { app_update(dt) };",
        1,
    )?;
    match api {
        Api::V1 | Api::SpatialV2 => Ok(source),
        Api::SegmentV3
        | Api::SweepV4
        | Api::MotionV5
        | Api::CharacterV1
        | Api::Character3V1
        | Api::Camera2V1
        | Api::Camera3V1
        | Api::Objects2V1
        | Api::Objects3V1
        | Api::Triggers2V1
        | Api::Triggers3V1
        | Api::Sprites2V1
        | Api::NodeAnimation3V1
        | Api::Audio2V1
        | Api::Audio3V1 => {
            let source = replace(
                &source,
                "_ag_spatial_begin();",
                "_ag_cast_token=0; _ag_spatial_begin();",
                2,
            )?;
            let source = replace(
                &source,
                "app_start() }",
                "app_start(); _ag_cast_token=0 }",
                1,
            )?;
            let source = replace(
                &source,
                "app_update(dt) }",
                "app_update(dt); _ag_cast_token=0 }",
                1,
            )?;
            let source = replace(
                &source,
                "; derive(); _ag_dirty=0",
                "; if _ag_dirty==1 {derive()}; _ag_dirty=0",
                2,
            )?;
            match api {
                Api::CharacterV1 => {
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char_freeze()",
                        1,
                    )?;
                    replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )
                }
                Api::Character3V1 => {
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char3_freeze()",
                        1,
                    )?;
                    replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char3_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )
                }
                Api::Camera2V1 => {
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char_freeze(); _ag_cam2_freeze()",
                        1,
                    )?;
                    replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )
                }
                Api::Camera3V1 => {
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char3_freeze(); _ag_cam3_freeze()",
                        1,
                    )?;
                    replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char3_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )
                }
                Api::Objects2V1 => {
                    // The table must exist before app_start can spawn into it,
                    // so the object freeze precedes the first callback; the
                    // camera/character freezes stay after it, as in 875.
                    let source = replace(
                        &source,
                        "if _ag_error==0 { app_start();",
                        "if _ag_error==0 { _ag_obj_freeze(); app_start();",
                        1,
                    )?;
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char_freeze(); _ag_cam2_freeze()",
                        1,
                    )?;
                    replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char_tick_begin(); _ag_obj_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )
                }
                Api::Objects3V1 => {
                    let source = replace(
                        &source,
                        "if _ag_error==0 { app_start();",
                        "if _ag_error==0 { _ag_obj_freeze(); app_start();",
                        1,
                    )?;
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char3_freeze(); _ag_cam3_freeze()",
                        1,
                    )?;
                    replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char3_tick_begin(); _ag_obj_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )
                }
                Api::Triggers2V1 => {
                    // Registry before binds, tick counters before the callback,
                    // and exactly one evaluate after the post-callback derive
                    // (the unique `toggled=0}` tail belongs to tick alone).
                    let source = replace(
                        &source,
                        "if _ag_error==0 { app_start();",
                        "if _ag_error==0 { _ag_obj_freeze(); _ag_trg_freeze(); app_start();",
                        1,
                    )?;
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char_freeze(); _ag_cam2_freeze()",
                        1,
                    )?;
                    let source = replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char_tick_begin(); _ag_obj_tick_begin(); _ag_trg_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )?;
                    replace(
                        &source,
                        "_ag_dirty=0; toggled=0}",
                        "_ag_dirty=0; _ag_trg_evaluate(); toggled=0}",
                        1,
                    )
                }
                Api::Sprites2V1 => {
                    // The sprite table freezes after the trigger table and before
                    // app_start, its per-callback write budget resets before the
                    // callback, and the trigger evaluate stays the single
                    // post-callback step. Every earlier profile keeps its bytes.
                    let source = replace(
                        &source,
                        "if _ag_error==0 { app_start();",
                        "if _ag_error==0 { _ag_obj_freeze(); _ag_trg_freeze(); _ag_spr_freeze(); app_start();",
                        1,
                    )?;
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char_freeze(); _ag_cam2_freeze()",
                        1,
                    )?;
                    let source = replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char_tick_begin(); _ag_obj_tick_begin(); _ag_trg_tick_begin(); _ag_spr_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )?;
                    replace(
                        &source,
                        "_ag_dirty=0; toggled=0}",
                        "_ag_dirty=0; _ag_trg_evaluate(); toggled=0}",
                        1,
                    )
                }
                Api::Triggers3V1 => {
                    let source = replace(
                        &source,
                        "if _ag_error==0 { app_start();",
                        "if _ag_error==0 { _ag_obj_freeze(); _ag_trg_freeze(); app_start();",
                        1,
                    )?;
                    let source = replace(
                        &source,
                        "app_start(); _ag_cast_token=0",
                        "app_start(); _ag_cast_token=0; _ag_char3_freeze(); _ag_cam3_freeze()",
                        1,
                    )?;
                    let source = replace(
                        &source,
                        "app_update(dt); _ag_cast_token=0",
                        "_ag_char3_tick_begin(); _ag_obj_tick_begin(); _ag_trg_tick_begin(); app_update(dt); _ag_cast_token=0",
                        1,
                    )?;
                    replace(
                        &source,
                        "_ag_dirty=0; toggled=0}",
                        "_ag_dirty=0; _ag_trg_evaluate(); toggled=0}",
                        1,
                    )
                }
                _ => Ok(source),
            }
        }
    }
}
pub(super) fn bridge(source: &str) -> Result<String, String> {
    for marker in ["fn main()", "fn reset(", "fn tick(", "fn run_id("] {
        require(source, marker, 1)?;
    }
    let main = source.find("fn main()").unwrap();
    let reset = source.find("fn reset(").unwrap();
    let tick = source.find("fn tick(").unwrap();
    let queries = source.find("fn run_id(").unwrap();
    if !(main < reset && reset < tick && tick < queries) {
        return Err("transport boundary order changed".into());
    }
    let globals = &source[..main];
    let config = &source[reset + "fn reset(".len()..tick];
    let queries = &source[queries + "fn run_id(".len()..];
    Ok(format!("{globals}fn reset({config}fn run_id({queries}\n"))
}
fn require(source: &str, marker: &str, expected: usize) -> Result<(), String> {
    let actual = source.matches(marker).count();
    if actual != expected {
        return Err(format!(
            "trusted marker {marker:?}: expected {expected}, found {actual}"
        ));
    }
    Ok(())
}
fn replace(source: &str, marker: &str, replacement: &str, count: usize) -> Result<String, String> {
    require(source, marker, count)?;
    Ok(source.replace(marker, replacement))
}
// v3 rotates the existing parent-first axis formula; it retains the same arrays.
// Compact trusted numeric units only: preserve user text and v1/v2 identities.
fn segment_compact(source: &str) -> Result<String, String> {
    if source.contains('"') || source.contains('\'') || source.contains('`') {
        return Err("trusted compact unit gained a literal; layout needs explicit review".into());
    }
    Ok(
        auto_lang_core::strip_line_block_comments(&source.replace("\r\n", "\n"))
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(";")
            .replace("{;", "{")
            .replace(";}", "}")
            .replace(";;", ";")
            .replace("};fn ", "}\nfn "),
    )
}
fn segment_transport(source: &str) -> Result<String, String> {
    let original = bridge(source)?.replace("\r\n", "\n");
    require(&original, "fn derive() {", 1)?;
    require(&original, "fn action(", 1)?;
    let begin = original
        .find("fn derive() {")
        .ok_or("shared derive boundary changed")?;
    let end = original[begin..]
        .find("fn action(")
        .ok_or("shared derive end changed")?
        + begin;
    let derive = &original[begin..end];
    let axis_loop = "for axis in 0..3 {\n            var world=fields[i*12+axis]\n            if parent>=0 { world=world+worlds[parent*3+axis] }\n            worlds.push(world)\n        }";
    let visibility = "var shown=flags[i]\n        if parent>=0 { if effective[parent]==0 { shown=0 } }\n        effective.push(shown)";
    if derive.matches(axis_loop).count() != 1 || derive.matches(visibility).count() != 1 {
        return Err("shared derive specialization boundary changed".into());
    }
    let axis_body = axis_loop.strip_prefix("for axis in 0..3 {").unwrap().strip_suffix('}').unwrap()
        .replace("if parent>=0 { world=world+worlds[parent*3+axis] }", "if parent>=0 {if parent!=cached {parent_value=worlds[parent*3+axis];cached=parent};world=world+parent_value}")
        .replace("worlds.push(world)","worlds[i*3+axis]=world");
    let visibility = visibility.replace("effective.push(shown)", "effective[i]=shown");
    let specialized = format!(
        r#"fn derive() {{
 let count=identities.len()
 if worlds.len()!=count*3 || effective.len()!=count {{
  worlds=[];effective=[]
  for i in 0..count {{worlds.push(0.0);worlds.push(0.0);worlds.push(0.0);effective.push(0)}}
  _ag_segment_dx=1;_ag_segment_dy=1;_ag_segment_dz=1;_ag_segment_dv=1
 }}
 if _ag_segment_dx+_ag_segment_dy+_ag_segment_dz+_ag_segment_dv==0 {{_ag_segment_dx=1;_ag_segment_dy=1;_ag_segment_dz=1;_ag_segment_dv=1}}
 for axis in 0..3 {{
  var active=0
  if axis==0 {{active=_ag_segment_dx}};if axis==1 {{active=_ag_segment_dy}};if axis==2 {{active=_ag_segment_dz}}
  if active==1 {{var cached= -2;var parent_value=0.0;for i in 0..count {{let parent=parents[i];{axis_body}}}}}
 }}
 if _ag_segment_dv==1 {{for i in 0..count {{let parent=parents[i];{visibility}}}}}
 _ag_segment_dx=0;_ag_segment_dy=0;_ag_segment_dz=0;_ag_segment_dv=0
}}"#
    );
    Ok(format!("var _ag_segment_dx=1\nvar _ag_segment_dy=1\nvar _ag_segment_dz=1\nvar _ag_segment_dv=1\n{}{}\n{}",&original[..begin],segment_compact(&specialized)?,&original[end..]))
}
fn segment_base_api(api: &str) -> Result<String, String> {
    let field = "fields[node*12+axis]=value; _ag_dirty=1";
    let movement = "fields[node*12+2]=fields[node*12+2]+dz";
    let visible = "flags[node]=flag; _ag_dirty=1";
    if api.matches(field).count() != 1
        || api.matches(movement).count() != 1
        || api.matches(visible).count() != 1
    {
        return Err("shared setter specialization boundary changed".into());
    }
    Ok(api.replace(field,&format!("{field};if axis==0 {{_ag_segment_dx=1}};if axis==1 {{_ag_segment_dy=1}};if axis==2 {{_ag_segment_dz=1}}"))
        .replace(movement,&format!("{movement};_ag_segment_dx=1;_ag_segment_dy=1;_ag_segment_dz=1"))
        .replace(visible,&format!("{visible};_ag_segment_dv=1")))
}
const ENTRY: &str = "\nfn main() {}\nfn profile() int {return 7}\nfn _ag_version() int {return 1}\nfn _ag_status() int {return _ag_error}\nfn _ag_error_node() int {return _ag_node}\nfn _ag_error_argument() int {return _ag_argument}\nfn _ag_start(){input_space=0; input_x=0; input_y=0; toggled=0; app_start(); derive(); _ag_dirty=0}\nfn tick(next int, dt float){sequence=next; app_update(dt); derive(); _ag_dirty=0; toggled=0}\n";

#[cfg(test)]
mod tests {
    use super::*;
    const APIS: [Api; 13] = [
        Api::V1,
        Api::SpatialV2,
        Api::SegmentV3,
        Api::SweepV4,
        Api::MotionV5,
        Api::CharacterV1,
        Api::Character3V1,
        Api::Camera2V1,
        Api::Camera3V1,
        Api::Objects2V1,
        Api::Objects3V1,
        Api::Triggers2V1,
        Api::Triggers3V1,
    ];
    #[test]
    fn trusted_boundary_mutations_reject_and_restored_units_match() {
        let transport = [
            "fn main()",
            "fn reset(",
            "fn tick(",
            "fn run_id(",
            "fn derive() {",
            "fn action(",
        ];
        let setters = [
            "fields[node*12+axis]=value; _ag_dirty=1",
            "fields[node*12+2]=fields[node*12+2]+dz",
            "flags[node]=flag; _ag_dirty=1",
        ];
        let entries = [
            "fn main() {}",
            "fn profile() int {return 7}",
            "fn _ag_version() int {return 1}",
            "app_start();",
            "app_update(dt);",
            "; derive(); _ag_dirty=0",
        ];
        for api in APIS {
            let expected = assemble(api).unwrap();
            for (kind, markers) in [
                (0, transport.as_slice()),
                (1, setters.as_slice()),
                (2, entries.as_slice()),
                (3, ["_ag_cast_token=0", "if token<=0"].as_slice()),
            ] {
                if (kind == 1 && matches!(api, Api::V1 | Api::SpatialV2))
                    || (kind == 3
                        && !matches!(
                            api,
                            Api::SweepV4 | Api::MotionV5 | Api::CharacterV1 | Api::Character3V1
                        ))
                {
                    continue;
                }
                for marker in markers {
                    if kind == 0
                        && ["fn derive() {", "fn action("].contains(marker)
                        && matches!(api, Api::V1 | Api::SpatialV2)
                    {
                        continue;
                    }
                    let original = match kind {
                        0 => INPUTS.transport,
                        1 => INPUTS.base,
                        2 => INPUTS.entry,
                        _ => INPUTS.segment,
                    };
                    for changed in [
                        original.replacen(marker, "changed_boundary", 1),
                        format!("{original}\n{marker}"),
                    ] {
                        let mut input = INPUTS;
                        match kind {
                            0 => input.transport = &changed,
                            1 => input.base = &changed,
                            2 => input.entry = &changed,
                            _ => input.segment = &changed,
                        }
                        assert!(
                            assemble_from(api, input).is_err(),
                            "{api:?} accepted {marker:?}"
                        );
                    }
                }
            }
            let restored = assemble_from(api, INPUTS).unwrap();
            assert_eq!(restored.transport, expected.transport);
            assert_eq!(restored.api, expected.api);
            assert_eq!(restored.entry, expected.entry);
        }
    }
    #[test]
    fn required_components_cannot_be_missing_or_duplicated() {
        for api in APIS {
            for kind in 0..12 {
                let required = match kind {
                    0 => true,
                    1 => api != Api::V1,
                    2 => matches!(
                        api,
                        Api::SegmentV3
                            | Api::SweepV4
                            | Api::MotionV5
                            | Api::CharacterV1
                            | Api::Character3V1
                            | Api::Camera2V1
                            | Api::Camera3V1
                            | Api::Objects2V1
                            | Api::Objects3V1
                            | Api::Triggers2V1
                            | Api::Triggers3V1
                    ),
                    3 => matches!(
                        api,
                        Api::SweepV4
                            | Api::MotionV5
                            | Api::CharacterV1
                            | Api::Character3V1
                            | Api::Camera2V1
                            | Api::Camera3V1
                            | Api::Objects2V1
                            | Api::Objects3V1
                            | Api::Triggers2V1
                            | Api::Triggers3V1
                    ),
                    4 | 5 => matches!(
                        api,
                        Api::MotionV5
                            | Api::CharacterV1
                            | Api::Character3V1
                            | Api::Camera2V1
                            | Api::Camera3V1
                            | Api::Objects2V1
                            | Api::Objects3V1
                            | Api::Triggers2V1
                            | Api::Triggers3V1
                    ),
                    6 => matches!(
                        api,
                        Api::CharacterV1 | Api::Camera2V1 | Api::Objects2V1 | Api::Triggers2V1
                    ),
                    7 => matches!(
                        api,
                        Api::Character3V1 | Api::Camera3V1 | Api::Objects3V1 | Api::Triggers3V1
                    ),
                    8 => matches!(api, Api::Camera2V1 | Api::Objects2V1 | Api::Triggers2V1),
                    9 => matches!(api, Api::Camera3V1 | Api::Objects3V1 | Api::Triggers3V1),
                    10 => matches!(
                        api,
                        Api::Objects2V1 | Api::Objects3V1 | Api::Triggers2V1 | Api::Triggers3V1
                    ),
                    11 => matches!(api, Api::Triggers2V1 | Api::Triggers3V1),
                    _ => false,
                };
                if !required {
                    continue;
                }
                let original = match kind {
                    0 => INPUTS.base,
                    1 => INPUTS.spatial,
                    2 => INPUTS.segment,
                    3 => INPUTS.sweep,
                    4 => INPUTS.motion_scan,
                    5 => INPUTS.motion,
                    6 => INPUTS.character,
                    7 => INPUTS.character3,
                    8 => INPUTS.camera2,
                    9 => INPUTS.camera3,
                    10 => INPUTS.objects,
                    _ => INPUTS.triggers,
                };
                for changed in [String::new(), format!("{original}\n{original}")] {
                    let mut input = INPUTS;
                    match kind {
                        0 => input.base = &changed,
                        1 => input.spatial = &changed,
                        2 => input.segment = &changed,
                        3 => input.sweep = &changed,
                        4 => input.motion_scan = &changed,
                        5 => input.motion = &changed,
                        6 => input.character = &changed,
                        7 => input.character3 = &changed,
                        8 => input.camera2 = &changed,
                        9 => input.camera3 = &changed,
                        10 => input.objects = &changed,
                        _ => input.triggers = &changed,
                    }
                    assert!(
                        assemble_from(api, input).is_err(),
                        "{api:?} accepted missing/duplicate component{kind}"
                    );
                }
            }
            assert!(assemble_from(api, INPUTS).is_ok());
        }
    }
    #[test]
    fn derive_specialization_requires_exact_axis_and_visibility_units() {
        let axis = "for axis in 0..3 {\n            var world=fields[i*12+axis]\n            if parent>=0 { world=world+worlds[parent*3+axis] }\n            worlds.push(world)\n        }";
        let visibility = "var shown=flags[i]\n        if parent>=0 { if effective[parent]==0 { shown=0 } }\n        effective.push(shown)";
        for marker in [axis, visibility] {
            assert_eq!(INPUTS.transport.matches(marker).count(), 1);
            for replacement in ["changed".to_owned(), format!("{marker}\n{marker}")] {
                let changed = INPUTS.transport.replace(marker, &replacement);
                for api in [Api::SegmentV3, Api::SweepV4, Api::MotionV5] {
                    let input = Inputs {
                        transport: &changed,
                        ..INPUTS
                    };
                    let error = assemble_from(api, input).err().unwrap();
                    assert!(error.contains("derive specialization boundary"), "{error}");
                }
            }
        }
        for api in [Api::SegmentV3, Api::SweepV4, Api::MotionV5] {
            assert!(assemble(api).is_ok());
        }
    }
    #[test]
    fn wrong_order_and_literals_reject_before_compilation() {
        let changed = INPUTS
            .transport
            .replace("fn reset(", "BOUNDARY(")
            .replace("fn tick(", "fn reset(")
            .replace("BOUNDARY(", "fn tick(");
        assert!(bridge(&changed).is_err());
        assert!(segment_compact("fn x(){let a=\"literal\"}").is_err());
    }
}

#[cfg(test)]
mod managed_envelope {
    use super::*;
    /// The r5 frozen envelope guard (PLAN-002): the assembled managed source
    /// of every composed profile must stay inside MAX_BRIDGE, and the
    /// counterexample numbers that forced the explicit 64KiB raise stay on
    /// record. Objects2V1 measured 61950 under the old limit; Triggers2V1
    /// 72141 and Triggers3V1 74050 exceeded it.
    #[test]
    fn managed_source_envelope_holds_for_every_composed_profile() {
        for api in [
            Api::V1,
            Api::SpatialV2,
            Api::SegmentV3,
            Api::SweepV4,
            Api::MotionV5,
            Api::CharacterV1,
            Api::Character3V1,
            Api::Camera2V1,
            Api::Camera3V1,
            Api::Objects2V1,
            Api::Objects3V1,
            Api::Triggers2V1,
            Api::Triggers3V1,
            Api::Sprites2V1,
        ] {
            let u = assemble(api).unwrap();
            let total = u.transport.len() + u.api.len() + u.entry.len();
            assert!(
                total <= super::super::MAX_BRIDGE,
                "{api:?} managed source {total} exceeds the frozen envelope"
            );
            if api == Api::Objects2V1 {
                assert_eq!(total, 61950, "the 64KiB counterexample baseline moved");
            }
            if api == Api::Triggers2V1 {
                // The r5 counterexample must stay real: the composition still
                // exceeds the old 64KiB envelope that forced the raise.
                assert!(
                    total > 65536,
                    "the r5 counterexample fell under the old 64KiB limit: {total}"
                );
            }
        }
    }
}
