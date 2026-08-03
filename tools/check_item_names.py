import json
with open('D:/personal/games/Diablo II Resurrected/mods/D2RMM/D2RMM.mpq/data/local/lng/strings/item-names.json', 'rb') as f:
    raw = f.read().decode('utf-8-sig', errors='replace')
if '[//' in raw[:10]:
    raw = '[' + raw[raw.index('\n'):]
data = json.loads(raw)

out_path = 'D:/work_space/personal_workspace/d2r/d2r-marketplace-tauri/tools/item_names_lin.txt'
with open(out_path, 'w', encoding='utf-8') as out:
    # Find lin
    for x in data:
        if x.get('Key') == 'lin':
            out.write(f'lin found! zhCN={x.get("zhCN","")}\n')
            break
    # Count short keys (3-4 chars, matching item code patterns)
    short = [x for x in data if x.get('Key') and len(x.get('Key')) in (3,4) and x.get('Key').isascii()]
    out.write(f'\nShort code-like keys: {len(short)}\n')
    for x in short[:30]:
        out.write(f'  Key={x.get("Key")}, zhCN={x.get("zhCN","")[:40]}\n')
print('done')
