import json, re, os

def load_clean(path):
    with open(path, 'rb') as f:
        raw = f.read().decode('utf-8', errors='replace')
    if raw.startswith('﻿'): raw = raw[1:]
    lines = raw.split('\n')
    cleaned = []
    for line in lines:
        line = re.sub(r'^\s*//.*$', '', line)
        line = re.sub(r'^\[\s*//.*$', '[', line)
        if line.strip() or any(c.strip() for c in cleaned):
            cleaned.append(line)
    return json.loads('\n'.join(cleaned))

base = 'D:/personal/games/Diablo II Resurrected/mods/D2RMM/D2RMM.mpq/data/local/lng/strings-legacy'
out_path = 'D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/tools/legacy_details.txt'

data = load_clean(base + '/item-names.json')

with open(out_path, 'w', encoding='utf-8') as out:
    # Find base items like Cap, Helmet, etc.
    for key in ['Cap', 'Helm', 'Full Helm', 'Leather Armor', 'Small Charm', 'Large Charm', 'Grand Charm', 'El Rune', 'Chipped Amethyst', 'Key of Terror', 'Rejuvenation Potion']:
        matches = [x for x in data if x.get('Key') == key]
        if matches:
            out.write(f'{key}: zhCN={matches[0].get("zhCN","")[:60]}\n')
        else:
            out.write(f'{key}: NOT FOUND\n')

    print('done')
