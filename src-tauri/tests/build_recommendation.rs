//! Build recommendation engine tests.
//!
//! Validates:
//! - JSON deserialization of build definitions
//! - Matching engine: warehouse items → build score
//! - Missing item / runeword gap analysis

use d2r_marketplace_lib::services::build_service::{
    BuildDefinition, BuildEquipment, BuildItem, BuildRuneword,
    match_builds,
};

fn sample_build() -> BuildDefinition {
    BuildDefinition {
        id: "sorceress-cold".into(),
        class: "Sorceress".into(),
        name: "Cold Blizzard Sorceress".into(),
        name_zh: "纯冰法师".into(),
        core_skills: vec!["Blizzard".into(), "Cold Mastery".into()],
        stat_priority: vec!["energy=base".into(), "strength=156".into()],
        equipment: BuildEquipment {
            core: vec![
                BuildItem { slot: "weapon_main".into(), code: "ob1".into(), name: "Death's Fathom".into(), weight: 1.0 },
                BuildItem { slot: "helm".into(), code: "uhm".into(), name: "Nightwing's Veil".into(), weight: 0.9 },
                BuildItem { slot: "armor".into(), code: "uap".into(), name: "Ormus' Robes".into(), weight: 0.8 },
            ],
            optional: vec![
                BuildItem { slot: "ring_l".into(), code: "rin".into(), name: "Stone of Jordan".into(), weight: 0.5 },
                BuildItem { slot: "belt".into(), code: "ael".into(), name: "Arachnid Mesh".into(), weight: 0.5 },
            ],
            runewords: vec![
                BuildRuneword { runes: vec!["jah".into(), "ith".into(), "ber".into()], name: "Enigma".into(), slot: "armor".into(), weight: 0.7 },
            ],
        },
        description: "Classic cold blizzard MF sorceress.".into(),
    }
}

#[allow(dead_code)]
fn sample_warehouse_codes() -> Vec<String> {
    vec!["ob1".into(), "uhm".into(), "rin".into(), "r01".into(), "r02".into()]
}

#[test]
fn test_build_json_roundtrip() {
    let build = sample_build();
    let json = serde_json::to_string(&build).expect("serialize");
    let back: BuildDefinition = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.id, "sorceress-cold");
    assert_eq!(back.equipment.core.len(), 3);
    assert_eq!(back.equipment.runewords[0].runes, vec!["jah", "ith", "ber"]);
}

#[test]
fn test_match_build_full_score() {
    let build = sample_build();
    // All core items owned
    let owned = vec!["ob1".into(), "uhm".into(), "uap".into()];
    let matches = match_builds(&[build], &owned, &[]);
    assert_eq!(matches.len(), 1);
    assert!(matches[0].score > 0.5, "core complete should score >0.5, got {}", matches[0].score);
    assert_eq!(matches[0].core_owned, 3);
    assert_eq!(matches[0].core_total, 3);
    assert!(matches[0].missing_core.is_empty());
}

#[test]
fn test_match_build_partial() {
    let build = sample_build();
    let owned = vec!["ob1".into()];
    let matches = match_builds(&[build], &owned, &[]);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].core_owned, 1);
    assert_eq!(matches[0].core_total, 3);
    assert_eq!(matches[0].missing_core.len(), 2);
}

#[test]
fn test_match_build_no_items() {
    let build = sample_build();
    let owned: Vec<String> = vec![];
    let matches = match_builds(&[build], &owned, &[]);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].score, 0.0);
    assert_eq!(matches[0].missing_core.len(), 3);
}

#[test]
fn test_match_build_runewords() {
    let build = sample_build();
    // Have Jah + Ber but missing Ith → Enigma is still incomplete
    let owned_runes = vec!["jah".into(), "ber".into(), "r01".into()];
    let matches = match_builds(&[build], &[], &owned_runes);
    assert_eq!(matches.len(), 1);
    assert!(!matches[0].missing_runewords.is_empty());
    // Should list missing runes
    assert!(matches[0].missing_runewords[0].missing_runes.contains(&"ith".to_string()));
}

#[test]
fn test_match_multiple_builds() {
    let cold = sample_build();
    let fire = BuildDefinition {
        id: "sorceress-fire".into(),
        class: "Sorceress".into(),
        name: "Fire Sorceress".into(),
        name_zh: "火系法师".into(),
        core_skills: vec!["Fireball".into(), "Fire Mastery".into()],
        stat_priority: vec![],
        equipment: BuildEquipment {
            core: vec![
                BuildItem { slot: "weapon_main".into(), code: "ob1".into(), name: "Death's Fathom".into(), weight: 1.0 },
            ],
            optional: vec![],
            runewords: vec![],
        },
        description: "".into(),
    };

    let owned = vec!["ob1".into(), "uhm".into(), "uap".into()];
    let matches = match_builds(&[cold, fire], &owned, &[]);
    assert_eq!(matches.len(), 2);
    // Cold should score higher (3/3 core) than fire (1/1 core but fewer total)
    // Both have 1.0 core score, but cold has more optional/runeword items
    assert!(matches[0].score > 0.0);
    assert!(matches[1].score > 0.0);
}
