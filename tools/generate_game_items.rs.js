const fs = require('fs'), vm = require('vm');
const chunkCode = fs.readFileSync(
  'D:/work_space/personal_workspace/d2r/d2r-marketplace/tools/d2r_parser/385ca0d5-aecd714f7b16aad0.js',
  'utf8'
);
const modules = {};
const sandbox = {
  self: { webpackChunk_N_E: { push(c) { Object.assign(modules, c[1]); } } },
  console, TextDecoder, TextEncoder, Uint8Array, DataView, ArrayBuffer, Buffer,
};
vm.createContext(sandbox);
vm.runInContext(chunkCode, sandbox, { filename: 'x' });
const m = {}; modules[4258](m, m);
const C = m.A;

const all = { ...C.armor_items, ...C.weapon_items, ...C.other_items };
for (const code of Object.keys(C.stackables || {})) {
  if (!all[code]) all[code] = { n: code, c: ['Stackable'] };
}

const items = Object.entries(all).map(([code, item]) => {
  const cats = item.c || [];
  return {
    code,
    name: (item.n || code)
      .replace(/\\/g, '\\\\')
      .replace(/"/g, '\\"'),
    isArmor: cats.includes('Any Armor') || cats.includes('Armor'),
    isWeapon: cats.includes('Weapon'),
    isShield: cats.includes('Shield'),
  };
});

const lines = [
  '/// All item type definitions from game constants',
  '/// Format: (item_code, display_name, is_armor, is_weapon, is_shield)',
  'pub const ALL_ITEMS: &[(&str, &str, bool, bool, bool)] = &[',
];
for (const item of items) {
  lines.push(
    `  ("${item.code}", "${item.name}", ${item.isArmor}, ${item.isWeapon}, ${item.isShield}),`
  );
}
lines.push('];');
lines.push(`// Total: ${items.length} items`);

fs.writeFileSync(
  'D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/src-tauri/src/stash/game_items.rs',
  lines.join('\n')
);
console.log('OK:', items.length, 'items written');
