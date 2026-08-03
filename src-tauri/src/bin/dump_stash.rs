use std::fs;
use d2r_marketplace_lib::protocol::d2i::parser::parse_file;

fn main() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).try_init();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: dump_stash <path-to.d2i> [--bits]");
        return;
    }
    let path = &args[1];
    let bits_mode = args.iter().any(|a| a == "--bits");

    let data = fs::read(path).unwrap();

    let file = parse_file(&data).unwrap();

    for page in &file.pages {
        let items: Vec<_> = file.items.iter().filter(|pi| pi.page_index == page.index).collect();
        if items.is_empty() { continue; }
        let label = if page.is_stackable { format!("高级页(idx={})", page.index) } else { format!("页面{}", page.index + 1) };
        println!("\n[{}] {} 件", label, items.len());
        for pi in &items {
            let q = pi.item.quality.as_u8();
            let affix = match (pi.magic_prefix_id, pi.magic_suffix_id) {
                (Some(p), Some(s)) => format!(" pre={} suf={}", p, s),
                (Some(p), None) => format!(" pre={}", p),
                (None, Some(s)) => format!(" suf={}", s),
                _ => String::new(),
            };
            let uid = pi.item.unique_id.map(|id| format!(" uid={}", id)).unwrap_or_default();
            let sid = pi.item.set_id.map(|id| format!(" sid={}", id)).unwrap_or_default();
            let total_stats: usize = pi.item.stat_lists.iter().map(|sl| sl.stats.len()).sum();
            if bits_mode {
                let byte_off = pi.raw_bit_offset / 8;
                let byte_len = pi.raw_bit_length.div_ceil(8);
                println!("  off={:>4}b ({:>3}B) len={:>3}b ({:>2}B)  code={:4} amount={:4} q={} x={} y={} simple={} stats={}{}{}{}",
                    pi.raw_bit_offset, byte_off, pi.raw_bit_length, byte_len,
                    pi.item.code, pi.item.amount, q, pi.item.x, pi.item.y, pi.item.flags.simple_item(),
                    total_stats, affix, uid, sid);
            } else {
                println!("  code={:4} amount={:4} q={} x={} y={} simple={} stats={}{}{}{}",
                    pi.item.code, pi.item.amount, q, pi.item.x, pi.item.y, pi.item.flags.simple_item(),
                    total_stats, affix, uid, sid);
            }
        }
    }
}
