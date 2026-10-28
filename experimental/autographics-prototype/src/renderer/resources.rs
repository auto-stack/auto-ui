//! Bounded, identity-keyed shared resource cache.
//!
//! Contract (PLAN-829 §5.3): lookups validate the generational identity of
//! the requesting object; a stale/reused identity is rejected. Unchanged
//! resources keep their GPU allocation — a frame that touches nothing must
//! not rebuild/upload anything. Eviction is LRU and bounded.

use crate::core::id::{IdAllocator, ObjectId};
use crate::renderer::device::GpuDevice;
use crate::renderer::error::RendererError;
use std::collections::HashMap;

/// Cache key: the owning object's identity token + a role tag.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub owner: String,
    pub role: String,
}

/// A cached RGBA8 texture slot. `id`/`texture`/`width`/`height` are retained
/// for identity/lifetime bookkeeping (the texture must stay alive while
/// cached) even when the current probe paths only read through `view`.
#[allow(dead_code)]
struct Entry {
    id: ObjectId,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    /// LRU clock.
    last_used: u64,
}

/// Observable counters (probe reports before/after).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CacheStats {
    pub created: u64,
    pub recreated: u64,
    pub hits: u64,
    pub misses: u64,
    pub evicted: u64,
    pub released: u64,
    pub stale_rejected: u64,
    pub live_entries: usize,
}

pub struct ResourceCache {
    capacity: usize,
    entries: HashMap<CacheKey, Entry>,
    order: Vec<CacheKey>, // LRU oldest-first (simplified; small capacities)
    clock: u64,
    stats: CacheStats,
    alloc: IdAllocator,
}

impl ResourceCache {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        ResourceCache {
            capacity,
            entries: HashMap::new(),
            order: Vec::new(),
            clock: 0,
            stats: CacheStats::default(),
            alloc: IdAllocator::new(),
        }
    }

    /// Identity allocator for cache-owned resource ids (generation-checked).
    pub fn allocator(&mut self) -> &mut IdAllocator {
        &mut self.alloc
    }

    fn touch(&mut self, key: &CacheKey) {
        self.clock += 1;
        if let Some(e) = self.entries.get_mut(key) {
            e.last_used = self.clock;
        }
    }
    /// Get or create the texture bound to `key` owned by `owner`.
    ///
    /// Ownership contract (review r1 F04):
    /// - the `owner` identity is validated (stale/reused owners rejected);
    /// - the key's owner STRING must equal the validated owner's token —
    ///   presenting a valid foreign handle with someone else's key is a
    ///   stale-identity rejection, never a hit on the old resource;
    /// - a same-key request with DIFFERENT dimensions is a miss-and-recreate
    ///   (identity unchanged, storage re-allocated; counted as `recreated`);
    /// - eviction and explicit release FREE the entry's identity, so the
    ///   allocator's live count returns to baseline (no identity leak).
    pub fn texture(
        &mut self,
        gpu: &GpuDevice,
        key: CacheKey,
        owner: ObjectId,
        width: u32,
        height: u32,
    ) -> Result<&wgpu::TextureView, RendererError> {
        if let Err(e) = self.alloc.validate(owner) {
            self.stats.stale_rejected += 1;
            return Err(RendererError::StaleIdentity {
                id: owner,
                detail: format!("cache owner rejected: {e:?}"),
            });
        }
        if key.owner != owner.token() {
            self.stats.stale_rejected += 1;
            return Err(RendererError::StaleIdentity {
                id: owner,
                detail: format!(
                    "key owner '{}' does not match the validated owner token",
                    key.owner
                ),
            });
        }
        match self.entries.get(&key) {
            Some(e) if e.width == width && e.height == height => {
                self.stats.hits += 1;
                self.touch(&key);
                return Ok(&self.entries.get(&key).unwrap().view);
            }
            Some(_) => {
                // same identity, new dimensions: recreate the storage.
                self.stats.recreated += 1;
                let old = self.entries.remove(&key).unwrap();
                self.free_entry_id(old.id);
                self.order.retain(|x| x != &key);
            }
            None => {
                self.stats.misses += 1;
                if self.entries.len() >= self.capacity {
                    self.evict_oldest();
                }
            }
        }
        self.create_entry(gpu, key, width, height)
    }

    fn create_entry(
        &mut self,
        gpu: &GpuDevice,
        key: CacheKey,
        width: u32,
        height: u32,
    ) -> Result<&wgpu::TextureView, RendererError> {
        let id = self.alloc.allocate();
        let texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("agp cache texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.stats.created += 1;
        self.order.push(key.clone());
        self.clock += 1;
        self.entries.insert(
            key.clone(),
            Entry {
                id,
                texture,
                view,
                width,
                height,
                last_used: self.clock,
            },
        );
        Ok(&self.entries.get(&key).unwrap().view)
    }

    /// Free an entry-owned identity (eviction/release path). Entry ids are
    /// each freed exactly once; the debug_assert documents that invariant.
    fn free_entry_id(&mut self, id: ObjectId) {
        let res = self.alloc.free(id);
        debug_assert!(res.is_ok(), "entry id double-free: {res:?}");
    }

    fn evict_oldest(&mut self) {
        if self.order.is_empty() {
            return;
        }
        // Find the least-recently-used key.
        let mut oldest: Option<CacheKey> = None;
        let mut oldest_clock = u64::MAX;
        for (k, e) in self.entries.iter() {
            if e.last_used < oldest_clock {
                oldest_clock = e.last_used;
                oldest = Some(k.clone());
            }
        }
        if let Some(k) = oldest {
            let entry = self.entries.remove(&k).unwrap();
            self.free_entry_id(entry.id);
            self.order.retain(|x| x != &k);
            self.stats.evicted += 1;
        }
    }

    /// Explicit release. Releasing a key that is already gone is classified
    /// (`ResourceReleased`), not silently absorbed; a successful release
    /// frees the entry's identity.
    pub fn release(&mut self, key: &CacheKey) -> Result<(), RendererError> {
        if let Some(entry) = self.entries.remove(key) {
            self.free_entry_id(entry.id);
            self.order.retain(|x| x != key);
            self.stats.released += 1;
            Ok(())
        } else {
            Err(RendererError::ResourceReleased {
                key: format!("{}:{}", key.owner, key.role),
            })
        }
    }

    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    pub fn live_entries(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_owner_rejected_without_gpu() {
        // Validate identity behavior without a device: only exercise the
        // allocator half of the cache (the GPU half needs a real device and
        // is covered by the gpu-foundation probe).
        let mut cache = ResourceCache::new(4);
        let owner = cache.alloc.allocate();
        cache.alloc.free(owner).unwrap();
        let reused = cache.alloc.allocate();
        assert_eq!(owner.index(), reused.index());
        assert!(cache.alloc.validate(owner).is_err());
    }
}
