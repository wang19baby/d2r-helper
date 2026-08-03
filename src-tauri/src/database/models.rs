use serde::{Deserialize, Serialize};

/// A virtual item (listed in the marketplace or purchased)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualItem {
    pub id: String,
    pub name: String,
    pub item_code: Option<String>,
    pub item_kind: Option<String>,
    pub item_type: Option<String>,
    pub quality: Option<String>,
    pub level: Option<i64>,
    pub attributes: Option<String>,
    pub source: Option<String>,
    pub exported_from: Option<String>,
    pub purchased_at: Option<String>,
    pub token_price: Option<i64>,
    pub status: Option<String>,
    pub quantity: Option<i64>,
    pub unit_price: Option<i64>,
    pub listed_at: Option<String>,
    pub sell_after_seconds: Option<i64>,
    pub profile_id: Option<i64>,
    pub profile_key: Option<String>,
    pub game_version: Option<String>,
    pub mod_name: Option<String>,
}

// ── Domain behavior ───────────────────────────────────────────

impl VirtualItem {
    /// The item can be listed for sale.
    pub fn is_listable(&self) -> bool {
        matches!(self.status.as_deref(), None | Some("available"))
    }

    /// The item is currently active on the market.
    pub fn is_listed(&self) -> bool {
        self.status.as_deref() == Some("listed")
    }

    /// The item has been sold (auto or manual).
    pub fn is_sold(&self) -> bool {
        self.status.as_deref() == Some("sold")
    }

    /// The item was purchased and delivered to stash.
    pub fn is_imported(&self) -> bool {
        self.status.as_deref() == Some("imported")
    }

    /// Total listing price (unit_price × quantity).
    pub fn total_price(&self) -> i64 {
        self.unit_price.unwrap_or(0) * self.quantity.unwrap_or(1)
    }

    /// Human-readable status label.
    pub fn status_label(&self) -> &'static str {
        match self.status.as_deref() {
            Some("available") => "可上架",
            Some("listed") => "已上架",
            Some("sold") => "已售出",
            Some("cancelled") => "已取消",
            Some("imported") => "已领取",
            _ => "未知",
        }
    }
}

/// A listed item (for display in the stash/listings page)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListedItem {
    pub id: String,
    pub name: String,
    pub quantity: i32,
    pub unit_price: i32,
    pub listed_at: Option<String>,
    pub sell_after_seconds: i64,
    pub status: Option<String>,
    /// 物品 4-char code,如 "r01" (El Rune)、"gcv" (Chipped Amethyst)
    /// Catalog 用它计算 quality border 与 rune tier 排序
    pub item_code: Option<String>,
    /// 物品 kind,"rune" / "gem" / "potion" / "key" / "essence" / "armor" / "weapon" / "shield"
    pub item_kind: Option<String>,
    /// 物品品质,"unique" / "set" / "rare" / "magic" / "normal"
    pub quality: Option<String>,
    /// 上架人(角色存档文件名,如 "EchoingStrike.d2s")。
    /// Catalog 列表项显示 "ECHOINGSTRIKE 上架"
    pub listed_by: Option<String>,
}

/// An item that was just sold via auto-sell timer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoldItem {
    pub id: String,
    pub name: String,
    pub quantity: i32,
    pub unit_price: i32,
    pub listed_at: String,
    pub sell_after_seconds: i64,
}

/// A single transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Option<i64>,
    pub tx_type: String,
    pub item_id: Option<String>,
    pub token_amount: i64,
    pub description: String,
    pub date: Option<String>,
}

/// App configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub key: String,
    pub value: String,
}

/// A warehoused item — extracted from .d2i into SQLite storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehousedItem {
    pub id: String,
    pub item_code: String,
    pub item_name: String,
    pub item_kind: String,          // "rune", "gem", "potion", "armor", "weapon", "misc", etc.
    pub quality: Option<String>,    // "unique", "set", "rare", "magic", "normal", etc.
    pub simple_item: bool,
    pub quantity: u32,
    pub profile_key: String,        // 资源画像隔离键：vanilla:2.7 / mod:testmod:3.2
    pub game_version: String,       // e.g. "2.4", "2.7", "3.0"
    pub mod_name: String,           // "" = original game, else mod folder name
    pub raw_item_bits: Vec<u8>,     // Raw item bitstream segment
    pub raw_bit_length: usize,      // Exact bit length of the raw segment
    pub item_json: String,          // JSON blob with full parsed data
    pub stash_name: Option<String>, // Original stash file name
    pub imported_at: String,
    pub page_name: String,
    pub tags: String,
    pub notes: String,
    /// Character source tracking — set when extracted from a .d2s file.
    /// None when deposited from the shared stash (.d2i).
    pub source_character: Option<String>, // e.g. "EchoingStrike"
    /// Full path to the source .d2s file (for future write-back).
    #[serde(default)]
    pub source_save_path: Option<String>,
    /// Equipment slot name when this was an equipped item.
    /// e.g. "helm", "weapon_main", "ring_l". None = backpack/belt/stash.
    #[serde(default)]
    pub slot_equipped: Option<String>,
    /// Page index in the stash file (for future multi-page support).
    #[serde(default)]
    pub page_index: i32,
    /// X position in the page grid (0 = leftmost column).
    #[serde(default)]
    pub position_x: i32,
    /// Y position in the page grid (0 = top row).
    #[serde(default)]
    pub position_y: i32,
    /// Item width in grid cells (1 for most items, 2 for larger).
    #[serde(default)]
    pub inv_width: i32,
    /// Item height in grid cells (1 for most items, 2+ for larger).
    #[serde(default)]
    pub inv_height: i32,
}

/// 资源画像：一套原版/模组/版本/语言组合的资源上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProfile {
    pub id: Option<i64>,
    pub profile_key: String,
    pub source_kind: String,
    pub mod_name: String,
    pub game_version: String,
    pub active_language: String,
    pub game_root: String,
    pub excel_path: String,
    pub strings_path: String,
    pub strings_legacy_path: String,
    pub vanilla_profile_id: Option<i64>,
    pub checksum: String,
    pub source_path: String,
    pub import_status: String,
    pub imported_at: Option<String>,
}

/// 资源文件来源：记录当前 profile 依赖了哪些 txt/json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFileRecord {
    pub id: Option<i64>,
    pub profile_id: i64,
    pub role: String,
    pub file_type: String,
    pub relation: String,
    pub path: String,
    pub exists: bool,
    pub languages_json: String,
}

/// 语言字符串行：面向多语言物品名/前后缀/亮金名等查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedStringRecord {
    pub id: Option<i64>,
    pub profile_id: i64,
    pub namespace: String,
    pub string_key: String,
    pub language: String,
    pub text: String,
    pub source_path: String,
}

/// 圣杯追踪记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrailEntry {
    pub profile_id: i64,
    pub item_key: String,
    pub item_type: String,      // "unique" or "set"
    pub item_code: String,
    pub name_en: String,
    pub found: bool,
    pub found_at: Option<String>,
}
