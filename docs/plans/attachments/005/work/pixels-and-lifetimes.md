# PLAN-005 r3 — T-04/T-05

T-04: `6d6764227cf99dd9fe505fe70ea194d3bba5a835`.
T-05: `621d2b98494631c79171cb9034599dc2194fb23e`.
Both are committed in the same `D:/autostack/.wt/ui-005/auto-ui` worktree.
No final acceptance review, consumer or source identity gate is claimed here.

## T-04 / V-03 actual pixels

- AGP `sampling_modes_srgb_and_material_factor_match_an_independent_cpu_oracle` executes **36 cases / 144 pixel probes**: minification and magnification use opposing min/mag filters, nearest/linear, clamp/repeat/mirrored-repeat on both axes. Its independent CPU sampler decodes texels from sRGB, interpolates in linear space, multiplies the material factor and light, then encodes to the sRGB target. Maximum error **1**, tolerance3. Frozen-UV and flat-colour substitutions both fail the pixel oracle. One shared image creates one actual GPU texture.
- The first minification case (case18) failed: GPU `[14,64,140]`, CPU `[16,74,135]`. Sampler lambda was clamped to zero, forcing mag_filter. The fix keeps lambda for filter selection (`lod_max_clamp=32`), while the actual texture still has exactly **one mip level**. This enables correct base min filtering, not mipmap generation or a new resource budget. The full sampling oracle then passed.
- `textured_depth_and_double_sided_normal_orientation_are_visible_in_real_pixels`: both draw orders select the nearer textured surface; removing it reveals the far surface. Single back face is culled; double back/front lighting matches four explicit numerical controls (max error0, tolerance2); changing light direction distinguishes a missing back-normal flip.
- `double_sided_shadow_culling_and_shadow_disable_controls_use_the_same_geometry`: single back face does not cast, double back face and single front face cast. The receiver's centre is **[99,99,99,255]** in shadow versus **[165,165,165,255]** illuminated. Disabling the real shadow pass makes the oracle fail; restoring it restores the exact pixel. Depth still hides the blocker from the colour pass.
- Driver `gltf_uv_transform_changes_pixels_but_preserves_collision_bounds_and_node_associations` imports a nonuniform 4×4 PNG and a GLB with offset + 90° rotation + nonuniform scale. Four actual GPU probes match a separately specialized mathematical transform and fixture-defined colours (max error0, tolerance2). Dropping the KHR transform and freezing UV each fail at least three probes. Actual Descriptor AABB rows remain equal. Original node1/mesh0/primitive0 association and invertible world transform recover the source-local position for006.
- Driver `gltf_mipmap_filters_have_explicit_base_filter_normalization` verifies all six accepted minFilter enums against their documented nearest/linear base normalization.

AGP's independent target uses `CARGO_PROFILE_DEV_DEBUG=0`, solely to avoid unnecessary debug-symbol storage; no release-performance claim is made. Commands are `cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked --offline --test textured_models ... -- --test-threads=1 --nocapture`; driver uses its own manifest and `--test textured_model_resources gltf_`.

Local evidence under `D:/autostack/plan-artifacts/auto-ui/plan-005/`:

| Log | SHA256 | Meaning |
|---|---|---|
| t04-agp-pixels.log | 995bb011a882bee7606722ff51f84fdb788989df435b52c006a5e6f5be764a42 | Initial 2 passing depth/shadow oracles and case18 sampling red |
| t04-sampling-repair.log | 68b703882ad75590917119ca82055493e81c4d04d7e4b4482f068bfa199e07fa | Sampling 36/144 green after lambda fix |
| t04-gltf-transform.log | 2d8ce09b73241baefa304b574ca693c1948d9c82066434d9fd74212a6228c5a3 | Driver two GLB tests green |
| t04-gltf-independent-colours.log | 033bfb244b13a99f7c6f870e490a057c715bda0c131525c28c92d0ae7fbdbbe9 | Strengthened fixture-colour reference, one GPU GLB case green |

The unchanged depth/shadow evidence is reused after the sampler repair: these cases have equal min/mag filters and uniform one-texel images, so retaining lambda cannot change their sampling. The normalization-only CPU case is likewise unaffected by the reference-colour strengthening. Later T-05 changes affect device admission and invalid payload/close transitions, not these valid same-device pixel paths. The frozen final gate will run the complete required modules after review.

## T-05 / V-04 transactions and actual owners

- Driver `textured_runtime_prepare_failures_preserve_the_complete_previous_frame_and_recover`: failures before preparation and after asset upload preserve the entire last Frame (nodes/instances/camera), tick, actual camera matrix and actual GPU capture. Stop and a fresh Run recover through the same entry.
- `textured_import_failure_drops_candidates_and_leaves_published_pixels_unchanged`: a same-root/different-image import fails at both preparation stages without changing editor document/history/pixels. Actual queue counters show no uploads before preparation, and two buffers plus one texture uploaded then discarded after preparation. Candidate Asset/raw-PNG/decoded-image weak owners expire. Successful retry, public selection and hiding the equal-depth original expose the new real texture. The initial retry oracle incorrectly expected a later equal-depth leaf to overwrite the original; the corrected test observes the new leaf through legal editor actions.
- `twenty_textured_object_camera_runs_keep_uploads_flat_and_release_actual_last_owners`: 20 real user-Auto objects3d Runs with camera motion, character motion, spawn/despawn/reuse and Stop; **20 VM owners released**, **0 warm static mesh uploads / texture uploads / decodes**. Two original models share one decoded PNG and one real GPU texture. CPU retained reservation **1,075,810 bytes**, GPU reservation **1,080,904 bytes**, actual asset mesh+UV+aligned index+texture bytes **1,080,904**. Editor isolation identity and resident resource usage stay equal after each Stop. Close removes cache ownership; external Asset/Mesh/Texture pins retain only their legitimate owners until each final drop, witnessed with actual Weak objects.
- Close originally removed meshes while leaving published instances referencing them (`InvalidSurface: 3d instance mesh missing`). Pruning now retires draws whose payload ownership was revoked; it deliberately retains malformed draws of still-owned resources so renderer validation cannot be bypassed.
- AGP `staged_texture_candidates_share_uploads_and_drop_failed_owners_on_the_same_device`: shared image is one actual GPU owner; a discarded staged new mesh/texture releases both actual Weak owners while its real upload attempt remains counted; malformed dimensions reject before any buffer upload; final external pin release is observed. A new-key foreign-device input initially escaped the old key-local check. Admission now checks the host generation for the whole store before key lookup, and new textures carry the same ordinal. A native Device comparison alone was insufficient across independently created wgpu Instances. The raw Iced variant remains caller-owned/unattested as in the inherited contract; it cannot silently become an attested public store.

Current commands: driver `... --test textured_model_resources textured_` selects **3** tests, all passed; AGP `... --test textured_models staged_texture` selects **1**, passed. Reports are in local `t05-reports/`; none are committed.

| Log | SHA256 |
|---|---|
| t05-runtime-r2.log (initial meaningful failure evidence) | 0ca5ef49f59151fd47175f6db552c0aba1003f6c0e4ae057a516bcabedf6e8cb |
| t05-final-runtime.log (3 passed) | c0f10832206ec699e99c3670c3427651a5a64abcc15a04a18c09293d6cd22451 |
| t05-gpu-owners.log (foreign new-key red) | 39d6baab600b3723639d860d7fa83926706f25b09305db252d2ec3864fb7992d |
| t05-gpu-owners-repair.log (1 passed) | dd448f071475e559174d2f11b7f772ba1c0097b3bc0935e4be9d2aa9f50a992d |

Both packages format clean; `git diff --check` passes; no new product warning. T-06 still owns fresh offline consumer, current identity/checker controls, public standalone project and Spec delta. Review/merge/archival/cleanup remain pending.
