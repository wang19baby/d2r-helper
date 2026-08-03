use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFileInfo {
    pub role: String,
    pub file_type: String,
    pub relation: String,
    pub path: String,
    pub exists: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManifest {
    pub profile_id: String,
    pub source_kind: String,
    pub game_version: String,
    pub mod_name: String,
    pub game_root: String,
    pub excel_path: String,
    pub strings_path: String,
    pub strings_legacy_path: String,
    #[serde(default)]
    pub checksum: String,
    #[serde(default)]
    pub source_path: String,
    pub vanilla_profile_id: Option<i64>,
    pub active_language: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_languages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub txt_files: Vec<ResourceFileInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub json_files: Vec<ResourceFileInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback_chain: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

fn detect_file(path: PathBuf) -> (String, bool) {
    let exists = path.exists();
    (path.to_string_lossy().to_string(), exists)
}

fn excel_file_path(excel_dir: &Path, filename: &str) -> (String, bool) {
    let base = excel_dir.join("base").join(filename);
    if base.exists() {
        return detect_file(base);
    }
    detect_file(excel_dir.join(filename))
}

fn strings_file_path(dir: &Path, filename: &str) -> (String, bool) {
    detect_file(dir.join(filename))
}

fn find_mod_root(data_path: &str) -> Option<PathBuf> {
    let p = Path::new(data_path);
    for ancestor in p.ancestors() {
        if ancestor.join("local").join("lng").join("strings").is_dir() {
            return ancestor.parent().map(|p| p.to_path_buf());
        }
        if ancestor.join("data").join("local").join("lng").join("strings").is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn read_json_languages(path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    let file = match std::fs::File::open(path) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut raw = String::new();
    if std::io::BufReader::new(file).read_to_string(&mut raw).is_err() {
        return Vec::new();
    }
    if raw.starts_with('\u{FEFF}') {
        raw = raw[3..].to_string();
    }
    let cleaned: Vec<String> = raw
        .lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") {
                String::new()
            } else if t.starts_with("[//") {
                "[".to_string()
            } else {
                l.to_string()
            }
        })
        .filter(|l| !l.is_empty())
        .collect();
    let raw = if cleaned.is_empty() { raw } else { cleaned.join("\n") };
    let entries: Vec<serde_json::Value> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let Some(first) = entries.first().and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let mut langs = BTreeSet::new();
    for key in first.keys() {
        if key == "id" || key == "Key" {
            continue;
        }
        langs.insert(key.to_string());
    }
    langs.into_iter().collect()
}

pub fn build_resource_manifest(
    game_data_path: Option<&str>,
    language: &str,
    mod_name: Option<&str>,
    game_version: Option<&str>,
    game_root: Option<&str>,
) -> Option<ResourceManifest> {
    let mut excel_path = game_data_path?.to_string();
    let game_path = Path::new(&excel_path).to_path_buf();
    // Detect cascview layout where TXT files live under global/excel/
    let global_excel = game_path.join("global").join("excel");
    let has_txt_here = {
        let sample = game_path.join("misc.txt");
        sample.is_file() || game_path.join("base").join("misc.txt").is_file()
    };
    if !has_txt_here && global_excel.is_dir() {
        excel_path = global_excel.to_string_lossy().to_string();
    }
    let excel_dir = Path::new(&excel_path);
    if !excel_dir.is_dir() {
        return None;
    }

    let source_kind = if mod_name.unwrap_or_default().is_empty() || mod_name == Some("(原版)") {
        "vanilla"
    } else {
        "mod"
    };
    let version = game_version.unwrap_or_default().to_string();
    let mut notes = vec![
        "TXT 负责 code/类型/尺寸/唯一套装定义等协议数据；JSON 负责多语言显示文本。".to_string(),
        "名称解析顺序为：TXT 提供 code->英文基础名，再用 JSON 的 Key->语言字段覆盖到目标语言。".to_string(),
        "未来支持多 mod/多原版版本时，应以 profile_id 作为资源缓存和名称缓存的主键。".to_string(),
    ];

    let txt_specs = [
        ("base_items_misc", "txt", "基础物品表：misc code/name/type/stackable", "misc.txt"),
        ("base_items_armor", "txt", "基础物品表：armor code/name/type", "armor.txt"),
        ("base_items_weapons", "txt", "基础物品表：weapons code/name/type", "weapons.txt"),
        ("unique_items", "txt", "唯一物品定义：unique_id -> 英文名/base code", "uniqueitems.txt"),
        ("set_items", "txt", "套装物品定义：set_id -> 英文名/base code", "setitems.txt"),
        ("sets_meta", "txt", "套装元数据与套装奖励", "sets.txt"),
        ("skills", "txt", "技能/灵气/技能页名称关联", "skills.txt"),
        ("skilldesc", "txt", "技能描述与显示文本", "skilldesc.txt"),
        ("item_stat_cost", "txt", "属性位宽与 stat 元数据", "ItemStatCost.txt"),
    ];
    let txt_specs_ext = [
        ("item_affix_magic_prefix", "txt", "魔法前缀词缀表", "magicprefix.txt"),
        ("item_affix_magic_suffix", "txt", "魔法后缀词缀表", "magicsuffix.txt"),
        ("item_affix_rare_prefix", "txt", "稀有前缀词缀表", "rareprefix.txt"),
        ("item_affix_rare_suffix", "txt", "稀有后缀词缀表", "raresuffix.txt"),
        ("item_affix_auto", "txt", "自动词缀表", "automagic.txt"),
        ("item_affix_unique_appellation", "txt", "独特物品称号表", "uniqueappellation.txt"),
        ("item_affix_unique_prefix", "txt", "独特前缀词缀表", "uniqueprefix.txt"),
        ("item_affix_unique_suffix", "txt", "独特后缀词缀表", "uniquesuffix.txt"),
        ("item_quality", "txt", "品质附加词缀表", "qualityitems.txt"),
        ("properties", "txt", "属性定义表", "properties.txt"),
        ("gems", "txt", "宝石镶嵌入表", "gems.txt"),
        ("runes", "txt", "符文支持表", "runes.txt"),
        ("cubemain", "txt", "合成配方表", "cubemain.txt"),
        ("itemtypes", "txt", "物品类型等级表", "itemtypes.txt"),
        ("charstats", "txt", "角色属性表", "charstats.txt"),
    ];

    let mut txt_files = Vec::new();
    for (role, file_type, relation, filename) in txt_specs.into_iter().chain(txt_specs_ext) {
        let (path, exists) = excel_file_path(excel_dir, filename);
        txt_files.push(ResourceFileInfo {
            role: role.to_string(),
            file_type: file_type.to_string(),
            relation: relation.to_string(),
            path,
            exists,
            languages: Vec::new(),
        });
    }

    let mut strings_path = String::new();
    let mut strings_legacy_path = String::new();
    let mut json_files = Vec::new();
    let mut supported_languages = BTreeSet::new();

    if let Some(mod_root) = find_mod_root(&excel_path) {
        let strings_dir = mod_root.join("data").join("local").join("lng").join("strings");
        let legacy_dir = mod_root.join("data").join("local").join("lng").join("strings-legacy");
        strings_path = strings_dir.to_string_lossy().to_string();
        strings_legacy_path = legacy_dir.to_string_lossy().to_string();
        let json_specs = [
            ("item_names", "json", "基础物品名本地化：Key(英文) -> enUS/zhCN/zhTW", "item-names.json"),
            ("item_runes", "json", "符文名本地化", "item-runes.json"),
            ("item_gems", "json", "宝石名本地化", "item-gems.json"),
            ("item_nameaffixes", "json", "前后缀词头本地化", "item-nameaffixes.json"),
            ("item_rarenames", "json", "亮金/稀有名词根", "item-rarenames.json"),
            ("item_modifiers", "json", "部分词条文案/修饰符文本", "item-modifiers.json"),
            ("skills", "json", "技能名称/描述本地化", "skills.json"),
        ];
        for (role, file_type, relation, filename) in json_specs {
            let file_path = strings_dir.join(filename);
            let langs = read_json_languages(&file_path);
            for lang in &langs {
                supported_languages.insert(lang.clone());
            }
            let (path, exists) = strings_file_path(&strings_dir, filename);
            json_files.push(ResourceFileInfo {
                role: role.to_string(),
                file_type: file_type.to_string(),
                relation: relation.to_string(),
                path,
                exists,
                languages: langs,
            });
        }
        for filename in ["item-names.json", "item-runes.json", "item-gems.json"] {
            let file_path = legacy_dir.join(filename);
            let langs = read_json_languages(&file_path);
            for lang in &langs {
                supported_languages.insert(lang.clone());
            }
            let (path, exists) = strings_file_path(&legacy_dir, filename);
            json_files.push(ResourceFileInfo {
                role: format!("legacy_{}", filename.replace(".json", "")),
                file_type: "json".to_string(),
                relation: "旧版字符串补丁源；仅在主 strings 缺项时回补".to_string(),
                path,
                exists,
                languages: langs,
            });
        }
    } else {
        notes.push("当前未定位到 mod_root/data/local/lng/strings，JSON 多语言文件将回退到内置表。".to_string());
    }

    if supported_languages.is_empty() {
        supported_languages.extend(["enUS".to_string(), "zhCN".to_string(), "zhTW".to_string()]);
    }
    let mut fallback_chain = vec![format!("{} -> enUS", language)];
    if language == "zhTW" {
        fallback_chain.push("embedded zhCN -> embedded enUS".to_string());
    } else {
        fallback_chain.push("embedded current-language -> embedded enUS".to_string());
    }

    let profile_id = if source_kind == "vanilla" {
        if version.is_empty() {
            "vanilla:default".to_string()
        } else {
            format!("vanilla:{}", version)
        }
    } else if version.is_empty() {
        format!("mod:{}", mod_name.unwrap_or_default())
    } else {
        format!("mod:{}:{}", mod_name.unwrap_or_default(), version)
    };

    // Compute manifest checksum — hashes key identity + file inventory
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    profile_id.hash(&mut hasher);
    game_root.unwrap_or_default().hash(&mut hasher);
    excel_path.hash(&mut hasher);
    language.hash(&mut hasher);
    for f in &txt_files {
        f.path.hash(&mut hasher);
        f.exists.hash(&mut hasher);
    }
    for f in &json_files {
        f.path.hash(&mut hasher);
        f.exists.hash(&mut hasher);
    }
    let checksum = format!("{:x}", hasher.finish());

    Some(ResourceManifest {
        profile_id,
        source_kind: source_kind.to_string(),
        game_version: version,
        mod_name: mod_name.unwrap_or_default().to_string(),
        game_root: game_root.unwrap_or_default().to_string(),
        excel_path,
        strings_path,
        strings_legacy_path,
        vanilla_profile_id: None,
        checksum,
        source_path: String::new(),
        active_language: language.to_string(),
        supported_languages: supported_languages.into_iter().collect(),
        txt_files,
        json_files,
        fallback_chain,
        notes,
    })
}
