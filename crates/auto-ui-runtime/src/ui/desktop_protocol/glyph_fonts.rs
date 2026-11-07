//! PLAN-062 T-02：字体内容身份与字形重放支持（glyph run 的 Host 侧基础）。
//!
//! 合同（061 §4 / PLAN-062 用户裁定 2026-11-07）：文本优先以 **glyph run**
//! 过线——wire 携 glyph id、位置与字体内容身份，Host 用同一字体字节按相同
//! (`font_id`, `glyph_id`, 字号, 亚像素位置, 字重, flags) 重新取字形位图，
//! **不得**按字符串重排。字体身份 = 字节内容的 blake3 前缀（禁绝对路径）。
//!
//! 本模块只做两件事：
//! 1. 把宿主已装载的字体登记为 `内容哈希 → fontdb::ID`（跨进程时由
//!    FontBlob 握手装载后调用 [`register_loaded_font`]）；
//! 2. 按内容哈希取回 `fontdb::ID`（未装载 → 调用方必须走具名降级，不得
//!    静默换字体）。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// 字体内容身份（blake3 前 16 字节）。
pub type FontHash = [u8; 16];

/// 计算字体字节的内容身份。
pub fn font_hash(bytes: &[u8]) -> FontHash {
    let digest = blake3::hash(bytes);
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest.as_bytes()[..16]);
    out
}

fn registry() -> &'static Mutex<HashMap<FontHash, cosmic_text::fontdb::ID>> {
    static REGISTRY: OnceLock<Mutex<HashMap<FontHash, cosmic_text::fontdb::ID>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// fontdb ID → 内容身份（字体字节的哈希很贵：回退字体可达数十 MB，逐帧重算
/// 会把 produce 推到 10–25ms 级；这里进程内只算一次）。
fn id_to_hash() -> &'static Mutex<HashMap<cosmic_text::fontdb::ID, Option<FontHash>>> {
    static IDS: OnceLock<Mutex<HashMap<cosmic_text::fontdb::ID, Option<FontHash>>>> =
        OnceLock::new();
    IDS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 取（必要时计算并缓存）某面在宿主字体系统里的内容身份。
pub fn hash_for_font_id(
    system: &mut cosmic_text::FontSystem,
    id: cosmic_text::fontdb::ID,
    weight: cosmic_text::fontdb::Weight,
) -> Option<FontHash> {
    if let Some(cached) = id_to_hash().lock().unwrap().get(&id).copied() {
        return cached;
    }
    let hash = system.get_font(id, weight).map(|font| font_hash(font.data()));
    id_to_hash().lock().unwrap().insert(id, hash);
    hash
}

/// 登记一个已装载字体（重复登记幂等）。
pub fn register_loaded_font(bytes: &[u8], id: cosmic_text::fontdb::ID) {
    registry().lock().unwrap().insert(font_hash(bytes), id);
}

/// 把字体字节装入全局字体系统并登记其内容身份。
///
/// 走 iced 的 `FontSystem::load_font`（与产品装载同路，避免同一字体被登记
/// 两次影响字体匹配），再用装载前后的 face 差集取回新的 fontdb ID。
pub fn register_font_bytes(bytes: Vec<u8>) -> Option<cosmic_text::fontdb::ID> {
    let hash = font_hash(&bytes);
    if let Some(id) = registry().lock().unwrap().get(&hash).copied() {
        return Some(id);
    }
    let mut system = iced::advanced::graphics::text::font_system()
        .write()
        .expect("Write font system");
    let before: std::collections::HashSet<_> =
        system.raw().db_mut().faces().map(|f| f.id).collect();
    system.load_font(std::borrow::Cow::Owned(bytes));
    let new_id = system
        .raw()
        .db_mut()
        .faces()
        .find(|f| !before.contains(&f.id))
        .map(|f| f.id)?;
    drop(system);
    registry().lock().unwrap().insert(hash, new_id);
    Some(new_id)
}

/// 按内容身份取宿主侧 fontdb ID（None = 该字体未装载，调用方须具名降级）。
///
/// 未登记的哈希会**扫一遍本地 fontdb**（同一字体已装载但未显式登记的情形，
/// 例如系统回退字体）并缓存结果；找不到才是真的不可渲染。
pub fn font_id_for_hash(hash: &FontHash) -> Option<cosmic_text::fontdb::ID> {
    if let Some(id) = registry().lock().unwrap().get(hash).copied() {
        return Some(id);
    }
    let mut system = iced::advanced::graphics::text::font_system()
        .write()
        .expect("Write font system");
    let candidates: Vec<cosmic_text::fontdb::ID> = system.raw().db_mut().faces().map(|f| f.id).collect();
    for id in candidates {
        if let Some(font) = system.raw().get_font(id, cosmic_text::Weight::NORMAL) {
            if font_hash(font.data()) == *hash {
                drop(system);
                registry().lock().unwrap().insert(*hash, id);
                return Some(id);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_font_identity_is_content_addressed() {
        let bytes = crate::ui::iced::renderer::INTER_FONT_REGULAR.to_vec();
        let hash = font_hash(&bytes);
        let id = register_font_bytes(bytes.clone()).expect("装载字体");
        assert_eq!(font_id_for_hash(&hash), Some(id), "同字节必须解析到同一面");
        // 内容变 → 身份变（不同字节不得命中同一身份）。
        let mut mutated = bytes.clone();
        mutated[0] ^= 0xFF;
        assert_ne!(font_hash(&mutated), hash);
        // 幂等：重复登记不换 ID。
        assert_eq!(register_font_bytes(bytes).expect("重复装载"), id);
    }
}
