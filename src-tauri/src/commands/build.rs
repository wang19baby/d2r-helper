//! Build recommendation Tauri commands.

use crate::services::build_service::{self, BuildMatch};
use serde::Serialize;
use tauri::State;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct BuildRecommendationResponse {
    pub builds: Vec<BuildMatch>,
    pub total_builds: usize,
    pub source: String, // "sample" | "full"
}

/// Get build recommendations based on owned warehouse items and runes.
/// If character_path is provided, also includes currently equipped items.
#[tauri::command]
pub fn get_build_recommendations(
    state: State<AppState>,
    character_path: Option<String>,
    class_filter: Option<String>,
) -> Result<BuildRecommendationResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repos = db.repos();

    // 1. Get all warehouse items
    let wh_items = repos.warehouse.list_all().map_err(|e| e.to_string())?;
    let mut owned_codes: Vec<String> = wh_items.iter()
        .map(|w| w.item_code.clone())
        .collect();

    // 2. Optionally add equipped items from character
    if let Some(ref path) = character_path
        && let Ok(data) = std::fs::read(path)
            && let Ok(file) = crate::protocol::d2s::parser::parse_file(&data) {
                // equipped + belt + backpack 容器下所有 item code 都算"拥有"
                // 用于装备集的源码查表
                let all = file.equipped.iter()
                    .chain(file.belt.iter())
                    .chain(file.backpack.iter());
                for pi in all {
                    if !owned_codes.contains(&pi.item.code) {
                        owned_codes.push(pi.item.code.clone());
                    }
                }
            }

    // 3. Get owned rune codes (from warehouse or rune stash)
    let owned_runes: Vec<String> = wh_items.iter()
        .filter(|w| w.item_kind == "rune")
        .map(|w| w.item_code.clone())
        .collect();
    // Extract rune number from code like "r01" → "el", "r02" → "eld"
    // This requires a mapping. For MVP, we use the ITEM_CODE_MAP lookup
    let rune_names: Vec<String> = owned_runes.iter()
        .filter_map(|code| {
            crate::protocol::d2i::legacy::constants::ITEM_CODE_MAP.iter()
                .find(|(c, _, _, _)| *c == code.as_str())
                .map(|(_, name, _, _)| name.to_lowercase())
        })
        .collect();

    // 4. Load builds
    let all_builds = build_service::load_build_definitions();

    // 5. Filter by class if specified
    let filtered: Vec<_> = if let Some(ref class) = class_filter {
        all_builds.into_iter().filter(|b| b.class == *class).collect()
    } else {
        all_builds
    };

    // 6. Match
    let builds = build_service::match_builds(&filtered, &owned_codes, &rune_names);

    Ok(BuildRecommendationResponse {
        total_builds: filtered.len(),
        source: if character_path.is_some() { "full".into() } else { "warehouse_only".into() },
        builds,
    })
}
