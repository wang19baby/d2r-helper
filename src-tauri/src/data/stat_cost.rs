//! Auto-generated from itemstatcost.txt + webpack MAGICAL_PROPS.
//! CSvBits and other metadata annotated in comments.

use crate::protocol::common::{StatProp, StatTable};

pub fn build_stat_table() -> StatTable {
    let mut props: Vec<StatProp> = MAGICAL_PROPS
        .iter()
        .map(|p| StatProp {
            save_bits: p.save_bits,
            num_sub_props: p.num_sub_props,
            save_add: p.save_add,
            save_param_bits: p.save_param_bits,
            signed: p.signed,
            encoding: p.encoding,
            descfunc: p.descfunc,
            cs_bits: p.cs_bits,
        })
        .collect();

    // ★ D2R sub-property counts: 某些 stat 有多个连续的 sub-prop（如 lightmindam(50)+lightmaxdam(51)）
    //   auto-generator 未导出此信息，这里手动覆盖。
    //   Python jm_parser 的 _STAT_NP 等价定义。
    static NP_OVERRIDES: &[(u16, u8)] = &[
        (17, 2), // hitpoints → +maxhp
        (48, 2), // item_integration_test
        (50, 2), // lightmindam → lightmaxdam
        (52, 2), // firemindam → firemaxdam
        (54, 3), // poison over time
        (57, 3), // coldmindam → coldmaxdam → coldlength
    ];
    for &(sid, np) in NP_OVERRIDES {
        if (sid as usize) < props.len() {
            props[sid as usize].num_sub_props = np;
        }
    }
    StatTable::from_props(props)
}

#[derive(Debug, Clone, Copy)]
pub struct MagicProp {
    pub save_bits: u8,
    pub num_sub_props: u8,
    pub save_add: i32,
    pub save_param_bits: u8,
    pub signed: u8,
    pub encoding: u8,
    /// D2R ItemStatCost.txt 的 `descfunc` 列。
    /// 14 = stat 188 item_addskill_tab — 需要把整个 param 拆为 SkillTab + SkillLevel
    /// (D2SLib Items.cs 的 case 14 特殊处理)
    pub descfunc: u8,
    /// CSvBits (col 9 from ItemStatCost.txt). Used for gf (character attributes) section.
    pub cs_bits: u8,
}

impl MagicProp {
    pub const fn new(save_bits: u8, num_sub_props: u8, save_add: i32, save_param_bits: u8, signed: u8, encoding: u8, cs_bits: u8) -> Self {
        // 默认 descfunc=0; 现有所有 470+ 个调用点不受影响
        Self { save_bits, num_sub_props, save_add, save_param_bits, signed, encoding, descfunc: 0, cs_bits }
    }

    /// 显式指定 descfunc (用于需要 descfunc 特殊处理的少数 stat, 如 stat 188)
    pub const fn with_descfunc(mut self, descfunc: u8) -> Self {
        self.descfunc = descfunc;
        self
    }

    pub const fn empty() -> Self {
        Self { save_bits: 0, num_sub_props: 1, save_add: 0, save_param_bits: 0, signed: 0, encoding: 0, descfunc: 0, cs_bits: 0 }
    }
}

#[rustfmt::skip]
pub const MAGICAL_PROPS: &[MagicProp] = &[

    MagicProp::new( 8,  1,   32,  0,  0,  0, 10).with_descfunc(19), // [  0] strength
    MagicProp::new( 7,  1,   32,  0,  0,  0, 10).with_descfunc(19), // [  1] energy
    MagicProp::new( 7,  1,   32,  0,  0,  0, 10).with_descfunc(19), // [  2] dexterity
    MagicProp::new( 7,  1,   32,  0,  0,  0, 10).with_descfunc(19), // [  3] vitality
    MagicProp::empty(), // [  4] statpts
    MagicProp::empty(), // [  5] newskills
    MagicProp::empty(), // [  6] hitpoints
    MagicProp::new( 9,  1,   32,  0,  0,  0, 21).with_descfunc(19), // [  7] maxhp
    MagicProp::empty(), // [  8] mana
    MagicProp::new( 8,  1,   32,  0,  0,  0, 21).with_descfunc(19), // [  9] maxmana
    MagicProp::empty(), // [ 10] stamina
    MagicProp::new( 8,  1,   32,  0,  0,  0, 21).with_descfunc(19), // [ 11] maxstamina
    MagicProp::empty(), // [ 12] level
    MagicProp::empty(), // [ 13] experience
    MagicProp::empty(), // [ 14] gold
    MagicProp::empty(), // [ 15] goldbank
    MagicProp::new( 9,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 16] item_armor_percent
    MagicProp::new( 9,  2,    0,  0,  1,  0,  0).with_descfunc(19), // [ 17] item_maxdamage_percent
    MagicProp::new( 9,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 18] item_mindamage_percent
    MagicProp::new(10,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 19] tohit
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 20] toblock
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 21] mindamage
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 22] maxdamage
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 23] secondary_mindamage
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 24] secondary_maxdamage
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 25] damagepercent
    MagicProp::new( 8,  1,    0,  0,  0,  0,  0), // [ 26] manarecovery
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 27] manarecoverybonus
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 28] staminarecoverybonus
    MagicProp::empty(), // [ 29] lastexp
    MagicProp::empty(), // [ 30] nextexp
    MagicProp::new(11,  1,   10,  0,  1,  0,  0).with_descfunc(19), // [ 31] armorclass
    MagicProp::new( 9,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 32] armorclass_vs_missile
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 33] armorclass_vs_hth
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 34] normal_damage_reduction
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 35] magic_damage_reduction
    MagicProp::new( 9,  1,  200,  0,  1,  0,  0).with_descfunc(29), // [ 36] damageresist
    MagicProp::new( 9,  1,  200,  0,  1,  0,  0).with_descfunc(19), // [ 37] magicresist
    MagicProp::new( 5,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 38] maxmagicresist
    MagicProp::new( 9,  1,  200,  0,  1,  0,  0).with_descfunc(19), // [ 39] fireresist
    MagicProp::new( 5,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 40] maxfireresist
    MagicProp::new( 9,  1,  200,  0,  1,  0,  0).with_descfunc(19), // [ 41] lightresist
    MagicProp::new( 5,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 42] maxlightresist
    MagicProp::new( 9,  1,  200,  0,  1,  0,  0).with_descfunc(19), // [ 43] coldresist
    MagicProp::new( 5,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 44] maxcoldresist
    MagicProp::new( 9,  1,  200,  0,  1,  0,  0).with_descfunc(19), // [ 45] poisonresist
    MagicProp::new( 5,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 46] maxpoisonresist
    MagicProp::empty(), // [ 47] damageaura
    MagicProp::new( 8,  2,    0,  0,  1,  0,  0).with_descfunc(19), // [ 48] firemindam
    MagicProp::new( 9,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 49] firemaxdam
    MagicProp::new( 6,  2,    0,  0,  1,  0,  0).with_descfunc(19), // [ 50] lightmindam
    MagicProp::new(10,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 51] lightmaxdam
    MagicProp::new( 8,  3,    0,  0,  1,  0,  0).with_descfunc(19), // [ 52] magicmindam
    MagicProp::new( 9,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 53] magicmaxdam
    MagicProp::new( 8,  3,    0,  0,  1,  0,  0).with_descfunc(19), // [ 54] coldmindam
    MagicProp::new( 9,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 55] coldmaxdam
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [ 56] coldlength
    MagicProp::new(10,  2,    0,  0,  1,  0,  0).with_descfunc(19), // [ 57] poisonmindam
    MagicProp::new(10,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 58] poisonmaxdam
    MagicProp::new( 9,  1,    0,  0,  1,  0,  0), // [ 59] poisonlength
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 60] lifedrainmindam
    MagicProp::empty(), // [ 61] lifedrainmaxdam
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 62] manadrainmindam
    MagicProp::empty(), // [ 63] manadrainmaxdam
    MagicProp::empty(), // [ 64] stamdrainmindam
    MagicProp::empty(), // [ 65] stamdrainmaxdam
    MagicProp::empty(), // [ 66] stunlength
    MagicProp::new( 7,  1,   30,  0,  1,  0,  0), // [ 67] velocitypercent
    MagicProp::new( 7,  1,   20,  0,  1,  0,  0), // [ 68] attackrate
    MagicProp::empty(), // [ 69] other_animrate
    MagicProp::empty(), // [ 70] quantity
    MagicProp::new( 8,  1,  100,  0,  1,  0,  0), // [ 71] value
    MagicProp::new( 9,  1,    0,  0,  1,  0,  0), // [ 72] durability
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [ 73] maxdurability
    MagicProp::new( 6,  1,   30,  0,  0,  0,  0).with_descfunc(19), // [ 74] hpregen
    MagicProp::new( 7,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [ 75] item_maxdurability_percent
    MagicProp::new( 6,  1,   10,  0,  1,  0,  0).with_descfunc(19), // [ 76] item_maxhp_percent
    MagicProp::new( 6,  1,   10,  0,  1,  0,  0).with_descfunc(19), // [ 77] item_maxmana_percent
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 78] item_attackertakesdamage
    MagicProp::new( 9,  1,  100,  0,  1,  0,  0).with_descfunc(19), // [ 79] item_goldbonus
    MagicProp::new( 8,  1,  100,  0,  1,  0,  0).with_descfunc(19), // [ 80] item_magicbonus
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 81] item_knockback
    MagicProp::new( 9,  1,   20,  0,  1,  0,  0), // [ 82] item_timeduration
    MagicProp::new( 3,  1,    0,  3,  1,  0,  0).with_descfunc(13), // [ 83] item_addclassskills
    MagicProp::empty(), // [ 84] unsentparam1
    MagicProp::new( 9,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [ 85] item_addexperience
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [ 86] item_healafterkill
    MagicProp::new( 7,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [ 87] item_reducedprices
    MagicProp::new( 1,  1,    0,  0,  1,  0,  0), // [ 88] item_doubleherbduration
    MagicProp::new( 4,  1,    4,  0,  1,  0,  0).with_descfunc(19), // [ 89] item_lightradius
    MagicProp::new(24,  1,    0,  0,  1,  0,  0), // [ 90] item_lightcolor
    MagicProp::new( 8,  1,  100,  0,  1,  0,  0).with_descfunc(19), // [ 91] item_req_percent
    MagicProp::new( 7,  1,    0,  0,  0,  0,  0), // [ 92] item_levelreq
    MagicProp::new( 7,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [ 93] item_fasterattackrate
    MagicProp::new( 7,  1,   64,  0,  0,  0,  0), // [ 94] item_levelreqpct
    MagicProp::empty(), // [ 95] lastblockframe
    MagicProp::new( 7,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [ 96] item_fastermovevelocity
    MagicProp::new( 6,  1,    0,  9,  1,  1,  0).with_descfunc(28), // [ 97] item_nonclassskill
    MagicProp::new( 1,  1,    0,  8,  0,  0,  0), // [ 98] state
    MagicProp::new( 7,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [ 99] item_fastergethitrate
    MagicProp::empty(), // [100] monster_playercount
    MagicProp::empty(), // [101] skill_poison_override_length
    MagicProp::new( 7,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [102] item_fasterblockrate
    MagicProp::empty(), // [103] skill_bypass_undead
    MagicProp::empty(), // [104] skill_bypass_demons
    MagicProp::new( 7,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [105] item_fastercastrate
    MagicProp::empty(), // [106] skill_bypass_beasts
    MagicProp::new( 3,  1,    0,  9,  1,  1,  0).with_descfunc(27), // [107] item_singleskill
    MagicProp::new( 1,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [108] item_restinpeace
    MagicProp::new( 9,  1,    0,  0,  0,  0,  0), // [109] curse_resistance
    MagicProp::new( 8,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [110] item_poisonlengthresist
    MagicProp::new( 9,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [111] item_normaldamage
    MagicProp::new( 7,  1,   -1,  0,  1,  0,  0).with_descfunc(5), // [112] item_howl
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(12), // [113] item_stupidity
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [114] item_damagetomana
    MagicProp::new( 1,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [115] item_ignoretargetac
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [116] item_fractionaltargetac
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [117] item_preventheal
    MagicProp::new( 1,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [118] item_halffreezeduration
    MagicProp::new( 9,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [119] item_tohit_percent
    MagicProp::new( 7,  1,  128,  0,  1,  0,  0).with_descfunc(19), // [120] item_damagetargetac
    MagicProp::new( 9,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [121] item_demondamage_percent
    MagicProp::new( 9,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [122] item_undeaddamage_percent
    MagicProp::new(10,  1,  128,  0,  1,  0,  0).with_descfunc(19), // [123] item_demon_tohit
    MagicProp::new(10,  1,  128,  0,  1,  0,  0).with_descfunc(19), // [124] item_undead_tohit
    MagicProp::new( 1,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [125] item_throwable
    MagicProp::new( 3,  1,    0,  3,  1,  0,  0).with_descfunc(19), // [126] item_elemskill
    MagicProp::new( 3,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [127] item_allskills
    MagicProp::new( 5,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [128] item_attackertakeslightdamage
    MagicProp::empty(), // [129] ironmaiden_level
    MagicProp::empty(), // [130] lifetap_level
    MagicProp::empty(), // [131] thorns_percent
    MagicProp::empty(), // [132] bonearmor
    MagicProp::empty(), // [133] bonearmormax
    MagicProp::new( 5,  1,    0,  0,  1,  0,  0).with_descfunc(12), // [134] item_freeze
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [135] item_openwounds
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [136] item_crushingblow
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [137] item_kickdamage
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [138] item_manaafterkill
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [139] item_healafterdemonkill
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0), // [140] item_extrablood
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [141] item_deadlystrike
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [142] item_absorbfire_percent
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [143] item_absorbfire
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [144] item_absorblight_percent
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [145] item_absorblight
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [146] item_absorbmagic_percent
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [147] item_absorbmagic
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [148] item_absorbcold_percent
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [149] item_absorbcold
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [150] item_slow
    MagicProp::new( 5,  1,    0,  9,  1,  0,  0).with_descfunc(16), // [151] item_aura
    MagicProp::new( 1,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [152] item_indesctructible
    MagicProp::new( 1,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [153] item_cannotbefrozen
    MagicProp::new( 7,  1,   20,  0,  1,  0,  0).with_descfunc(19), // [154] item_staminadrainpct
    MagicProp::new( 7,  1,    0, 10,  0,  0,  0).with_descfunc(23), // [155] item_reanimate
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [156] item_pierce
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [157] item_magicarrow
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [158] item_explosivearrow
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0), // [159] item_throw_mindamage
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0), // [160] item_throw_maxdamage
    MagicProp::empty(), // [161] skill_handofathena
    MagicProp::empty(), // [162] skill_staminapercent
    MagicProp::empty(), // [163] skill_passive_staminapercent
    MagicProp::empty(), // [164] skill_concentration
    MagicProp::empty(), // [165] skill_enchant
    MagicProp::empty(), // [166] skill_pierce
    MagicProp::empty(), // [167] skill_conviction
    MagicProp::empty(), // [168] skill_chillingarmor
    MagicProp::empty(), // [169] skill_frenzy
    MagicProp::empty(), // [170] skill_decrepify
    MagicProp::empty(), // [171] skill_armor_percent
    MagicProp::empty(), // [172] alignment
    MagicProp::empty(), // [173] target0
    MagicProp::empty(), // [174] target1
    MagicProp::empty(), // [175] goldlost
    MagicProp::empty(), // [176] conversion_level
    MagicProp::empty(), // [177] conversion_maxhp
    MagicProp::empty(), // [178] unit_dooverlay
    MagicProp::new( 9,  1,    0, 10,  0,  0,  0).with_descfunc(22), // [179] attack_vs_montype
    MagicProp::new( 9,  1,    0, 10,  0,  0,  0).with_descfunc(22), // [180] damage_vs_montype
    MagicProp::new( 3,  1,    0,  0,  0,  0,  0), // [181] fade
    MagicProp::empty(), // [182] armor_override_percent
    MagicProp::empty(), // [183] lasthitreactframe
    MagicProp::empty(), // [184] create_season
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [185] bonus_mindamage
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [186] bonus_maxdamage
    MagicProp::new(10,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [187] item_pierce_cold_immunity
    MagicProp::new( 3,  1,    0, 16,  1,  0,  0).with_descfunc(14), // [188] item_addskill_tab
    MagicProp::new(10,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [189] item_pierce_fire_immunity
    MagicProp::new(10,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [190] item_pierce_light_immunity
    MagicProp::new(10,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [191] item_pierce_poison_immunity
    MagicProp::new(10,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [192] item_pierce_damage_immunity
    MagicProp::new(10,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [193] item_pierce_magic_immunity
    MagicProp::new( 4,  1,    0,  0,  1,  0,  0), // [194] item_numsockets
    MagicProp::new( 7,  1,    0, 16,  1,  2,  0).with_descfunc(15), // [195] item_skillonattack
    MagicProp::new( 7,  1,    0, 16,  1,  2,  0).with_descfunc(15), // [196] item_skillonkill
    MagicProp::new( 7,  1,    0, 16,  1,  2,  0).with_descfunc(15), // [197] item_skillondeath
    MagicProp::new( 7,  1,    0, 16,  1,  2,  0).with_descfunc(15), // [198] item_skillonhit
    MagicProp::new( 7,  1,    0, 16,  1,  2,  0).with_descfunc(15), // [199] item_skillonlevelup
    MagicProp::new( 7,  1,    0,  0,  0,  0,  0).with_descfunc(19), // [200] item_charge_noconsume
    MagicProp::new( 7,  1,    0, 16,  1,  2,  0).with_descfunc(15), // [201] item_skillongethit
    MagicProp::empty(), // [202] modifierlist_castid
    MagicProp::empty(), // [203] passive_mastery_item_req_percent
    MagicProp::new(16,  1,    0, 16,  1,  3,  0).with_descfunc(24), // [204] item_charged_skill
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [205] item_noconsume
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [206] passive_mastery_noconsume
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [207] passive_mastery_replenish_oncrit
    MagicProp::empty(), // [208] missile_thorns_percent
    MagicProp::empty(), // [209] passive_mastery_item_level_req_percent
    MagicProp::empty(), // [210] ua_escalation
    MagicProp::empty(), // [211] ua_defeated
    MagicProp::empty(), // [212]
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [213] passive_mastery_attack_speed
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [214] item_armor_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [215] item_armorpercent_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [216] item_hp_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [217] item_mana_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [218] item_maxdamage_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [219] item_maxdamage_percent_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [220] item_strength_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [221] item_dexterity_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [222] item_energy_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [223] item_vitality_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [224] item_tohit_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [225] item_tohitpercent_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [226] item_cold_damagemax_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [227] item_fire_damagemax_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [228] item_ltng_damagemax_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [229] item_pois_damagemax_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [230] item_resist_cold_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [231] item_resist_fire_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [232] item_resist_ltng_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [233] item_resist_pois_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [234] item_absorb_cold_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [235] item_absorb_fire_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [236] item_absorb_ltng_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0), // [237] item_absorb_pois_perlevel
    MagicProp::new( 5,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [238] item_thorns_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [239] item_find_gold_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [240] item_find_magic_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [241] item_regenstamina_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [242] item_stamina_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [243] item_damage_demon_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [244] item_damage_undead_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [245] item_tohit_demon_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [246] item_tohit_undead_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [247] item_crushingblow_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [248] item_openwounds_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [249] item_kick_damage_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [250] item_deadlystrike_perlevel
    MagicProp::empty(), // [251] item_find_gems_perlevel
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(11), // [252] item_replenish_durability
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [253] item_replenish_quantity
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [254] item_extra_stack
    MagicProp::empty(), // [255] item_find_item
    MagicProp::empty(), // [256] item_slash_damage
    MagicProp::empty(), // [257] item_slash_damage_percent
    MagicProp::empty(), // [258] item_crush_damage
    MagicProp::empty(), // [259] item_crush_damage_percent
    MagicProp::empty(), // [260] item_thrust_damage
    MagicProp::empty(), // [261] item_thrust_damage_percent
    MagicProp::empty(), // [262] item_absorb_slash
    MagicProp::empty(), // [263] item_absorb_crush
    MagicProp::empty(), // [264] item_absorb_thrust
    MagicProp::empty(), // [265] item_absorb_slash_percent
    MagicProp::empty(), // [266] item_absorb_crush_percent
    MagicProp::empty(), // [267] item_absorb_thrust_percent
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [268] item_armor_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [269] item_armorpercent_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [270] item_hp_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [271] item_mana_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [272] item_maxdamage_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [273] item_maxdamage_percent_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [274] item_strength_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [275] item_dexterity_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [276] item_energy_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [277] item_vitality_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [278] item_tohit_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [279] item_tohitpercent_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [280] item_cold_damagemax_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [281] item_fire_damagemax_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [282] item_ltng_damagemax_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [283] item_pois_damagemax_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [284] item_resist_cold_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [285] item_resist_fire_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [286] item_resist_ltng_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [287] item_resist_pois_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [288] item_absorb_cold_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [289] item_absorb_fire_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [290] item_absorb_ltng_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0), // [291] item_absorb_pois_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [292] item_find_gold_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [293] item_find_magic_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [294] item_regenstamina_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [295] item_stamina_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [296] item_damage_demon_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [297] item_damage_undead_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [298] item_tohit_demon_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [299] item_tohit_undead_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [300] item_crushingblow_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [301] item_openwounds_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(17), // [302] item_kick_damage_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0).with_descfunc(18), // [303] item_deadlystrike_bytime
    MagicProp::new(22,  1,    0,  0,  1,  4,  0), // [304] item_find_gems_bytime
    MagicProp::new( 8,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [305] item_pierce_cold
    MagicProp::new( 8,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [306] item_pierce_fire
    MagicProp::new( 8,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [307] item_pierce_ltng
    MagicProp::new( 8,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [308] item_pierce_pois
    MagicProp::empty(), // [309] item_damage_vs_monster
    MagicProp::empty(), // [310] item_damage_percent_vs_monster
    MagicProp::empty(), // [311] item_tohit_vs_monster
    MagicProp::empty(), // [312] item_tohit_percent_vs_monster
    MagicProp::empty(), // [313] item_ac_vs_monster
    MagicProp::empty(), // [314] item_ac_percent_vs_monster
    MagicProp::empty(), // [315] firelength
    MagicProp::empty(), // [316] burningmin
    MagicProp::empty(), // [317] burningmax
    MagicProp::empty(), // [318] progressive_damage
    MagicProp::empty(), // [319] progressive_steal
    MagicProp::empty(), // [320] progressive_other
    MagicProp::empty(), // [321] progressive_fire
    MagicProp::empty(), // [322] progressive_cold
    MagicProp::empty(), // [323] progressive_lightning
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0), // [324] item_extra_charges
    MagicProp::empty(), // [325] progressive_tohit
    MagicProp::empty(), // [326] poison_count
    MagicProp::empty(), // [327] damage_framerate
    MagicProp::empty(), // [328] pierce_idx
    MagicProp::new( 9,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [329] passive_fire_mastery
    MagicProp::new( 9,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [330] passive_ltng_mastery
    MagicProp::new( 9,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [331] passive_cold_mastery
    MagicProp::new( 9,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [332] passive_pois_mastery
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [333] passive_fire_pierce
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [334] passive_ltng_pierce
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [335] passive_cold_pierce
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [336] passive_pois_pierce
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [337] passive_critical_strike
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0), // [338] passive_dodge
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0), // [339] passive_avoid
    MagicProp::new( 7,  1,    0,  0,  1,  0,  0), // [340] passive_evade
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [341] passive_warmth
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [342] passive_mastery_melee_th
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [343] passive_mastery_melee_dmg
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [344] passive_mastery_melee_crit
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [345] passive_mastery_throw_th
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [346] passive_mastery_throw_dmg
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [347] passive_mastery_throw_crit
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [348] passive_weaponblock
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0), // [349] passive_summon_resist
    MagicProp::empty(), // [350] modifierlist_skill
    MagicProp::empty(), // [351] modifierlist_level
    MagicProp::empty(), // [352] last_sent_hp_pct
    MagicProp::empty(), // [353] source_unit_type
    MagicProp::empty(), // [354] source_unit_id
    MagicProp::empty(), // [355] shortparam1
    MagicProp::new( 2,  1,    0,  0,  0,  0,  0), // [356] questitemdifficulty
    MagicProp::new( 9,  1,   50,  0,  1,  0,  0).with_descfunc(19), // [357] passive_mag_mastery
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [358] passive_mag_pierce
    MagicProp::empty(), // [359] skill_cooldown
    MagicProp::empty(), // [360] skill_missile_damage_scale
    MagicProp::empty(), // [361] psychicward
    MagicProp::empty(), // [362] psychicwardmax
    MagicProp::empty(), // [363] skill_channeling_tick
    MagicProp::empty(), // [364] customization_index
    MagicProp::new( 6,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [365] item_magic_damagemax_perlevel
    MagicProp::new( 8,  1,    0,  0,  1,  0,  0).with_descfunc(19), // [366] passive_dmg_pierce
    MagicProp::empty(), // [367] heraldtier
    MagicProp::new(10,  1,    0,  0,  0,  0, 10).with_descfunc(3), // [368] coi_inf_t1_count
    MagicProp::new(10,  1,    0,  0,  0,  0, 10).with_descfunc(3), // [369] coi_inf_t1_gate
    MagicProp::new(10,  1,    0,  0,  0,  0, 10).with_descfunc(3), // [370] coi_inf_t2_count
    MagicProp::new(10,  1,    0,  0,  0,  0, 10).with_descfunc(3), // [371] coi_inf_t2_gate
    MagicProp::new(10,  1,    0,  0,  0,  0, 10).with_descfunc(3), // [372] coi_inf_t3_count
    MagicProp::new(10,  1,    0,  0,  0,  0, 10).with_descfunc(3), // [373] coi_inf_t3_gate
    MagicProp::new( 1,  1,    0,  0,  0,  0,  1).with_descfunc(3), // [374] coi_inf_gate_init
    MagicProp::empty(), // [375]
    MagicProp::empty(), // [376]
    MagicProp::empty(), // [377]
    MagicProp::empty(), // [378]
    MagicProp::empty(), // [379]
    MagicProp::empty(), // [380]
    MagicProp::empty(), // [381]
    MagicProp::empty(), // [382]
    MagicProp::empty(), // [383]
    MagicProp::empty(), // [384]
    MagicProp::empty(), // [385]
    MagicProp::empty(), // [386]
    MagicProp::empty(), // [387]
    MagicProp::empty(), // [388]
    MagicProp::empty(), // [389]
    MagicProp::empty(), // [390]
    MagicProp::empty(), // [391]
    MagicProp::empty(), // [392]
    MagicProp::empty(), // [393]
    MagicProp::empty(), // [394]
    MagicProp::empty(), // [395]
    MagicProp::new( 7,  1,    0,  0,  0,  0,  7).with_descfunc(7), // [396] crit
    MagicProp::new(10,  1,    0,  0,  0,  0, 10).with_descfunc(7), // [397] hp-kill
    MagicProp::new(10,  1,    0,  0,  0,  0, 10).with_descfunc(7), // [398] mana-lost
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [399] coi_jzb_lin
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [400] coi_jzb_xfu
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [401] coi_jzb_lsh
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [402] coi_jzb_lyd
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [403] coi_jzb_jlf
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [404] coi_jzb_rly
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [405] coi_jzb_nls
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [406] coi_jzb_lck
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [407] coi_jzb_cly
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [408] coi_jzb_qlf
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [409] coi_jzb_cll
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [410] coi_jzb_lgy
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [411] coi_jzb_lgs
    MagicProp::new(14,  1,    0,  0,  0,  0, 14).with_descfunc(3), // [412] coi_jzb_uni
    MagicProp::new( 6,  1,    0,  0,  0,  0,  6).with_descfunc(3), // [413] coi_root_gold
    MagicProp::new( 6,  1,    0,  0,  0,  0,  6).with_descfunc(3), // [414] coi_root_wood
    MagicProp::new( 6,  1,    0,  0,  0,  0,  6).with_descfunc(3), // [415] coi_root_water
    MagicProp::new( 6,  1,    0,  0,  0,  0,  6).with_descfunc(3), // [416] coi_root_fire
    MagicProp::new( 6,  1,    0,  0,  0,  0,  6).with_descfunc(3), // [417] coi_root_earth
    MagicProp::new( 6,  1,    0,  0,  0,  0,  6).with_descfunc(3), // [418] coi_root_light
    MagicProp::new( 6,  1,    0,  0,  0,  0,  6).with_descfunc(3), // [419] coi_root_dark
];

// CSV field indices (itemstatcost.txt):
//   Col 3:  Signed
//   Col 9:  CSvBits
//   Col 14: Encode
//   Col 20: Save Bits
//   Col 21: Save Add
//   Col 22: Save Param Bits
