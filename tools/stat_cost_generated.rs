//! Auto-generated from itemstatcost.txt + webpack MAGICAL_PROPS.
//! CSvBits and other metadata annotated in comments.

use crate::protocol::common::{StatProp, StatTable};

pub fn build_stat_table() -> StatTable {
    let props: Vec<StatProp> = MAGICAL_PROPS
        .iter()
        .map(|p| StatProp {
            save_bits: p.save_bits,
            num_sub_props: p.num_sub_props,
            save_add: p.save_add,
            save_param_bits: p.save_param_bits,
            signed: p.signed,
            encoding: p.encoding,
            cs_bits: p.cs_bits,
        })
        .collect();
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
    /// CSvBits (col 9 from ItemStatCost.txt). Used for gf (character attributes) section.
    pub cs_bits: u8,
}

impl MagicProp {
    pub const fn new(save_bits: u8, num_sub_props: u8, save_add: i32, save_param_bits: u8, signed: u8, encoding: u8, cs_bits: u8) -> Self {
        Self { save_bits, num_sub_props, save_add, save_param_bits, signed, encoding, cs_bits }
    }

    pub const fn empty() -> Self {
        Self { save_bits: 0, num_sub_props: 1, save_add: 0, save_param_bits: 0, signed: 0, encoding: 0, cs_bits: 0 }
    }
}

#[rustfmt::skip]
pub const MAGICAL_PROPS: &[MagicProp] = &[

    MagicProp::new( 8, 1,   32,  0, 0, 0, 10), // [  0] strength  (saved=1, CSvBits=10)
    MagicProp::new( 7, 1,   32,  0, 0, 0, 10), // [  1] energy  (saved=1, CSvBits=10)
    MagicProp::new( 7, 1,   32,  0, 0, 0, 10), // [  2] dexterity  (saved=1, CSvBits=10)
    MagicProp::new( 7, 1,   32,  0, 0, 0, 10), // [  3] vitality  (saved=1, CSvBits=10)
    MagicProp::empty(), // [  4] statpts  (saved=1, CSvBits=10)
    MagicProp::empty(), // [  5] newskills  (saved=1, CSvBits=8)
    MagicProp::empty(), // [  6] hitpoints  (saved=1, CSvBits=21)
    MagicProp::new( 9, 1,   32,  0, 0, 0, 21), // [  7] maxhp  (saved=1, CSvBits=21)
    MagicProp::empty(), // [  8] mana  (saved=1, CSvBits=21)
    MagicProp::new( 8, 1,   32,  0, 0, 0, 21), // [  9] maxmana  (saved=1, CSvBits=21)
    MagicProp::empty(), // [ 10] stamina  (saved=1, CSvBits=21)
    MagicProp::new( 8, 1,   32,  0, 0, 0, 21), // [ 11] maxstamina  (saved=1, CSvBits=21)
    MagicProp::empty(), // [ 12] level  (saved=1, CSvBits=7)
    MagicProp::empty(), // [ 13] experience  (saved=1, CSvBits=32)
    MagicProp::empty(), // [ 14] gold  (saved=1, CSvBits=25)
    MagicProp::empty(), // [ 15] goldbank  (saved=1, CSvBits=25)
    MagicProp::new( 9, 1,    0,  0, 1, 0,  0), // [ 16] item_armor_percent
    MagicProp::new( 9, 2,    0,  0, 1, 0,  0), // [ 17] item_maxdamage_percent
    MagicProp::new( 9, 1,    0,  0, 1, 0,  0), // [ 18] item_mindamage_percent
    MagicProp::new(10, 1,    0,  0, 1, 0,  0), // [ 19] tohit
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [ 20] toblock
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [ 21] mindamage
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [ 22] maxdamage
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [ 23] secondary_mindamage
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [ 24] secondary_maxdamage
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [ 25] damagepercent
    MagicProp::new( 8, 1,    0,  0, 0, 0,  0), // [ 26] manarecovery
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [ 27] manarecoverybonus
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [ 28] staminarecoverybonus
    MagicProp::empty(), // [ 29] lastexp
    MagicProp::empty(), // [ 30] nextexp
    MagicProp::new(11, 1,   10,  0, 1, 0,  0), // [ 31] armorclass
    MagicProp::new( 9, 1,    0,  0, 1, 0,  0), // [ 32] armorclass_vs_missile
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [ 33] armorclass_vs_hth
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [ 34] normal_damage_reduction
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [ 35] magic_damage_reduction
    MagicProp::new( 9, 1,  200,  0, 1, 0,  0), // [ 36] damageresist
    MagicProp::new( 9, 1,  200,  0, 1, 0,  0), // [ 37] magicresist
    MagicProp::new( 5, 1,    0,  0, 1, 0,  0), // [ 38] maxmagicresist
    MagicProp::new( 9, 1,  200,  0, 1, 0,  0), // [ 39] fireresist
    MagicProp::new( 5, 1,    0,  0, 1, 0,  0), // [ 40] maxfireresist
    MagicProp::new( 9, 1,  200,  0, 1, 0,  0), // [ 41] lightresist
    MagicProp::new( 5, 1,    0,  0, 1, 0,  0), // [ 42] maxlightresist
    MagicProp::new( 9, 1,  200,  0, 1, 0,  0), // [ 43] coldresist
    MagicProp::new( 5, 1,    0,  0, 1, 0,  0), // [ 44] maxcoldresist
    MagicProp::new( 9, 1,  200,  0, 1, 0,  0), // [ 45] poisonresist
    MagicProp::new( 5, 1,    0,  0, 1, 0,  0), // [ 46] maxpoisonresist
    MagicProp::empty(), // [ 47] damageaura
    MagicProp::new( 8, 2,    0,  0, 1, 0,  0), // [ 48] firemindam
    MagicProp::new( 9, 1,    0,  0, 1, 0,  0), // [ 49] firemaxdam
    MagicProp::new( 6, 2,    0,  0, 1, 0,  0), // [ 50] lightmindam
    MagicProp::new(10, 1,    0,  0, 1, 0,  0), // [ 51] lightmaxdam
    MagicProp::new( 8, 2,    0,  0, 1, 0,  0), // [ 52] magicmindam
    MagicProp::new( 9, 1,    0,  0, 1, 0,  0), // [ 53] magicmaxdam
    MagicProp::new( 8, 3,    0,  0, 1, 0,  0), // [ 54] coldmindam
    MagicProp::new( 9, 1,    0,  0, 1, 0,  0), // [ 55] coldmaxdam
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [ 56] coldlength
    MagicProp::new(10, 3,    0,  0, 1, 0,  0), // [ 57] poisonmindam
    MagicProp::new(10, 1,    0,  0, 1, 0,  0), // [ 58] poisonmaxdam
    MagicProp::new( 9, 1,    0,  0, 1, 0,  0), // [ 59] poisonlength
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [ 60] lifedrainmindam
    MagicProp::empty(), // [ 61] lifedrainmaxdam
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [ 62] manadrainmindam
    MagicProp::empty(), // [ 63] manadrainmaxdam
    MagicProp::empty(), // [ 64] stamdrainmindam
    MagicProp::empty(), // [ 65] stamdrainmaxdam
    MagicProp::empty(), // [ 66] staminarecovery
    MagicProp::new( 7, 1,   30,  0, 1, 0,  0), // [ 67] velocitypercent
    MagicProp::new( 7, 1,   30,  0, 1, 0,  0), // [ 68] attackrate
    MagicProp::empty(), // [ 69] other_animrate
    MagicProp::empty(), // [ 70] quantity
    MagicProp::new( 8, 1,  100,  0, 1, 0,  0), // [ 71] value
    MagicProp::new( 9, 1,    0,  0, 1, 0,  0), // [ 72] durability
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [ 73] maxdurability
    MagicProp::new( 6, 1,   30,  0, 0, 0,  0), // [ 74] hpregen
    MagicProp::new( 7, 1,   20,  0, 1, 0,  0), // [ 75] item_maxdurability_percent
    MagicProp::new( 6, 1,   10,  0, 1, 0,  0), // [ 76] item_maxhp_percent
    MagicProp::new( 6, 1,   10,  0, 1, 0,  0), // [ 77] item_maxmana_percent
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [ 78] item_attackertakesdamage
    MagicProp::new( 9, 1,  100,  0, 1, 0,  0), // [ 79] item_goldbonus
    MagicProp::new( 8, 1,  100,  0, 1, 0,  0), // [ 80] item_magicbonus
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [ 81] item_knockback
    MagicProp::new( 9, 1,   20,  0, 1, 0,  0), // [ 82] item_timeduration
    MagicProp::new( 3, 1,    0,  3, 1, 0,  0), // [ 83] item_addclassskills
    MagicProp::empty(), // [ 84] item_addclassskills2
    MagicProp::new( 9, 1,   50,  0, 1, 0,  0), // [ 85] item_addexperience
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [ 86] item_healafterkill
    MagicProp::empty(), // [ 87] item_healafterkill_percent
    MagicProp::new( 1, 1,    0,  0, 1, 0,  0), // [ 88] item_doubleherbduration
    MagicProp::new( 4, 1,    4,  0, 1, 0,  0), // [ 89] item_lightradius
    MagicProp::new(24, 1,    0,  0, 1, 0,  0), // [ 90] item_lightcolor
    MagicProp::new( 8, 1,  100,  0, 1, 0,  0), // [ 91] item_req_percent
    MagicProp::empty(), // [ 92] item_levelreq
    MagicProp::new( 7, 1,   20,  0, 1, 0,  0), // [ 93] item_fasterattackrate
    MagicProp::empty(), // [ 94] item_levelreq_percent
    MagicProp::empty(), // [ 95] last_block_frame
    MagicProp::new( 7, 1,   20,  0, 1, 0,  0), // [ 96] item_fastermovevelocity
    MagicProp::new( 6, 1,    0,  9, 1, 1,  0), // [ 97] item_nonclassskill
    MagicProp::new( 1, 1,    0,  8, 0, 0,  0), // [ 98] state
    MagicProp::new( 7, 1,   20,  0, 1, 0,  0), // [ 99] item_fastergethitrate
    MagicProp::empty(), // [100] item_fastergethitrate_percent
    MagicProp::empty(), // [101] item_fastergethitrate_avgbased
    MagicProp::new( 7, 1,   20,  0, 1, 0,  0), // [102] item_fasterblockrate
    MagicProp::empty(), // [103] item_fasterblockrate_percent
    MagicProp::empty(), // [104] item_fasterblockrate_avgbased
    MagicProp::new( 7, 1,   20,  0, 1, 0,  0), // [105] item_fastercastrate
    MagicProp::empty(), // [106] item_fastercastrate_percent
    MagicProp::new( 3, 1,    0,  9, 1, 1,  0), // [107] item_singleskill
    MagicProp::empty(), // [108] item_restinpeace
    MagicProp::new( 9, 1,    0,  0, 0, 0,  0), // [109] curse_resistance
    MagicProp::new( 8, 1,   20,  0, 1, 0,  0), // [110] item_poisonlengthresist
    MagicProp::new( 9, 1,   20,  0, 1, 0,  0), // [111] item_normaldamage
    MagicProp::new( 7, 1,   -1,  0, 1, 0,  0), // [112] item_howl
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [113] item_stupidity
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [114] item_damagetomana
    MagicProp::new( 1, 1,    0,  0, 1, 0,  0), // [115] item_ignoretargetac
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [116] item_fractionaltargetac
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [117] item_preventheal
    MagicProp::new( 1, 1,    0,  0, 1, 0,  0), // [118] item_halffreezeduration
    MagicProp::new( 9, 1,   20,  0, 1, 0,  0), // [119] item_tohit_percent
    MagicProp::new( 7, 1,  128,  0, 1, 0,  0), // [120] item_damagetargetac
    MagicProp::new( 9, 1,   20,  0, 1, 0,  0), // [121] item_demondamage_percent
    MagicProp::new( 9, 1,   20,  0, 1, 0,  0), // [122] item_undeaddamage_percent
    MagicProp::new(10, 1,  128,  0, 1, 0,  0), // [123] item_demon_tohit
    MagicProp::new(10, 1,  128,  0, 1, 0,  0), // [124] item_undead_tohit
    MagicProp::new( 1, 1,    0,  0, 1, 0,  0), // [125] item_throwable
    MagicProp::new( 3, 1,    0,  3, 1, 0,  0), // [126] item_elemskill
    MagicProp::new( 3, 1,    0,  0, 1, 0,  0), // [127] item_allskills
    MagicProp::new( 5, 1,    0,  0, 1, 0,  0), // [128] item_attackertakeslightdamage
    MagicProp::empty(), // [129] ironmaiden_level
    MagicProp::empty(), // [130] lifetap_level
    MagicProp::empty(), // [131] thorns_percent
    MagicProp::empty(), // [132] bonearmor
    MagicProp::empty(), // [133] bonearmormax
    MagicProp::new( 5, 1,    0,  0, 1, 0,  0), // [134] item_freeze
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [135] item_openwounds
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [136] item_crushingblow
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [137] item_kickdamage
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [138] item_manaafterkill
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [139] item_healafterdemonkill
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [140] item_extrablood
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [141] item_deadlystrike
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [142] item_absorbfire_percent
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [143] item_absorbfire
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [144] item_absorblight_percent
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [145] item_absorblight
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [146] item_absorbmagic_percent
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [147] item_absorbmagic
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [148] item_absorbcold_percent
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [149] item_absorbcold
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [150] item_slow
    MagicProp::new( 5, 1,    0,  9, 1, 0,  0), // [151] item_aura
    MagicProp::new( 1, 1,    0,  0, 1, 0,  0), // [152] item_indesctructible
    MagicProp::new( 1, 1,    0,  0, 1, 0,  0), // [153] item_cannotbefrozen
    MagicProp::new( 7, 1,   20,  0, 1, 0,  0), // [154] item_staminadrainpct
    MagicProp::new( 7, 1,    0, 10, 0, 0,  0), // [155] item_reanimate
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [156] item_pierce
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [157] item_magicarrow
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [158] item_explosivearrow
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [159] item_throw_mindamage
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [160] item_throw_maxdamage
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
    MagicProp::empty(), // [172] skill_armorclass
    MagicProp::empty(), // [173] skill_hp
    MagicProp::empty(), // [174] skill_mana
    MagicProp::empty(), // [175] skill_stamina
    MagicProp::empty(), // [176] skill_tohit
    MagicProp::empty(), // [177] skill_toblock
    MagicProp::empty(), // [178] skill_mindamage
    MagicProp::new( 9, 1,    0, 10, 0, 0,  0), // [179] attack_vs_montype
    MagicProp::new( 9, 1,    0, 10, 0, 0,  0), // [180] damage_vs_montype
    MagicProp::empty(), // [181] fire_skill_damage
    MagicProp::empty(), // [182] armor_override_percent
    MagicProp::empty(), // [183] lightning_skill_damage
    MagicProp::empty(), // [184] cold_skill_damage
    MagicProp::empty(), // [185] poison_skill_damage
    MagicProp::empty(), // [186] all_skill_damage
    MagicProp::empty(), // [187] player_skill_rank
    MagicProp::new( 3, 1,    0, 16, 1, 0,  0), // [188] item_addskill_tab
    MagicProp::empty(), // [189] item_allattributepoints
    MagicProp::empty(), // [190] item_makemagic
    MagicProp::empty(), // [191] item_blank_skill
    MagicProp::empty(), // [192] item_skillonkill
    MagicProp::empty(), // [193] item_skillondeath_sentry
    MagicProp::new( 4, 1,    0,  0, 1, 0,  0), // [194] item_numsockets
    MagicProp::new( 7, 1,    0, 16, 1, 2,  0), // [195] item_skillonattack
    MagicProp::new( 7, 1,    0, 16, 1, 2,  0), // [196] item_skillonkill
    MagicProp::new( 7, 1,    0, 16, 1, 2,  0), // [197] item_skillondeath
    MagicProp::new( 7, 1,    0, 16, 1, 2,  0), // [198] item_skillonhit
    MagicProp::new( 7, 1,    0, 16, 1, 2,  0), // [199] item_skillonlevelup
    MagicProp::empty(), // [200] item_skillonkill_monster
    MagicProp::new( 7, 1,    0, 16, 1, 2,  0), // [201] item_skillongethit
    MagicProp::empty(), // [202] item_skillonkill_player
    MagicProp::empty(), // [203] item_skillonkill_monster_sorceress
    MagicProp::new(16, 1,    0, 16, 1, 3,  0), // [204] item_charged_skill
    MagicProp::empty(), // [205] unused204
    MagicProp::empty(), // [206] unused205
    MagicProp::empty(), // [207] unused206
    MagicProp::empty(), // [208] unused207
    MagicProp::empty(), // [209] unused208
    MagicProp::empty(), // [210] unused209
    MagicProp::empty(), // [211] unused210
    MagicProp::empty(), // [212] unused211
    MagicProp::empty(), // [213] unused212
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [214] item_armor_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [215] item_armorpercent_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [216] item_hp_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [217] item_mana_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [218] item_maxdamage_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [219] item_maxdamage_percent_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [220] item_strength_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [221] item_dexterity_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [222] item_energy_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [223] item_vitality_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [224] item_tohit_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [225] item_tohitpercent_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [226] item_cold_damagemax_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [227] item_fire_damagemax_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [228] item_ltng_damagemax_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [229] item_pois_damagemax_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [230] item_resist_cold_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [231] item_resist_fire_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [232] item_resist_ltng_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [233] item_resist_pois_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [234] item_absorb_cold_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [235] item_absorb_fire_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [236] item_absorb_ltng_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [237] item_absorb_pois_perlevel
    MagicProp::new( 5, 1,    0,  0, 1, 0,  0), // [238] item_thorns_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [239] item_find_gold_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [240] item_find_magic_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [241] item_regenstamina_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [242] item_stamina_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [243] item_damage_demon_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [244] item_damage_undead_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [245] item_tohit_demon_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [246] item_tohit_undead_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [247] item_crushingblow_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [248] item_openwounds_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [249] item_kick_damage_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [250] item_deadlystrike_perlevel
    MagicProp::empty(), // [251] item_find_gems_perlevel
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [252] item_replenish_durability
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [253] item_replenish_quantity
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [254] item_extra_stack
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
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [268] item_armor_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [269] item_armorpercent_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [270] item_hp_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [271] item_mana_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [272] item_maxdamage_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [273] item_maxdamage_percent_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [274] item_strength_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [275] item_dexterity_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [276] item_energy_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [277] item_vitality_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [278] item_tohit_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [279] item_tohitpercent_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [280] item_cold_damagemax_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [281] item_fire_damagemax_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [282] item_ltng_damagemax_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [283] item_pois_damagemax_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [284] item_resist_cold_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [285] item_resist_fire_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [286] item_resist_ltng_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [287] item_resist_pois_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [288] item_absorb_cold_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [289] item_absorb_fire_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [290] item_absorb_ltng_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [291] item_absorb_pois_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [292] item_find_gold_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [293] item_find_magic_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [294] item_regenstamina_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [295] item_stamina_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [296] item_damage_demon_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [297] item_damage_undead_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [298] item_tohit_demon_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [299] item_tohit_undead_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [300] item_crushingblow_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [301] item_openwounds_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [302] item_kick_damage_bytime
    MagicProp::new(22, 1,    0,  0, 1, 4,  0), // [303] item_deadlystrike_bytime
    MagicProp::empty(), // [304] item_find_gems_bytime
    MagicProp::new( 8, 1,   50,  0, 1, 0,  0), // [305] item_pierce_cold
    MagicProp::new( 8, 1,   50,  0, 1, 0,  0), // [306] item_pierce_fire
    MagicProp::new( 8, 1,   50,  0, 1, 0,  0), // [307] item_pierce_ltng
    MagicProp::new( 8, 1,   50,  0, 1, 0,  0), // [308] item_pierce_pois
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
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [324] item_extra_charges
    MagicProp::empty(), // [325] progressive_tohit
    MagicProp::empty(), // [326] poison_count
    MagicProp::empty(), // [327] damage_framerate
    MagicProp::empty(), // [328] pierce_idx
    MagicProp::new( 9, 1,   50,  0, 1, 0,  0), // [329] passive_fire_mastery
    MagicProp::new( 9, 1,   50,  0, 1, 0,  0), // [330] passive_ltng_mastery
    MagicProp::new( 9, 1,   50,  0, 1, 0,  0), // [331] passive_cold_mastery
    MagicProp::new( 9, 1,   50,  0, 1, 0,  0), // [332] passive_pois_mastery
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [333] passive_fire_pierce
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [334] passive_ltng_pierce
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [335] passive_cold_pierce
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [336] passive_pois_pierce
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [337] passive_critical_strike
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [338] passive_dodge
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [339] passive_avoid
    MagicProp::new( 7, 1,    0,  0, 1, 0,  0), // [340] passive_evade
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [341] passive_warmth
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [342] passive_mastery_melee_th
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [343] passive_mastery_melee_dmg
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [344] passive_mastery_melee_crit
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [345] passive_mastery_throw_th
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [346] passive_mastery_throw_dmg
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [347] passive_mastery_throw_crit
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [348] passive_weaponblock
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [349] passive_summon_resist
    MagicProp::empty(), // [350] modifierlist_skill
    MagicProp::empty(), // [351] modifierlist_level
    MagicProp::empty(), // [352] last_sent_hp_pct
    MagicProp::empty(), // [353] source_unit_type
    MagicProp::empty(), // [354] source_unit_id
    MagicProp::empty(), // [355] shortparam1
    MagicProp::new( 2, 1,    0,  0, 0, 0,  0), // [356] questitemdifficulty
    MagicProp::new( 9, 1,   50,  0, 1, 0,  0), // [357] passive_mag_mastery
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [358] passive_mag_pierce
    MagicProp::empty(), // [359] skill_cooldown
    MagicProp::empty(), // [360] skill_missile_damage_scale
    MagicProp::empty(), // [361] psychicward
    MagicProp::empty(), // [362] psychicwardmax
    MagicProp::empty(), // [363] skill_channeling_tick
    MagicProp::empty(), // [364] customization_index
    MagicProp::new( 6, 1,    0,  0, 1, 0,  0), // [365] item_magic_damagemax_perlevel
    MagicProp::new( 8, 1,    0,  0, 1, 0,  0), // [366] passive_dmg_pierce
    MagicProp::empty(), // [367] heraldtier
    MagicProp::new(10, 1,    0,  0, 0, 0, 10), // [368] coi_inf_t1_count  (saved=1, CSvBits=10)
    MagicProp::new(10, 1,    0,  0, 0, 0, 10), // [369] coi_inf_t1_gate  (saved=1, CSvBits=10)
    MagicProp::new(10, 1,    0,  0, 0, 0, 10), // [370] coi_inf_t2_count  (saved=1, CSvBits=10)
    MagicProp::new(10, 1,    0,  0, 0, 0, 10), // [371] coi_inf_t2_gate  (saved=1, CSvBits=10)
    MagicProp::new(10, 1,    0,  0, 0, 0, 10), // [372] coi_inf_t3_count  (saved=1, CSvBits=10)
    MagicProp::new(10, 1,    0,  0, 0, 0, 10), // [373] coi_inf_t3_gate  (saved=1, CSvBits=10)
    MagicProp::new( 1, 1,    0,  0, 0, 0,  1), // [374] coi_inf_gate_init  (saved=1, CSvBits=1)
    MagicProp::empty(), // [375] (unused)
    MagicProp::empty(), // [376] (unused)
    MagicProp::empty(), // [377] (unused)
    MagicProp::empty(), // [378] (unused)
    MagicProp::empty(), // [379] (unused)
    MagicProp::empty(), // [380] (unused)
    MagicProp::empty(), // [381] (unused)
    MagicProp::empty(), // [382] (unused)
    MagicProp::empty(), // [383] (unused)
    MagicProp::empty(), // [384] (unused)
    MagicProp::empty(), // [385] (unused)
    MagicProp::empty(), // [386] (unused)
    MagicProp::empty(), // [387] (unused)
    MagicProp::empty(), // [388] (unused)
    MagicProp::empty(), // [389] (unused)
    MagicProp::empty(), // [390] (unused)
    MagicProp::empty(), // [391] (unused)
    MagicProp::empty(), // [392] (unused)
    MagicProp::empty(), // [393] (unused)
    MagicProp::empty(), // [394] (unused)
    MagicProp::empty(), // [395] (unused)
    MagicProp::new( 7, 1,    0,  0, 0, 0,  7), // [396] crit  (saved=1, CSvBits=7)
    MagicProp::new(10, 1,    0,  0, 0, 0, 10), // [397] hp-kill  (saved=1, CSvBits=10)
    MagicProp::new(10, 1,    0,  0, 0, 0, 10), // [398] mana-lost  (saved=1, CSvBits=10)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [399] coi_jzb_lin  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [400] coi_jzb_xfu  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [401] coi_jzb_lsh  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [402] coi_jzb_lyd  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [403] coi_jzb_jlf  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [404] coi_jzb_rly  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [405] coi_jzb_nls  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [406] coi_jzb_lck  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [407] coi_jzb_cly  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [408] coi_jzb_qlf  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [409] coi_jzb_cll  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [410] coi_jzb_lgy  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [411] coi_jzb_lgs  (saved=1, CSvBits=14)
    MagicProp::new(14, 1,    0,  0, 0, 0, 14), // [412] coi_jzb_uni  (saved=1, CSvBits=14)
    MagicProp::new( 6, 1,    0,  0, 0, 0,  6), // [413] coi_root_gold  (saved=1, CSvBits=6)
    MagicProp::new( 6, 1,    0,  0, 0, 0,  6), // [414] coi_root_wood  (saved=1, CSvBits=6)
    MagicProp::new( 6, 1,    0,  0, 0, 0,  6), // [415] coi_root_water  (saved=1, CSvBits=6)
    MagicProp::new( 6, 1,    0,  0, 0, 0,  6), // [416] coi_root_fire  (saved=1, CSvBits=6)
    MagicProp::new( 6, 1,    0,  0, 0, 0,  6), // [417] coi_root_earth  (saved=1, CSvBits=6)
    MagicProp::new( 6, 1,    0,  0, 0, 0,  6), // [418] coi_root_light  (saved=1, CSvBits=6)
    MagicProp::new( 6, 1,    0,  0, 0, 0,  6), // [419] coi_root_dark  (saved=1, CSvBits=6)
];

// CSV field indices (itemstatcost.txt):
//   Col 3:  Signed
//   Col 9:  CSvBits
//   Col 14: Encode
//   Col 20: Save Bits
//   Col 21: Save Add
//   Col 22: Save Param Bits
