//! Auto-generated from D2R runes.txt + item-names.json — runeword combo detection
//! with Chinese name support.

/// Runeword combo: (sorted_rune_codes, english_name, chinese_name)
#[rustfmt::skip]
pub const RUNEWORD_COMBOS: &[(&str, &str, &str)] = &[
  ("r01+r02+r05+r15+r26+r33", "Breath of the Dying", "濒死之息"),
  ("r01+r02+r06+r15+r16+r19", "Unbending Will", "不屈之志"),
  ("r01+r02+r18+r20", "Voice of Reason", "理智之声"),
  ("r01+r03", "Steel", "钢铁"),
  ("r01+r03+r06+r11+r12", "Honor", "荣耀"),
  ("r01+r05+r06", "Malice", "怨恨"),
  ("r01+r09+r15+r25+r26", "Death", "死亡"),
  ("r01+r12+r14+r28", "Fortitude", "刚毅"),
  ("r01+r29", "Wind", "疾风"),
  ("r02+r03+r14+r15+r24+r26", "Silence", "沉默"),
  ("r02+r06+r21", "Wisdom", "智慧"),
  ("r02+r08+r25", "Principle", "信条"),
  ("r02+r09+r14+r20", "Passion", "热情"),
  ("r02+r13+r18", "Hustle (armor)", "狂乱"),
  ("r02+r13+r18", "Hustle (weapon)", "癫狂"),
  ("r02+r20+r27+r31", "Faith", "信念"),
  ("r03+r04", "Nadir", "天底"),
  ("r03+r05+r08+r23+r28", "Grief", "悔恨"),
  ("r03+r06+r12+r18", "Harmony", "和谐"),
  ("r03+r07+r08+r12", "Insight", "眼光"),
  ("r03+r07+r11", "Edge", "锐锋"),
  ("r03+r08", "Leaf", "叶子"),
  ("r03+r11", "Strength", "力量"),
  ("r03+r13+r22", "Crescent Moon", "新月"),
  ("r03+r17+r22+r23+r30", "Beast", "野兽"),
  ("r03+r18+r20", "Wealth", "财富"),
  ("r03+r23", "Prudence", "谨慎"),
  ("r04+r06+r12", "Radiance", "光辉"),
  ("r04+r10+r16", "Black", "黑色"),
  ("r04+r11+r15", "Myth", "神话"),
  ("r04+r13+r18", "Melody", "旋律"),
  ("r04+r16+r17+r20+r24+r33", "Obsession", "执着"),
  ("r04+r17", "Smoke", "烟雾"),
  ("r04+r21+r26", "Flickering Flame", "闪烁火焰"),
  ("r05+r07", "Stealth", "隐秘"),
  ("r05+r07+r08+r09", "Holy Thunder", "圣雷"),
  ("r05+r08+r27+r29", "Bramble", "荆棘"),
  ("r05+r09", "Zephyr", "和风"),
  ("r05+r10+r15+r18+r19", "Obedience", "顺从"),
  ("r05+r12+r16+r17", "Memory", "回忆"),
  ("r05+r13", "Rhyme", "韵律"),
  ("r05+r17", "Splendor", "壮美"),
  ("r05+r25+r31", "Fury", "愤怒"),
  ("r06+r09+r23", "Rain", "暴雨"),
  ("r06+r10+r13+r25+r32", "Mist", "迷雾"),
  ("r06+r30+r31", "Enigma", "谜团"),
  ("r07+r08+r09", "Ancients' Pledge", "先祖之誓"),
  ("r07+r09+r10", "Pattern", "典范"),
  ("r07+r09+r10+r11", "Spirit", "精神"),
  ("r07+r13+r16", "Cure", "解药"),
  ("r07+r14+r23", "Venom", "毒液"),
  ("r08+r10+r11", "King's Grace", "王恩"),
  ("r08+r11+r23+r24+r27", "Call to Arms", "战争召唤"),
  ("r08+r12+r21", "Enlightenment", "启迪"),
  ("r08+r13+r15", "Authority", "权威"),
  ("r08+r13+r16", "Temper", "淬火"),
  ("r08+r16+r24", "Coven", "女巫团"),
  ("r09+r12", "Lore", "学识"),
  ("r09+r13+r16", "Ground", "接地"),
  ("r09+r19+r27+r31", "Famine", "饥荒"),
  ("r10+r11+r13", "Peace", "平和"),
  ("r10+r13+r16", "Hearth", "壁炉"),
  ("r10+r13+r20", "Treachery", "背叛"),
  ("r10+r13+r22", "Duress", "强压"),
  ("r10+r18+r21+r26", "Heart of the Oak", "橡树之心"),
  ("r10+r24+r33", "Void", "虚空"),
  ("r11+r12+r24+r29+r30", "Eternity", "永恒"),
  ("r11+r13+r27", "Ritual", "仪式"),
  ("r11+r13+r28+r31", "Ice", "寒冰"),
  ("r11+r18+r20", "Lawbringer", "执法者"),
  ("r11+r23+r25", "Mosaic", "模糊"),
  ("r11+r28+r29+r32", "Hand of Justice", "正义之手"),
  ("r12+r13+r16", "Bulwark", "壁垒"),
  ("r12+r22+r22", "Bone", "白骨"),
  ("r12+r28+r29", "Dragon", "巨龙"),
  ("r13+r17+r21+r22", "Stone", "磐石"),
  ("r13+r17+r21+r23", "Oath", "誓言"),
  ("r13+r22+r32", "Plague", "瘟疫"),
  ("r14+r16", "White", "白色"),
  ("r14+r22+r24+r30", "Chains of Honor", "荣耀之链"),
  ("r14+r24+r26+r27", "Exile", "流放"),
  ("r14+r25", "Vigilance", "警戒"),
  ("r15+r17+r19", "Lionheart", "狮心"),
  ("r15+r18+r20+r25", "Rift", "裂隙"),
  ("r15+r22+r27+r28+r32", "Doom", "厄运"),
  ("r16+r19+r32", "Metamorphosis", "变形"),
  ("r16+r20+r24", "Delirium", "迷狂"),
  ("r16+r21+r31", "Dream", "梦境"),
  ("r16+r28+r29+r32", "Pride", "骄傲"),
  ("r17+r21+r23+r30", "Wrath", "怒火"),
  ("r18+r18+r23", "Sanctuary", "庇护"),
  ("r18+r26+r28+r30+r31", "Destruction", "毁灭"),
  ("r19+r21+r22", "Gloom", "阴霾"),
  ("r19+r22+r23+r25", "Kingslayer", "弑君者"),
  ("r19+r22+r27", "Chaos", "混沌"),
  ("r23+r24+r30+r30", "Infinity", "无限"),
  ("r23+r25+r28+r31", "Brand", "烙印"),
  ("r23+r29+r30+r31+r31+r31", "Last Wish", "临终之愿"),
  ("r26+r26+r28+r31", "Phoenix", "凤凰"),
];

/// Match rune codes against known runeword combos, return English name.
pub fn match_runeword(rune_codes: &[&str]) -> Option<&'static str> {
    let combo = build_combo(rune_codes);
    RUNEWORD_COMBOS.iter().find(|(k, _, _)| *k == combo.as_str()).map(|(_, n, _)| *n)
}

/// Match rune codes against known runeword combos, return Chinese name.
pub fn match_runeword_zh(rune_codes: &[&str]) -> Option<&'static str> {
    let combo = build_combo(rune_codes);
    RUNEWORD_COMBOS.iter().find(|(k, _, _)| *k == combo.as_str()).map(|(_, _, z)| *z)
}

/// 模糊匹配: 1) 精确组合  2) 前 num_sockets 个符文(按物品顺序)  3) 集合包含
/// (mod 存档常见: 物品孔数多、多余符文被一并镶入, 例如 3 孔 Tal+Eth+Eth 实为
/// 隐秘(Tal+Eth); 6 孔 Ral+Tir+Tir+Tal+Sol+Sol 实为眼光(Ral+Tir+Tal+Sol))。
/// 集合包含选符文数最多的符文之语 (防多解)。
pub fn match_runeword_fuzzy(rune_codes: &[&str], num_sockets: u8) -> Option<&'static str> {
    match_runeword_fuzzy_impl(rune_codes, num_sockets).map(|(_, n, _)| n)
}

/// 模糊匹配的中文名版本。
pub fn match_runeword_fuzzy_zh(rune_codes: &[&str], num_sockets: u8) -> Option<&'static str> {
    match_runeword_fuzzy_impl(rune_codes, num_sockets).map(|(_, _, z)| z)
}

fn match_runeword_fuzzy_impl(rune_codes: &[&str], num_sockets: u8) -> Option<(&'static str, &'static str, &'static str)> {
    // 1. 精确匹配
    let combo = build_combo(rune_codes);
    if let Some((en, zh)) = RUNEWORD_COMBOS.iter().find(|(k, _, _)| *k == combo.as_str()).map(|(_, n, z)| (n, z)) {
        return Some((en, zh, zh));
    }
    // 2. 前 num_sockets 个符文 (mod: 多余符文镶入同一物品)
    if num_sockets > 0 && (num_sockets as usize) < rune_codes.len() {
        let head = &rune_codes[..num_sockets as usize];
        let head_combo = build_combo(head);
        if let Some((en, zh)) = RUNEWORD_COMBOS.iter().find(|(k, _, _)| *k == head_combo.as_str()).map(|(_, n, z)| (n, z)) {
            return Some((en, zh, zh));
        }
    }
    // 3. 集合包含: socketed 符文 ⊇ 符文之语所需符文, 选符文数最多的
    let mut best: Option<((&'static str, &'static str, &'static str), usize)> = None;
    for entry in RUNEWORD_COMBOS {
        let need: Vec<&str> = entry.0.split('+').collect();
        if need.len() > best.map(|(_, n)| n).unwrap_or(0)
            && need.iter().all(|r| rune_codes.contains(r)) {
            best = Some(((entry.1, entry.2, entry.2), need.len()));
        }
    }
    best.map(|(e, _)| e)
}

fn build_combo(rune_codes: &[&str]) -> String {
    if rune_codes.is_empty() { return String::new(); }
    let mut sorted: Vec<&str> = rune_codes.to_vec();
    sorted.sort();
    sorted.join("+")
}

/// Match rune codes and return runeword ID (1-indexed index in RUNEWORD_COMBOS).
pub fn find_runeword_id(rune_codes: &[&str]) -> Option<u16> {
    if rune_codes.is_empty() { return None; }
    let mut sorted: Vec<&str> = rune_codes.to_vec();
    sorted.sort();
    let combo = sorted.join("+");
    RUNEWORD_COMBOS.iter().position(|(k, _, _)| *k == combo.as_str()).map(|i| i as u16 + 1)
}

/// 计算符文之语的装备等级需求 = 最高符文的等级需求。
pub fn runeword_req_level(rune_codes: &[String]) -> u8 {
    rune_codes.iter().filter_map(|c| rune_level(c)).max().unwrap_or(0)
}

fn rune_level(code: &str) -> Option<u8> {
    Some(match code {
        "r01" | "r02" => 11, "r03" | "r04" => 13,
        "r05" | "r06" => 15, "r07" => 17, "r08" => 19,
        "r09" => 21, "r10" => 23, "r11" => 25, "r12" => 27,
        "r13" => 29, "r14" => 31, "r15" => 35, "r16" => 37,
        "r17" => 39, "r18" => 41, "r19" => 43, "r20" => 45,
        "r21" => 47, "r22" => 49, "r23" => 51, "r24" => 53,
        "r25" => 55, "r26" => 57, "r27" => 59, "r28" => 61,
        "r29" => 63, "r30" => 65, "r31" => 67, "r32" => 69,
        "r33" => 71,
        _ => return None,
    })
}
