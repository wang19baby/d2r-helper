use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reference prices for items in the market (in tokens)
/// Ported from data/item_prices.json
struct PriceData {
    runes: HashMap<String, i64>,
    gems: HashMap<String, i64>,
    potions: HashMap<String, i64>,
    keys: HashMap<String, i64>,
    essences: HashMap<String, i64>,
    tokens: HashMap<String, i64>,
    shards: HashMap<String, i64>,
    uniques: HashMap<String, i64>,
}

/// Price suggestion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSuggestion {
    pub base_price: i64,
    pub suggested_price: i64,
    pub min_price: i64,
    pub max_price: i64,
    pub variation: i64,
    pub has_reference: bool,
}

fn load_default_prices() -> PriceData {
    // Default price data ported from item_prices.json
    let mut runes = HashMap::new();
    runes.insert("El".into(), 6900); runes.insert("Eld".into(), 6900);
    runes.insert("Tir".into(), 6900); runes.insert("Nef".into(), 6900);
    runes.insert("Eth".into(), 6900); runes.insert("Ith".into(), 6900);
    runes.insert("Tal".into(), 6900); runes.insert("Ral".into(), 6900);
    runes.insert("Ort".into(), 6900); runes.insert("Thul".into(), 6900);
    runes.insert("Amn".into(), 6900); runes.insert("Sol".into(), 6900);
    runes.insert("Shael".into(), 6900); runes.insert("Dol".into(), 6900);
    runes.insert("Hel".into(), 6900);
    runes.insert("Io".into(), 8300); runes.insert("Lum".into(), 6900);
    runes.insert("Ko".into(), 9700); runes.insert("Fal".into(), 9700);
    runes.insert("Lem".into(), 15200);
    runes.insert("Pul".into(), 16600); runes.insert("Um".into(), 13800);
    runes.insert("Mal".into(), 24900); runes.insert("Ist".into(), 24900);
    runes.insert("Gul".into(), 26200);
    runes.insert("Vex".into(), 41400); runes.insert("Ohm".into(), 44200);
    runes.insert("Lo".into(), 51100); runes.insert("Sur".into(), 77300);
    runes.insert("Ber".into(), 105000); runes.insert("Jah".into(), 89800);
    runes.insert("Cham".into(), 49700); runes.insert("Zod".into(), 93900);

    let mut gems = HashMap::new();
    gems.insert("Chipped Amethyst".into(), 3000); gems.insert("Chipped Diamond".into(), 3000);
    gems.insert("Chipped Emerald".into(), 3000); gems.insert("Chipped Ruby".into(), 3000);
    gems.insert("Chipped Sapphire".into(), 3000); gems.insert("Chipped Topaz".into(), 3000);
    gems.insert("Chipped Skull".into(), 3500);
    gems.insert("Flawed Amethyst".into(), 6000); gems.insert("Flawed Diamond".into(), 6000);
    gems.insert("Flawed Emerald".into(), 6000); gems.insert("Flawed Ruby".into(), 6000);
    gems.insert("Flawed Sapphire".into(), 6000); gems.insert("Flawed Topaz".into(), 6000);
    gems.insert("Flawed Skull".into(), 7000);
    gems.insert("Amethyst".into(), 9000); gems.insert("Diamond".into(), 9000);
    gems.insert("Emerald".into(), 9000); gems.insert("Ruby".into(), 9000);
    gems.insert("Sapphire".into(), 9000); gems.insert("Topaz".into(), 9000);
    gems.insert("Skull".into(), 11000);
    gems.insert("Flawless Amethyst".into(), 15000); gems.insert("Flawless Diamond".into(), 15000);
    gems.insert("Flawless Emerald".into(), 15000); gems.insert("Flawless Ruby".into(), 15000);
    gems.insert("Flawless Sapphire".into(), 15000); gems.insert("Flawless Topaz".into(), 15000);
    gems.insert("Flawless Skull".into(), 18000);
    gems.insert("Perfect Amethyst".into(), 24900); gems.insert("Perfect Diamond".into(), 24900);
    gems.insert("Perfect Emerald".into(), 24900); gems.insert("Perfect Ruby".into(), 24900);
    gems.insert("Perfect Sapphire".into(), 24900); gems.insert("Perfect Topaz".into(), 24900);
    gems.insert("Perfect Skull".into(), 28000);

    let mut potions = HashMap::new();
    potions.insert("Rejuvenation Potion".into(), 250);
    potions.insert("Full Rejuvenation Potion".into(), 600);

    let mut keys = HashMap::new();
    keys.insert("Key of Terror".into(), 37300);
    keys.insert("Key of Hate".into(), 37300);
    keys.insert("Key of Destruction".into(), 37300);

    let mut essences = HashMap::new();
    essences.insert("Twisted Essence of Suffering".into(), 24900);
    essences.insert("Charged Essence of Hatred".into(), 24900);
    essences.insert("Burning Essence of Terror".into(), 24900);
    essences.insert("Festering Essence of Destruction".into(), 24900);

    let mut tokens = HashMap::new();
    tokens.insert("Token of Absolution".into(), 24900);

    let mut shards = HashMap::new();
    shards.insert("Western Worldstone Shard".into(), 74600);
    shards.insert("Eastern Worldstone Shard".into(), 74600);
    shards.insert("Southern Worldstone Shard".into(), 74600);
    shards.insert("Deep Worldstone Shard".into(), 74600);
    shards.insert("Northern Worldstone Shard".into(), 74600);

    let mut uniques = HashMap::new();
    uniques.insert("Harlequin Crest".into(), 118800);
    uniques.insert("The Grandfather".into(), 84000);
    uniques.insert("Windforce".into(), 118800);

    PriceData { runes, gems, potions, keys, essences, tokens, shards, uniques }
}

/// Get the reference market price for an item
pub fn get_market_reference_price(item_name: &str, item_kind: Option<&str>) -> i64 {
    let prices = load_default_prices();
    let normalized_name = item_name.trim();
    let kind = item_kind.unwrap_or("").trim().to_lowercase();

    // Rune lookup: strip " Rune" suffix
    if kind == "rune" || kind == "runa" {
        let rune_key = normalized_name
            .strip_suffix(" Rune")
            .unwrap_or(normalized_name);
        return *prices.runes.get(rune_key).unwrap_or(&0);
    }

    match kind.as_str() {
        "gem" => *prices.gems.get(normalized_name).unwrap_or(&0),
        "potion" => *prices.potions.get(normalized_name).unwrap_or(&0),
        "key" => *prices.keys.get(normalized_name).unwrap_or(&0),
        "essence" => *prices.essences.get(normalized_name).unwrap_or(&0),
        "token" => *prices.tokens.get(normalized_name).unwrap_or(&0),
        "shard" => *prices.shards.get(normalized_name).unwrap_or(&0),
        "unique" => *prices.uniques.get(normalized_name).unwrap_or(&0),
        _ => 0,
    }
}

/// Get a sell price suggestion with variation range
pub fn get_sell_price_suggestion(item_name: &str, item_kind: Option<&str>) -> PriceSuggestion {
    let base_price = get_market_reference_price(item_name, item_kind);

    if base_price <= 0 {
        return PriceSuggestion {
            base_price: 0,
            suggested_price: 0,
            min_price: 0,
            max_price: 0,
            variation: 0,
            has_reference: false,
        };
    }

    // 4% variation (same as original Python code)
    let variation = std::cmp::max(1, (base_price as f64 * 0.04).round() as i64);

    PriceSuggestion {
        base_price,
        suggested_price: base_price,
        min_price: base_price,
        max_price: base_price + variation,
        variation,
        has_reference: true,
    }
}

/// Normalize item type aliases
pub fn normalize_item_type(item_type: &str) -> String {
    let value = item_type.trim().to_lowercase();
    match value.as_str() {
        "rune" | "runas" | "runa" => "rune".to_string(),
        _ => value,
    }
}

/// Check if an item name looks like a rune
pub fn looks_like_rune(item_name: &str, item_type: &str) -> bool {
    let normalized_type = normalize_item_type(item_type);
    if normalized_type == "rune" {
        return true;
    }
    item_name.trim().to_lowercase().ends_with(" rune")
}

/// Calculate the sell price after marketplace fee (70% of token price)
pub fn calculate_sell_price(token_price: i64) -> i64 {
    (token_price as f64 * 0.7) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── calculate_sell_price ──

    #[test]
    fn test_sell_price_zero() {
        assert_eq!(calculate_sell_price(0), 0);
    }

    #[test]
    fn test_sell_price_rounds_down() {
        // 70% of 1 = 0.7 → 0
        assert_eq!(calculate_sell_price(1), 0);
    }

    #[test]
    fn test_sell_price_100() {
        assert_eq!(calculate_sell_price(100), 70);
    }

    #[test]
    fn test_sell_price_10000() {
        assert_eq!(calculate_sell_price(10000), 7000);
    }

    #[test]
    fn test_sell_price_large() {
        assert_eq!(calculate_sell_price(105000), 73500);
    }

    // ── normalize_item_type ──

    #[test]
    fn test_normalize_rune_aliases() {
        assert_eq!(normalize_item_type("rune"), "rune");
        assert_eq!(normalize_item_type("Rune"), "rune");
        assert_eq!(normalize_item_type("runas"), "rune");
        assert_eq!(normalize_item_type("runa"), "rune");
    }

    #[test]
    fn test_normalize_passthrough() {
        assert_eq!(normalize_item_type("gem"), "gem");
        assert_eq!(normalize_item_type("armor"), "armor");
        assert_eq!(normalize_item_type(""), "");
    }

    #[test]
    fn test_normalize_trims_whitespace() {
        assert_eq!(normalize_item_type("  rune  "), "rune");
    }

    // ── looks_like_rune ──

    #[test]
    fn test_looks_like_rune_by_type() {
        assert!(looks_like_rune("El", "rune"));
        assert!(looks_like_rune("Zod", "runas"));
    }

    #[test]
    fn test_looks_like_rune_by_name_suffix() {
        assert!(looks_like_rune("El Rune", "gem"));
        assert!(looks_like_rune("Zod Rune", ""));
    }

    #[test]
    fn test_not_rune() {
        assert!(!looks_like_rune("Perfect Skull", "gem"));
        assert!(!looks_like_rune("El", "armor"));
    }

    #[test]
    fn test_rune_case_insensitive() {
        assert!(looks_like_rune("el rune", "gem"));
        assert!(looks_like_rune("EL RUNE", ""));
    }

    // ── get_market_reference_price ──

    #[test]
    fn test_rune_price() {
        let price = get_market_reference_price("El Rune", Some("rune"));
        assert!(price > 0, "El Rune should have a price");
        assert_eq!(price, 6900);
    }

    #[test]
    fn test_gem_price() {
        let price = get_market_reference_price("Perfect Skull", Some("gem"));
        assert_eq!(price, 28000);
    }

    #[test]
    fn test_potion_price() {
        let price = get_market_reference_price("Full Rejuvenation Potion", Some("potion"));
        assert_eq!(price, 600);
    }

    #[test]
    fn test_key_price() {
        let price = get_market_reference_price("Key of Terror", Some("key"));
        assert_eq!(price, 37300);
    }

    #[test]
    fn test_essence_price() {
        let price = get_market_reference_price("Twisted Essence of Suffering", Some("essence"));
        assert_eq!(price, 24900);
    }

    #[test]
    fn test_unique_price() {
        let price = get_market_reference_price("Harlequin Crest", Some("unique"));
        assert_eq!(price, 118800);
    }

    #[test]
    fn test_unknown_item_no_kind() {
        assert_eq!(get_market_reference_price("Unknown Item", None), 0);
    }

    #[test]
    fn test_unknown_item_known_kind() {
        assert_eq!(get_market_reference_price("Fake Item", Some("gem")), 0);
    }

    #[test]
    fn test_price_handles_leading_trailing_spaces() {
        let price = get_market_reference_price("  El Rune  ", Some("rune"));
        assert_eq!(price, 6900);
    }

    // ── get_sell_price_suggestion ──

    #[test]
    fn test_price_suggestion_known_item() {
        let s = get_sell_price_suggestion("El Rune", Some("rune"));
        assert!(s.has_reference);
        assert_eq!(s.base_price, 6900);
        assert!(s.variation > 0);
    }

    #[test]
    fn test_price_suggestion_unknown() {
        let s = get_sell_price_suggestion("Nothing", None);
        assert!(!s.has_reference);
        assert_eq!(s.base_price, 0);
        assert_eq!(s.suggested_price, 0);
    }

    #[test]
    fn test_price_suggestion_variation() {
        let s = get_sell_price_suggestion("Zod", Some("rune"));
        assert_eq!(s.base_price, 93900);
        // 4% of 93900 = 3756
        assert_eq!(s.variation, 3756);
        assert_eq!(s.max_price, s.base_price + s.variation);
    }

    #[test]
    fn test_rune_no_suffix() {
        // Without " Rune" suffix, with kind="rune"
        let price = get_market_reference_price("El", Some("rune"));
        assert_eq!(price, 6900);
    }

    #[test]
    fn test_ber_rune_price() {
        let price = get_market_reference_price("Ber Rune", Some("rune"));
        assert_eq!(price, 105000);
    }
}
