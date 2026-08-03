//! D2S items 段解析 — 魔改 layout 专用。
//!
//! **重要：这不是 D2SLib 标准 layout 解析器。**
//!
//! 真实 D2R v105 .d2s header layout (D2SLib D2S.cs):
//! ```text
//! 0x0000: Magic (u32)        = 0xAA55AA55
//! 0x0004: Version (u32)
//! 0x0008: Filesize (u32)
//! 0x000C: Checksum (u32)
//! 0x0010: ActiveWeapon (u32)
//! 0x0014: Name (UTF-16LE, 16 bytes)
//! ...
//! ```
//!
//! **本文件解析的魔改 layout** (xieedi.d2s 真实字节观察):
//! ```text
//! 0x0000: Magic (u32 LE)     = 0xAA55AA55  (LE: 55 aa 55 aa)
//! 0x0004: Version (u32)      = 0x69 (105)
//! 0x00F8: Items count (u16 LE) — 实测 3
//! 0x00FB: 4 bytes padding    — 0x37 0x73 0x38 0x20 (跳过)
//! 0x00FF: 0xff               — simple item 段 marker
//! 0x0100: 8 bytes flags      — 0x02 0x00 0x00 0x04 0x00 0x00 0x00 0x00
//! 0x0108: 12 bytes per item  — code[3] + 0x20 + 0xff + quality_byte + 7B stat_list 头
//! 0x012B: Name (UTF-8, terminated by 0x00) — "开心邪帝"
//! ```
//!
//! 魔改 layout 的 Status/ClassId/Level/Created/LastPlayed 全是 0xff/0x00 噪声,
//! 无真实意义。attributes 段不存在 / 无法解析。
//!
//! 这种 layout 看上去不像 D2R 客户端生成, 但 user 验证这是真实有效游戏存档。
//! 因此: parser 尽力提取 name + items code + quality 字节, attributes 全部填 0 占位。

use crate::core::ParseResult;
use crate::protocol::d2i::parser::ParsedItem;

/// 魔改 layout 关键偏移 (基于 xieedi.d2s / happy_manman.d2s 实测)。
///
/// **真实 layout**: 0xF8 处 u16 LE 字段 (实测=3) **不可靠** — 实际 items 段是
/// 0xFB 起始,stride=12,终止于 0x12B (name UTF-8 区)。
/// 装备数 = (0x12B - 0xFB) / 12。happy 野蛮人 (Barbarian) 实际 4 件 (双 lfw + uea + uap),
/// xieedi 术士 (Warlock) 实际 4 件 (7s8 + wae + xpl + uap)。
pub const MOD_ITEMS_COUNT_OFFSET: usize = 0xF8;
/// 魔改 layout 第一项起始 offset (count u16 之后 1 字节 padding, 然后 12B item[0])。
pub const MOD_FIRST_ITEM_OFFSET: usize = 0xFB;
/// 魔改 layout 名字起始 offset (items 终止边界)。
pub const MOD_NAME_OFFSET: usize = 0x12B;
/// 每个 item 的固定长度 (实测)。
pub const MOD_ITEM_LEN: usize = 12;
/// 魔改 layout class id (u8) 偏移 — 实测 xieedi.d2s = 7 (Warlock)
pub const MOD_CLASS_OFFSET: usize = 0x18;
/// 魔改 layout level (u8) 偏移 — 实测 xieedi.d2s = 116
pub const MOD_LEVEL_OFFSET: usize = 0x1B;
/// 魔改 layout created time (u32 LE) 偏移 — 实测 = 1948127239
pub const MOD_CREATED_OFFSET: usize = 0x18; // 同 class,实际是 0x18-0x1B u32

/// 魔改 layout 探测 — 检测是否是 xieedi/happy_manman 风格 layout。
///
/// 特征:
/// - magic 0xAA55AA55 (LE: 55 aa 55 aa) ✓
/// - version 0x69 = 105 ✓
/// - items count @ 0xF8 u16 LE = 3..=10
/// - 0x12B 处出现合法 UTF-8 名字（不再限定中文）
/// - 0xFB 起始的 12B item 块前几项结构自检通过
/// - 额外要求前 4 个 12B block 呈现“真实魔改装备画像”，避免标准 d2s 在
///   `0xFB..0x12B` 偶然长得像 12B 固定块时被误判成 modified layout
pub fn detect_modified_layout(data: &[u8]) -> bool {
    if data.len() < MOD_NAME_OFFSET + 6 {
        return false;
    }
    // magic + version
    if data[0..4] != [0x55, 0xaa, 0x55, 0xaa] {
        return false;
    }
    if u32::from_le_bytes([data[4], data[5], data[6], data[7]]) != 105 {
        return false;
    }
    // items count @ 0xF8 u16 LE = 3..=10
    let count = u16::from_le_bytes([data[MOD_ITEMS_COUNT_OFFSET], data[MOD_ITEMS_COUNT_OFFSET + 1]]);
    if !(3..=10).contains(&count) {
        return false;
    }
    // UTF-8 名字 @ 0x12B（兼容英文/中文/混合）
    if !has_valid_utf8_name(data, MOD_NAME_OFFSET) {
        return false;
    }
    if !has_valid_modified_item_block(data) {
        return false;
    }
    if !has_strong_modified_item_profile(data) {
        return false;
    }
    true
}

fn has_valid_utf8_name(data: &[u8], start: usize) -> bool {
    if start >= data.len() {
        return false;
    }
    let end = data[start..]
        .iter()
        .position(|b| *b == 0)
        .map(|idx| start + idx)
        .unwrap_or(data.len());
    if end <= start {
        return false;
    }
    let name_bytes = &data[start..end];
    if name_bytes.len() > 48 {
        return false;
    }
    let Ok(name) = std::str::from_utf8(name_bytes) else {
        return false;
    };
    let char_count = name.chars().count();
    char_count >= 2 && !name.chars().all(|ch| ch.is_control())
}

fn has_valid_modified_item_block(data: &[u8]) -> bool {
    let available = MOD_NAME_OFFSET.saturating_sub(MOD_FIRST_ITEM_OFFSET);
    let count = available / MOD_ITEM_LEN;
    if count < 3 {
        return false;
    }
    let sanity_count = count.min(3);
    for idx in 0..sanity_count {
        let off = MOD_FIRST_ITEM_OFFSET + idx * MOD_ITEM_LEN;
        if off + MOD_ITEM_LEN > data.len() {
            return false;
        }
        let code = &data[off..off + 3];
        if !code.iter().all(|b| (*b as char).is_ascii_alphanumeric()) {
            return false;
        }
        if data[off + 3] != 0x20 {
            return false;
        }
        let quality = data[off + 5];
        if !(1..=8).contains(&quality) {
            return false;
        }
    }
    true
}

fn has_strong_modified_item_profile(data: &[u8]) -> bool {
    let available = MOD_NAME_OFFSET.saturating_sub(MOD_FIRST_ITEM_OFFSET);
    let count = available / MOD_ITEM_LEN;
    if count < 2 {
        return false;
    }

    // 扫描前 count 个 item（最多 4 个），收集有效条目
    let mut valid_items = Vec::new();
    for idx in 0..count.min(4) {
        let off = MOD_FIRST_ITEM_OFFSET + idx * MOD_ITEM_LEN;
        if off + MOD_ITEM_LEN > data.len() {
            break;
        }
        let code = &data[off..off + 3];
        if !code.iter().all(|b| (*b as char).is_ascii_alphanumeric()) {
            break;
        }
        if data[off + 3] != 0x20 {
            break;
        }
        let quality = data[off + 5];
        if !(1..=8).contains(&quality) {
            break;
        }
        let payload_nonzero = data[off + 6..off + 12].iter().any(|b| *b != 0);
        valid_items.push((quality, payload_nonzero));
    }

    if valid_items.len() < 2 {
        return false;
    }

    // 新角色（level 2-3）可能只有 2 件 normal/low quality 物品，
    // raw data 全 0（如 开心图书馆长.d2s: dgr/buc）。降低门槛：
    // 只要有 >= 2 个有效条目，且至少有 1 个是非 magic 品质即可。
    let non_magic_count = valid_items.iter().filter(|(q, _)| *q != 4).count();
    let nonzero_payload_count = valid_items.iter().filter(|(_, nz)| *nz).count();

    // 宽松条件（2 items 新角色）：至少 1 个非 magic 品质
    // 严格条件（4 items 老角色）：>= 2 非 magic + >= 2 nonzero payload
    if valid_items.len() <= 2 {
        non_magic_count >= 1
    } else {
        non_magic_count >= 2 && nonzero_payload_count >= 2
    }
}

/// 读取 UTF-8 名字从魔改 layout 起始 offset。
fn read_utf8_name(data: &[u8], start: usize) -> Option<String> {
    if start >= data.len() {
        return None;
    }
    // UTF-8 字符串以 0x00 终止
    let mut end = start;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    std::str::from_utf8(&data[start..end]).ok().map(String::from)
}

/// 读取 class id (Warlock=7, Assassin=6, Necromancer=2, ...) — 魔改 layout 单字节 @ 0x18。
///
/// 返回 None 如果越界或魔改 layout 不匹配。
pub fn read_class_modified(data: &[u8]) -> Option<u8> {
    if data.len() <= MOD_CLASS_OFFSET {
        return None;
    }
    Some(data[MOD_CLASS_OFFSET])
}

/// 读取 level (1-200 范围,支持 100+ 通关后等级) — 魔改 layout 单字节 @ 0x1B。
pub fn read_level_modified(data: &[u8]) -> Option<u8> {
    if data.len() <= MOD_LEVEL_OFFSET {
        return None;
    }
    let lv = data[MOD_LEVEL_OFFSET];
    if lv == 0 || lv == 0xFF {
        return None;
    }
    Some(lv)
}

/// 读取 created time (u32 LE Unix seconds) — 魔改 layout @ 0x18(与 class 共享 4B)。
///
/// ⚠️ 与 class 字段位置重叠 — 如果 class 是单字节,这个 u32 包含 class + 3 字节 created time 头。
/// 实际语义不明,xieedi.d2s 实测 = 1948127239 (unix seconds)。
pub fn read_created_modified(data: &[u8]) -> Option<u32> {
    if data.len() < MOD_CREATED_OFFSET + 4 {
        return None;
    }
    Some(u32::from_le_bytes([
        data[MOD_CREATED_OFFSET],
        data[MOD_CREATED_OFFSET + 1],
        data[MOD_CREATED_OFFSET + 2],
        data[MOD_CREATED_OFFSET + 3],
    ]))
}

/// 解析 items 段 (魔改 layout 12B 固定结构,从 0xFB 起始,stride=12,直到 0x12B name 区)。
///
/// 装备数 = (0x12B - 0xFB) / 12。count @ 0xF8 字段不可靠(实测 happy=3 但实际有 4 件 lfw/uea/uap/lfw 双持)。
/// 双持武器 (e.g. happy 野蛮人 双 lfw) 是合法 — Barbarian 可双手各一把。
///
/// 输入: 完整 .d2s 字节流。
/// 输出: 找到的 item codes (3-char ASCII) 列表。
pub fn read_items_modified(data: &[u8]) -> Vec<String> {
    if data.len() < MOD_FIRST_ITEM_OFFSET + MOD_ITEM_LEN {
        return Vec::new();
    }
    let available = MOD_NAME_OFFSET.saturating_sub(MOD_FIRST_ITEM_OFFSET);
    let count = available / MOD_ITEM_LEN;
    let mut codes = Vec::with_capacity(count);
    let mut off = MOD_FIRST_ITEM_OFFSET;
    for _ in 0..count {
        if off + MOD_ITEM_LEN > data.len() {
            break;
        }
        // 3-char code (lowercase or digit) + 0x20 (D2SLib 4-char code padding)
        if !data[off].is_ascii_alphanumeric()
            || !data[off + 1].is_ascii_alphanumeric()
            || !data[off + 2].is_ascii_alphanumeric()
            || data[off + 3] != 0x20
        {
            break;
        }
        let code = std::str::from_utf8(&data[off..off + 3])
            .unwrap_or("")
            .to_string();
        codes.push(code);
        off += MOD_ITEM_LEN;
    }
    codes
}

/// 解析 items 段 (魔改 layout 12 字节固定结构)。
///
/// 字节布局 (实测):
/// ```text
/// [+0..+2]  code (3-char ASCII alnum, e.g. "uap" / "7s8")
/// [+3]      0x20 (pad)
/// [+4]      i_lvl (u8)
/// [+5]      quality (u8, D2SLib enum)
/// [+6..+11] 6 字节数据 (LE u16 主数值 @ [0..2])
/// ```
///
/// 装备数 = (MOD_NAME_OFFSET - MOD_FIRST_ITEM_OFFSET) / MOD_ITEM_LEN。
/// 不依赖 count @ 0xF8 字段 (实测 happy=3 但实际 4 件 — Barbarian 双持 lfw)。
///
/// 返回完整 ModifiedItem 列表 (含 i_lvl + quality + raw_data)。
pub fn read_items_with_quality(data: &[u8]) -> Vec<ModifiedItem> {
    if data.len() < MOD_FIRST_ITEM_OFFSET + MOD_ITEM_LEN {
        return Vec::new();
    }
    let available = MOD_NAME_OFFSET.saturating_sub(MOD_FIRST_ITEM_OFFSET);
    let count = available / MOD_ITEM_LEN;
    let mut items = Vec::with_capacity(count);
    let mut off = MOD_FIRST_ITEM_OFFSET;
    for _ in 0..count {
        if off + MOD_ITEM_LEN > data.len() {
            break;
        }
        if !data[off].is_ascii_alphanumeric()
            || !data[off + 1].is_ascii_alphanumeric()
            || !data[off + 2].is_ascii_alphanumeric()
            || data[off + 3] != 0x20
        {
            break;
        }
        let code_str = std::str::from_utf8(&data[off..off + 3]).unwrap_or("");
        let code = code_str.to_string();
        let i_lvl = data[off + 4];
        let quality_byte = data[off + 5];
        let mut raw_data = [0u8; 6];
        raw_data.copy_from_slice(&data[off + 6..off + 12]);
        let unique_id = if quality_byte == 7 {
            Some(u16::from_le_bytes([raw_data[0], raw_data[1]]))
        } else {
            None
        };
        let set_id = if quality_byte == 5 {
            Some(u16::from_le_bytes([raw_data[0], raw_data[1]]))
        } else {
            None
        };
        // rare/crafted: raw_data[0]=rare_name1, raw_data[1]=rare_name2 (single bytes)
        let rare_name1 = if quality_byte == 6 || quality_byte == 8 {
            Some(raw_data[0])
        } else {
            None
        };
        let rare_name2 = if quality_byte == 6 || quality_byte == 8 {
            Some(raw_data[1])
        } else {
            None
        };
        if quality_byte == 7 {
            log::debug!("[items_modified] {} unique_id={}", code, unique_id.unwrap_or(0));
        }
        if quality_byte == 5 {
            log::debug!("[items_modified] {} set_id={}", code, set_id.unwrap_or(0));
        }
        if quality_byte == 6 || quality_byte == 8 {
            log::debug!("[items_modified] {} rare={}/{}", code, rare_name1.unwrap_or(0), rare_name2.unwrap_or(0));
        }
        items.push(ModifiedItem {
            slot: code_to_slot_static(&code).map(String::from),
            code,
            i_lvl,
            quality_byte,
            raw_data,
            unique_id,
            set_id,
            rare_name1,
            rare_name2,
            socketed: false,
        });
        off += MOD_ITEM_LEN;
    }
    items
}

/// 从 .d2s 数据提取名字 (魔改 layout)。返回 None 如果不是魔改 layout。
pub fn read_name_modified(data: &[u8]) -> Option<String> {
    if !detect_modified_layout(data) {
        return None;
    }
    read_utf8_name(data, MOD_NAME_OFFSET)
}

/// 把 item code 映射到 8 装备位 (helm/amulet/armor/weapon/shield/gloves/boots/belt)。
///
/// 实现策略: 用具体 code 集合查表 — `game_items::ALL_ITEMS` 没有 slot 元数据,
/// 所以这里按游戏常识硬编码每个 slot 的 code 集合。fallback 到 game_items 的
/// is_armor/is_weapon/is_shield 元组区分 armor/weapon/shield。
///
/// helm 集合 (示例 — 不是完整列表):
/// uap xap uh9 uhm uhl ukp uhx uhg xhm xhl xh9 bhm ghm hlm cap bhm ...
/// boots 集合: uvb ulb umb xhb xtb lbt mbt hbt tbt uhb uvb ulb umb ...
/// gloves 集合: uvg uhg utg xhg lgl tgl vgl mgl hgl ...
/// belt 集合: uvc ulc umc uhc lbl mbl tbl ...
pub fn code_to_slot(code: &str) -> Option<&'static str> {
    code_to_slot_static(code)
}

/// 内部查表函数。
fn code_to_slot_static(code: &str) -> Option<&'static str> {
    if code == "amu" {
        return Some("amulet");
    }
    if crate::protocol::d2i::legacy::game_data_loader::is_grimoire_offhand(code) {
        return Some("shield");
    }
    let bytes = code.as_bytes();
    if bytes.len() != 3 {
        return None;
    }
    // 优先按 D2R 装备 type 第二字符分类 (D2SLib BodyLoc 简化)
    // 第 2 位 'h' 通常是 helm (xhm/xhl/xh9/uhm/uh9/uap/xap/cap/...)
    // 第 2 位 'b' 通常是 boots (xhb/xtb/lbt/...)
    // 第 2 位 'g' 通常是 gloves (xhg/lgl/tgl/...)
    // 第 2 位 'c' 通常是 belt (lbl/mbl/tbl/... uvc/ulc/uhc 是 armor 类)
    // 例外: 'w' 开头 + 'h' 是 weapon (whm War Hammer)
    //       'b' 开头 + 'h' 不存在
    //       'u' 开头 + 'h' = helm (uhm/uh9/uhg 是 gloves!)
    // 关键陷阱: uhg (Ogre Gauntlets) 第 2 位 'h' 但 实际是 gloves
    //          uhb (Myrmidon Greaves) 第 2 位 'h' 但 实际是 boots
    //          uvc/ulc/uhc 是 belt 但 is_armor=true
    //          whm 是 weapon
    //
    // 实战策略: **不靠 prefix**,改靠 **完整 code 集合查表** + game_items 元组 fallback。

    // Helm 完整集合 (来自 D2R Helm.txt 等的常见 3-char code):
    const HELM_CODES: &[&str] = &[
        "uap", "xap", "uh9", "uhm", "uhl", "ukp", "uhx", "xhm", "xhl", "xh9",
        "bhm", "ghm", "hlm", "cap", "skp", "fhl", "gth", "ghm", "bhm",
    ];
    if HELM_CODES.contains(&code) {
        return Some("helm");
    }
    // Boots 完整集合:
    const BOOTS_CODES: &[&str] = &[
        "uvb", "ulb", "umb", "xhb", "xtb", "lbt", "mbt", "hbt", "tbt",
        "uhb",
    ];
    if BOOTS_CODES.contains(&code) {
        return Some("boots");
    }
    // Gloves 完整集合:
    const GLOVES_CODES: &[&str] = &[
        "uvg", "uhg", "utg", "xhg", "lgl", "tgl", "vgl", "mgl", "hgl",
    ];
    if GLOVES_CODES.contains(&code) {
        return Some("gloves");
    }
    // Belt 完整集合:
    const BELT_CODES: &[&str] = &[
        "uvc", "ulc", "umc", "uhc", "lbl", "mbl", "tbl", "vbl",
    ];
    if BELT_CODES.contains(&code) {
        return Some("belt");
    }
    // 落回 game_items 区分 armor/weapon/shield
    use crate::protocol::d2i::legacy::game_items;
    for entry in game_items::ALL_ITEMS {
        if entry.0 == code {
            return if entry.4 {
                Some("shield")
            } else if entry.3 && !entry.2 {
                Some("weapon")
            } else if entry.2 {
                if matches!(code, "whm" | "wnd" | "wsp" | "wsc" | "wst" | "wsd" | "wax" | "wrb") {
                    // War Hammer, Wand, War Scepter, War Scythe, War Staff, War Sword, War Axe, Wrist Blade
                    // game_items 标 is_weapon=true AND is_armor=true? 不一定,但都是 weapon
                    Some("weapon")
                } else {
                    Some("armor")
                }
            } else {
                None
            };
        }
    }
    None
}

/// 魔改 layout 提取的 item (含完整 12 字节字段)。
///
/// 字节布局 (实测 xieedi.d2s / happy_manman.d2s,stride=12):
/// ```text
/// [+0..+2]  code    : 3-char ASCII (e.g. "uap"=Warlock helm, "wae"=Necro wand)
/// [+3]      pad     : 0x20 (D2SLib 4-char code padding)
/// [+4]      i_lvl   : 物品等级 (u8, 0xff=legendary/special, 70-99=普通装备)
/// [+5]      quality : D2SLib quality enum (1=low, 2=normal, 3=superior, 4=magic, 5=set, 6=rare, 7=unique, 8=crafted)
/// [+6..+11] raw_data: 6 字节 (LE u16 @ [0..2] = 主数值, 如 defense 0x0090=144)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifiedItem {
    pub code: String,
    /// 装备位 (helm/amulet/armor/weapon/shield/gloves/boots/belt)
    pub slot: Option<String>,
    /// 物品等级 (0xff = legendary, 0 = 未知,其他 = 普通)
    pub i_lvl: u8,
    /// 原始 quality 字节 (D2SLib quality enum: 1=low, 2=normal, 3=superior, 4=magic, 5=set, 6=rare, 7=unique, 8=crafted)
    pub quality_byte: u8,
    /// 剩余 6 字节原始数据 (LE u16 @ [0..2] = 主数值, 如 defense 0x0090=144;
    /// 对 unique(7) raw_data[0..2] LE = unique_id; 对 set(5) = set_id)
    pub raw_data: [u8; 6],
    /// 从 raw_data 解析的 unique_id (仅 quality=7 有效)
    pub unique_id: Option<u16>,
    /// 从 raw_data 解析的 set_id (仅 quality=5 有效)
    pub set_id: Option<u16>,
    /// 从 raw_data[0] 解析的 rare_name1 (仅 quality=6/8 有效)
    pub rare_name1: Option<u8>,
    /// 从 raw_data[1] 解析的 rare_name2 (仅 quality=6/8 有效)
    pub rare_name2: Option<u8>,
    /// 是否有孔。逆向策略注:此段是从 xieedi.d2s / happy_manman.d2s 等
    /// 几个样本里按固定 12 字节 stride 硬编码读出的字段布局 (逆向期间
    /// 的开发假设,而非 D2SLib 标准协议)。socket 信息在该 stride 里
    /// 没对应位置,所以默认为 false —— 真实 .d2s 标准段 (D2SLib JM
    /// bit-stream) 上 socket 用 stat 188 + ItemStat 表达,跟这里是
    /// 两条不同路径,不要混用。
    pub socketed: bool,
}

impl ModifiedItem {
    /// 把 raw quality 字节转字符串 (用于前端 EquipmentPanel)。
    pub fn quality_name(&self) -> &'static str {
        match self.quality_byte {
            1 => "low",
            2 => "normal",
            3 => "superior",
            4 => "magic",
            5 => "set",
            6 => "rare",
            7 => "unique",
            8 => "crafted",
            _ => "unknown",
        }
    }
}

/// 构造一个极简 ParsedItem-like 结构 (本 parser 不复用 d2i JM 解析,
/// 只提取 code — UI 只需要 code + name + slot + quality 即可)。
///
/// 完整 ParsedItem 需要 d2i JM 解析, 那条路对魔改 layout 不适用。
pub fn read_modified_items_full(data: &[u8]) -> ParseResult<Vec<ParsedItem>> {
    // 魔改 layout 没有完整 JM 编码 items — 只返回 code 占位。
    // ParsedItem 完整构造需要 d2i JM, 这里返回空 Vec (上层按 code 处理)。
    let _ = data;
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn test_detect_modified_layout_xieedi() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        assert!(detect_modified_layout(&data), "should detect xieedi.d2s");
    }

    #[test]
    fn test_detect_modified_layout_happy_manman() {
        let _fp = fixture_path("happy_manman.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture happy_manman.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        assert!(detect_modified_layout(&data), "should detect happy_manman.d2s");
    }

    #[test]
    fn test_detect_modified_layout_rejects_standard_tc03() {
        let _fp = fixture_path("standard_test_warlock_tc03.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture standard_test_warlock_tc03.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        assert!(
            !detect_modified_layout(&data),
            "标准 TC03 不应被识别为 modified layout"
        );
    }

    #[test]
    fn test_detect_modified_layout_with_ascii_name() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let mut data = std::fs::read(_fp).unwrap();
        let ascii_name = b"WarlockA\0";
        let end = MOD_NAME_OFFSET + ascii_name.len();
        data[MOD_NAME_OFFSET..end].copy_from_slice(ascii_name);
        assert!(detect_modified_layout(&data), "ASCII name should still be recognized as modified layout");
    }

    #[test]
    fn test_detect_modified_layout_rejects_invalid_utf8_name() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let mut data = std::fs::read(_fp).unwrap();
        data[MOD_NAME_OFFSET..MOD_NAME_OFFSET + 3].copy_from_slice(&[0xff, 0xfe, 0x00]);
        assert!(
            !detect_modified_layout(&data),
            "invalid UTF-8 name should not be recognized as modified layout"
        );
    }

    #[test]
    fn test_detect_modified_layout_rejects_corrupt_item_block() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let mut data = std::fs::read(_fp).unwrap();
        data[MOD_FIRST_ITEM_OFFSET + 3] = 0x00;
        assert!(
            !detect_modified_layout(&data),
            "corrupt 12-byte item block should fail modified layout detection"
        );
    }

    #[test]
    fn test_detect_modified_layout_rejects_out_of_range_count() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let mut data = std::fs::read(_fp).unwrap();
        data[MOD_ITEMS_COUNT_OFFSET..MOD_ITEMS_COUNT_OFFSET + 2]
            .copy_from_slice(&2u16.to_le_bytes());
        assert!(
            !detect_modified_layout(&data),
            "out-of-range modified item count should fail detection"
        );
    }

    #[test]
    fn test_read_name_xieedi() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let name = read_name_modified(&data);
        println!("name = {:?}", name);
        assert_eq!(name, Some("开心邪帝".to_string()));
    }

    #[test]
    fn test_read_items_xieedi() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let codes = read_items_modified(&data);
        println!("codes = {:?}", codes);
        // xieedi (Warlock 116): 0xFB 起始 4 件
        // [0] 7s8 (Thresher 长柄, 0x02=normal?) [1] wae (Necro grim, unique) [2] xpl (armor, set) [3] uap (Shako helm, unique)
        assert_eq!(codes, vec!["7s8", "wae", "xpl", "uap"]);
    }

    #[test]
    fn test_read_items_with_quality_xieedi() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let items = read_items_with_quality(&data);
        println!("items = {:#?}", items);
        assert_eq!(items.len(), 4, "xieedi 应有 4 件装备 (0xFB 起始)");
        // [0] 7s8 Thresher 长柄 (weapon, iLvl=255, normal=0x02)
        assert_eq!(items[0].code, "7s8");
        assert_eq!(items[0].slot.as_deref(), Some("weapon"));
        assert_eq!(items[0].quality_byte, 0x02);
        assert_eq!(items[0].i_lvl, 0xff, "7s8 iLvl=255 (legendary)");
        // [1] wae Blasphemous Compendium (Necro grim, unique)
        assert_eq!(items[1].code, "wae");
        assert_eq!(items[1].slot.as_deref(), Some("shield"));
        assert_eq!(items[1].quality_byte, 7);
        // [2] xpl Russet Armor (armor, set)
        assert_eq!(items[2].code, "xpl");
        assert_eq!(items[2].slot.as_deref(), Some("armor"));
        assert_eq!(items[2].quality_byte, 5);
        // [3] uap Shako helm (unique)
        assert_eq!(items[3].code, "uap");
        assert_eq!(items[3].slot.as_deref(), Some("helm"));
        assert_eq!(items[3].quality_byte, 7);
    }

    #[test]
    fn test_code_to_slot() {
        // 实测 xieedi 三件
        assert_eq!(code_to_slot("uap"), Some("helm"));
        assert_eq!(code_to_slot("xpl"), Some("armor"));
        assert_eq!(code_to_slot("wae"), Some("shield"));
        // 常见其他装备
        assert_eq!(code_to_slot("amu"), Some("amulet"));
        // helm 系列
        assert_eq!(code_to_slot("uhm"), Some("helm"));
        assert_eq!(code_to_slot("uh9"), Some("helm"));
        assert_eq!(code_to_slot("uhb"), Some("boots")); // uhb = Myrmidon Greaves (boots)
        // boots
        assert_eq!(code_to_slot("uvb"), Some("boots"));
        assert_eq!(code_to_slot("xtb"), Some("boots"));
        // gloves
        assert_eq!(code_to_slot("uvg"), Some("gloves"));
        assert_eq!(code_to_slot("uhg"), Some("gloves"));
        // belt
        assert_eq!(code_to_slot("uvc"), Some("belt"));
        assert_eq!(code_to_slot("uhc"), Some("belt"));
        // weapon
        assert_eq!(code_to_slot("2ax"), Some("weapon"));
        assert_eq!(code_to_slot("whm"), Some("weapon")); // War Hammer 第 2 位 h 但第 1 位 w → weapon
        // shield (is_shield=true)
        assert_eq!(code_to_slot("uit"), Some("shield"));
        // 未知
        assert_eq!(code_to_slot("xxx"), None);
        // 不存在
        assert_eq!(code_to_slot("zzz"), None);
    }

    /// 完整 12 字节字段测试: xieedi.d2s (Warlock 116)
    /// 4 件 (0xFB 起始):
    /// - [0] 7s8 Thresher 长柄 (weapon, iLvl=2, normal)
    /// - [1] wae Blasphemous Compendium (Necro grim, unique, iLvl=255)
    /// - [2] xpl Russet Armor (armor, set, iLvl=255)
    /// - [3] uap Shako helm (unique, iLvl=77, defense=248)
    #[test]
    fn test_read_items_full_fields_xieedi() {
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let items = read_items_with_quality(&data);
        assert_eq!(items.len(), 4, "xieedi 应有 4 件装备 (0xFB 起始)");

        // [0] 7s8 = Thresher (weapon, normal, iLvl=2)
        let it0 = &items[0];
        assert_eq!(it0.code, "7s8");
        assert_eq!(it0.slot.as_deref(), Some("weapon"));
        assert_eq!(it0.quality_byte, 0x02, "7s8 quality=normal");
        assert_eq!(it0.i_lvl, 0xff, "7s8 iLvl=255");
        let val0 = u16::from_le_bytes([it0.raw_data[0], it0.raw_data[1]]);
        // 7s8 raw_data = 00 00 04 00 00 00 → LE u16 = 0
        assert_eq!(val0, 0x0000, "7s8 stat LE u16=0");

        // [1] wae = Blasphemous Compendium (Necro grim, unique, iLvl=255)
        let it1 = &items[1];
        assert_eq!(it1.code, "wae");
        assert_eq!(it1.slot.as_deref(), Some("shield"));
        assert_eq!(it1.quality_byte, 7, "wae quality=unique");
        assert_eq!(it1.i_lvl, 0xff, "wae iLvl=255 (legendary)");
        let val1 = u16::from_le_bytes([it1.raw_data[0], it1.raw_data[1]]);
        assert_eq!(val1, 0x0199, "wae stat 0x0199=409");

        // [2] xpl = Russet Armor (armor, set, iLvl=255)
        let it2 = &items[2];
        assert_eq!(it2.code, "xpl");
        assert_eq!(it2.slot.as_deref(), Some("armor"));
        assert_eq!(it2.quality_byte, 5, "xpl quality=set");
        assert_eq!(it2.i_lvl, 0xff);
        let val2 = u16::from_le_bytes([it2.raw_data[0], it2.raw_data[1]]);
        assert_eq!(val2, 0x0088, "xpl stat 0x0088=136");

        // [3] uap = Shako helm (unique, iLvl=77, defense=248)
        let it3 = &items[3];
        assert_eq!(it3.code, "uap");
        assert_eq!(it3.slot.as_deref(), Some("helm"));
        assert_eq!(it3.quality_byte, 7);
        assert_eq!(it3.i_lvl, 0x4d, "uap iLvl=77");
        let val3 = u16::from_le_bytes([it3.raw_data[0], it3.raw_data[1]]);
        assert_eq!(val3, 0x00f8, "uap stat 0x00f8=248 (defense)");

        // quality_name() 字符串转换
        assert_eq!(it1.quality_name(), "unique");
        assert_eq!(it2.quality_name(), "set");
        assert_eq!(it3.quality_name(), "unique");
    }

    /// 完整 12 字节字段测试: happy_manman.d2s (Barbarian 93, 双持武器)
    /// 4 件 (0xFB 起始):
    /// - [0] lfw Thunderfury (weapon, set, iLvl=255) — 左手
    /// - [1] lfw Thunderfury (weapon, set, iLvl=255) — 右手 (双持 Barbarian)
    /// - [2] uea Wyrmhide (armor, unique, iLvl=70)
    /// - [3] uap Shako (helm, unique, iLvl=77)
    #[test]
    fn test_read_items_full_fields_happy_manman() {
        let _fp = fixture_path("happy_manman.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture happy_manman.d2s 缺失"); return; }
        let _fp2 = fixture_path("xieedi.d2s");
        if !_fp2.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let items = read_items_with_quality(&data);
        assert_eq!(items.len(), 4, "happy_manman 应有 4 件装备 (双持 lfw + uea + uap)");

        // [0] lfw = Thunderfury 左手
        let it0 = &items[0];
        assert_eq!(it0.code, "lfw");
        assert_eq!(it0.slot.as_deref(), Some("weapon"));
        assert_eq!(it0.quality_byte, 5, "lfw quality=set");
        assert_eq!(it0.i_lvl, 0xff);
        let val0 = u16::from_le_bytes([it0.raw_data[0], it0.raw_data[1]]);
        assert_eq!(val0, 0x0090, "lfw stat 0x0090=144");

        // [1] lfw = Thunderfury 右手 (双持, Barbarian 可双手各一把)
        let it1 = &items[1];
        assert_eq!(it1.code, "lfw");
        assert_eq!(it1.slot.as_deref(), Some("weapon"));
        assert_eq!(it1.quality_byte, 5);
        let val1 = u16::from_le_bytes([it1.raw_data[0], it1.raw_data[1]]);
        // happy [0] 和 [1] 是同一件 lfw,内容应一致
        assert_eq!(val1, val0, "双持 lfw 字节一致");

        // [2] uea = Wyrmhide (armor, unique, iLvl=70)
        let it2 = &items[2];
        assert_eq!(it2.code, "uea");
        assert_eq!(it2.slot.as_deref(), Some("armor"));
        assert_eq!(it2.quality_byte, 7);
        assert_eq!(it2.i_lvl, 0x46, "uea iLvl=70");
        let val2 = u16::from_le_bytes([it2.raw_data[0], it2.raw_data[1]]);
        assert_eq!(val2, 0x00d2, "uea stat 0x00d2=210");

        // [3] uap = Shako (unique, mod 模板 — 与 xieedi uap 字节完全相同)
        let it3 = &items[3];
        assert_eq!(it3.code, "uap");
        assert_eq!(it3.slot.as_deref(), Some("helm"));
        assert_eq!(it3.i_lvl, 0x4d);
        let val3 = u16::from_le_bytes([it3.raw_data[0], it3.raw_data[1]]);
        assert_eq!(val3, 0x00f8);

        // 跨文件: 同一 uap 模板字节级一致
        let xieedi_uap = &read_items_with_quality(
            &std::fs::read(fixture_path("xieedi.d2s")).unwrap()
        )[3];
        assert_eq!(it3.raw_data, xieedi_uap.raw_data, "uap 模板跨文件一致");
        assert_eq!(it3.quality_byte, xieedi_uap.quality_byte);
    }

    /// 边距测试: count 越界 / buffer 短应返回空 Vec 不 panic。
    #[test]
    fn test_read_items_truncated_buffer() {
        let empty = read_items_with_quality(&[]);
        assert!(empty.is_empty());
        let short = read_items_with_quality(&[0u8; 10]);
        assert!(short.is_empty());
        // 长度够 count 字段但 item 区域短: 早停
        let mut partial = vec![0u8; 0x108]; // 0xF8..0xFA = count
        partial[0xF8] = 10; // count=10 但 buffer 只到 0x108(1 项也装不下)
        partial[0xFA] = 0;
        let partial_items = read_items_with_quality(&partial);
        assert!(partial_items.is_empty(), "buffer 太短应早停");
    }

    #[test]
    fn test_scan_class_xieedi() {
        // xieedi.d2s 装备 Warlock "书" wae → class = 7 (Warlock, D2SLib enum after Assassin=6)
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let class = read_class_modified(&data);
        println!("class = {:?}", class);
        assert_eq!(class, Some(7), "xieedi 应该识别为 Warlock (class=7)");
    }

    #[test]
    fn test_scan_level_xieedi() {
        // xieedi.d2s 等级 116 (实测 @ offset 0x1B)
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let level = read_level_modified(&data);
        println!("level = {:?}", level);
        assert_eq!(level, Some(116), "xieedi 应该识别为 level 116");
    }

    #[test]
    fn test_read_created_xieedi() {
        // xieedi.d2s created time (实测 1948127239)
        let _fp = fixture_path("xieedi.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture xieedi.d2s 缺失"); return; }
        let data = std::fs::read(_fp).unwrap();
        let created = read_created_modified(&data);
        println!("created = {:?}", created);
        assert_eq!(created, Some(1948127239));
    }
}
