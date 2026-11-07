//! PLAN-062（AC-03）：媒体**静态帧（海报/首帧/暂停帧）**的进程级供给面。
//!
//! 为什么需要这一层：真实 mpv 帧由 sw renderer 直接写进持久 wgpu 纹理
//! （`ui/mpv/channel.rs`），**没有 CPU 侧 RGBA 读回**；RQ 记录器（tiny-skia
//! Secondary 层）看不到纹理，只能记占位。用户裁定（2026-11-08）本计划的交付
//! 口径是“**一帧**（首帧/暂停帧/海报）→ 资源身份 → 跨线重放”，连续读回
//! （每帧下来 + 带宽预算）具名留后，与 OS063 传输/媒体管线一起设计。
//!
//! 语义：
//! - 键 = `media_key`（声明了就用它，跨视图共享）否则 `src`；
//! - 值 = 一次发布的 RGBA 静态帧 + 单调代次（同键更新 +1，可观测）；
//! - 帧数据紧排（w×h×4，straight RGBA，与 `BitmapUpload` 同口径）；
//! - 渲染侧（`iced::renderer::media::render_video`）命中即用普通 `image`
//!   元素出图——于是资源身份/去重/预算/驻留全部走既有图片管线，不新增第二套
//!   资源通道；
//! - 进程级注册表：不跨进程共享（供给方与渲染同进程，真实媒体引擎即如此）。
//!
//! 边界（不得当成已交付）：本模块只证明“已解码的一帧能安全过线并重放”。
//! 从 mpv 纹理取回这一帧（一次性 `copy_texture_to_buffer`）属于媒体运行时，
//! 当前环境没有媒体引擎，未在本计划内验证。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// 一帧静态媒体画面（紧排 RGBA，straight 非预乘）。
#[derive(Clone, Debug, PartialEq)]
pub struct PosterFrame {
    pub w: u32,
    pub h: u32,
    /// 行字节距（紧排 = w×4）。
    pub stride: u32,
    pub rgba: Vec<u8>,
    /// 供给代次（同键每次发布 +1；诊断/断言用）。
    pub generation: u64,
}

fn registry() -> &'static Mutex<HashMap<String, PosterFrame>> {
    static REG: OnceLock<Mutex<HashMap<String, PosterFrame>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

fn generations() -> &'static Mutex<HashMap<String, u64>> {
    static GENS: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
    GENS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 发布/更新某媒体键的静态帧。尺寸或长度非法（空、非紧排）→ 拒绝并返回 false。
pub fn publish(key: &str, w: u32, h: u32, rgba: Vec<u8>) -> bool {
    if key.is_empty() || w == 0 || h == 0 {
        return false;
    }
    let stride = w.saturating_mul(4);
    if rgba.len() != (stride as usize).saturating_mul(h as usize) {
        return false;
    }
    let generation = {
        let mut gens = generations().lock().unwrap();
        let g = gens.entry(key.to_string()).or_insert(0);
        *g += 1;
        *g
    };
    registry().lock().unwrap().insert(
        key.to_string(),
        PosterFrame {
            w,
            h,
            stride,
            rgba,
            generation,
        },
    );
    true
}

/// 读当前帧（拷贝）。未发布 = None。
pub fn frame(key: &str) -> Option<PosterFrame> {
    registry().lock().unwrap().get(key).cloned()
}

/// 当前代次（未发布 = 0）。
pub fn generation(key: &str) -> u64 {
    generations()
        .lock()
        .unwrap()
        .get(key)
        .copied()
        .unwrap_or(0)
}

/// 撤下某媒体键的帧（媒体关闭/换源）。
pub fn clear(key: &str) {
    registry().lock().unwrap().remove(key);
    generations().lock().unwrap().remove(key);
}

/// 清空（测试隔离用）。
pub fn clear_all() {
    registry().lock().unwrap().clear();
    generations().lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_read_update_and_clear() {
        clear_all();
        let (w, h) = (4u32, 2u32);
        let rgba = vec![7u8; (w * h * 4) as usize];
        assert!(publish("probe://poster", w, h, rgba.clone()));
        let f = frame("probe://poster").expect("发布后必须可读");
        assert_eq!((f.w, f.h, f.stride), (w, h, w * 4));
        assert_eq!(f.rgba, rgba);
        assert_eq!(f.generation, 1);

        // 更新 = 代次 +1 + 内容替换
        let rgba2 = vec![9u8; (w * h * 4) as usize];
        assert!(publish("probe://poster", w, h, rgba2.clone()));
        assert_eq!(generation("probe://poster"), 2);
        assert_eq!(frame("probe://poster").unwrap().rgba, rgba2);

        // 非法长度拒收，旧帧保持
        assert!(!publish("probe://poster", w, h, vec![0u8; 8]));
        assert_eq!(frame("probe://poster").unwrap().rgba, rgba2);

        clear("probe://poster");
        assert!(frame("probe://poster").is_none());
        assert_eq!(generation("probe://poster"), 0);
    }
}
