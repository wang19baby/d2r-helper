import json
with open('D:/personal/games/Diablo II Resurrected/mods/D2RMM/D2RMM.mpq/data/local/lng/strings/item-gems.json', 'r', encoding='utf-8-sig', errors='replace') as f:
    # Strip BOM
    raw = f.read()
    if raw.startswith('﻿'): raw = raw[1:]
    data = json.loads(raw)
with open('D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/tools/gem_keys.txt', 'w', encoding='utf-8') as out:
    for x in data[:30]:
        k = x.get('Key', '?')
        zh = x.get('zhCN', '')[:30]
        out.write(f'Key={k}, zhCN={zh}\n')
print('done:', len(data), 'entries')
