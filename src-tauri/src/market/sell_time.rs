use rand::Rng;

/// Calculate how many seconds until a listing is automatically sold.
/// Based on the ratio of the listing price to the reference price.
/// Ported from app.py calculate_sell_after_seconds()
pub fn calculate_sell_after_seconds(
    unit_price: i64,
    reference_price: i64,
    item_kind: Option<&str>,
) -> i64 {
    let base_seconds = if reference_price <= 0 {
        // No reference: 30-90 minutes
        rand::thread_rng().gen_range(1800..5400)
    } else {
        let ratio = unit_price as f64 / reference_price as f64;

        if ratio <= 0.80 {
            rand::thread_rng().gen_range(60..600)          // 1-10 min
        } else if ratio <= 0.95 {
            rand::thread_rng().gen_range(60..1500)         // 1-25 min
        } else if ratio <= 1.05 {
            rand::thread_rng().gen_range(120..1800)        // 2-30 min
        } else if ratio <= 1.25 {
            rand::thread_rng().gen_range(1800..10800)      // 30min-3h
        } else if ratio <= 1.60 {
            rand::thread_rng().gen_range(10800..28800)     // 3-8h
        } else {
            rand::thread_rng().gen_range(28800..86400)     // 8-24h
        }
    };

    let kind = item_kind.unwrap_or("").trim().to_lowercase();

    // Potions sell faster (45%)
    if kind == "potion" {
        return std::cmp::max(120, (base_seconds as f64 * 0.45) as i64);
    }

    // Gems sell faster (75%)
    if kind == "gem" {
        return std::cmp::max(300, (base_seconds as f64 * 0.75) as i64);
    }

    base_seconds
}

/// Get the default sell timer for items sold directly to the market (not listed)
pub fn get_direct_sell_timer() -> i64 {
    rand::thread_rng().gen_range(300..600) // 5-10 minutes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sell_time_below_reference() {
        let time = calculate_sell_after_seconds(100, 200, None);
        assert!((60..=600).contains(&time), "time={}", time);
    }

    #[test]
    fn test_sell_time_at_reference() {
        let time = calculate_sell_after_seconds(200, 200, None);
        assert!((120..=1800).contains(&time), "time={}", time);
    }

    #[test]
    fn test_sell_time_potion() {
        let time = calculate_sell_after_seconds(100, 200, Some("potion"));
        assert!(time >= 120, "time={}", time);
        assert!(time <= 600, "time={}", time); // 45% of max 600
    }

    #[test]
    fn test_sell_time_no_reference() {
        let time = calculate_sell_after_seconds(500, 0, None);
        assert!((1800..=5400).contains(&time), "time={}", time);
    }
}
