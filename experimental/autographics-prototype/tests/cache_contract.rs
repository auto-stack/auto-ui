//! F04 regression (review r1): cache ownership binding + identity lifecycle.
//! The review's exact sequences are reproduced: (1) free/reuse owner A as B,
//! then request A's key with the valid B handle — must be rejected; (2) a
//! 64 create/release cycle loop — the allocator's live count must stay
//! bounded (entry ids are freed on release/eviction).

use agp::renderer::resources::{CacheKey, ResourceCache};
use agp::renderer::GpuDevice;

fn gpu_or_skip() -> Option<GpuDevice> {
    match GpuDevice::new() {
        Ok(g) => Some(g),
        Err(e) => {
            eprintln!("cache_contract: no GPU device, skipping GPU-gated test: {e}");
            None
        }
    }
}

fn key(owner: &agp::core::id::ObjectId, role: &str) -> CacheKey {
    CacheKey {
        owner: owner.token(),
        role: role.to_string(),
    }
}

/// Review F04 sequence: allocate A, cache A's texture, free A, allocate B
/// (slot reuse), request A's KEY with the valid B handle → rejected; a
/// fresh B key succeeds and `created` increments for the legit new bind.
#[test]
fn f04_key_owner_bound_to_validated_owner() {
    let Some(gpu) = gpu_or_skip() else { return };
    let mut cache = ResourceCache::new(4);
    let owner_a = cache.allocator().allocate();
    let key_a = key(&owner_a, "tex");
    assert!(cache.texture(&gpu, key_a.clone(), owner_a, 16, 16).is_ok());
    cache.allocator().free(owner_a).unwrap();
    let owner_b = cache.allocator().allocate(); // same slot, generation+1
    assert_eq!(owner_a.index(), owner_b.index());
    // the review's attack: A's key + a VALID foreign handle (B)
    let stale = cache.texture(&gpu, key_a.clone(), owner_b, 32, 32);
    assert!(
        stale.is_err(),
        "A's key with B's valid handle must be rejected"
    );
    // legit new bind: B's own key
    let created_before = cache.stats().created;
    let key_b = key(&owner_b, "tex");
    assert!(cache.texture(&gpu, key_b, owner_b, 32, 32).is_ok());
    assert_eq!(cache.stats().created, created_before + 1);
    // and the genuinely-stale A identity is still rejected
    assert!(cache.texture(&gpu, key_a, owner_a, 16, 16).is_err());
}

/// Same-key size change = miss-and-recreate (documented F04 semantics):
/// the identity survives, the storage is re-allocated, `recreated` counts.
#[test]
fn f04_same_key_size_change_recreates() {
    let Some(gpu) = gpu_or_skip() else { return };
    let mut cache = ResourceCache::new(4);
    let owner = cache.allocator().allocate();
    let k = key(&owner, "tex");
    assert!(cache.texture(&gpu, k.clone(), owner, 16, 16).is_ok());
    let created = cache.stats().created;
    let recreated = cache.stats().recreated;
    assert!(cache.texture(&gpu, k.clone(), owner, 32, 32).is_ok());
    assert_eq!(
        cache.stats().created,
        created + 1,
        "recreate allocates new storage"
    );
    assert_eq!(cache.stats().recreated, recreated + 1);
    // same size again = hit
    assert!(cache.texture(&gpu, k, owner, 32, 32).is_ok());
    assert_eq!(cache.stats().hits, 1);
}

/// Review F04 leak loop: capacity 2, 64 create/release cycles — the
/// allocator's live count must stay bounded (review measured 3→67).
#[test]
fn f04_release_and_eviction_free_entry_ids() {
    let Some(gpu) = gpu_or_skip() else { return };
    let mut cache = ResourceCache::new(2);
    let baseline = cache.allocator().live_count();
    for i in 0..64 {
        let owner = cache.allocator().allocate();
        let k = key(&owner, &format!("cycle-{i}"));
        let _ = cache.texture(&gpu, k.clone(), owner, 8, 8);
        cache.release(&k).unwrap();
        cache.allocator().free(owner).unwrap();
        assert_eq!(
            cache.allocator().live_count(),
            baseline,
            "release frees the entry identity (cycle {i})"
        );
    }
    // eviction path: capacity 2 with `keep` refreshed every cycle (so the
    // cycle entries are the LRU); each new create evicts the previous cycle
    // entry and its identity is freed at eviction time.
    let keep = cache.allocator().allocate();
    let keep_key = key(&keep, "keep");
    let _ = cache.texture(&gpu, keep_key.clone(), keep, 8, 8);
    let evicted_before = cache.stats().evicted;
    let mut prev: Option<(CacheKey, agp::core::id::ObjectId)> = None;
    for i in 0..32 {
        let _ = cache.texture(&gpu, keep_key.clone(), keep, 8, 8); // refresh LRU
        let o = cache.allocator().allocate();
        let k = key(&o, &format!("evict-{i}"));
        // two live cycle entries + `keep` exceeds capacity 2 → the previous
        // cycle entry is evicted HERE and its identity freed at eviction.
        let _ = cache.texture(&gpu, k.clone(), o, 8, 8);
        if let Some((pk, po)) = prev.take() {
            // may already be evicted: release is a no-op-classified then
            let _ = cache.release(&pk);
            let _ = cache.allocator().free(po);
        }
        prev = Some((k, o));
        assert!(
            cache.allocator().live_count() <= baseline + 4,
            "bounded identities during eviction cycles (iteration {i})"
        );
    }
    assert!(
        cache.stats().evicted > evicted_before,
        "evictions actually happened"
    );
    // drain the last cycle entry; only `keep` (owner + entry) survives
    if let Some((pk, po)) = prev.take() {
        let _ = cache.release(&pk);
        let _ = cache.allocator().free(po);
    }
    assert_eq!(
        cache.allocator().live_count(),
        baseline + 2,
        "evicted entries free their identities; only `keep` lives"
    );
}
