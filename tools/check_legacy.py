import json, re, os

def load_clean(path):
    with open(path, 'rb') as f:
        raw = f.read().decode('utf-8', errors='replace')
    # Strip BOM
    if raw.startswith('﻿'): raw = raw[1:]
    # Strip [// comments
    lines = raw.split('\n')
    cleaned = []
    for line in lines:
        # Remove // comments at line start (after optional whitespace)
        line = re.sub(r'^\s*//.*$', '', line)
        # Handle [//comments
        line = re.sub(r'^\[\s*//.*$', '[', line)
        # Remove empty lines at the start
        if line.strip() or any(c.strip() for c in cleaned):
            cleaned.append(line)
    return json.loads('\n'.join(cleaned))

base = 'D:/personal/games/Diablo II Resurrected/mods/D2RMM/D2RMM.mpq/data/local/lng/strings-legacy'
out_path = 'D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/tools/legacy_report.txt'

data = load_clean(base + '/item-runes.json')
data2 = load_clean(base + '/item-names.json')

with open(out_path, 'w', encoding='utf-8') as out:
    out.write('item-runes.json: ' + str(len(data)) + ' entries\n')
    runes = [x for x in data if 'Rune' in x.get('Key','')]
    out.write('Rune entries: ' + str(len(runes)) + '\n')
    for x in runes[:10]:
        out.write('  Key=' + str(x.get('Key')) + ' zhCN=' + str(x.get('zhCN',''))[:30] + '\n')

    out.write('\nitem-names.json: ' + str(len(data2)) + ' entries\n')
    cap = [x for x in data2 if x.get('Key') == 'Cap']
    if cap:
        out.write('  Cap: zhCN=' + str(cap[0].get('zhCN','')) + '\n')
    with_zh = sum(1 for x in data2 if x.get('zhCN'))
    out.write('  With zhCN: ' + str(with_zh) + '\n')

print('done: runes=' + str(len(runes)) + ', items=' + str(len(data2)))
