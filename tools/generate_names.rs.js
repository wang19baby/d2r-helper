const fs = require('fs');
const names = JSON.parse(fs.readFileSync(
  'D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/tools/game_item_names.json', 'utf8'));
const entries = Object.entries(names).sort((a, b) => a[0].localeCompare(b[0]));

const lines = ['/// Full game item names from D2R data files (armor.txt + weapons.txt + misc.txt)',
  '/// Format: (item_code, display_name)',
  'pub const GAME_ITEM_NAMES: &[(&str, &str)] = &['];
for (const [code, name] of entries) {
  const esc = name.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
  lines.push(`  ("${code}", "${esc}"),`);
}
lines.push('];');
lines.push(`// Total: ${entries.length} items`);
fs.writeFileSync(
  'D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/src-tauri/src/stash/game_item_names.rs',
  lines.join('\n'));
console.log('OK:', entries.length, 'items');
