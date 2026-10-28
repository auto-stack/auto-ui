//! One device per host. The `DeviceToken` proves both render targets draw
//! through the same device/queue (V-01 "same device identity" evidence).

use crate::renderer::error::RendererError;
use std::sync::atomic::{AtomicU64, Ordering};

/// Opaque identity token: (process-wide device ordinal, adapter digest).
/// Rendered into probe reports from every target that shares the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceToken {
    pub ordinal: u64,
    pub adapter_digest: u64,
}

impl DeviceToken {
    pub fn to_report(&self) -> String {
        format!("dev{}:{:016x}", self.ordinal, self.adapter_digest)
    }
}

/// Serializable adapter facts (from `wgpu::AdapterInfo`, no wgpu types leak).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AdapterInfoReport {
    pub name: String,
    pub backend: String,
    pub device_type: String,
    pub driver: String,
    pub driver_info: String,
    pub vendor: u32,
    pub device: u32,
}

/// Owns instance/adapter/device/queue. Exactly one per host session.
pub struct GpuDevice {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    token: DeviceToken,
    info: AdapterInfoReport,
}

/// Minimal block-on (avoids a futures executor dependency; device setup only).
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    struct Noop;
    impl Wake for Noop {
        fn wake(self: Arc<Self>) {}
    }
    let waker = Waker::from(Arc::new(Noop));
    let mut cx = Context::from_waker(&waker);
    let mut f = Box::pin(f);
    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(out) => return out,
            Poll::Pending => std::hint::spin_loop(),
        }
    }
}

impl GpuDevice {
    /// Create the host device (high-performance preference; falls back to any
    /// adapter before giving up with `AdapterUnavailable`).
    pub fn new() -> Result<Self, RendererError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let preferred = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }));
        let adapter = match preferred {
            Ok(a) => a,
            Err(_) => block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true,
                compatible_surface: None,
            }))
            .map_err(|_| RendererError::AdapterUnavailable)?,
        };

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("agp host"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            ..Default::default()
        }))
        .map_err(|e| RendererError::DeviceLost(format!("request_device failed: {e}")))?;

        let raw = adapter.get_info();
        let adapter_digest = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            raw.name.hash(&mut h);
            raw.backend.hash(&mut h);
            raw.driver.hash(&mut h);
            raw.vendor.hash(&mut h);
            raw.device.hash(&mut h);
            h.finish()
        };
        static ORDINAL: AtomicU64 = AtomicU64::new(0);
        let token = DeviceToken {
            ordinal: ORDINAL.fetch_add(1, Ordering::Relaxed),
            adapter_digest,
        };
        let info = AdapterInfoReport {
            name: raw.name.clone(),
            backend: format!("{:?}", raw.backend),
            device_type: format!("{:?}", raw.device_type),
            driver: raw.driver.clone(),
            driver_info: raw.driver_info.clone(),
            vendor: raw.vendor,
            device: raw.device,
        };
        Ok(GpuDevice {
            instance,
            adapter,
            device,
            queue,
            token,
            info,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn token(&self) -> DeviceToken {
        self.token
    }
    pub fn adapter_info(&self) -> &AdapterInfoReport {
        &self.info
    }
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    /// Block until all queued GPU work is done (probe checkpoints).
    pub fn wait_idle(&self) {
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
    }
}
