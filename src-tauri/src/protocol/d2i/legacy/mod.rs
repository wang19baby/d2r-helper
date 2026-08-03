pub mod bit_reader;
pub mod bit_writer;
pub mod complete_header;
pub mod constants;
pub mod game_data_loader;
pub mod game_items;
pub mod huffman;
pub mod item;
pub mod item_names;  // DEPRECATED — use resource::NameResolver instead
pub mod item_sizes;
pub mod magical_props;
pub mod node_reader;
pub mod page;
pub mod resource_manifest;
pub mod runewords;
pub mod unique_items;
pub mod set_items;
pub mod cube_recipes;

pub use bit_reader::BitReader;
pub use bit_writer::BitWriter;
pub use constants::{STACKABLE_ITEM_CODES, ITEM_CODE_MAP, ITEM_NAME_TO_CODE};
pub use huffman::{decode_huffman_string, encode_huffman_string, HUFFMAN_LOOKUP};
pub use page::{D2IPageHeader, split_legacy_d2i_pages, D2IPages};
pub use item::{StashItem, read_stash_items, read_stash_items_from_page, read_all_stash_items, write_stash_items, SUPPORTED_ITEM_CODES};

use serde::{Deserialize, Serialize};

/// Resolved item metadata (name, icon, kind, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMeta {
    pub name: String,
    pub icon: String,
    pub kind: String,
    pub code: String,
}
