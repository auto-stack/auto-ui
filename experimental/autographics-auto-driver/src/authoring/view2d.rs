//! Bounded editor-only world/child mapping. Coordinates are logical pixels.
use glam::Vec2;

pub const SIZE: Vec2 = Vec2::new(800.0, 380.0);
pub const STEPS: [f32; 5] = [1.0, 5.0, 10.0, 50.0, 100.0];
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View2d {
    pub center: Vec2,
    pub zoom: f32,
    pub grid: bool,
    pub snap: bool,
    pub step: f32,
    pub revision: u64,
}
impl Default for View2d {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            grid: false,
            snap: false,
            step: 10.0,
            revision: 0,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct GridLine {
    pub center: [f32; 2],
    pub size: [f32; 2],
    /// 0 minor, 1 major, 2 world X axis, 3 world Y axis.
    pub class: u8,
}
impl View2d {
    pub fn valid(self) -> bool {
        self.center.is_finite()
            && self.center.abs().max_element() <= 65536.0
            && self.zoom.is_finite()
            && (0.125..=8.0).contains(&self.zoom)
            && STEPS.contains(&self.step)
    }
    pub fn screen(self, world: Vec2) -> Option<Vec2> {
        if !self.valid() || !world.is_finite() {
            return None;
        }
        // f64 intermediates prevent finite document coordinates overflowing before clipping.
        let x = 400.0 + (world.x as f64 - self.center.x as f64) * self.zoom as f64;
        let y = 190.0 - (world.y as f64 - self.center.y as f64) * self.zoom as f64;
        let p = Vec2::new(x as f32, y as f32);
        p.is_finite().then_some(p)
    }
    pub fn world(self, screen: Vec2) -> Option<Vec2> {
        if !self.valid() || !screen.is_finite() {
            return None;
        }
        let p = self.center + (screen - SIZE * 0.5) * Vec2::new(1.0, -1.0) / self.zoom;
        p.is_finite().then_some(p)
    }
    pub fn navigate(self, center: Vec2, zoom: f32) -> Result<Self, String> {
        if !center.is_finite() || !zoom.is_finite() || zoom <= 0.0 {
            return Err("invalid 2D navigation".into());
        }
        Ok(Self {
            center: center.clamp(Vec2::splat(-65536.0), Vec2::splat(65536.0)),
            zoom: zoom.clamp(0.125, 8.0),
            ..self
        })
    }
    pub fn pan(self, delta: Vec2) -> Result<Self, String> {
        self.navigate(
            self.center - delta * Vec2::new(1.0, -1.0) / self.zoom,
            self.zoom,
        )
    }
    pub fn wheel(self, p: Vec2, notches: f32) -> Result<Self, String> {
        if !notches.is_finite() {
            return Err("invalid wheel delta".into());
        }
        let anchor = self.world(p).ok_or("invalid wheel anchor")?;
        let z = (self.zoom * 1.1f32.powf(notches.clamp(-8.0, 8.0))).clamp(0.125, 8.0);
        self.navigate(anchor - (p - SIZE * 0.5) * Vec2::new(1.0, -1.0) / z, z)
    }
    pub fn focus(self, lo: Vec2, hi: Vec2) -> Result<Self, String> {
        if !lo.is_finite() || !hi.is_finite() || (hi - lo).min_element() < 0.0 {
            return Err("no finite visible 2D bounds".into());
        }
        let center = lo * 0.5 + hi * 0.5;
        let extent = hi.as_dvec2() - lo.as_dvec2();
        let z = (640.0 / extent.x.max(1e-9))
            .min(304.0 / extent.y.max(1e-9))
            .min(8.0);
        if center.abs().max_element() > 65536.0 || z < 0.125 {
            return Err("selection exceeds supported 2D focus range".into());
        }
        self.navigate(center, z as f32)
    }
    pub fn snapped(self, world: Vec2) -> Vec2 {
        if self.snap {
            (world / self.step).round() * self.step
        } else {
            world
        }
    }
    /// Fixed upper bound: at most 128 rectangular commands and <16KiB CPU storage.
    pub fn grid_lines(self) -> (f32, Vec<GridLine>) {
        if !self.grid || !self.valid() {
            return (0.0, vec![]);
        }
        let mut spacing = 1.0f32;
        'outer: for exponent in -3..=8 {
            for factor in [1.0, 2.0, 5.0] {
                let s = factor * 10f32.powi(exponent);
                if s * self.zoom >= 16.0 {
                    spacing = s;
                    break 'outer;
                }
            }
        }
        let a = self.world(Vec2::new(0.0, 380.0)).unwrap();
        let b = self.world(Vec2::new(800.0, 0.0)).unwrap();
        let mut lines = Vec::with_capacity(128);
        for axis in 0..2 {
            let first = (a[axis] as f64 / spacing as f64).ceil() as i64;
            let last = (b[axis] as f64 / spacing as f64).floor() as i64;
            for i in first..=last {
                if i == 0 {
                    continue;
                }
                let mut w = self.center;
                w[axis] = i as f32 * spacing;
                let p = self.screen(w).unwrap();
                let (center, size) = if axis == 0 {
                    ([p.x, 190.0], [1.0, 380.0])
                } else {
                    ([400.0, p.y], [800.0, 1.0])
                };
                lines.push(GridLine {
                    center,
                    size,
                    class: u8::from(i % 5 == 0),
                });
            }
        }
        let origin = self.screen(Vec2::ZERO).unwrap();
        if (0.0..380.0).contains(&origin.y) {
            lines.push(GridLine {
                center: [400.0, origin.y],
                size: [800.0, 1.0],
                class: 2,
            });
        }
        if (0.0..800.0).contains(&origin.x) {
            lines.push(GridLine {
                center: [origin.x, 190.0],
                size: [1.0, 380.0],
                class: 3,
            });
        }
        // At >=16px spacing, ceil(800/16)+ceil(380/16)+2 <= 76.
        debug_assert!(lines.len() <= 128);
        (spacing, lines)
    }
}
