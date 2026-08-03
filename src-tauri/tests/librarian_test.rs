use std::path::PathBuf;

fn librarian_path() -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("librarian.d2s");
    if p.exists() { Some(p) } else {
        eprintln!("SKIP: fixture librarian.d2s 未随仓库分发");
        None
    }
}

#[test]
fn test_librarian_class_level() {
    let Some(_lp) = librarian_path() else { return };
    let data = std::fs::read(_lp).unwrap();
    assert_eq!(data[0x18], 7, "class should be 7 (Warlock)");
    assert_eq!(data[0x1B], 2, "level should be 2");
}

#[test]
fn test_librarian_name() {
    let Some(_lp) = librarian_path() else { return };
    let data = std::fs::read(_lp).unwrap();
    let name_bytes = &data[0x12B..0x13E];
    let name = std::str::from_utf8(name_bytes).unwrap_or("?");
    println!("Name @0x12B: '{}'", name);
    assert!(name.contains("开心"), "name should contain 开心");
}

#[test]
fn test_librarian_header() {
    let Some(_lp) = librarian_path() else { return };
    let data = std::fs::read(_lp).unwrap();
    use d2r_marketplace_lib::protocol::d2s::header::D2SHeader;
    let hdr = D2SHeader::from_bytes(&data).expect("parse header");
    println!("=== 文件头 ===");
    println!("version={} class={} name='{}' status=0x{:x}",
        hdr.version_raw, hdr.class, hdr.name, hdr.status_flags);
    assert_eq!(hdr.version_raw, 105);
    assert_eq!(hdr.class, 7);
    assert!(hdr.name.contains("开心图"), "name should be readable");
}

#[test]
fn test_librarian_parse() {
    let Some(_lp) = librarian_path() else { return };
    let data = std::fs::read(_lp).unwrap();
    let file = d2r_marketplace_lib::protocol::d2s::parser::parse_file(&data)
        .expect("parse_file should succeed");
    println!("=== D2SCharacter 全解析 (5 容器语义化) ===");
    println!("Header: version={} class={} name='{}'",
        file.header.version_raw, file.header.class, file.header.name);
    println!("Skills decoded (first 10): {:?}", &file.skills_decoded[..10.min(file.skills_decoded.len())]);
    println!("Containers: equipped={} belt={} backpack={} cube={} merc={}",
        file.equipped.len(), file.belt.len(), file.backpack.len(), file.cube.len(), file.merc.len());
    println!("Quests progression: {}", file.woo.progression);
    println!("W4 block_type: {}", file.w4.block_type);

    // Print all items across 5 containers
    let all = file.equipped.iter()
        .chain(file.belt.iter())
        .chain(file.backpack.iter())
        .chain(file.cube.iter())
        .chain(file.merc.iter());
    for pi in all {
        let it = &pi.item;
        println!("  code='{}' q={:?} lv={} mode={:?} loc={:?} x={} y={}",
            it.code, it.quality, it.item_level, it.mode, it.location, it.x, it.y);
    }
}

#[test]
fn test_librarian_parse_items() {
    let Some(_lp) = librarian_path() else { return };
    let data = std::fs::read(_lp).unwrap();
    let file = d2r_marketplace_lib::protocol::d2s::parser::parse_file(&data)
        .expect("parse_file");

    // This is a Lv2 Warlock with starting gear
    assert!(
        !file.backpack.is_empty(),
        "backpack should have items, got {}",
        file.backpack.len()
    );
    assert_eq!(file.belt.len(), 4, "belt should have 4 potions");
    let total = file.equipped.len() + file.belt.len() + file.backpack.len() + file.cube.len() + file.merc.len();
    assert!(total >= 10, "should have 10+ items total, got {}", total);

    // Check belt is all health potions
    for pi in &file.belt {
        assert!(pi.item.code.starts_with("hp"), "belt code should be hp*");
    }

    // Check we found the Magic Short Bow (在 equipped 或 backpack 任一容器)
    let has_sbw = file.equipped.iter().chain(file.backpack.iter())
        .any(|pi| pi.item.code == "sbw");
    assert!(has_sbw, "should have short bow");
    // And it's magic quality
    if let Some(sbw) = file.equipped.iter().chain(file.backpack.iter())
        .find(|pi| pi.item.code == "sbw")
    {
        assert!(sbw.item.quality == d2r_marketplace_lib::protocol::common::item_quality::ItemQuality::Magic, "short bow should be magic");
    }
}

#[test]
fn test_librarian_read_items() {
    let Some(_lp) = librarian_path() else { return };
    let data = std::fs::read(_lp).unwrap();
    let items = d2r_marketplace_lib::protocol::d2s::items::read_items(&data)
        .expect("read_items should succeed");
    println!("=== read_items count={} ===", items.len());
    for pi in &items {
        let it = &pi.item;
        println!("  code='{}' q={:?} level={} mode={:?} loc={:?}",
            it.code, it.quality, it.item_level, it.mode, it.location);
    }
    assert!(items.len() >= 10, "should have 10+ items");
}
