const fs = require('fs');
const vm = require('vm');

function loadWebpack(file) {
  const mods = {}; const cache = {};
  const sb = {
    self: { webpackChunk_N_E: { push: function(chunk) { Object.assign(mods, chunk[1]); } } },
    console, TextDecoder, TextEncoder, Uint8Array, DataView, ArrayBuffer, Buffer, setTimeout, clearTimeout,
  };
  vm.createContext(sb);
  vm.runInContext(fs.readFileSync(file, 'utf8'), sb, { filename: file });
  function req(id) {
    if (cache[id]) return cache[id].exports;
    const m = { exports: {} }; cache[id] = m;
    mods[id](m, m.exports, req);
    return m.exports;
  }
  return req;
}

const c = loadWebpack('D:/work_space/personal_workspace/d2r/d2r-marketplace/tools/d2r_parser/385ca0d5-aecd714f7b16aad0.js')(4258).A;
const all = Object.assign({}, c.armor_items, c.weapon_items, c.other_items);
console.log('lsh in game:', 'lsh' in all);
for (const [code, item] of Object.entries(all)) {
  if (code === 'lsh' || (item.n && item.n.toLowerCase().includes('large shield'))) {
    console.log(code, '→', item.n);
  }
}
