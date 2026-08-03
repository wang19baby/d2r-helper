//! Build knowledge base — JSON-defined character builds.
//!
//! Each `.json` file in this directory defines a single build
//! with core equipment, optional items, runewords, skills, and
//! acquisition tips.
//!
//! Embedded at compile time for production builds.

/// All build JSON strings, embedded for production use.
pub const BUILD_JSONS: &[(&str, &str)] = &[
    ("sorceress_cold", include_str!("sorceress_cold.json")),
    ("sorceress_lightning", include_str!("sorceress_lightning.json")),
    ("paladin_hammerdin", include_str!("paladin_hammerdin.json")),
    ("paladin_smiter", include_str!("paladin_smiter.json")),
    ("barbarian_frenzy", include_str!("barbarian_frenzy.json")),
    ("barbarian_whirlwind", include_str!("barbarian_whirlwind.json")),
    ("necromancer_bone", include_str!("necromancer_bone.json")),
    ("necromancer_summon", include_str!("necromancer_summon.json")),
    ("amazon_javazon", include_str!("amazon_javazon.json")),
    ("assassin_trapsin", include_str!("assassin_trapsin.json")),
    ("druid_windy", include_str!("druid_windy.json")),
];
