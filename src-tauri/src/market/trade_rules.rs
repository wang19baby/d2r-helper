/// Trade rules: determine what items can be listed/sold
/// Ported from services/trade_rules.py

/// Normalize text for comparison
fn normalize_text(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Recognised purchasable item categories (direct buy from marketplace).
/// Non-purchasable categories can still be listed and sold peer-to-peer.
const PURCHASABLE_CATEGORIES: &[&str] = &["rune", "gem", "potion", "key", "essence", "token", "shard"];

/// Get the item category for trade validation
pub fn get_item_category(item_type: Option<&str>) -> &str {
    let base_type = normalize_text(item_type.unwrap_or(""));

    match base_type.as_str() {
        "rune" => "rune",
        "gem" => "gem",
        "potion" => "potion",
        "key" => "key",
        "essence" => "essence",
        "shard" => "shard",
        _ => "blocked",
    }
}

/// Check if an item is tradeable (can be listed on the marketplace)
pub fn is_tradeable(item_type: Option<&str>) -> bool {
    get_item_category(item_type) != "blocked"
}

/// Check if an item type is purchasable (direct buy via stash write).
pub fn is_purchasable_type(item_type: Option<&str>) -> bool {
    let cat = get_item_category(item_type);
    PURCHASABLE_CATEGORIES.contains(&cat)
}

/// Check if a specific item code is allowed for purchase.
///
/// Accepts all stackable item types that live in the stackable stash page:
/// runes (r01-r33), gems (gcv-gpw/gsk), potions, keys, essences, token, shards.
pub fn is_purchasable(item_code: &str) -> bool {
    // Potions: rvs (Rejuvenation), rvl (Full Rejuvenation) — must check BEFORE rune prefix
    if item_code == "rvs" || item_code == "rvl" {
        return true;
    }
    // Rune codes (r01-r33)
    if item_code.len() == 3 && item_code.starts_with('r') {
        let num: i32 = item_code[1..3].parse().unwrap_or(0);
        return (1..=33).contains(&num);
    }
    // Gem codes: gcv..gpw + skull variants
    if item_code.len() == 3 && item_code.starts_with('g') {
        return true;
    }
    // Keys: pk1 (Terror), pk2 (Hate), pk3 (Destruction)
    if item_code.starts_with("pk") && item_code.len() == 3 {
        return true;
    }
    // Essences: tes (Twisted), ceh (Charged), bet (Burning), fed (Festering)
    if matches!(item_code, "tes" | "ceh" | "bet" | "fed") {
        return true;
    }
    // Token of Absolution
    if item_code == "toa" {
        return true;
    }
    // Shards: xa1-xa5 (Worldstone Shards)
    if item_code.starts_with("xa") && item_code.len() == 3 {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rune_is_tradeable() {
        assert!(is_tradeable(Some("rune")));
    }

    #[test]
    fn test_gem_is_tradeable() {
        assert!(is_tradeable(Some("gem")));
    }

    #[test]
    fn test_potion_is_tradeable() {
        assert!(is_tradeable(Some("potion")));
    }

    #[test]
    fn test_key_is_tradeable() {
        assert!(is_tradeable(Some("key")));
    }

    #[test]
    fn test_essence_is_tradeable() {
        assert!(is_tradeable(Some("essence")));
    }

    #[test]
    fn test_not_tradeable() {
        assert!(!is_tradeable(Some("armor")));
        assert!(!is_tradeable(Some("weapon")));
    }

    #[test]
    fn test_purchasable_rune() {
        assert!(is_purchasable("r01"));
        assert!(is_purchasable("r33"));
    }

    #[test]
    fn test_purchasable_gem() {
        assert!(is_purchasable("gcv"));
        assert!(is_purchasable("gpv"));
        assert!(is_purchasable("gsk"));
    }

    #[test]
    fn test_purchasable_potion() {
        assert!(is_purchasable("rvs"));
        assert!(is_purchasable("rvl"));
    }

    #[test]
    fn test_purchasable_key() {
        assert!(is_purchasable("pk1"));
        assert!(is_purchasable("pk2"));
        assert!(is_purchasable("pk3"));
    }

    #[test]
    fn test_purchasable_essence() {
        assert!(is_purchasable("tes"));
        assert!(is_purchasable("ceh"));
        assert!(is_purchasable("bet"));
        assert!(is_purchasable("fed"));
    }

    #[test]
    fn test_purchasable_token() {
        assert!(is_purchasable("toa"));
    }

    #[test]
    fn test_not_purchasable() {
        assert!(!is_purchasable("hp1"));
        assert!(!is_purchasable("key"));
        assert!(!is_purchasable("tr1"));
    }
}
