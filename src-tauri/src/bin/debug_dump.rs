//! CLI tool to dump d2s item table matching Python --bits output format.
//! Usage: cargo run --bin debug_dump -- <path-to-d2s>

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: debug_dump <path-to.d2s>");
        std::process::exit(1);
    }
    let path = &args[1];
    match d2r_marketplace_lib::commands::character::debug_item_table(path.clone()) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
