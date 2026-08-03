/**
 * Extract all item codes from the D2R game constants webpack chunk.
 * This includes armor_items, weapon_items, and other_items.
 * Outputs codes not present in the Rust ALL_ITEMS constant.
 *
 * Usage: node compare_item_codes.cjs
 */
const fs = require("fs");
const vm = require("vm");

function loadWebpackChunk(file) {
  const modules = {}; const cache = {};
  const sandbox = {
    self: { webpackChunk_N_E: { push(chunk) { Object.assign(modules, chunk[1]); } } },
    console, TextDecoder, TextEncoder, Uint8Array, DataView, ArrayBuffer, Buffer, setTimeout, clearTimeout,
  };
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(file, "utf8"), sandbox, { filename: file });
  function req(id) {
    if (cache[id]) return cache[id].exports;
    if (!modules[id]) throw new Error(`Module ${id} not found`);
    const m = { exports: {} }; cache[id] = m;
    modules[id](m, m.exports, req);
    return m.exports;
  }
  req.d = (e, d) => { for (const k in d) { if (!Object.prototype.hasOwnProperty.call(e, k)) Object.defineProperty(e, k, { enumerable: true, get: d[k] }); } };
  req.o = (o, p) => Object.prototype.hasOwnProperty.call(o, p);
  req.r = (e) => { if (typeof Symbol < "u" && Symbol.toStringTag) Object.defineProperty(e, Symbol.toStringTag, { value: "Module" }); Object.defineProperty(e, "__esModule", { value: true }); };
  req.n = (m) => { const g = m && m.__esModule ? () => m.default : () => m; req.d(g, { a: g }); return g; };
  return req;
}

const chunkPath = "D:/work_space/personal_workspace/d2r/d2r-marketplace/tools/d2r_parser/385ca0d5-aecd714f7b16aad0.js";
const constReq = loadWebpackChunk(chunkPath);
const constants = constReq(4258).A;

// All items grouped by category
const armorCodes = Object.keys(constants.armor_items || {});
const weaponCodes = Object.keys(constants.weapon_items || {});
const otherCodes = Object.keys(constants.other_items || {});
const allGameCodes = new Set([...armorCodes, ...weaponCodes, ...otherCodes]);

console.error("Game constants:");
console.error(`  armor_items:  ${armorCodes.length}`);
console.error(`  weapon_items: ${weaponCodes.length}`);
console.error(`  other_items:  ${otherCodes.length}`);
console.error(`  total codes:  ${allGameCodes.size}`);

// Parse the Rust ALL_ITEMS constant to extract codes
const rustPath = "D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/src-tauri/src/stash/game_items.rs";
const rustContent = fs.readFileSync(rustPath, "utf-8");

const rustCodes = new Set();
const codeRegex = /\("([a-z0-9]+)"/g;
let match;
while ((match = codeRegex.exec(rustContent)) !== null) {
  rustCodes.add(match[1]);
}

console.error(`Rust ALL_ITEMS: ${rustCodes.size} codes`);

// Find missing codes
const missing = [...allGameCodes].filter(c => !rustCodes.has(c)).sort();
console.error(`Missing from Rust: ${missing.length}`);

// Categorize missing codes
const missingArmor = missing.filter(c => armorCodes.includes(c));
const missingWeapon = missing.filter(c => weaponCodes.includes(c));
const missingOther = missing.filter(c => otherCodes.includes(c));

console.error(`  missing armor:  ${missingArmor.length}`);
console.error(`  missing weapon: ${missingWeapon.length}`);
console.error(`  missing other:  ${missingOther.length}`);

// Output missing codes in Rust format
if (missing.length > 0) {
  process.stdout.write("// Missing item codes to add to ALL_ITEMS\n");
  process.stdout.write("// Format: (code, name, is_armor, is_weapon, is_shield)\n\n");

  for (const code of missing) {
    let name = "";
    let isArmor = false;
    let isWeapon = false;
    let isShield = false;

    if (armorCodes.includes(code)) {
      const item = constants.armor_items[code];
      name = item.n || code;
      isArmor = true;
      // Check if it's a shield type
      if (item.c && Array.isArray(item.c) && item.c.includes("shie")) {
        isShield = true;
      }
    } else if (weaponCodes.includes(code)) {
      const item = constants.weapon_items[code];
      name = item.n || code;
      isWeapon = true;
    } else if (otherCodes.includes(code)) {
      const item = constants.other_items[code];
      name = item.n || code;
    }

    process.stdout.write(`  ("${code}", "${name}", ${isArmor}, ${isWeapon}, ${isShield}),\n`);
  }

  // Also output the stackables
  const stackablesSet = new Set(Object.keys(constants.stackables || {}));
  const stackedInRust = [...stackablesSet].filter(c => !rustCodes.has(c));
  if (stackedInRust.length > 0) {
    process.stdout.write("\n// Stackable items missing from Rust\n");
    for (const c of stackedInRust) {
      const item = constants.other_items[c] || constants.weapon_items[c] || constants.armor_items[c] || { n: c };
      process.stdout.write(`// stackable: ${c} (${item.n})\n`);
    }
  }
}

// Extra check: codes in Rust but not in game (stale entries)
const extra = [...rustCodes].filter(c => !allGameCodes.has(c));
if (extra.length > 0) {
  process.stdout.write("\n// Codes in Rust but NOT in game constants (possible stale):\n");
  for (const c of extra) {
    process.stdout.write(`// stale: ${c}\n`);
  }
}
