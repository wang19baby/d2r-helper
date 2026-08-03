/// Magical properties table extracted from D2R game constants (webpack chunk 385ca0d5...js).
///
/// Format: (save_bits, num_sub_props, save_add, save_param_bits, signed, encoding)
///
/// Fields:
/// - `save_bits`: Number of bits to store the value (0 = not used for item serialization)
/// - `num_sub_props`: How many consecutive property IDs this spans
/// - `save_add`: Offset added to value before storing (subtracted on read)
/// - `save_param_bits`: Bits for parameter before value (skill ID, state ID, etc.)
/// - `signed`: Whether the value is signed
/// - `encoding`: Special encoding (0=normal, 1=non-class-skill, 2=cast-on-strike, 3=charged, 4=by-time)
#[derive(Debug, Clone, Copy)]
pub struct MagicProp {
    pub save_bits: u8,
    pub num_sub_props: u8,
    pub save_add: i32,
    pub save_param_bits: u8,
    pub signed: u8,
    pub encoding: u8,
}

impl MagicProp {
    pub const fn new(save_bits: u8, num_sub_props: u8, save_add: i32, save_param_bits: u8, signed: u8, encoding: u8) -> Self {
        Self { save_bits, num_sub_props, save_add, save_param_bits, signed, encoding }
    }

    pub const fn empty() -> Self {
        Self { save_bits: 0, num_sub_props: 1, save_add: 0, save_param_bits: 0, signed: 0, encoding: 0 }
    }
}

#[rustfmt::skip]
pub const MAGICAL_PROPS: &[MagicProp] = &[
    MagicProp::new(8, 1, 32, 0, 0, 0), // [0] strength
    MagicProp::new(7, 1, 32, 0, 0, 0), // [1] energy
    MagicProp::new(7, 1, 32, 0, 0, 0), // [2] dexterity
    MagicProp::new(7, 1, 32, 0, 0, 0), // [3] vitality
    MagicProp::empty(), // [4] statpts (character-only)
    MagicProp::empty(), // [5] newskills (character-only)
    MagicProp::empty(), // [6] hitpoints (character-only)
    MagicProp::new(9, 1, 32, 0, 0, 0), // [7] maxhp
    MagicProp::empty(), // [8] mana (character-only)
    MagicProp::new(8, 1, 32, 0, 0, 0), // [9] maxmana
    MagicProp::empty(), // [10] stamina (character-only)
    MagicProp::new(8, 1, 32, 0, 0, 0), // [11] maxstamina
    MagicProp::empty(), // [12] level (character-only)
    MagicProp::empty(), // [13] experience (character-only)
    MagicProp::empty(), // [14] gold (character-only)
    MagicProp::empty(), // [15] goldbank (character-only)
    MagicProp::new(9, 1, 0, 0, 1, 0), // [16] item_armor_percent
    MagicProp::new(9, 2, 0, 0, 1, 0), // [17] item_maxdamage_percent
    MagicProp::new(9, 1, 0, 0, 1, 0), // [18] item_mindamage_percent
    MagicProp::new(10, 1, 0, 0, 1, 0), // [19] tohit
    MagicProp::new(6, 1, 0, 0, 1, 0), // [20] toblock
    MagicProp::new(6, 1, 0, 0, 1, 0), // [21] mindamage
    MagicProp::new(7, 1, 0, 0, 1, 0), // [22] maxdamage
    MagicProp::new(6, 1, 0, 0, 1, 0), // [23] secondary_mindamage
    MagicProp::new(7, 1, 0, 0, 1, 0), // [24] secondary_maxdamage
    MagicProp::new(8, 1, 0, 0, 1, 0), // [25] damagepercent
    MagicProp::new(8, 1, 0, 0, 0, 0), // [26] manarecovery
    MagicProp::new(8, 1, 0, 0, 1, 0), // [27] manarecoverybonus
    MagicProp::new(8, 1, 0, 0, 1, 0), // [28] staminarecoverybonus
    MagicProp::empty(), // [29] lastexp (character-only)
    MagicProp::empty(), // [30] nextexp (character-only)
    MagicProp::new(11, 1, 10, 0, 1, 0), // [31] armorclass
    MagicProp::new(9, 1, 0, 0, 1, 0), // [32] armorclass_vs_missile
    MagicProp::new(8, 1, 0, 0, 1, 0), // [33] armorclass_vs_hth
    MagicProp::new(6, 1, 0, 0, 1, 0), // [34] normal_damage_reduction
    MagicProp::new(6, 1, 0, 0, 1, 0), // [35] magic_damage_reduction
    MagicProp::new(9, 1, 200, 0, 1, 0), // [36] damageresist
    MagicProp::new(9, 1, 200, 0, 1, 0), // [37] magicresist
    MagicProp::new(5, 1,   0, 0, 1, 0), // [38] maxmagicresist
    MagicProp::new(9, 1, 200, 0, 1, 0), // [39] fireresist
    MagicProp::new(5, 1,   0, 0, 1, 0), // [40] maxfireresist
    MagicProp::new(9, 1, 200, 0, 1, 0), // [41] lightresist
    MagicProp::new(5, 1,   0, 0, 1, 0), // [42] maxlightresist
    MagicProp::new(9, 1, 200, 0, 1, 0), // [43] coldresist
    MagicProp::new(5, 1,   0, 0, 1, 0), // [44] maxcoldresist
    MagicProp::new(9, 1, 200, 0, 1, 0), // [45] poisonresist
    MagicProp::new(5, 1,   0, 0, 1, 0), // [46] maxpoisonresist
    MagicProp::empty(), // [47] damageaura (character-only)
    MagicProp::new(8, 2, 0, 0, 1, 0), // [48] firemindam
    MagicProp::new(9, 1, 0, 0, 1, 0), // [49] firemaxdam
    MagicProp::new(6, 2, 0, 0, 1, 0), // [50] lightmindam
    MagicProp::new(10, 1, 0, 0, 1, 0), // [51] lightmaxdam
    MagicProp::new(8, 2, 0, 0, 1, 0), // [52] magicmindam
    MagicProp::new(9, 1, 0, 0, 1, 0), // [53] magicmaxdam
    MagicProp::new(8, 3, 0, 0, 1, 0), // [54] coldmindam
    MagicProp::new(9, 1, 0, 0, 1, 0), // [55] coldmaxdam
    MagicProp::new(8, 1, 0, 0, 1, 0), // [56] coldlength
    MagicProp::new(10, 3, 0, 0, 1, 0), // [57] poisonmindam
    MagicProp::new(10, 1, 0, 0, 1, 0), // [58] poisonmaxdam
    MagicProp::new(9, 1, 0, 0, 1, 0), // [59] poisonlength
    MagicProp::new(7, 1, 0, 0, 1, 0), // [60] lifedrainmindam
    MagicProp::empty(), // [61] lifedrainmaxdam (character-only)
    MagicProp::new(7, 1, 0, 0, 1, 0), // [62] manadrainmindam
    MagicProp::empty(), // [63] manadrainmaxdam (character-only)
    MagicProp::empty(), // [64] stamdrainmindam (character-only)
    MagicProp::empty(), // [65] stamdrainmaxdam (character-only)
    MagicProp::empty(), // [66] staminarecovery (character-only)
    MagicProp::new(7, 1, 30, 0, 1, 0), // [67] velocitypercent
    MagicProp::new(7, 1, 30, 0, 1, 0), // [68] attackrate
    MagicProp::empty(), // [69] other_animrate (character-only)
    MagicProp::empty(), // [70] quantity (character-only)
    MagicProp::new(8, 1, 100, 0, 1, 0), // [71] value
    MagicProp::new(9, 1, 0, 0, 1, 0), // [72] durability
    MagicProp::new(8, 1, 0, 0, 1, 0), // [73] maxdurability
    MagicProp::new(6, 1, 30, 0, 0, 0), // [74] hpregen
    MagicProp::new(7, 1, 20, 0, 1, 0), // [75] item_maxdurability_percent
    MagicProp::new(6, 1, 10, 0, 1, 0), // [76] item_maxhp_percent
    MagicProp::new(6, 1, 10, 0, 1, 0), // [77] item_maxmana_percent
    MagicProp::new(7, 1, 0, 0, 1, 0), // [78] item_attackertakesdamage
    MagicProp::new(9, 1, 100, 0, 1, 0), // [79] item_goldbonus
    MagicProp::new(8, 1, 100, 0, 1, 0), // [80] item_magicbonus
    MagicProp::new(7, 1, 0, 0, 1, 0), // [81] item_knockback
    MagicProp::new(9, 1, 20, 0, 1, 0), // [82] item_timeduration
    MagicProp::new(3, 1, 0, 3, 1, 0), // [83] item_addclassskills
    MagicProp::empty(), // [84] item_addclassskills2 (unused)
    MagicProp::new(9, 1, 50, 0, 1, 0), // [85] item_addexperience
    MagicProp::new(7, 1, 0, 0, 1, 0), // [86] item_healafterkill
    MagicProp::empty(), // [87] item_healafterkill_percent (character-only)
    MagicProp::new(1, 1, 0, 0, 1, 0), // [88] item_doubleherbduration
    MagicProp::new(4, 1, 4, 0, 1, 0), // [89] item_lightradius
    MagicProp::new(24, 1, 0, 0, 1, 0), // [90] item_lightcolor
    MagicProp::new(8, 1, 100, 0, 1, 0), // [91] item_req_percent
    MagicProp::empty(), // [92] item_levelreq (character-only)
    MagicProp::new(7, 1, 20, 0, 1, 0), // [93] item_fasterattackrate
    MagicProp::empty(), // [94] item_levelreq_percent (character-only)
    MagicProp::empty(), // [95] last_block_frame (character-only)
    MagicProp::new(7, 1, 20, 0, 1, 0), // [96] item_fastermovevelocity
    MagicProp::new(6, 1, 0, 9, 1, 1), // [97] item_nonclassskill
    MagicProp::new(1, 1, 0, 8, 0, 0), // [98] state
    MagicProp::new(7, 1, 20, 0, 1, 0), // [99] item_fastergethitrate
    MagicProp::empty(), // [100] item_fastergethitrate_percent (character-only)
    MagicProp::empty(), // [101] item_fastergethitrate_avgbased (character-only)
    MagicProp::new(7, 1, 20, 0, 1, 0), // [102] item_fasterblockrate
    MagicProp::empty(), // [103] item_fasterblockrate_percent (character-only)
    MagicProp::empty(), // [104] item_fasterblockrate_avgbased (character-only)
    MagicProp::new(7, 1, 20, 0, 1, 0), // [105] item_fastercastrate
    MagicProp::empty(), // [106] item_fastercastrate_percent (character-only)
    MagicProp::new(3, 1, 0, 9, 1, 1), // [107] item_singleskill
    MagicProp::empty(), // [108] item_restinpeace (character-only)
    MagicProp::new(9, 1, 0, 0, 0, 0), // [109] curse_resistance
    MagicProp::new(8, 1, 20, 0, 1, 0), // [110] item_poisonlengthresist
    MagicProp::new(9, 1, 20, 0, 1, 0), // [111] item_normaldamage
    MagicProp::new(7, 1, -1, 0, 1, 0), // [112] item_howl
    MagicProp::new(7, 1, 0, 0, 1, 0), // [113] item_stupidity
    MagicProp::new(6, 1, 0, 0, 1, 0), // [114] item_damagetomana
    MagicProp::new(1, 1, 0, 0, 1, 0), // [115] item_ignoretargetac
    MagicProp::new(7, 1, 0, 0, 1, 0), // [116] item_fractionaltargetac
    MagicProp::new(7, 1, 0, 0, 1, 0), // [117] item_preventheal
    MagicProp::new(1, 1, 0, 0, 1, 0), // [118] item_halffreezeduration
    MagicProp::new(9, 1, 20, 0, 1, 0), // [119] item_tohit_percent
    MagicProp::new(7, 1, 128, 0, 1, 0), // [120] item_damagetargetac
    MagicProp::new(9, 1, 20, 0, 1, 0), // [121] item_demondamage_percent
    MagicProp::new(9, 1, 20, 0, 1, 0), // [122] item_undeaddamage_percent
    MagicProp::new(10, 1, 128, 0, 1, 0), // [123] item_demon_tohit
    MagicProp::new(10, 1, 128, 0, 1, 0), // [124] item_undead_tohit
    MagicProp::new(1, 1, 0, 0, 1, 0), // [125] item_throwable
    MagicProp::new(3, 1, 0, 3, 1, 0), // [126] item_elemskill
    MagicProp::new(3, 1, 0, 0, 1, 0), // [127] item_allskills
    MagicProp::new(5, 1, 0, 0, 1, 0), // [128] item_attackertakeslightdamage
    MagicProp::empty(), // [129] ironmaiden_level (character-only)
    MagicProp::empty(), // [130] lifetap_level (character-only)
    MagicProp::empty(), // [131] thorns_percent (character-only)
    MagicProp::empty(), // [132] bonearmor (character-only)
    MagicProp::empty(), // [133] bonearmormax (character-only)
    MagicProp::new(5, 1, 0, 0, 1, 0), // [134] item_freeze
    MagicProp::new(7, 1, 0, 0, 1, 0), // [135] item_openwounds
    MagicProp::new(7, 1, 0, 0, 1, 0), // [136] item_crushingblow
    MagicProp::new(7, 1, 0, 0, 1, 0), // [137] item_kickdamage
    MagicProp::new(7, 1, 0, 0, 1, 0), // [138] item_manaafterkill
    MagicProp::new(7, 1, 0, 0, 1, 0), // [139] item_healafterdemonkill
    MagicProp::new(7, 1, 0, 0, 1, 0), // [140] item_extrablood
    MagicProp::new(7, 1, 0, 0, 1, 0), // [141] item_deadlystrike
    MagicProp::new(7, 1, 0, 0, 1, 0), // [142] item_absorbfire_percent
    MagicProp::new(7, 1, 0, 0, 1, 0), // [143] item_absorbfire
    MagicProp::new(7, 1, 0, 0, 1, 0), // [144] item_absorblight_percent
    MagicProp::new(7, 1, 0, 0, 1, 0), // [145] item_absorblight
    MagicProp::new(7, 1, 0, 0, 1, 0), // [146] item_absorbmagic_percent
    MagicProp::new(7, 1, 0, 0, 1, 0), // [147] item_absorbmagic
    MagicProp::new(7, 1, 0, 0, 1, 0), // [148] item_absorbcold_percent
    MagicProp::new(7, 1, 0, 0, 1, 0), // [149] item_absorbcold
    MagicProp::new(7, 1, 0, 0, 1, 0), // [150] item_slow
    MagicProp::new(5, 1, 0, 9, 1, 0), // [151] item_aura
    MagicProp::new(1, 1, 0, 0, 1, 0), // [152] item_indesctructible
    MagicProp::new(1, 1, 0, 0, 1, 0), // [153] item_cannotbefrozen
    MagicProp::new(7, 1, 20, 0, 1, 0), // [154] item_staminadrainpct
    MagicProp::new(7, 1, 0, 10, 0, 0), // [155] item_reanimate
    MagicProp::new(7, 1, 0, 0, 1, 0), // [156] item_pierce
    MagicProp::new(7, 1, 0, 0, 1, 0), // [157] item_magicarrow
    MagicProp::new(7, 1, 0, 0, 1, 0), // [158] item_explosivearrow
    MagicProp::new(6, 1, 0, 0, 1, 0), // [159] item_throw_mindamage
    MagicProp::new(7, 1, 0, 0, 1, 0), // [160] item_throw_maxdamage
    MagicProp::empty(), // [161] skill_handofathena (character-only)
    MagicProp::empty(), // [162] skill_staminapercent (character-only)
    MagicProp::empty(), // [163] skill_passive_staminapercent (character-only)
    MagicProp::empty(), // [164] skill_concentration (character-only)
    MagicProp::empty(), // [165] skill_enchant (character-only)
    MagicProp::empty(), // [166] skill_pierce (character-only)
    MagicProp::empty(), // [167] skill_conviction (character-only)
    MagicProp::empty(), // [168] skill_chillingarmor (character-only)
    MagicProp::empty(), // [169] skill_frenzy (character-only)
    MagicProp::empty(), // [170] skill_decrepify (character-only)
    MagicProp::empty(), // [171] skill_armor_percent (character-only)
    MagicProp::empty(), // [172] skill_armorclass (character-only)
    MagicProp::empty(), // [173] skill_hp (character-only)
    MagicProp::empty(), // [174] skill_mana (character-only)
    MagicProp::empty(), // [175] skill_stamina (character-only)
    MagicProp::empty(), // [176] skill_tohit (character-only)
    MagicProp::empty(), // [177] skill_toblock (character-only)
    MagicProp::empty(), // [178] skill_mindamage (character-only)
    MagicProp::new(9, 1, 0, 10, 0, 0), // [179] attack_vs_montype
    MagicProp::new(9, 1, 0, 10, 0, 0), // [180] damage_vs_montype
    MagicProp::empty(), // [181] fire_skill_damage (character-only)
    MagicProp::empty(), // [182] armor_override_percent (character-only)
    MagicProp::empty(), // [183] lightning_skill_damage (character-only)
    MagicProp::empty(), // [184] cold_skill_damage (character-only)
    MagicProp::empty(), // [185] poison_skill_damage (character-only)
    MagicProp::empty(), // [186] all_skill_damage (character-only)
    MagicProp::empty(), // [187] player_skill_rank (character-only)
    MagicProp::new(3, 1, 0, 16, 1, 0), // [188] item_addskill_tab
    MagicProp::empty(), // [189] item_allattributepoints (unused)
    MagicProp::empty(), // [190] item_makemagic (character-only)
    MagicProp::empty(), // [191] item_blank_skill (character-only)
    MagicProp::empty(), // [192] item_skillonkill (character-only)
    MagicProp::empty(), // [193] item_skillondeath_sentry (character-only)
    MagicProp::new(4, 1, 0, 0, 1, 0), // [194] item_numsockets
    MagicProp::new(7, 1, 0, 16, 1, 2), // [195] item_skillonattack
    MagicProp::new(7, 1, 0, 16, 1, 2), // [196] item_skillonkill
    MagicProp::new(7, 1, 0, 16, 1, 2), // [197] item_skillondeath
    MagicProp::new(7, 1, 0, 16, 1, 2), // [198] item_skillonhit
    MagicProp::new(7, 1, 0, 16, 1, 2), // [199] item_skillonlevelup
    MagicProp::empty(), // [200] item_skillonkill_monster (character-only)
    MagicProp::new(7, 1, 0, 16, 1, 2), // [201] item_skillongethit
    MagicProp::empty(), // [202] item_skillonkill_player (character-only)
    MagicProp::empty(), // [203] item_skillonkill_monster_sorceress (character-only)
    MagicProp::new(16, 1, 0, 16, 1, 3), // [204] item_charged_skill
    MagicProp::empty(), // [205] unused204 (character-only)
    MagicProp::empty(), // [206] unused205 (character-only)
    MagicProp::empty(), // [207] unused206 (character-only)
    MagicProp::empty(), // [208] unused207 (character-only)
    MagicProp::empty(), // [209] unused208 (character-only)
    MagicProp::empty(), // [210] unused209 (character-only)
    MagicProp::empty(), // [211] unused210 (character-only)
    MagicProp::empty(), // [212] unused211 (character-only)
    MagicProp::empty(), // [213] unused212 (character-only)
    MagicProp::new(6, 1, 0, 0, 1, 0), // [214] item_armor_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [215] item_armorpercent_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [216] item_hp_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [217] item_mana_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [218] item_maxdamage_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [219] item_maxdamage_percent_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [220] item_strength_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [221] item_dexterity_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [222] item_energy_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [223] item_vitality_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [224] item_tohit_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [225] item_tohitpercent_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [226] item_cold_damagemax_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [227] item_fire_damagemax_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [228] item_ltng_damagemax_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [229] item_pois_damagemax_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [230] item_resist_cold_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [231] item_resist_fire_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [232] item_resist_ltng_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [233] item_resist_pois_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [234] item_absorb_cold_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [235] item_absorb_fire_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [236] item_absorb_ltng_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [237] item_absorb_pois_perlevel
    MagicProp::new(5, 1, 0, 0, 1, 0), // [238] item_thorns_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [239] item_find_gold_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [240] item_find_magic_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [241] item_regenstamina_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [242] item_stamina_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [243] item_damage_demon_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [244] item_damage_undead_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [245] item_tohit_demon_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [246] item_tohit_undead_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [247] item_crushingblow_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [248] item_openwounds_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [249] item_kick_damage_perlevel
    MagicProp::new(6, 1, 0, 0, 1, 0), // [250] item_deadlystrike_perlevel
    MagicProp::empty(), // [251] item_find_gems_perlevel (character-only)
    MagicProp::new(6, 1, 0, 0, 1, 0), // [252] item_replenish_durability
    MagicProp::new(6, 1, 0, 0, 1, 0), // [253] item_replenish_quantity
    MagicProp::new(8, 1, 0, 0, 1, 0), // [254] item_extra_stack
    MagicProp::empty(), // [255] item_find_item (character-only)
    MagicProp::empty(), // [256] item_slash_damage (character-only)
    MagicProp::empty(), // [257] item_slash_damage_percent (character-only)
    MagicProp::empty(), // [258] item_crush_damage (character-only)
    MagicProp::empty(), // [259] item_crush_damage_percent (character-only)
    MagicProp::empty(), // [260] item_thrust_damage (character-only)
    MagicProp::empty(), // [261] item_thrust_damage_percent (character-only)
    MagicProp::empty(), // [262] item_absorb_slash (character-only)
    MagicProp::empty(), // [263] item_absorb_crush (character-only)
    MagicProp::empty(), // [264] item_absorb_thrust (character-only)
    MagicProp::empty(), // [265] item_absorb_slash_percent (character-only)
    MagicProp::empty(), // [266] item_absorb_crush_percent (character-only)
    MagicProp::empty(), // [267] item_absorb_thrust_percent (character-only)
    MagicProp::new(22, 1, 0, 0, 1, 4), // [268] item_armor_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [269] item_armorpercent_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [270] item_hp_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [271] item_mana_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [272] item_maxdamage_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [273] item_maxdamage_percent_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [274] item_strength_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [275] item_dexterity_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [276] item_energy_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [277] item_vitality_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [278] item_tohit_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [279] item_tohitpercent_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [280] item_cold_damagemax_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [281] item_fire_damagemax_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [282] item_ltng_damagemax_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [283] item_pois_damagemax_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [284] item_resist_cold_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [285] item_resist_fire_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [286] item_resist_ltng_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [287] item_resist_pois_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [288] item_absorb_cold_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [289] item_absorb_fire_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [290] item_absorb_ltng_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [291] item_absorb_pois_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [292] item_find_gold_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [293] item_find_magic_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [294] item_regenstamina_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [295] item_stamina_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [296] item_damage_demon_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [297] item_damage_undead_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [298] item_tohit_demon_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [299] item_tohit_undead_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [300] item_crushingblow_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [301] item_openwounds_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [302] item_kick_damage_bytime
    MagicProp::new(22, 1, 0, 0, 1, 4), // [303] item_deadlystrike_bytime
    MagicProp::empty(), // [304] item_find_gems_bytime (character-only)
    MagicProp::new(8, 1, 50, 0, 1, 0), // [305] item_pierce_cold
    MagicProp::new(8, 1, 50, 0, 1, 0), // [306] item_pierce_fire
    MagicProp::new(8, 1, 50, 0, 1, 0), // [307] item_pierce_ltng
    MagicProp::new(8, 1, 50, 0, 1, 0), // [308] item_pierce_pois
    MagicProp::empty(), // [309] item_damage_vs_monster (character-only)
    MagicProp::empty(), // [310] item_damage_percent_vs_monster (character-only)
    MagicProp::empty(), // [311] item_tohit_vs_monster (character-only)
    MagicProp::empty(), // [312] item_tohit_percent_vs_monster (character-only)
    MagicProp::empty(), // [313] item_ac_vs_monster (character-only)
    MagicProp::empty(), // [314] item_ac_percent_vs_monster (character-only)
    MagicProp::empty(), // [315] firelength (character-only)
    MagicProp::empty(), // [316] burningmin (character-only)
    MagicProp::empty(), // [317] burningmax (character-only)
    MagicProp::empty(), // [318] progressive_damage (character-only)
    MagicProp::empty(), // [319] progressive_steal (character-only)
    MagicProp::empty(), // [320] progressive_other (character-only)
    MagicProp::empty(), // [321] progressive_fire (character-only)
    MagicProp::empty(), // [322] progressive_cold (character-only)
    MagicProp::empty(), // [323] progressive_lightning (character-only)
    MagicProp::new(6, 1, 0, 0, 1, 0), // [324] item_extra_charges
    MagicProp::empty(), // [325] progressive_tohit (character-only)
    MagicProp::empty(), // [326] poison_count (character-only)
    MagicProp::empty(), // [327] damage_framerate (character-only)
    MagicProp::empty(), // [328] pierce_idx (character-only)
    MagicProp::new(9, 1, 50, 0, 1, 0), // [329] passive_fire_mastery
    MagicProp::new(9, 1, 50, 0, 1, 0), // [330] passive_ltng_mastery
    MagicProp::new(9, 1, 50, 0, 1, 0), // [331] passive_cold_mastery
    MagicProp::new(9, 1, 50, 0, 1, 0), // [332] passive_pois_mastery
    MagicProp::new(8, 1, 0, 0, 1, 0), // [333] passive_fire_pierce
    MagicProp::new(8, 1, 0, 0, 1, 0), // [334] passive_ltng_pierce
    MagicProp::new(8, 1, 0, 0, 1, 0), // [335] passive_cold_pierce
    MagicProp::new(8, 1, 0, 0, 1, 0), // [336] passive_pois_pierce
    MagicProp::new(8, 1, 0, 0, 1, 0), // [337] passive_critical_strike
    MagicProp::new(7, 1, 0, 0, 1, 0), // [338] passive_dodge
    MagicProp::new(7, 1, 0, 0, 1, 0), // [339] passive_avoid
    MagicProp::new(7, 1, 0, 0, 1, 0), // [340] passive_evade
    MagicProp::new(8, 1, 0, 0, 1, 0), // [341] passive_warmth
    MagicProp::new(8, 1, 0, 0, 1, 0), // [342] passive_mastery_melee_th
    MagicProp::new(8, 1, 0, 0, 1, 0), // [343] passive_mastery_melee_dmg
    MagicProp::new(8, 1, 0, 0, 1, 0), // [344] passive_mastery_melee_crit
    MagicProp::new(8, 1, 0, 0, 1, 0), // [345] passive_mastery_throw_th
    MagicProp::new(8, 1, 0, 0, 1, 0), // [346] passive_mastery_throw_dmg
    MagicProp::new(8, 1, 0, 0, 1, 0), // [347] passive_mastery_throw_crit
    MagicProp::new(8, 1, 0, 0, 1, 0), // [348] passive_weaponblock
    MagicProp::new(8, 1, 0, 0, 1, 0), // [349] passive_summon_resist
    MagicProp::empty(), // [350] modifierlist_skill (character-only)
    MagicProp::empty(), // [351] modifierlist_level (character-only)
    MagicProp::empty(), // [352] last_sent_hp_pct (character-only)
    MagicProp::empty(), // [353] source_unit_type (character-only)
    MagicProp::empty(), // [354] source_unit_id (character-only)
    MagicProp::empty(), // [355] shortparam1 (character-only)
    MagicProp::new(2, 1, 0, 0, 0, 0), // [356] questitemdifficulty
    MagicProp::new(9, 1, 50, 0, 1, 0), // [357] passive_mag_mastery
    MagicProp::new(8, 1, 0, 0, 1, 0), // [358] passive_mag_pierce
    // Mod-added stats from D2RMM 仙道轮回 (itemstatcost.txt)
    MagicProp::new( 0, 1,   0,  0, 0, 0), // [359] skill_cooldown
    MagicProp::new( 0, 1,   0,  0, 0, 0), // [360] skill_missile_damage_scale
    MagicProp::new( 0, 1,   0,  0, 1, 0), // [361] psychicward
    MagicProp::new( 0, 1,   0,  0, 1, 0), // [362] psychicwardmax
    MagicProp::new( 0, 1,   0,  0, 0, 0), // [363] skill_channeling_tick
    MagicProp::new( 0, 1,   0,  0, 0, 0), // [364] customization_index
    MagicProp::new( 6, 1,   0,  0, 1, 0), // [365] item_magic_damagemax_perlevel
    MagicProp::new( 8, 1,   0,  0, 1, 0), // [366] passive_dmg_pierce
    MagicProp::new( 0, 1,   0,  0, 1, 0), // [367] heraldtier
    MagicProp::new(10, 1,   0,  0, 0, 0), // [368] coi_inf_t1_count
    MagicProp::new(10, 1,   0,  0, 0, 0), // [369] coi_inf_t1_gate
    MagicProp::new(10, 1,   0,  0, 0, 0), // [370] coi_inf_t2_count
    MagicProp::new(10, 1,   0,  0, 0, 0), // [371] coi_inf_t2_gate
    MagicProp::new(10, 1,   0,  0, 0, 0), // [372] coi_inf_t3_count
    MagicProp::new(10, 1,   0,  0, 0, 0), // [373] coi_inf_t3_gate
    MagicProp::new( 1, 1,   0,  0, 0, 0), // [374] coi_inf_gate_init
    // [375]-[395] are not in any itemstatcost — empty gap to keep array indices aligned
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
    MagicProp::new( 7, 1,   0,  0, 0, 0), // [396] crit
    MagicProp::new(10, 1,   0,  0, 0, 0), // [397] hp-kill
    MagicProp::new(10, 1,   0,  0, 0, 0), // [398] mana-lost
    MagicProp::new(14, 1,   0,  0, 0, 0), // [399] coi_jzb_lin
    MagicProp::new(14, 1,   0,  0, 0, 0), // [400] coi_jzb_xfu
    MagicProp::new(14, 1,   0,  0, 0, 0), // [401] coi_jzb_lsh
    MagicProp::new(14, 1,   0,  0, 0, 0), // [402] coi_jzb_lyd
    MagicProp::new(14, 1,   0,  0, 0, 0), // [403] coi_jzb_jlf
    MagicProp::new(14, 1,   0,  0, 0, 0), // [404] coi_jzb_rly
    MagicProp::new(14, 1,   0,  0, 0, 0), // [405] coi_jzb_nls
    MagicProp::new(14, 1,   0,  0, 0, 0), // [406] coi_jzb_lck
    MagicProp::new(14, 1,   0,  0, 0, 0), // [407] coi_jzb_cly
    MagicProp::new(14, 1,   0,  0, 0, 0), // [408] coi_jzb_qlf
    MagicProp::new(14, 1,   0,  0, 0, 0), // [409] coi_jzb_cll
    MagicProp::new(14, 1,   0,  0, 0, 0), // [410] coi_jzb_lgy
    MagicProp::new(14, 1,   0,  0, 0, 0), // [411] coi_jzb_lgs
    MagicProp::new(14, 1,   0,  0, 0, 0), // [412] coi_jzb_uni
    MagicProp::new( 6, 1,   0,  0, 0, 0), // [413] coi_root_gold
    MagicProp::new( 6, 1,   0,  0, 0, 0), // [414] coi_root_wood
    MagicProp::new( 6, 1,   0,  0, 0, 0), // [415] coi_root_water
    MagicProp::new( 6, 1,   0,  0, 0, 0), // [416] coi_root_fire
    MagicProp::new( 6, 1,   0,  0, 0, 0), // [417] coi_root_earth
    MagicProp::new( 6, 1,   0,  0, 0, 0), // [418] coi_root_light
    MagicProp::new( 6, 1,   0,  0, 0, 0), // [419] coi_root_dark
];
