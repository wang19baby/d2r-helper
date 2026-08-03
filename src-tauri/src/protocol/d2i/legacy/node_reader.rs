use std::process::Command;

use super::item::StashItem;

/// Parse a stash file using Node.js + the game's webpack chunks.
/// This gives 100% accurate results for ALL item types.
pub fn read_stash_with_node(stash_path: &str) -> Result<Vec<StashItem>, String> {
    let parser_dir = get_parser_dir();
    if parser_dir.is_empty() {
        return Err("Node.js parser directory not found".to_string());
    }

    let reader_file = std::path::Path::new(&parser_dir).join("read_shared_stash.cjs");
    if !reader_file.exists() {
        return Err(format!("Reader script not found: {:?}", reader_file));
    }

    println!("[D2R] Node cmd: node.exe, script: {:?}, dir: {:?}", reader_file, parser_dir);

    if !std::path::Path::new(stash_path).exists() {
        return Err(format!("Stash file not found: {}", stash_path));
    }

    // On Windows, hide the console window
    let mut cmd = Command::new(node_cmd());
    cmd.arg(&reader_file)
        .arg(stash_path)
        .current_dir(&parser_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if cfg!(target_os = "windows") {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().map_err(|e| format!("Failed to run node: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Node.js parser failed.\nstdout: {}\nstderr: {}",
            stdout, stderr
        ));
    }

    // Always print stderr for debugging (Node may have warnings/errors)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("[D2R] Node stderr: {}", stderr);
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let payload: serde_json::Value =
        serde_json::from_str(&stdout_str).map_err(|e| {
            println!("[D2R] JSON parse error: {} — raw: {}", e, &stdout_str[..stdout_str.len().min(200)]);
            format!("JSON parse error: {}", e)
        })?;

    if !payload.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
        return Err(format!(
            "Parser returned error: {}",
            payload.get("error").and_then(|v| v.as_str()).unwrap_or("unknown")
        ));
    }

    let stackables = payload
        .get("stackables")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "No 'stackables' in parser output".to_string())?;

    let items: Vec<StashItem> = stackables
        .iter()
        .filter_map(|val| {
            let code = val
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();

            let amount = val
                .get("amount_in_shared_stash")
                .or_else(|| val.get("amount"))
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as u32;

            if amount == 0 {
                return None;
            }

            let simple = val
                .get("simple_item")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) == 1;

            let identified = val
                .get("identified")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) == 1;

            let socketed = val
                .get("socketed")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) == 1;

            let ethereal = val
                .get("ethereal")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) == 1;

            let quality = val
                .get("quality")
                .and_then(|v| v.as_i64())
                .map(|v| v as u8);

            Some(StashItem {
                item_type: code,
                name: val.get("type_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                amount,
                quality,
                simple_item: simple,
                identified,
                socketed,
                ethereal,
                position_x: 0,
                position_y: 0,
                location_id: 0,
                alt_position_id: 0,
                inv_width: 1,
                inv_height: 1,
                raw_bit_offset: 0,
                raw_bit_length: 0,
                unknown_data: Vec::new(),
            })
        })
        .collect();

    Ok(items)
}

fn get_parser_dir() -> String {
    let cwd = std::env::current_dir().unwrap_or_default();

    // Build candidate paths
    let mut candidates: Vec<std::path::PathBuf> = vec![
        // Absolute fallback
        std::path::PathBuf::from(
            "D:/work_space/personal_workspace/d2r/d2r-marketplace/tools/d2r_parser"
        ),
    ];

    // CWD-relative: try various depths
    if let Some(parent) = cwd.parent() {
        candidates.push(parent.join("d2r-marketplace").join("tools").join("d2r_parser"));
        candidates.push(parent.join("tools").join("d2r_parser"));
    }
    candidates.push(cwd.join("..").join("d2r-marketplace").join("tools").join("d2r_parser"));
    candidates.push(cwd.join("..").join("..").join("d2r-marketplace").join("tools").join("d2r_parser"));

    for path in &candidates {
        if path.join("read_shared_stash.cjs").exists() {
            return path.to_string_lossy().to_string();
        }
    }

    String::new()
}

fn node_cmd() -> String {
    // Check if node is in PATH
    if which::which("node").is_ok() {
        return "node".to_string();
    }
    // Fallback
    "node".to_string()
}
