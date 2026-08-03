//! Build recommendation engine.
//!
//! Loads build definitions from `data/builds/*.json`, matches
//! against warehouse items to score each build, and reports gaps.

use serde::{Deserialize, Serialize};

// ── Build Definition (from JSON) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDefinition {
    pub id: String,
    pub class: String,
    pub name: String,
    pub name_zh: String,
    #[serde(default)]
    pub core_skills: Vec<String>,
    #[serde(default)]
    pub stat_priority: Vec<String>,
    pub equipment: BuildEquipment,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildEquipment {
    #[serde(default)]
    pub core: Vec<BuildItem>,
    #[serde(default)]
    pub optional: Vec<BuildItem>,
    #[serde(default)]
    pub runewords: Vec<BuildRuneword>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildItem {
    pub slot: String,
    pub code: String,
    pub name: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 { 0.5 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRuneword {
    pub runes: Vec<String>,
    pub name: String,
    pub slot: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
}

// ── Match Result (to frontend) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMatch {
    pub build_id: String,
    pub class: String,
    pub name: String,
    pub name_zh: String,
    pub score: f64,
    pub core_owned: usize,
    pub core_total: usize,
    pub optional_owned: usize,
    pub optional_total: usize,
    pub owned_items: Vec<OwnedItem>,
    pub missing_core: Vec<MissingItem>,
    pub missing_runewords: Vec<MissingRuneword>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedItem {
    pub slot: String,
    pub name: String,
    pub code: String,
    pub source: String, // "warehouse" | "rune_stash"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingItem {
    pub slot: String,
    pub name: String,
    pub code: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingRuneword {
    pub name: String,
    pub runes: Vec<String>,
    pub missing_runes: Vec<String>,
}

// ── Matching Engine ──

/// Score all builds against owned item codes and owned rune codes.
/// Returns builds sorted by score descending.
pub fn match_builds(
    builds: &[BuildDefinition],
    owned_item_codes: &[String],
    owned_rune_codes: &[String],
) -> Vec<BuildMatch> {
    let mut results: Vec<BuildMatch> = builds.iter().map(|build| {
        let mut owned_items: Vec<OwnedItem> = Vec::new();
        let mut missing_core: Vec<MissingItem> = Vec::new();
        let mut core_owned = 0usize;
        let mut optional_owned = 0usize;

        // Check core items
        for req in &build.equipment.core {
            if owned_item_codes.iter().any(|c| c == &req.code) {
                core_owned += 1;
                owned_items.push(OwnedItem {
                    slot: req.slot.clone(),
                    name: req.name.clone(),
                    code: req.code.clone(),
                    source: "warehouse".into(),
                });
            } else {
                missing_core.push(MissingItem {
                    slot: req.slot.clone(),
                    name: req.name.clone(),
                    code: req.code.clone(),
                    weight: req.weight,
                });
            }
        }

        // Check optional items
        for opt in &build.equipment.optional {
            if owned_item_codes.iter().any(|c| c == &opt.code) {
                optional_owned += 1;
                owned_items.push(OwnedItem {
                    slot: opt.slot.clone(),
                    name: opt.name.clone(),
                    code: opt.code.clone(),
                    source: "warehouse".into(),
                });
            }
        }

        // Check runewords
        let mut missing_runewords: Vec<MissingRuneword> = Vec::new();
        for rw in &build.equipment.runewords {
            let missing_runes: Vec<String> = rw.runes.iter()
                .filter(|r| !owned_rune_codes.contains(r))
                .cloned()
                .collect();
            if missing_runes.is_empty() {
                // All runes owned — check if we already counted the base item
                // (the runeword is craftable)
                owned_items.push(OwnedItem {
                    slot: rw.slot.clone(),
                    name: format!("{} (craftable)", rw.name),
                    code: String::new(),
                    source: "rune_stash".into(),
                });
            } else if missing_runes.len() < rw.runes.len() {
                missing_runewords.push(MissingRuneword {
                    name: rw.name.clone(),
                    runes: rw.runes.clone(),
                    missing_runes,
                });
            }
        }

        let core_total = build.equipment.core.len();
        let optional_total = build.equipment.optional.len();

        // Score: 60% core + 30% optional + 10% runewords
        let core_score = if core_total > 0 { core_owned as f64 / core_total as f64 } else { 0.0 };
        let opt_score = if optional_total > 0 { optional_owned as f64 / optional_total as f64 } else { 0.0 };
        let rw_total = build.equipment.runewords.len();
        let rw_complete = build.equipment.runewords.iter().filter(|rw| {
            rw.runes.iter().all(|r| owned_rune_codes.contains(r))
        }).count();
        let rw_score = if rw_total > 0 { rw_complete as f64 / rw_total as f64 } else { 0.0 };

        let score = core_score * 0.6 + opt_score * 0.3 + rw_score * 0.1;

        BuildMatch {
            build_id: build.id.clone(),
            class: build.class.clone(),
            name: build.name.clone(),
            name_zh: build.name_zh.clone(),
            score,
            core_owned,
            core_total,
            optional_owned,
            optional_total,
            owned_items,
            missing_core,
            missing_runewords,
            description: build.description.clone(),
        }
    }).collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Load all build definitions from JSON.
///
/// 1. Try runtime `data/builds/` directory (dev convenience).
/// 2. Fall back to embedded `data::builds::BUILD_JSONS` (production).
pub fn load_build_definitions() -> Vec<BuildDefinition> {
    let mut builds: Vec<BuildDefinition> = Vec::new();

    // Try loading from Cargo manifest directory (development)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let builds_dir = std::path::Path::new(&manifest_dir).join("src").join("data").join("builds");
        if builds_dir.is_dir()
            && let Ok(entries) = std::fs::read_dir(&builds_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "json")
                        && let Ok(content) = std::fs::read_to_string(&path)
                            && let Ok(build) = serde_json::from_str::<BuildDefinition>(&content) {
                                builds.push(build);
                            }
                }
            }
    }

    // Fall back to embedded JSONs (production / no CARGO_MANIFEST_DIR)
    if builds.is_empty() {
        for &(_id, json) in crate::data::builds::BUILD_JSONS {
            if let Ok(build) = serde_json::from_str::<BuildDefinition>(json) {
                builds.push(build);
            }
        }
    }

    builds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_build_no_items() {
        let build = BuildDefinition {
            id: "test".into(), class: "Test".into(),
            name: "Test".into(), name_zh: "测试".into(),
            core_skills: vec![], stat_priority: vec![],
            equipment: BuildEquipment {
                core: vec![BuildItem { slot: "weapon".into(), code: "sw01".into(), name: "Sword".into(), weight: 1.0 }],
                optional: vec![],
                runewords: vec![],
            },
            description: String::new(),
        };
        let result = match_builds(&[build], &[], &[]);
        assert_eq!(result[0].score, 0.0);
        assert_eq!(result[0].missing_core.len(), 1);
    }
}
