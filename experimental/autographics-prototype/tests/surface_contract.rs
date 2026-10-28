//! PLAN-830 V-01 `surface-model` contract tests: composition identity,
//! multi-instance/shared mounts, DAG scheduling, cycle rejection, atomic
//! batches (no half-commit, no consumed ids), dirty propagation, depth
//! budget classification, and the input path/capture rules from §5.3.
//!
//! These are pure-model tests (no GPU/window): they prove the graph
//! contract; the windowed/GPU evidence lives in the `surface-*` probe cases.

use agp::core::composition::input::{hit_test, no_planes, CancelReason, InputOutcome, InputState};
use agp::core::composition::{
    CompositionChange, CompositionError, CompositionGraph, ContentKind, MountKind, PortalDesc,
    Rect, SurfaceDesc, ViewDesc,
};
use agp::core::scene::Color;
use agp::host::composition_input::CompositionInput;
use agp::host::iced_surface_probe::probe_camera;
use agp::host::surface_runtime::{Game2dContent, SurfaceContent, SurfaceRuntime};
use agp::renderer::GpuDevice;
use std::collections::HashMap;

fn surf(w: f32, h: f32, kind: ContentKind) -> SurfaceDesc {
    SurfaceDesc {
        logical_size: [w, h],
        content: kind,
    }
}

fn flat(offset: [f32; 2], scale: [f32; 2], clip: Rect) -> MountKind {
    MountKind::Flat2d {
        offset,
        scale,
        clip,
    }
}

/// A graph with one 800x600 top surface + window view.
fn base_graph() -> (
    CompositionGraph,
    agp::core::composition::SurfaceId,
    agp::core::composition::ViewId,
) {
    let mut g = CompositionGraph::new();
    let mints = g
        .submit_one(CompositionChange::AddSurface {
            desc: surf(800.0, 600.0, ContentKind::Scene3d),
        })
        .expect("top surface");
    let top = mints.first_surface().unwrap();
    let mints = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: top,
                presents_to_window: true,
            },
        })
        .expect("top view");
    (g, top, mints.first_view().unwrap())
}

// ---------------------------------------------------------------------------
// Identity / multi-instance / shared mounts
// ---------------------------------------------------------------------------

#[test]
fn two_independent_instances_do_not_share_identity() {
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(480.0, 360.0, ContentKind::Iced),
            },
            CompositionChange::AddSurface {
                desc: surf(480.0, 360.0, ContentKind::Iced),
            },
        ])
        .expect("two instances");
    let a = m.surface(0).expect("first");
    let b = m.surface(1).expect("second");
    assert_ne!(a, b, "identical content/kind must NOT share identity");
    // Newly created surfaces are dirty by construction (first render); take a
    // clean baseline, then prove a content change touches only its instance.
    g.clear_dirty();
    g.mark_content_dirty(a);
    assert!(g.is_dirty(a));
    assert!(
        !g.is_dirty(b),
        "state of one instance must not dirty the other"
    );
}

#[test]
fn shared_surface_is_scheduled_once_across_two_portals() {
    let (mut g, top, top_view) = base_graph();
    let mints = g
        .submit_one(CompositionChange::AddSurface {
            desc: surf(320.0, 240.0, ContentKind::Game2d),
        })
        .expect("game surface");
    let game = mints.first_surface().unwrap();
    let mut g2 = g.clone();
    // Two portals from the SAME view onto the same child surface.
    let mounts = g2
        .submit(&[
            CompositionChange::AddPortal {
                desc: PortalDesc {
                    parent: top_view,
                    child: game,
                    mount: flat([10.0, 10.0], [1.0, 1.0], Rect::at(0.0, 0.0, 400.0, 400.0)),
                    interactive: true,
                },
            },
            CompositionChange::AddPortal {
                desc: PortalDesc {
                    parent: top_view,
                    child: game,
                    mount: flat(
                        [420.0, 10.0],
                        [1.0, 1.0],
                        Rect::at(400.0, 0.0, 800.0, 400.0),
                    ),
                    interactive: true,
                },
            },
        ])
        .expect("two mounts of one surface");
    assert_eq!(mounts.created.len(), 2);
    g = g2;
    assert_eq!(g.surfaces().len(), 2);
    let order = g.render_order();
    assert_eq!(
        order.len(),
        2,
        "each surface appears exactly once: {order:?}"
    );
    assert_eq!(order[0], game, "consumed child renders before its consumer");
    assert_eq!(order[1], top);
    // Both mounts present the same content instance (one surface row).
    assert_eq!(g.surface(game).unwrap().mounted_by.len(), 2);
}

// ---------------------------------------------------------------------------
// Cycle + depth
// ---------------------------------------------------------------------------

#[test]
fn self_cycle_is_rejected() {
    let (mut g, top, top_view) = base_graph();
    let err = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: top_view,
                child: top,
                mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 10.0, 10.0)),
                interactive: true,
            },
        })
        .expect_err("a surface may not consume itself");
    assert!(
        matches!(err.as_slice(), [CompositionError::CycleDetected { .. }]),
        "expected a classified cycle rejection, got {err:?}"
    );
    assert_eq!(g.portals().len(), 0, "no portal may survive a rejection");
}

#[test]
fn indirect_cycle_is_rejected() {
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(100.0, 100.0, ContentKind::Iced),
            },
            CompositionChange::AddSurface {
                desc: surf(100.0, 100.0, ContentKind::Canvas2d),
            },
        ])
        .expect("two surfaces");
    let a = m.surface(0).unwrap();
    let b = m.surface(1).unwrap();
    let va = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: a,
                presents_to_window: true,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    let vb = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: b,
                presents_to_window: false,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    // a consumes b ...
    g.submit_one(CompositionChange::AddPortal {
        desc: PortalDesc {
            parent: va,
            child: b,
            mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 100.0, 100.0)),
            interactive: true,
        },
    })
    .expect("a -> b");
    let before = g.snapshot();
    // ... and b consuming a closes the loop: must be rejected atomically.
    let err = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: vb,
                child: a,
                mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 100.0, 100.0)),
                interactive: true,
            },
        })
        .expect_err("indirect cycle");
    assert!(
        matches!(err.as_slice(), [CompositionError::CycleDetected { .. }]),
        "got {err:?}"
    );
    assert_eq!(
        before,
        g.snapshot(),
        "rejected batch must not mutate the graph"
    );
}

#[test]
fn depth_budget_rejection_is_classified_and_configurable() {
    let mut g = CompositionGraph::new();
    // chain of surfaces s0 -> s1 -> s2 -> s3 (s0 consumes s1, ...)
    let mut ids = Vec::new();
    for _ in 0..4 {
        let m = g
            .submit_one(CompositionChange::AddSurface {
                desc: surf(100.0, 100.0, ContentKind::Canvas2d),
            })
            .unwrap();
        ids.push(m.first_surface().unwrap());
    }
    let mut views = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        let m = g
            .submit_one(CompositionChange::AddView {
                desc: ViewDesc {
                    surface: *id,
                    presents_to_window: i == 0,
                },
            })
            .unwrap();
        views.push(m.first_view().unwrap());
    }
    // three levels: s0 consumes s1 consumes s2 (depth 2)
    for i in 0..3 {
        g.submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: views[i],
                child: ids[i + 1],
                mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 100.0, 100.0)),
                interactive: true,
            },
        })
        .expect("within budget");
    }
    // s3's view is not mounted; mount s2 -> s3 to reach depth 3.
    g.submit_one(CompositionChange::AddPortal {
        desc: PortalDesc {
            parent: views[2],
            child: ids[3],
            mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 100.0, 100.0)),
            interactive: true,
        },
    })
    .expect("depth 3 under default budget 8");
    assert_eq!(g.render_order().len(), 4);

    // Tighten the budget: a new chain exceeding it is a CLASSIFIED rejection.
    g.max_depth = 1;
    let m = g
        .submit_one(CompositionChange::AddSurface {
            desc: surf(50.0, 50.0, ContentKind::Canvas2d),
        })
        .unwrap();
    let deep = m.first_surface().unwrap();
    let before = g.snapshot();
    let err = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: views[0],
                child: deep,
                mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 50.0, 50.0)),
                interactive: true,
            },
        })
        .expect_err("budget exceeded");
    assert!(
        err.iter()
            .any(|e| matches!(e, CompositionError::DepthBudgetExceeded { .. })),
        "got {err:?}"
    );
    assert_eq!(before, g.snapshot());
}

// ---------------------------------------------------------------------------
// Atomicity, stale ids, invalid mounts
// ---------------------------------------------------------------------------

#[test]
fn mixed_batch_with_one_bad_entry_commits_nothing() {
    let (mut g, top, top_view) = base_graph();
    let before = g.snapshot();
    let err = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(200.0, 150.0, ContentKind::Iced),
            },
            // invalid mount in the same batch (zero scale is uninvertible)
            CompositionChange::AddPortal {
                desc: PortalDesc {
                    parent: top_view,
                    child: top,
                    mount: flat([0.0, 0.0], [0.0, 1.0], Rect::at(0.0, 0.0, 10.0, 10.0)),
                    interactive: true,
                },
            },
            CompositionChange::AddSurface {
                desc: surf(64.0, 64.0, ContentKind::Canvas2d),
            },
        ])
        .expect_err("batch must be rejected");
    assert!(
        err.iter()
            .any(|e| matches!(e, CompositionError::InvalidMount { .. })),
        "got {err:?}"
    );
    assert_eq!(before, g.snapshot(), "no half-applied batch");
    // No id space consumed: the next allocation continues the same sequence.
    let m = g
        .submit_one(CompositionChange::AddSurface {
            desc: surf(10.0, 10.0, ContentKind::Canvas2d),
        })
        .unwrap();
    let fresh = m.first_surface().unwrap();
    assert_eq!(
        g.surfaces().len(),
        2,
        "only the pre-existing surface plus the new one"
    );
    assert_eq!(g.surface(fresh).unwrap().desc.logical_size, [10.0, 10.0]);
}

#[test]
fn stale_and_foreign_ids_are_rejected() {
    let (mut g, _top, top_view) = base_graph();
    let m = g
        .submit_one(CompositionChange::AddSurface {
            desc: surf(20.0, 20.0, ContentKind::Canvas2d),
        })
        .unwrap();
    let victim = m.first_surface().unwrap();
    g.submit_one(CompositionChange::RemoveSurface(victim))
        .expect("unreferenced surface removal is allowed");
    // stale id (its slot is free now)
    let err = g
        .submit_one(CompositionChange::SetSurfaceSize(victim, [5.0, 5.0]))
        .expect_err("stale surface id");
    assert!(matches!(
        err.as_slice(),
        [CompositionError::InvalidIdentity { .. }]
    ));

    // foreign-session id
    let mut other = CompositionGraph::new();
    let foreign = other
        .submit_one(CompositionChange::AddSurface {
            desc: surf(30.0, 30.0, ContentKind::Iced),
        })
        .unwrap()
        .first_surface()
        .unwrap();
    let err = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: top_view,
                child: foreign,
                mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 100.0, 100.0)),
                interactive: true,
            },
        })
        .expect_err("foreign session id");
    assert!(
        matches!(err.as_slice(), [CompositionError::InvalidIdentity { .. }]),
        "got {err:?}"
    );
}

#[test]
fn nan_mount_and_empty_plane_are_rejected() {
    let (mut g, top, top_view) = base_graph();
    let nan = MountKind::Flat2d {
        offset: [f32::NAN, 0.0],
        scale: [1.0, 1.0],
        clip: Rect::at(0.0, 0.0, 10.0, 10.0),
    };
    let err = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: top_view,
                child: top,
                mount: nan,
                interactive: false,
            },
        })
        .expect_err("NaN mount");
    assert!(matches!(
        err.as_slice(),
        [CompositionError::InvalidMount { .. }]
    ));
    let empty = MountKind::Plane3d {
        center: [0.0, 0.0, 0.0],
        half: [0.0, 1.0],
    };
    let err = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: top_view,
                child: top,
                mount: empty,
                interactive: false,
            },
        })
        .expect_err("empty plane");
    assert!(matches!(
        err.as_slice(),
        [CompositionError::InvalidMount { .. }]
    ));
}

#[test]
fn surface_in_use_cannot_be_removed_and_view_cascade_keeps_the_child() {
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(400.0, 300.0, ContentKind::Scene3d),
            },
            CompositionChange::AddSurface {
                desc: surf(200.0, 200.0, ContentKind::Iced),
            },
        ])
        .unwrap();
    let parent = m.surface(0).unwrap();
    let child = m.surface(1).unwrap();
    let v = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: parent,
                presents_to_window: true,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    g.submit_one(CompositionChange::AddPortal {
        desc: PortalDesc {
            parent: v,
            child,
            mount: flat([20.0, 20.0], [1.0, 1.0], Rect::at(0.0, 0.0, 400.0, 300.0)),
            interactive: true,
        },
    })
    .unwrap();
    // Removing a referenced surface is refused.
    let err = g
        .submit_one(CompositionChange::RemoveSurface(child))
        .expect_err("still mounted");
    assert!(matches!(
        err.as_slice(),
        [CompositionError::SurfaceInUse { .. }]
    ));
    // Closing the view cascades its portals but keeps the child content.
    g.submit_one(CompositionChange::RemoveView(v))
        .expect("view close");
    assert_eq!(g.portals().len(), 0);
    assert_eq!(g.surfaces().len(), 2, "child content survives view close");
    assert!(g.surface(child).unwrap().mounted_by.is_empty());
}

// ---------------------------------------------------------------------------
// Dirty propagation
// ---------------------------------------------------------------------------

#[test]
fn content_dirty_propagates_to_every_consumer_and_no_sibling() {
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(600.0, 400.0, ContentKind::Scene3d),
            },
            CompositionChange::AddSurface {
                desc: surf(300.0, 200.0, ContentKind::Iced),
            },
            CompositionChange::AddSurface {
                desc: surf(300.0, 200.0, ContentKind::Game2d),
            },
        ])
        .unwrap();
    let parent = m.surface(0).unwrap();
    let ui = m.surface(1).unwrap();
    let game = m.surface(2).unwrap();
    let pv = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: parent,
                presents_to_window: true,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    for (i, child) in [ui, game].into_iter().enumerate() {
        g.submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: pv,
                child,
                mount: flat(
                    [i as f32 * 310.0, 0.0],
                    [1.0, 1.0],
                    Rect::at(i as f32 * 310.0, 0.0, 310.0, 200.0),
                ),
                interactive: true,
            },
        })
        .unwrap();
    }
    g.clear_dirty();
    assert!(!g.is_dirty(parent));
    g.mark_content_dirty(ui);
    assert!(g.is_dirty(ui), "changed content is dirty");
    assert!(
        g.is_dirty(parent),
        "consumer of the changed content is dirty"
    );
    assert!(!g.is_dirty(game), "unrelated sibling content stays clean");
    assert_eq!(g.surface(ui).unwrap().revision, 1);
    assert_eq!(g.surface(game).unwrap().revision, 0);
}

// ---------------------------------------------------------------------------
// Input: nesting, clipping, occlusion, capture, focus invalidation
// ---------------------------------------------------------------------------

struct Nested {
    graph: CompositionGraph,
    top: agp::core::composition::SurfaceId,
    top_view: agp::core::composition::ViewId,
    leaf: agp::core::composition::SurfaceId,
    top_portal: agp::core::composition::PortalId,
    leaf_portal: agp::core::composition::PortalId,
}

/// top(1000x800) ──portal@ (100,100) scale(2,2), clip 500x400──> mid(200x150)
/// mid ──portal@ (10,10) scale(1,1), clip 150x100──> leaf(160x120)
fn nested_graph() -> Nested {
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(1000.0, 800.0, ContentKind::Scene3d),
            },
            CompositionChange::AddSurface {
                desc: surf(200.0, 150.0, ContentKind::Iced),
            },
            CompositionChange::AddSurface {
                desc: surf(160.0, 120.0, ContentKind::Canvas2d),
            },
        ])
        .unwrap();
    let top = m.surface(0).unwrap();
    let mid = m.surface(1).unwrap();
    let leaf = m.surface(2).unwrap();
    let mut views = Vec::new();
    for (i, s) in [top, mid, leaf].into_iter().enumerate() {
        views.push(
            g.submit_one(CompositionChange::AddView {
                desc: ViewDesc {
                    surface: s,
                    presents_to_window: i == 0,
                },
            })
            .unwrap()
            .first_view()
            .unwrap(),
        );
    }
    let top_portal = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: views[0],
                child: mid,
                mount: flat(
                    [100.0, 100.0],
                    [2.0, 2.0],
                    Rect::at(100.0, 100.0, 500.0, 400.0),
                ),
                interactive: true,
            },
        })
        .unwrap()
        .first_portal()
        .unwrap();
    let leaf_portal = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: views[1],
                child: leaf,
                mount: flat([10.0, 10.0], [1.0, 1.0], Rect::at(10.0, 10.0, 150.0, 100.0)),
                interactive: true,
            },
        })
        .unwrap()
        .first_portal()
        .unwrap();
    Nested {
        graph: g,
        top,
        top_view: views[0],
        leaf,
        top_portal,
        leaf_portal,
    }
}

#[test]
fn nested_hit_maps_through_every_level() {
    let n = nested_graph();
    // window (500,300) → mid local ((500-100)/2, (300-100)/2) = (200,100)
    // mid local (200,100) is outside mid's own 200x150 content? No: 200 == w,
    // so use (480,280) instead: mid local (190,90); leaf local (180,80) —
    // outside leaf (160x120)... choose a point inside both.
    // window (260,220) → mid local (80,60) → leaf local (70,50).
    let hit = hit_test(&n.graph, n.top_view, [260.0, 220.0], &no_planes)
        .expect("hit")
        .expect("some hit");
    assert_eq!(hit.surface, n.leaf);
    assert_eq!(hit.local, [70.0, 50.0]);
    assert!(hit.nested);
    assert_eq!(hit.path.len(), 2, "two mounts traversed: {hit:?}");
    assert_eq!(hit.path[0].portal, n.top_portal);
    assert_eq!(hit.path[1].portal, n.leaf_portal);
}

#[test]
fn points_outside_clip_never_reach_the_child() {
    let n = nested_graph();
    // Inside mid's rect but outside mid's CLIP (clip starts at 100,100):
    // window (120,120) → mid local (10,10) → leaf local (0,0) — allowed.
    let inside = hit_test(&n.graph, n.top_view, [120.0, 120.0], &no_planes)
        .unwrap()
        .unwrap();
    assert_eq!(inside.surface, n.leaf);
    // window (620,520) is beyond the mid clip rect (100..600, 100..500).
    let outside = hit_test(&n.graph, n.top_view, [620.0, 520.0], &no_planes)
        .unwrap()
        .unwrap();
    assert_eq!(
        outside.surface, n.top,
        "top surface owns points outside the mount"
    );
    assert!(!outside.nested);
}

#[test]
fn non_interactive_overlay_occludes_without_delivering() {
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(400.0, 300.0, ContentKind::Scene3d),
            },
            CompositionChange::AddSurface {
                desc: surf(200.0, 200.0, ContentKind::Iced),
            },
            CompositionChange::AddSurface {
                desc: surf(50.0, 50.0, ContentKind::Canvas2d),
            },
        ])
        .unwrap();
    let top = m.surface(0).unwrap();
    let under = m.surface(1).unwrap();
    let overlay = m.surface(2).unwrap();
    let v = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: top,
                presents_to_window: true,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    // Two portals: a lower interactive one and an upper NON-interactive one
    // covering it. The upper consumes the point (occlusion) but delivers
    // nothing to either child.
    g.submit(&[
        // lower: interactive 200x200 child covering 0..200
        CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: v,
                child: under,
                mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 200.0, 200.0)),
                interactive: true,
            },
        },
        // upper: NON-interactive 50x50 overlay covering 0..50
        CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: v,
                child: overlay,
                mount: flat([0.0, 0.0], [1.0, 1.0], Rect::at(0.0, 0.0, 50.0, 50.0)),
                interactive: false,
            },
        },
    ])
    .unwrap();
    let occluded = hit_test(&g, v, [25.0, 25.0], &no_planes).unwrap();
    assert!(
        occluded.is_none(),
        "the non-interactive overlay swallows the hit: {occluded:?}"
    );
    // Outside the occluder's rect the lower interactive mount still receives it.
    let hit = hit_test(&g, v, [100.0, 100.0], &no_planes)
        .unwrap()
        .unwrap();
    assert_eq!(hit.surface, under);
    assert_eq!(hit.local, [100.0, 100.0]);
}

#[test]
fn capture_keeps_delivering_to_the_same_child_after_leaving_its_rect() {
    let n = nested_graph();
    let mut input = InputState::new();
    // press inside leaf
    let out = input
        .press(&n.graph, n.top_view, [260.0, 220.0], &no_planes)
        .unwrap();
    assert!(matches!(out, InputOutcome::Delivered { surface, .. } if surface == n.leaf));
    assert!(input.capture.is_some());
    // move far outside every mount: the SAME leaf still receives it (clamped)
    let out = input
        .moved(&n.graph, n.top_view, [5000.0, 5000.0], &no_planes)
        .unwrap();
    match out {
        InputOutcome::Delivered { surface, local } => {
            assert_eq!(surface, n.leaf, "capture keeps the original target");
            assert!(local[0] >= 0.0 && local[1] >= 0.0);
        }
        other => panic!("expected captured delivery, got {other:?}"),
    }
    // release goes to the captured surface
    assert_eq!(input.release(), Some(n.leaf));
    assert!(input.capture.is_none());
}

#[test]
fn destroying_the_capture_target_cancels_and_rejects_late_release() {
    let mut n = nested_graph();
    let mut input = InputState::new();
    input
        .press(&n.graph, n.top_view, [260.0, 220.0], &no_planes)
        .unwrap();
    // Destroy the leaf's mount (portal) — the captured path dies.
    n.graph
        .submit_one(CompositionChange::RemovePortal(n.leaf_portal))
        .expect("portal removal");
    let out = input
        .moved(&n.graph, n.top_view, [260.0, 220.0], &no_planes)
        .unwrap();
    assert_eq!(out, InputOutcome::Cancelled(CancelReason::TargetDestroyed));
    assert!(input.captures_cancelled_like(&[CancelReason::TargetDestroyed]));
    let gen_after_cancel = input.generation();
    // A late release must not be delivered to anything.
    assert_eq!(input.release(), None);
    assert_eq!(input.generation(), gen_after_cancel);
}

#[test]
fn focus_moves_to_the_single_clicked_target_and_dies_with_its_path() {
    let n = nested_graph();
    let mut input = InputState::new();
    input
        .press(&n.graph, n.top_view, [260.0, 220.0], &no_planes)
        .unwrap();
    assert_eq!(input.keyboard_target(), Some(n.leaf));
    // click in the top surface's own area: focus switches (single target).
    input
        .press(&n.graph, n.top_view, [900.0, 750.0], &no_planes)
        .unwrap();
    assert_eq!(input.keyboard_target(), Some(n.top));
    // Focus on the NESTED leaf, then destroy its mount: the focus path dies
    // and no dangling edit target survives.
    input
        .press(&n.graph, n.top_view, [260.0, 220.0], &no_planes)
        .unwrap();
    assert_eq!(input.keyboard_target(), Some(n.leaf));
    let mut g = n.graph.clone();
    g.submit_one(CompositionChange::RemovePortal(n.leaf_portal))
        .unwrap();
    assert_eq!(input.revalidate_focus(&g), Some(n.leaf));
    assert_eq!(
        input.keyboard_target(),
        None,
        "no dangling focus path survives"
    );
}

#[test]
fn plane_mounts_route_through_the_host_projector() {
    // A 3D plane mount resolves only through the host's ray closure; the
    // composition layer must not invent 3D math.
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(800.0, 600.0, ContentKind::Scene3d),
            },
            CompositionChange::AddSurface {
                desc: surf(400.0, 300.0, ContentKind::Iced),
            },
        ])
        .unwrap();
    let top = m.surface(0).unwrap();
    let ui = m.surface(1).unwrap();
    let v = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: top,
                presents_to_window: true,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    let plane = MountKind::Plane3d {
        center: [0.0, 1.7, 0.0],
        half: [2.4, 1.8],
    };
    g.submit_one(CompositionChange::AddPortal {
        desc: PortalDesc {
            parent: v,
            child: ui,
            mount: plane,
            interactive: true,
        },
    })
    .unwrap();
    // Without a projector the plane can never be hit.
    let miss = hit_test(&g, v, [400.0, 300.0], &no_planes).unwrap();
    assert_eq!(miss.unwrap().surface, top);
    // With a projector that maps the window center to uv (0.5, 0.5), the child
    // receives its center.
    let projector = |_: &MountKind, p: [f32; 2], _host: agp::core::composition::SurfaceId| {
        if (p[0] - 400.0).abs() < 1.0 && (p[1] - 300.0).abs() < 1.0 {
            Some([0.5, 0.5])
        } else {
            None
        }
    };
    let hit = hit_test(&g, v, [400.0, 300.0], &projector)
        .unwrap()
        .unwrap();
    assert_eq!(hit.surface, ui);
    assert_eq!(hit.local, [200.0, 150.0]);
}

#[test]
fn non_finite_input_is_rejected() {
    let n = nested_graph();
    let mut input = InputState::new();
    let err = input
        .press(&n.graph, n.top_view, [f32::INFINITY, 0.0], &no_planes)
        .expect_err("non-finite point");
    assert!(matches!(err, CompositionError::InvalidMount { .. }));
    assert!(input.capture.is_none(), "rejection must not capture");
}

/// Tiny helper so tests read clearly about cancellation bookkeeping.
trait CancelBook {
    fn captures_cancelled_like(&self, want: &[CancelReason]) -> bool;
}

impl CancelBook for InputState {
    fn captures_cancelled_like(&self, want: &[CancelReason]) -> bool {
        self.cancellations == want
    }
}

// ---------------------------------------------------------------------------
// Review r1 permanent regressions (F-830-R1-01/02/03/04/05/06/07)
// ---------------------------------------------------------------------------

/// F01: a structural edit invalidates the surface whose composed output
/// changed, and every consumer of it — not only newly created surfaces.
#[test]
fn structural_changes_dirty_every_affected_consumer() {
    let mut g = CompositionGraph::new();
    let a = g
        .submit_one(CompositionChange::AddSurface {
            desc: surf(100.0, 100.0, ContentKind::Canvas2d),
        })
        .unwrap()
        .first_surface()
        .unwrap();
    let b = g
        .submit_one(CompositionChange::AddSurface {
            desc: surf(20.0, 20.0, ContentKind::Canvas2d),
        })
        .unwrap()
        .first_surface()
        .unwrap();
    let v = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: a,
                presents_to_window: true,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    let p = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: v,
                child: b,
                mount: flat([10.0, 10.0], [1.0, 1.0], Rect::at(10.0, 10.0, 20.0, 20.0)),
                interactive: true,
            },
        })
        .unwrap()
        .first_portal()
        .unwrap();
    g.clear_dirty();
    g.submit_one(CompositionChange::SetMount(
        p,
        flat([40.0, 10.0], [1.0, 1.0], Rect::at(40.0, 10.0, 20.0, 20.0)),
    ))
    .unwrap();
    assert!(g.is_dirty(a), "SetMount must dirty the parent output");

    g.clear_dirty();
    g.submit_one(CompositionChange::SetSurfaceSize(b, [30.0, 20.0]))
        .unwrap();
    assert!(
        g.is_dirty(b) && g.is_dirty(a),
        "size change dirties the resized surface AND its consumer"
    );

    g.clear_dirty();
    g.submit_one(CompositionChange::RemovePortal(p)).unwrap();
    assert!(g.is_dirty(a), "unmount must dirty the parent for erasure");

    let p2 = g
        .submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: v,
                child: b,
                mount: flat([10.0, 10.0], [1.0, 1.0], Rect::at(10.0, 10.0, 20.0, 20.0)),
                interactive: true,
            },
        })
        .unwrap()
        .first_portal()
        .unwrap();
    assert!(p2 != p);

    g.clear_dirty();
    g.submit_one(CompositionChange::RemoveView(v)).unwrap();
    assert!(
        g.is_dirty(a),
        "closing a view cascades its portals and dirties the owning surface"
    );
}

/// F05: a non-interactive mount INSIDE an interactive one consumes the event
/// for the whole chain; the interactive parent must not receive a fallback.
#[test]
fn nested_occluder_stops_delivery_for_the_whole_chain() {
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(200.0, 200.0, ContentKind::Scene3d),
            },
            CompositionChange::AddSurface {
                desc: surf(100.0, 100.0, ContentKind::Iced),
            },
            CompositionChange::AddSurface {
                desc: surf(40.0, 40.0, ContentKind::Canvas2d),
            },
        ])
        .unwrap();
    let (top, mid, leaf) = (
        m.surface(0).unwrap(),
        m.surface(1).unwrap(),
        m.surface(2).unwrap(),
    );
    let tv = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: top,
                presents_to_window: true,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    let mv = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: mid,
                presents_to_window: false,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    g.submit_one(CompositionChange::AddPortal {
        desc: PortalDesc {
            parent: tv,
            child: mid,
            mount: flat([20.0, 20.0], [1.0, 1.0], Rect::at(20.0, 20.0, 100.0, 100.0)),
            interactive: true,
        },
    })
    .unwrap();
    g.submit_one(CompositionChange::AddPortal {
        desc: PortalDesc {
            parent: mv,
            child: leaf,
            mount: flat([10.0, 10.0], [1.0, 1.0], Rect::at(10.0, 10.0, 40.0, 40.0)),
            interactive: false,
        },
    })
    .unwrap();
    // Point inside the non-interactive leaf: NOTHING is delivered — not the
    // leaf, not the parent mid.
    let occluded = hit_test(&g, tv, [40.0, 40.0], &no_planes).unwrap();
    assert!(
        occluded.is_none(),
        "inner non-interactive mount must occlude the whole chain: {occluded:?}"
    );
    // Outside the leaf but inside mid: mid is the legitimate control.
    let hit = hit_test(&g, tv, [90.0, 90.0], &no_planes).unwrap().unwrap();
    assert_eq!(hit.surface, mid);
}

/// F04: the plane resolver receives the ACTUAL host surface of each mount.
/// A mount whose parent camera is unknown is a miss — never a hit borrowed
/// from another parent camera.
#[test]
fn plane_projector_is_bound_to_the_actual_host() {
    let mut g = CompositionGraph::new();
    let m = g
        .submit(&[
            CompositionChange::AddSurface {
                desc: surf(400.0, 300.0, ContentKind::Canvas2d),
            },
            CompositionChange::AddSurface {
                desc: surf(200.0, 150.0, ContentKind::Scene3d),
            },
            CompositionChange::AddSurface {
                desc: surf(200.0, 150.0, ContentKind::Scene3d),
            },
            CompositionChange::AddSurface {
                desc: surf(100.0, 100.0, ContentKind::Iced),
            },
        ])
        .unwrap();
    let top = m.surface(0).unwrap();
    let left = m.surface(1).unwrap();
    let right = m.surface(2).unwrap();
    let leaf = m.surface(3).unwrap();
    let tv = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: top,
                presents_to_window: true,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    let lv = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: left,
                presents_to_window: false,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    let rv = g
        .submit_one(CompositionChange::AddView {
            desc: ViewDesc {
                surface: right,
                presents_to_window: false,
            },
        })
        .unwrap()
        .first_view()
        .unwrap();
    for (v, x) in [(lv, 0.0), (rv, 200.0)] {
        g.submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: tv,
                child: if x == 0.0 { left } else { right },
                mount: flat([x, 0.0], [1.0, 1.0], Rect::at(x, 0.0, 200.0, 150.0)),
                interactive: true,
            },
        })
        .unwrap();
        g.submit_one(CompositionChange::AddPortal {
            desc: PortalDesc {
                parent: v,
                child: leaf,
                mount: MountKind::Plane3d {
                    center: [0.0, 0.0, 0.0],
                    half: [1.0, 1.0],
                },
                interactive: true,
            },
        })
        .unwrap();
    }
    // Only the LEFT host has a registered camera.
    let projector = |_: &MountKind, _: [f32; 2], host: agp::core::composition::SurfaceId| {
        if host == left {
            Some([0.5, 0.5])
        } else {
            None
        }
    };
    let left_hit = hit_test(&g, tv, [100.0, 75.0], &projector)
        .unwrap()
        .expect("left plane resolves");
    assert_eq!(left_hit.surface, leaf);
    assert_eq!(left_hit.local, [50.0, 50.0]);
    let right_hit = hit_test(&g, tv, [300.0, 75.0], &projector)
        .unwrap()
        .expect("right host itself owns the point");
    assert_eq!(
        right_hit.surface, right,
        "an unregistered parent camera must not borrow the left camera"
    );
}

/// F06: window focus loss cancels capture AND clears the keyboard target.
#[test]
fn window_cancel_clears_capture_and_keyboard_focus() {
    let (g, top, v) = base_graph();
    let mut input = InputState::new();
    input.press(&g, v, [10.0, 10.0], &no_planes).unwrap();
    assert_eq!(input.keyboard_target(), Some(top));
    let dropped = input.cancel(CancelReason::WindowFocusLost);
    assert_eq!(dropped, Some(top), "cancel reports the surface to unfocus");
    assert!(
        input.keyboard_target().is_none(),
        "no stale keyboard target"
    );
    assert!(input.capture.is_none());
    assert_eq!(input.release(), None);
}

// --- GPU-backed regressions (real targets, real composite path) -------------

fn runtime() -> SurfaceRuntime {
    SurfaceRuntime::new(GpuDevice::new().expect("gpu adapter")).expect("runtime")
}

fn canvas(
    rt: &mut SurfaceRuntime,
    size: [f32; 2],
    color: Color,
) -> agp::core::composition::SurfaceId {
    let content = SurfaceContent::Game2d(Box::new(Game2dContent::simple(rt.gpu(), color)));
    rt.add_surface(size, content).expect("surface")
}

fn flat_mount(x: f32, y: f32, w: f32, h: f32) -> MountKind {
    MountKind::Flat2d {
        offset: [x, y],
        scale: [1.0, 1.0],
        clip: Rect::at(x, y, w, h),
    }
}

fn near(a: [u8; 4], b: [u8; 4], tol: u8) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| x.abs_diff(*y) <= tol)
}

/// F01: unmounting a child must repaint the (clean) parent, erasing the old
/// footprint — not leave it until some caller manually marks dirty.
#[test]
fn unmount_erases_child_pixels_from_clean_parent() {
    let mut rt = runtime();
    let top = canvas(&mut rt, [100.0, 100.0], Color::rgb(0.0, 0.0, 0.0));
    let child = canvas(&mut rt, [40.0, 40.0], Color::rgb(1.0, 0.0, 0.0));
    let v = rt.add_view(top, true).unwrap();
    let p = rt
        .add_portal(v, child, flat_mount(10.0, 10.0, 40.0, 40.0), true)
        .unwrap();
    rt.frame(None, &HashMap::new()).unwrap();
    let before = rt.probe_pixel(top, [20.0, 20.0]).unwrap();
    assert!(
        near(before, [255, 0, 0, 255], 4),
        "child visible: {before:?}"
    );
    rt.remove_portal(p).unwrap();
    rt.frame(None, &HashMap::new()).unwrap();
    let after = rt.probe_pixel(top, [20.0, 20.0]).unwrap();
    assert_eq!(
        after,
        [0, 0, 0, 255],
        "parent must repaint without the unmounted child"
    );
}

/// F01: resizing a clean surface must repaint its content (not leave the
/// recreated texture blank), and its consumer must recomposite.
#[test]
fn resize_repaints_and_invalidates_consumers() {
    let mut rt = runtime();
    let top = canvas(&mut rt, [120.0, 100.0], Color::rgb(0.0, 0.0, 0.0));
    let child = canvas(&mut rt, [40.0, 40.0], Color::rgb(0.0, 1.0, 0.0));
    let v = rt.add_view(top, true).unwrap();
    rt.add_portal(v, child, flat_mount(10.0, 10.0, 40.0, 40.0), true)
        .unwrap();
    rt.frame(None, &HashMap::new()).unwrap();
    assert!(near(
        rt.probe_pixel(child, [5.0, 5.0]).unwrap(),
        [0, 255, 0, 255],
        4
    ));

    rt.resize_surface(child, [50.0, 40.0]).unwrap();
    assert!(
        rt.graph.is_dirty(child) && rt.graph.is_dirty(top),
        "resize dirties the surface and its consumer"
    );
    rt.frame(None, &HashMap::new()).unwrap();
    assert!(
        near(
            rt.probe_pixel(child, [5.0, 5.0]).unwrap(),
            [0, 255, 0, 255],
            4
        ),
        "resized target must not be blank"
    );
    assert!(
        near(
            rt.probe_pixel(top, [12.0, 12.0]).unwrap(),
            [0, 255, 0, 255],
            4
        ),
        "consumer recomposited the resized child"
    );
    assert_eq!(rt.stats.last_frame_renders.get(&child), Some(&1));
}

/// F02: an unchanged translucent composition is stable across frames (the
/// second frame must not blend the same layer over the first frame's result).
#[test]
fn unchanged_translucent_portal_is_stable_across_frames() {
    let mut rt = runtime();
    let top = canvas(&mut rt, [100.0, 100.0], Color::rgb(0.5, 0.5, 0.5));
    let child = canvas(&mut rt, [40.0, 40.0], Color::rgba(0.25, 0.0, 0.0, 0.5));
    let v = rt.add_view(top, true).unwrap();
    rt.add_portal(v, child, flat_mount(10.0, 10.0, 40.0, 40.0), true)
        .unwrap();
    rt.frame(None, &HashMap::new()).unwrap();
    let first = rt.probe_pixel(top, [20.0, 20.0]).unwrap();
    rt.frame(None, &HashMap::new()).unwrap();
    let second = rt.probe_pixel(top, [20.0, 20.0]).unwrap();
    assert_eq!(
        first, second,
        "static alpha composition must not accumulate"
    );
    // Exact premultiplied-over expectation on linear 0.5 gray.
    assert!(
        near(first, [188, 138, 138, 255], 6),
        "first frame must equal the alpha oracle, got {first:?}"
    );
}

/// F03: an invalid GPU target is a classified error, not a panic, and the
/// graph/identity space is untouched (no half-committed surface).
#[test]
fn oversized_target_is_classified_and_leaves_the_graph_untouched() {
    let mut rt = runtime();
    let _ = canvas(&mut rt, [20.0, 20.0], Color::rgb(1.0, 0.0, 0.0));
    let before = rt.graph.snapshot();
    let count = rt.graph.live_surfaces();
    let too_wide = rt.gpu().device().limits().max_texture_dimension_2d + 1;
    let content = SurfaceContent::Game2d(Box::new(Game2dContent::simple(
        rt.gpu(),
        Color::rgb(0.0, 0.0, 0.0),
    )));
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rt.add_surface([too_wide as f32, 1.0], content)
    }));
    let result = out.expect("must not panic");
    let err = result.expect_err("oversized target must be a classified Err");
    assert!(
        matches!(err, CompositionError::InvalidSurface { .. }),
        "got {err:?}"
    );
    assert_eq!(before, rt.graph.snapshot(), "no half-committed graph");
    assert_eq!(rt.graph.live_surfaces(), count);
    // A legal creation still works after the failure.
    let ok = canvas(&mut rt, [16.0, 16.0], Color::rgb(0.0, 0.0, 1.0));
    assert!(rt.graph.surface(ok).is_ok());
}

/// F03: a failed RESIZE keeps the last-good surface and its pixels; a later
/// legal resize still works.
#[test]
fn failed_resize_preserves_the_last_good_surface() {
    let mut rt = runtime();
    let s = canvas(&mut rt, [40.0, 40.0], Color::rgb(1.0, 0.0, 0.0));
    rt.frame(None, &HashMap::new()).unwrap();
    let before_px = rt.probe_pixel(s, [10.0, 10.0]).unwrap();
    let snapshot = rt.graph.snapshot();
    let too = rt.gpu().device().limits().max_texture_dimension_2d + 1;
    let err = rt
        .resize_surface(s, [too as f32, 40.0])
        .expect_err("classified");
    assert!(
        matches!(err, CompositionError::InvalidSurface { .. }),
        "got {err:?}"
    );
    assert_eq!(rt.graph.snapshot(), snapshot, "graph size not committed");
    rt.frame(None, &HashMap::new()).unwrap();
    assert_eq!(rt.probe_pixel(s, [10.0, 10.0]).unwrap(), before_px);
    rt.resize_surface(s, [50.0, 40.0]).expect("legal resize");
    rt.frame(None, &HashMap::new()).unwrap();
    assert!(near(
        rt.probe_pixel(s, [10.0, 10.0]).unwrap(),
        [255, 0, 0, 255],
        4
    ));
}

/// F04: two 3D parents with different cameras share a child surface; a point
/// on the SECOND parent's plane must map through the SECOND camera. Repeats
/// to catch HashMap-order instability.
#[test]
fn plane_hit_uses_the_focused_parent_camera() {
    let mut rt = runtime();
    let top = canvas(&mut rt, [900.0, 500.0], Color::rgb(0.0, 0.0, 0.0));
    let tv = rt.add_view(top, true).unwrap();
    let p1 = rt
        .add_surface(
            [400.0, 400.0],
            SurfaceContent::Game2d(Box::new(Game2dContent::simple(
                rt.gpu(),
                Color::rgb(0.1, 0.1, 0.1),
            ))),
        )
        .unwrap();
    let p2 = rt
        .add_surface(
            [400.0, 400.0],
            SurfaceContent::Game2d(Box::new(Game2dContent::simple(
                rt.gpu(),
                Color::rgb(0.1, 0.1, 0.1),
            ))),
        )
        .unwrap();
    let leaf = canvas(&mut rt, [100.0, 100.0], Color::rgb(1.0, 0.0, 0.0));
    let v1 = rt.add_view(p1, false).unwrap();
    let v2 = rt.add_view(p2, false).unwrap();
    rt.add_portal(tv, p1, flat_mount(0.0, 0.0, 400.0, 400.0), true)
        .unwrap();
    rt.add_portal(tv, p2, flat_mount(450.0, 0.0, 400.0, 400.0), true)
        .unwrap();
    let mount = MountKind::Plane3d {
        center: [0.0, 1.7, 0.0],
        half: [2.4, 1.8],
    };
    rt.add_portal(v1, leaf, mount, true).unwrap();
    rt.add_portal(v2, leaf, mount, true).unwrap();

    let cam1 = probe_camera(1.0);
    let cam2 = probe_camera(2.0);
    for _ in 0..24 {
        let mut router = CompositionInput::new([900.0, 500.0], [900.0, 500.0]);
        router.mapper.register(p1, cam1, [400.0, 400.0]);
        router.mapper.register(p2, cam2, [400.0, 400.0]);
        let projected = router
            .mapper
            .project(p2, &mount, [0.2, 0.3])
            .expect("second-parent projection");
        match router.pointer_pressed(&rt.graph, [450.0 + projected[0], projected[1]]) {
            Some(InputOutcome::Delivered { surface, local })
                if surface == leaf
                    && (local[0] - 20.0).abs() < 0.01
                    && (local[1] - 30.0).abs() < 0.01 => {}
            bad => panic!("wrong parent camera: expected leaf local [20,30], got {bad:?}"),
        }
    }
}

/// F07: the IME anchor follows the ACTIVE focus mount, not the first BFS
/// mount of a shared surface. The root surface's anchor is its own local
/// point (empty path), not `None`.
#[test]
fn ime_anchor_uses_the_focused_shared_mount() {
    let mut rt = runtime();
    let top = canvas(&mut rt, [240.0, 100.0], Color::rgb(0.0, 0.0, 0.0));
    let child = canvas(&mut rt, [40.0, 40.0], Color::rgb(1.0, 0.0, 0.0));
    let v = rt.add_view(top, true).unwrap();
    rt.add_portal(v, child, flat_mount(10.0, 10.0, 40.0, 40.0), true)
        .unwrap();
    let second = rt
        .add_portal(v, child, flat_mount(150.0, 10.0, 40.0, 40.0), true)
        .unwrap();
    let mut router = CompositionInput::new([240.0, 100.0], [240.0, 100.0]);
    router.pointer_pressed(&rt.graph, [160.0, 20.0]);
    assert_eq!(
        router.state.focus.as_ref().unwrap().hit.path[0].portal,
        second,
        "the click focused the second mount"
    );
    let anchor = router.anchor_to_window(&rt.graph, &rt, child, [10.0, 10.0]);
    assert_eq!(
        anchor,
        Some([160.0, 20.0]),
        "anchor follows the focused mount"
    );
    let root_anchor = router.anchor_to_window(&rt.graph, &rt, top, [10.0, 10.0]);
    assert_eq!(root_anchor, Some([10.0, 10.0]), "root anchor is valid");
}
