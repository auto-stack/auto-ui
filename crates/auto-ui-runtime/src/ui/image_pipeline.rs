//! Shared, renderer-neutral image asset pipeline — thin re-export facade.
//!
//! PLAN-811: the complete pipeline (public media types, registry, caches,
//! scheduler, worker pool, viewer sessions, owned `MediaRuntime`, and the
//! transport-neutral HTTP rules) lives in the `auto-media-core` crate,
//! migrated verbatim. This module preserves the historical in-`ui` path with
//! The global helpers keep the media-core type identity. The desktop lookup
//! also accepts tickets owned by a mounted private file session.

pub use auto_media_core::image_pipeline::{
    checked_rgba_bytes, close_media_session, decode_rendition, global_media_registry,
    global_media_runtime, global_media_worker_pool, inspect_image_metadata,
    media_change_generation, media_http_response, media_session_names, media_session_snapshot,
    media_session_stats, media_session_uri, navigate_media_session,
    open_media_session, pending_media_count, queue_media_path, queue_media_thumbnail,
    request_media_view, resolve_media_pixels, resolve_media_uri,
    scan_media_directory, validate_image_limits, DecodedPixelCache, EncodedByteLru,
    MediaAssetError, MediaAssetId, MediaAssetKey, MediaAssetRegistry, MediaAssetState,
    MediaAssetTicket, MediaHttpResponse, MediaLatestWins, MediaLookup, MediaMetadata,
    MediaOrientation, MediaPin, MediaPipelineStats, MediaPriority, MediaPriorityQueue,
    MediaRuntime, MediaSurfaceStatus, MediaWork, MediaWorkerPool, RenderedImage, RenditionSpec,
    MAX_IMAGE_FILE_BYTES, MAX_IMAGE_PIXELS,
};

pub fn media_surface_status(src: &str) -> MediaSurfaceStatus {
    #[cfg(feature = "ace-bridge")]
    if crate::back_proxy::native_files::is_media(src) {
        return crate::back_proxy::native_files::media_status(src);
    }
    auto_media_core::image_pipeline::media_surface_status(src)
}

pub fn resolve_media_render(src: &str) -> Option<(MediaAssetId, u32, u32, std::sync::Arc<[u8]>)> {
    #[cfg(feature = "ace-bridge")]
    if crate::back_proxy::native_files::is_media(src) {
        return crate::back_proxy::native_files::media_render(src);
    }
    auto_media_core::image_pipeline::resolve_media_render(src)
}
