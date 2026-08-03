//! 显示层:名字查找函数 (Chinese/English 名称映射)
//!
//! 从 display.rs 拆出来 (Sprint 2 W8)。这 4 个函数都是纯查找,无状态,
//! 也不依赖 display.rs 的其他逻辑。

use d2r_marketplace_lib::data::skills_zh::CLASS_SKILLS;
use d2r_marketplace_lib::protocol::d2i::legacy::game_items::ALL_ITEMS;

/// Look up English item name from ALL_ITEMS table.
pub fn item_name_en(code: &str) -> &str {
    ALL_ITEMS.iter().find(|(c, _, _, _, _)| *c == code).map(|(_, n, _, _, _)| *n).unwrap_or(code)
}

/// Look up skill Chinese name from CLASS_SKILLS by global skill_id.
pub fn skill_name_zh(skill_id: u16) -> Option<&'static str> {
    CLASS_SKILLS.iter().find(|(id, _, _, _, _)| *id == skill_id).map(|(_, name, _, _, _)| Some(*name)).unwrap_or(None)
}

/// Equipped slot name: 1=头,2=颈,3=身,4=主武,5=主盾,6=右戒,7=左戒,8=腰,9=脚,10=手,11=副武,12=副盾
pub fn slot_name_py(slot: u8) -> &'static str {
    match slot {
        1 => "头", 2 => "颈", 3 => "身", 4 => "主武", 5 => "主盾",
        6 => "右戒", 7 => "左戒", 8 => "腰", 9 => "脚", 10 => "手",
        11 => "副武", 12 => "副盾",
        _ => "?",
    }
}

/// Chinese item names (hardcoded fallback, matches Python _load_item_names_zh)
pub fn item_name_zh(code: &str) -> Option<&'static str> {
    Some(match code {
        "dgr" => "匕首", "buc" => "小盾", "lea" => "皮甲", "cap" => "帽子", "hla" => "硬皮甲",
        "skp" => "颅冠", "sbw" => "短弓", "lgl" => "皮手套", "rin" => "戒指", "amu" => "项链",
        "hp1" => "小红", "hp2" => "中红", "hp3" => "大红", "hp4" => "超红", "hp5" => "终红",
        "mp1" => "小蓝", "mp2" => "中蓝", "mp3" => "大蓝", "mp4" => "超蓝", "mp5" => "终蓝",
        "rvl" => "大紫", "rvs" => "小紫", "vps" => "精力", "key" => "钥匙", "tbk" => "回城书",
        "ibk" => "鉴定书", "lbl" => "腰带", "lbt" => "皮靴", "spr" => "长矛", "ktr" => "拳刃",
        "aqv" => "箭袋", "box" => "盒子", "tsc" => "传送卷", "isc" => "鉴定卷",
        "mfp" => "魔法书", "wwu" => "不明卷轴", "lsh" => "灵石",
        _ => return None,
    })
}
