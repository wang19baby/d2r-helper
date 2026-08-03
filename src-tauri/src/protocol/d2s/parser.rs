//! D2S 解析（角色存档）。
//!
//! 按**语义容器**组织解析结果,不再以 JM/IF/GF/KF marker 为对外字段。
//! 协议上 d2s 一定有 5 个物品容器(equipped / belt / backpack / cube / merc),
//! 实际文件可能不含全部 (例如新角色没装备/没佣兵),采用 best-effort 解析:
//! 没找到就空 vec,不报错。

use memchr::memchr_iter;
use serde::{Deserialize, Serialize};
use crate::core::ParseResult;
use crate::core::bitio::BitReader;
use crate::data::stat_cost::build_stat_table;
use crate::protocol::d2s::attributes::CharacterAttributes;
use crate::protocol::d2s::header::D2SHeader;
use crate::protocol::d2s::ATTRIBUTES_OFFSET;
use crate::protocol::d2i::parser::ParsedItem;

/// 当前已确认的标准 D2R item 位布局。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownItemBitLayout {
    pub location_id_bit_offset: u8,
    pub equipped_slot_bit_offset: u8,
    pub huffman_code_bit_offset: u8,
    pub socket_count_bits: u8,
    pub uid_bits: u8,
    pub ilvl_bits: u8,
    pub quality_bits: u8,
    pub stat_terminator: u16,
}

/// 单条技能数据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEntry {
    /// skill_id（per-class 0-based index）
    pub id: u16,
    pub level: u16,
}

/// 三难度小站标记。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WaypointSet {
    pub normal: Vec<bool>,
    pub nightmare: Vec<bool>,
    pub hell: Vec<bool>,
}

/// 单条任务进度摘要（与前端 QuestDisplay 兼容）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestEntry {
    pub difficulty: u8, // 0=normal, 1=nightmare, 2=hell
    pub act: u8,        // 1-5
    pub quest_id: u8,   // quest within act
    pub completed: bool,
}

/// Woo! 段提取的任务关键信息（progression + 原始 quest 位掩码）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WooQuestData {
    /// progression 字节 (difficulty × 5 + act 进度索引)
    pub progression: u8,
    /// 3 难度 × 5 act × quest 位掩码 uint16
    pub difficulties: Vec<Vec<Vec<u16>>>,
}

/// w4 段：NPC 对话及奖励消费状态。
/// 3 难度 × 2 类 (dialog + reward_consumed), 每类 8 字节位掩码。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct W4DialogData {
    pub block_type: u8,
    pub normal: u64,
    pub normal_extra: u64,
    pub nightmare: u64,
    pub nightmare_extra: u64,
    pub hell: u64,
    pub hell_extra: u64,
}

/// 角色存档按语义容器组织的解析结果。
///
/// 5 个物品容器各自 best-effort 解析:找不到对应的二进制段
/// (例如新角色没雇佣佣兵,或装备栏空)就返回空 Vec,不视为错误。
///
/// 字段顺序按"角色面板从左到右"的视觉:equipped → belt → backpack → cube → merc。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct D2SCharacter {
    pub header: D2SHeader,
    pub attributes: CharacterAttributes,
    pub skills: Vec<u8>,
    pub skills_decoded: Vec<SkillEntry>,
    pub waypoints: WaypointSet,
    /// Woo! 段任务数据（替代旧 kf 段 quest 解析）
    pub woo: WooQuestData,
    /// w4 NPC 对话/奖励消费数据
    pub w4: W4DialogData,

    /// 已装备物品 (ItemMode::Equipped, bodyloc 1..=12)
    pub equipped: Vec<ParsedItem>,
    /// 腰带物品 (ItemMode::Belt)
    pub belt: Vec<ParsedItem>,
    /// 背包物品 (ItemMode::Stored, storage=1)
    pub backpack: Vec<ParsedItem>,
    /// 储物箱物品 (storage=4)
    pub cube: Vec<ParsedItem>,
    /// 个人仓库物品 (ItemMode::Stored, Page=MyStash(5),d2s 文件 JM 段末尾)
    pub personal_stash: Vec<ParsedItem>,
    /// 雇佣兵装备 (storage=0 bodyloc 1..=12 但在第二个 JM block 中)
    pub merc: Vec<ParsedItem>,
}

impl D2SCharacter {
    /// 顶层入口 (标准 D2SLib layout)。
    ///
    /// 必须能解析的是 header 与 attributes,其余全部 best-effort:
    /// skills/waypoints/quests 找不到对应段就空 Vec;
    /// 5 个物品容器各自调对应的 parse_* 函数。
    pub fn parse(buffer: &[u8]) -> ParseResult<Self> {
        let header = D2SHeader::from_bytes(buffer)?;

        let attributes = if ATTRIBUTES_OFFSET + 2 <= buffer.len()
            && buffer[ATTRIBUTES_OFFSET..ATTRIBUTES_OFFSET + 2]
                == crate::protocol::d2s::attributes::ATTRIBUTES_HEADER
        {
            let table = if crate::data::stat_loader::has_runtime_table() {
                crate::data::stat_loader::build_runtime_table()
            } else {
                build_stat_table()
            };
            crate::protocol::d2s::attributes::parse_with_table(
                &buffer[ATTRIBUTES_OFFSET..],
                Some(&table),
            )?
        } else {
            CharacterAttributes::default()
        };

        let skills = crate::protocol::d2s::items::read_skills(buffer).unwrap_or_default();
        let skills_decoded = parse_skills(&skills);

        let m = marker_offsets(buffer);

        // WS 小站 (0x2BD, 80B) — 只检查标准偏移
        let waypoints = if buffer.len() >= 0x2D0 && &buffer[0x2BD..0x2BF] == b"WS" {
            parse_ws_waypoints(buffer, 0x2BD)
        } else if let Some(jf) = m.jf {
            // 回退: 从 jf 段读 (兼容旧文件/最小文件)
            parse_waypoints_from_jf(buffer, jf)
        } else {
            WaypointSet::default()
        };
        // Woo! 任务段 (0x193, 298B) — 检查标准偏移
        let woo = if buffer.len() >= 0x2C0
            && &buffer[0x193..0x197] == b"Woo!"
        {
            parse_woo(buffer, 0x193)
        } else {
            WooQuestData::default()
        };
        // w4 NPC 对话段 (0x30E, 51B) — 检查标准偏移
        let _w4 = if buffer.len() >= 0x340 && &buffer[0x30E..0x310] == b"w4" {
            parse_woo(buffer, 0x193)
        } else {
            WooQuestData::default()
        };
        // w4 NPC 对话段 (0x30E, 51B)
        let w4 = if let Some(w4_off) = find_marker(buffer, b"w4") {
            parse_w4(buffer, w4_off)
        } else {
            W4DialogData::default()
        };

        // 优先用 items 标准解析器（有 stat 解析），失败再 fallback 到 items_modified
        let all_stored = {
            let std_items = crate::protocol::d2s::items::read_standard_items(buffer).unwrap_or_default();
            if !std_items.is_empty() {
                std_items
            } else if crate::protocol::d2s::items_modified::detect_modified_layout(buffer) {
                crate::protocol::d2s::items_modified::read_items_with_quality(buffer)
                    .into_iter().map(|mi| {
                        use crate::protocol::common::{ItemQuality, ItemMode, ItemLocation, Item};
                        let quality = match mi.quality_byte {
                            0 => ItemQuality::None, 1 => ItemQuality::Low, 2 => ItemQuality::Normal,
                            3 => ItemQuality::Superior, 4 => ItemQuality::Magic, 5 => ItemQuality::Set,
                            6 => ItemQuality::Rare, 7 => ItemQuality::Unique, 8 => ItemQuality::Crafted,
                            v => ItemQuality::Unknown(v),
                        };
                        ParsedItem {
                            page_index: 0,
                            item: Item {
                                flags: Default::default(), version_raw: 105,
                                mode: ItemMode::Stored,
                                location: ItemLocation::None,
                                x: 0, y: 0, page: None,
                                code: mi.code.clone(), num_sockets: 0, id: 0,
                                item_level: mi.i_lvl, quality,
                                stat_lists: Vec::new(), amount: 0,
                                socketed_items: Vec::new(),
                                current_durability: 0, max_durability: 0,
                                defense: 0,
                                unique_id: mi.unique_id, set_id: mi.set_id,
                            },
                            raw_bit_offset: 0, raw_bit_length: 0,
                            is_socketed_subitem: false, is_pseudo_unverified: false,
                            magic_prefix_id: None, magic_suffix_id: None,
                        }
                    }).collect()
            } else {
                vec![]
            }
        };
        // 关联镶嵌物品到父物品
        let all_stored = crate::protocol::d2s::items::associate_socketed_items(&all_stored);
        let belt: Vec<_> = all_stored.iter().filter(|pi| pi.item.mode == crate::protocol::common::ItemMode::Belt).cloned().collect();
        let backpack: Vec<_> = all_stored.iter().filter(|pi| pi.item.mode == crate::protocol::common::ItemMode::Stored && pi.item.page == Some(crate::protocol::common::ItemPage::Backpack)).cloned().collect();
        let cube: Vec<_> = all_stored.iter().filter(|pi| pi.item.mode == crate::protocol::common::ItemMode::Stored && pi.item.page == Some(crate::protocol::common::ItemPage::Mod(4))).cloned().collect();
        // 个人仓库:Page=MyStash(5)。d2s 文件里只存当前存档的个人 stash,
        // 共享 stash 在 SharedStashSoftCoreV2.d2i,需要单独读。
        let personal_stash: Vec<_> = all_stored.iter().filter(|pi| pi.item.mode == crate::protocol::common::ItemMode::Stored && pi.item.page == Some(crate::protocol::common::ItemPage::MyStash)).cloned().collect();

        let equipped = crate::protocol::d2s::items::parse_equipped(buffer);
        // merc 物品也关联镶嵌物品（read_merc_items 不自动关联）
        let merc = crate::protocol::d2s::items::associate_socketed_items(
            &crate::protocol::d2s::items::read_merc_items(buffer),
        );

        Ok(Self {
            header,
            attributes,
            skills,
            skills_decoded,
            waypoints,
            woo,
            w4,
            equipped,
            belt,
            backpack,
            cube,
            personal_stash,
            merc,
        })
    }
}
/// 从 "if" 段解码技能列表。
///
/// 匹配 Python construct_adapter/d2s.py:
///   skill_raw = gf_raw[if_pos+2:]
///   skill_levels = list(skill_raw[:CLASS_SKILL_COUNT])  # 30 bytes
///   每个 byte 是 0-based per-class skill index 的等级。
pub fn parse_skills(skill_data: &[u8]) -> Vec<SkillEntry> {
    let count = skill_data.len().min(30);
    let mut skills = Vec::with_capacity(count);
    for (id, &level) in skill_data[..count].iter().enumerate() {
        if level > 0 {
            skills.push(SkillEntry { id: id as u16, level: level as u16 });
        }
    }
    skills
}

/// 从 jf 段解析小站标记 (兼容回退)。
pub fn parse_waypoints_from_jf(data: &[u8], jf_offset: usize) -> WaypointSet {
    let start = jf_offset + 4;
    if start + 15 > data.len() {
        return WaypointSet::default();
    }
    let mut reader = BitReader::new(&data[start..start + 15]);
    let mut all = Vec::with_capacity(120);
    for _ in 0..120 {
        all.push(reader.read_bit() != 0);
    }
    let hell: Vec<bool> = all.drain(80..).collect();
    let nightmare: Vec<bool> = all.drain(40..).collect();
    let normal: Vec<bool> = std::mem::take(&mut all);
    WaypointSet { normal, nightmare, hell }
}

/// 从 WS 段 (0x2BD) 解析小站标记。
///
/// D2SLib 格式: WS(2B) + version(4B) + length(2B) + 3 × 24B 难度块。
/// 每 24B 块: 2B header + ActI(9b) + ActII(9b) + ActIII(9b) + ActIV(3b) + ActV(9b)。
pub fn parse_ws_waypoints(data: &[u8], ws_offset: usize) -> WaypointSet {
    let start = ws_offset + 8; // skip WS(2) + version(4) + length(2)
    if start + 72 > data.len() {
        return WaypointSet::default();
    }
    let block_data = &data[start..start + 72];
    let mut waypoints = vec![vec![0u32; 5]; 3]; // [diff][act]
    let offsets: [(usize, usize); 5] = [(16, 9), (25, 9), (34, 9), (43, 3), (46, 9)];
    for diff in 0..3 {
        let blk = &block_data[diff * 24..(diff + 1) * 24];
        for (ai, (bit_start, n)) in offsets.iter().enumerate() {
            let mut val = 0u32;
            for bi in 0..*n {
                let byte_idx = (bit_start + bi) / 8;
                let bit_idx = (bit_start + bi) % 8;
                if byte_idx < 24 && (blk[byte_idx] >> bit_idx) & 1 == 1 {
                    val |= 1 << bi;
                }
            }
            waypoints[diff][ai] = val;
        }
    }
    // 转成 Vec<bool> 格式
    let waypoint_names_per_act: [usize; 5] = [9, 9, 9, 3, 9]; // 各 act waypoint 数
    let mut result = WaypointSet::default();
    for diff in 0..3 {
        let mut all_wps = Vec::new();
        for ai in 0..5 {
            let mask = waypoints[diff][ai];
            let count = waypoint_names_per_act[ai];
            for bi in 0..count {
                all_wps.push((mask >> bi) & 1 == 1);
            }
        }
        match diff {
            0 => result.normal = all_wps,
            1 => result.nightmare = all_wps,
            2 => result.hell = all_wps,
            _ => {}
        }
    }
    log::info!("[WS] normal ({}) {:?}", result.normal.len(), result.normal.iter().map(|&b| b as u8).collect::<Vec<_>>());
    log::info!("[WS] nightmare ({}) {:?}", result.nightmare.len(), result.nightmare.iter().map(|&b| b as u8).collect::<Vec<_>>());
    log::info!("[WS] hell ({}) {:?}", result.hell.len(), result.hell.iter().map(|&b| b as u8).collect::<Vec<_>>());
    result
}

/// 解析 Woo! 段 (0x193, 298B) — 任务进度 + 技能分配数据。
///
/// 格式: Woo!(4B) + payload_size(4B) + data_size(1B) + progression(1B) + quest_data(288B)
/// quest_data: 3 难度 × 96B (每难度 5 act × quest uint16 位掩码)
pub fn parse_woo(data: &[u8], woo_offset: usize) -> WooQuestData {
    if woo_offset + 298 > data.len() {
        return WooQuestData::default();
    }
    let block = &data[woo_offset..woo_offset + 298];
    // Woo!(4) + payload_size(4) = 8B header
    // data_size(1) at offset 8 — 标准 D2R = 0, mod 可能 >0 (含义待定)
    // progression 始终在 byte 9, quest data 始终从 offset 10 开始
    let _data_size = *block.get(8).unwrap_or(&0);
    let progression = *block.get(9).unwrap_or(&0);
    let quest_start = 10;
    // 3 difficulties × 96B each = 288B
    let act_quest_counts: [usize; 5] = [8, 8, 8, 8, 16]; // quest slots per act
    let mut difficulties = Vec::with_capacity(3);
    for diff in 0..3 {
        let diff_off = quest_start + diff * 96;
        let mut acts = Vec::with_capacity(5);
        for ai in 0..5 {
            let count = act_quest_counts[ai];
            let mut quests = Vec::with_capacity(count);
            for qi in 0..count {
                let off = diff_off + ai * 16 + qi * 2; // each quest = uint16 LE
                if off + 2 <= block.len() {
                    let val = u16::from_le_bytes([block[off], block[off + 1]]);
                    quests.push(val);
                } else {
                    quests.push(0);
                }
            }
            acts.push(quests);
        }
        difficulties.push(acts);
    }
    // 诊断: Normal Act I 原始 quest values
    if let Some(acts) = difficulties.first()
        && let Some(quests) = acts.first() {
            log::info!("[parse_woo] Normal ActI quest values: {:?}", quests);
        }
    WooQuestData { progression, difficulties }
}

/// 解析 w4 段 (0x30E, 51B) — NPC 对话/奖励消费状态。
///
/// 格式: w4(2B) + block_type(1B) + 6 × 8B (3 难度 × 2 类)
pub fn parse_w4(data: &[u8], w4_offset: usize) -> W4DialogData {
    if w4_offset + 51 > data.len() {
        return W4DialogData::default();
    }
    let block = &data[w4_offset..w4_offset + 51];
    let block_type = *block.get(2).unwrap_or(&0);
    let mut fields = [0u64; 6];
    for i in 0..6 {
        let off = 3 + i * 8;
        if off + 8 <= block.len() {
            let bytes: [u8; 8] = [
                block[off], block[off+1], block[off+2], block[off+3],
                block[off+4], block[off+5], block[off+6], block[off+7],
            ];
            fields[i] = u64::from_le_bytes(bytes);
        }
    }
    W4DialogData {
        block_type,
        normal: fields[0],
        normal_extra: fields[1],
        nightmare: fields[2],
        nightmare_extra: fields[3],
        hell: fields[4],
        hell_extra: fields[5],
    }
}

/// Marker offsets 是内部 helper,只在本模块 + items.rs 间使用,
/// 不放进 `D2SCharacter` 公共字段。
///
/// `gf` 与 `first_jm_count` 当前未被读,留作 debug 工具入口 ——
/// 未来若把诊断面板重新接回,可以打印这些字段。
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MarkerOffsets {
    pub gf: Option<usize>,
    pub if_: Option<usize>,
    /// 第一个 JM 之后的字符 (玩家物品) — 复用于 merc 容器定位时跳过他
    pub first_jm: Option<usize>,
    /// 第二个 JM 起点 (佣兵物品段)。新角色没雇佣兵时为 None。
    pub merc_jm: Option<usize>,
    pub jf: Option<usize>,
    pub kf: Option<usize>,
    /// 第一个 JM 的 count 字段 (header 声明,不一定准确,仅诊断用)
    pub first_jm_count: Option<u16>,
}

fn find_marker(buffer: &[u8], marker: &[u8; 2]) -> Option<usize> {
    let b1 = marker[0];
    let b2 = marker[1];
    memchr_iter(b1, buffer).find(|&pos| pos + 1 < buffer.len() && buffer[pos + 1] == b2)
}

/// 内部 helper:扫描 d2s 里关键 marker 的字节偏移。
///
/// ⚠️ "JM" 字节对也出现在 item flags 内,必须从 `if` 区之后开始搜,
/// 否则可能误中 item data 中的伪 "JM"。
pub fn marker_offsets(buffer: &[u8]) -> MarkerOffsets {
    let gf = find_marker(buffer, b"gf");
    let if_ = find_marker(buffer, b"if");
    let first_jm = if let Some(if_off) = if_ {
        if if_off + 2 < buffer.len() {
            find_marker(&buffer[if_off + 2..], b"JM")
                .map(|pos| if_off + 2 + pos)
        } else {
            find_marker(buffer, b"JM")
        }
    } else {
        find_marker(buffer, b"JM")
    };

    let merc_jm = first_jm.and_then(|first| {
        if first + 4 < buffer.len() {
            find_marker(&buffer[first + 4..], b"JM")
                .map(|pos| first + 4 + pos)
        } else {
            None
        }
    });

    let jf = first_jm.and_then(|jm| {
        if jm + 4 < buffer.len() {
            find_marker(&buffer[jm + 4..], b"jf")
                .map(|pos| jm + 4 + pos)
        } else {
            None
        }
    });
    let kf = jf.and_then(|jf_off| {
        if jf_off + 2 < buffer.len() {
            find_marker(&buffer[jf_off + 2..], b"kf")
                .map(|pos| jf_off + 2 + pos)
        } else {
            None
        }
    });

    let first_jm_count = first_jm.and_then(|offset| {
        if offset + 4 <= buffer.len() {
            Some(u16::from_le_bytes([buffer[offset + 2], buffer[offset + 3]]))
        } else {
            None
        }
    });

    MarkerOffsets {
        gf,
        if_,
        first_jm,
        merc_jm,
        jf,
        kf,
        first_jm_count,
    }
}

/// 当前已验证成功的标准 item 布局。
///
/// `socket_count_bits` = 4：与 `d2i::jm_reader::parse_noncompact_body` 一致
/// （[py] 口径, TC59 4 孔物品验证），早期 3b 记录是错误口径。
pub fn known_item_bit_layout() -> KnownItemBitLayout {
    KnownItemBitLayout {
        location_id_bit_offset: 35,
        equipped_slot_bit_offset: 38,
        huffman_code_bit_offset: 53,
        socket_count_bits: 4,
        uid_bits: 32,
        ilvl_bits: 7,
        quality_bits: 4,
        stat_terminator: 0x1FF,
    }
}

/// 顶层入口 (标准 D2SLib layout)。
///
/// 历史兼容: 留作 D2SCharacter::parse 的薄壳。
pub fn parse_file(buffer: &[u8]) -> ParseResult<D2SCharacter> {
    D2SCharacter::parse(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::d2s::header::D2S_MAGIC;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    fn make_minimal_d2s(level: u8, class: u8) -> Vec<u8> {
        let mut data = vec![0u8; 0x69]; // 全 0 header
        data[0..4].copy_from_slice(D2S_MAGIC);
        data[4..8].copy_from_slice(&0x69u32.to_le_bytes()); // header_size
        // 0x08: filesize, 0x0C: checksum (0)
        data[0x10..0x14].copy_from_slice(&1u32.to_le_bytes()); // active_weapon
        data[0x18] = class;
        data[0x19] = 0x10; // status: expansion
        data[0x1B] = level;
        // compat 段 (0x69..0x193) 全零, name 留空
        while data.len() < ATTRIBUTES_OFFSET { data.push(0); }
        while data.len() < ATTRIBUTES_OFFSET {
            data.push(0);
        }
        data.extend_from_slice(&[0x67, 0x66]);
        let lv_bits = level as u32 & 0x7F;
        let mut bit_buf: u32 = 0;
        let mut bit_pos = 0;
        for i in 0..9 {
            if 12u32 & (1 << i) != 0 {
                bit_buf |= 1 << bit_pos;
            }
            bit_pos += 1;
        }
        for i in 0..7 {
            if lv_bits & (1 << i) != 0 {
                bit_buf |= 1 << bit_pos;
            }
            bit_pos += 1;
        }
        for _ in 0..9 {
            bit_buf |= 1 << (bit_pos);
        }
        bit_pos += 9;
        let bytes_needed = (bit_pos + 7) / 8;
        for i in 0..bytes_needed {
            data.push(((bit_buf >> (i * 8)) & 0xFF) as u8);
        }
        data
    }

    #[test]
    fn test_parse_minimal_d2s() {
        let data = make_minimal_d2s(45, 6);
        let c = parse_file(&data).unwrap();
        assert_eq!(c.header.version_raw, 105);
        assert_eq!(c.header.class, 6);
        assert_eq!(c.attributes.level.raw, 45);
        assert!(c.skills.is_empty());
        assert!(c.equipped.is_empty());
        assert!(c.belt.is_empty());
        assert!(c.backpack.is_empty());
        assert!(c.cube.is_empty());
        assert!(c.merc.is_empty());
        assert!(c.header.is_expansion());
    }

    #[test]
    fn test_parse_invalid_magic() {
        let mut data = vec![0u8; 200];
        data[0..4].copy_from_slice(b"XXXX");
        data[4..8].copy_from_slice(&0x69u32.to_le_bytes());
        let err = parse_file(&data).unwrap_err();
        assert!(matches!(err, crate::core::ParseError::D2SMagic(_)));
    }

    #[test]
    fn test_parse_truncated_after_header() {
        let data = vec![0u8; 50]; // < 0x69 → Truncated
        let result = parse_file(&data);
        assert!(result.is_err(), "短 buffer 应报错");
    }

    #[test]
    fn test_standard_tc03_items_5_containers() {
        let path = fixture_path("standard_test_warlock_tc03.d2s");
        if !path.exists() { eprintln!("SKIP: fixture standard_test_warlock_tc03.d2s 缺失"); return; }
        let data = std::fs::read(&path).expect("read d2s");
        let c = parse_file(&data).expect("parse standard d2s");

        assert!(c.merc.is_empty(), "TC03 应没有 merc");
        // TC03 has equipment + items
        assert!(c.equipped.len() >= 5, "TC03 应至少 5 件装备,实际 {}", c.equipped.len());
        assert!(!c.backpack.is_empty(), "TC03 应有背包物品");
        assert_eq!(c.header.name, "TestWarlock", "优先取 mod 扩展名");
    }

    #[test]
    fn test_parse_standard_tc03_d2s() {
        let _fp = fixture_path("standard_test_warlock_tc03.d2s");
        if !_fp.exists() { eprintln!("SKIP: fixture standard_test_warlock_tc03.d2s 缺失"); return; }
        let data = std::fs::read(fixture_path("standard_test_warlock_tc03.d2s"))
            .expect("read d2s");
        let c = parse_file(&data).expect("parse standard d2s");
        assert_eq!(c.header.name, "TestWarlock", "优先取 file+0x12B mod 扩展名");
        assert_eq!(c.header.class, 7);
        assert_eq!(c.attributes.level.raw, 12);
        assert_eq!(c.skills.len(), 30, "Python: 30 raw bytes after if");

        // BitReader 改写后扫描行为微调，比原略少 ~1 件
        assert!(
            c.equipped.len() >= 6,
            "TC03 应至少 6 件装备,实际 {} 件",
            c.equipped.len(),
        );

        assert!(c.waypoints.normal.iter().any(|&b| b));
    }

    #[test]
    fn test_parse_ws_waypoints_short_buffer_returns_default() {
        let buf = b"WS\x00\x00\x00\x00\x00\x00";
        let wp = parse_ws_waypoints(buf, 0);
        assert!(wp.normal.iter().all(|&b| !b));
    }

    #[test]
    fn test_parse_woo_short_buffer_returns_default() {
        let buf = b"Woo!";
        let woo = parse_woo(buf, 0);
        assert_eq!(woo.progression, 0);
    }

    #[test]
    fn test_parse_w4_short_buffer_returns_default() {
        let buf = b"w4";
        let w4 = parse_w4(buf, 0);
        assert_eq!(w4.block_type, 0);
    }

    /// 一次性诊断:dump 开心邪帝.d2s 并与 Python 对比
    #[test]
    fn diagnostic_compare_with_python() {
        let path = std::path::Path::new("D:\\work_space\\personal_workspace\\d2r")
            .join("开心邪帝.d2s");
        if !path.exists() { eprintln!("SKIP: 本地诊断文件缺失"); return; }
        let data = std::fs::read(&path).expect("read d2s");

        // Debug: check markers
        let m = marker_offsets(&data);
        eprintln!("markers: gf={:?} if={:?} first_jm={:?} jf={:?} kf={:?} count={:?}",
            m.gf, m.if_, m.first_jm, m.jf, m.kf, m.first_jm_count);

        // Check parse_item_stream_sequential directly
        if let Some(jm_off) = m.first_jm {
            let search_end = m.jf.or(m.kf).unwrap_or(data.len()).min(data.len());
            let jm_data = &data[jm_off..search_end];
            eprintln!("JM section: {} bytes at 0x{:X}", jm_data.len(), jm_off);
            eprintln!("JM header: {:02x?}", &jm_data[..8.min(jm_data.len())]);
            let d2i_items = crate::protocol::d2i::parser::parse_item_stream_sequential(jm_data, 0, false);
            let jm_count = u16::from_le_bytes([data[jm_off + 2], data[jm_off + 3]]);
            let huffman_works = d2i_items.len() as u16 >= jm_count.saturating_sub(5) / 2
                && d2i_items.first().is_some_and(|i| !i.item.code.starts_with('#'));
            eprintln!("d2i parser: {} items, JM count={}, huffman_works={}",
                d2i_items.len(), jm_count, huffman_works);
            if let Some(first) = d2i_items.first() {
                eprintln!("First item: code={:?} raw_bit_offset={}", first.item.code, first.raw_bit_offset);
            }
        }

        let c = parse_file(&data).expect("parse");
        let total = c.backpack.len() + c.belt.len() + c.equipped.len() + c.cube.len() + c.merc.len();
        eprintln!("ITEMS: {}", total);
        for pi in c.equipped.iter().chain(c.belt.iter()).chain(c.backpack.iter()).chain(c.cube.iter()).chain(c.merc.iter()) {
            let it = &pi.item;
            let stats_n: usize = it.stat_lists.iter().map(|sl| sl.stats.len()).sum();
            let pg = it.page.map(|p| p.as_u8()).unwrap_or(0);
            let mode_val: u8 = match it.mode {
                crate::protocol::common::ItemMode::Stored => 0,
                crate::protocol::common::ItemMode::Equipped => 1,
                crate::protocol::common::ItemMode::Belt => 2,
                _ => 9,
            };
            let qual = it.quality.as_u8();
            eprintln!("{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                it.code, pg, mode_val, it.x, it.y, qual,
                it.item_level, it.id, it.set_id.unwrap_or(0),
                it.amount, it.current_durability, it.max_durability,
                it.num_sockets, stats_n);
        }
    }

    /// 一次性诊断:dump happy_librarian.d2s 5 个容器全字段。
    /// 用法:`cargo test --lib diagnostic_happy_librarian_5_containers -- --nocapture`
    #[test]
    fn diagnostic_happy_librarian_5_containers() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("happy_librarian.d2s");
        if !path.exists() { eprintln!("SKIP: fixture happy_librarian.d2s 缺失"); return; }
        let data = std::fs::read(&path).expect("read happy_librarian.d2s");
        let c = parse_file(&data).expect("parse");

        eprintln!("\n==================================================");
        eprintln!(" happy_librarian.d2s — 5 容器解析");
        eprintln!("==================================================");
        eprintln!(
            "  size = {} bytes  class = {:?}  level = {}",
            data.len(),
            c.header.character_class(),
            c.attributes.level.raw,
        );

        for (name, items) in [
            ("EQUIPPED", &c.equipped),
            ("BELT", &c.belt),
            ("BACKPACK", &c.backpack),
            ("CUBE", &c.cube),
            ("MERC", &c.merc),
        ] {
            eprintln!("\n  [{}] {} 件", name, items.len());
            for (i, pi) in items.iter().enumerate() {
                eprintln!(
                    "    [{:2}] code = {:5}  x = {:>2}  y = {:>2}  ilvl = {:>3}  amt = {:>3}",
                    i,
                    format!("{:?}", pi.item.code),
                    pi.item.x,
                    pi.item.y,
                    pi.item.item_level,
                    pi.item.amount,
                );
            }
        }
    }
}
