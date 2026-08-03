/// Quick standalone test to verify real stash file parsing
/// Run with: cargo run --example read_stash
use std::path::Path;

fn main() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("ModernSharedStashSoftCoreV2.d2i");

    if !fixture.exists() {
        eprintln!("Fixture not found: {:?}", fixture);
        std::process::exit(1);
    }

    // Try Node.js parser first
    let items_result = d2r_marketplace_lib::protocol::d2i::legacy::node_reader::read_stash_with_node(
        &fixture.to_string_lossy()
    );

    let items = match items_result {
        Ok(items) => {
            eprintln!("Using Node.js parser: {} items", items.len());
            items
        }
        Err(e) => {
            eprintln!("Node parser failed: {} — falling back to Rust parser", e);
            let data = std::fs::read(&fixture).expect("Failed to read fixture");
            let pages = d2r_marketplace_lib::protocol::d2i::legacy::page::split_legacy_d2i_pages(&data)
                .expect("Failed to split pages");
            d2r_marketplace_lib::protocol::d2i::legacy::item::read_stash_items(&pages.pages)
                .expect("Failed to read items")
        }
    };

    println!("\nItems ({} total):", items.len());
    let known_codes = [
        "r01","r02","r03","r04","r05","r06","r07","r08","r09","r10",
        "r11","r12","r13","r14","r15","r16","r17","r18","r19","r20",
        "r21","r22","r23","r24","r25","r26","r27","r28","r29","r30",
        "r31","r32","r33",
        "gcv","gcw","gcg","gcr","gcb","gcy","skc",
        "gfv","gfw","gfg","gfr","gfb","gfy","skf",
        "gsv","gsw","gsg","gsr","gsb","gsy","sku",
        "gzv","glw","glg","glr","glb","gly","skl",
        "gpv","gpw","gpg","gpr","gpb","gpy","skz",
        "rvs","rvl","pk1","pk2","pk3",
        "toa","tes","ceh","bet","fed",
        "xa1","xa2","xa3","xa4","xa5",
    ];

    let mut known_count = 0;
    for item in &items {
        let known = if known_codes.contains(&item.item_type.as_str()) {
            known_count += 1;
            " ★"
        } else {
            if !item.simple_item {
                " (non-simple)"
            } else {
                ""
            }
        };
        println!("  {:>4} ×{:<4} simple={} id={}{}",
            item.item_type, item.amount,
            item.simple_item as u8, item.identified as u8, known);
    }

    println!("\nSummary:");
    println!("  Total items: {}", items.len());
    println!("  Simple items: {}", items.iter().filter(|i| i.simple_item).count());
    println!("  Known stackables: {}", known_count);

    if items.iter().all(|i| i.amount > 0) {
        println!("  ✅ All amounts valid");
    }
    if items.iter().any(|i| known_codes.contains(&i.item_type.as_str())) {
        println!("  ✅ Known stackable items found & correctly parsed");
    }
}
