#!/usr/bin/env python3
"""Output d2s parsed items as JSON, using d2r-zero's canonical parser."""
import json, sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'd2r-zero', 'src'))

from d2r_zero.construct_adapter.jm_parser import _scan_item
from d2r_zero.items import _get_stat_save_add, _get_stat_param_bits, _get_stat_bits

path = sys.argv[1]
data = open(path, 'rb').read()
jm_off = data.find(b'JM') + 4
items = []

bit = 0
jm_bits = (len(data) - jm_off) * 8
while bit + 80 <= jm_bits:
    try:
        r = _scan_item(data, jm_off * 8 + bit)
        if r is None: break
        (header, code, uid, ilvl, qb, uniq_id, set_id, rw_id,
         du, max_du, defense, qty, stat_lists, socket_count,
         mpc, msc, soc_3, book_pg, is_compact, realm_data, end_pos) = r
    except Exception:
        bit += 8
        continue

    # Collect stats with display values
    stats = []
    for sl in stat_lists:
        for s in sl.stats:
            display = s.value - _get_stat_save_add(s.id) if hasattr(s, 'value') else s.value
            stats.append({
                "id": s.id, "value": s.value,
                "display": display,
                "param": s.param,
                "save_add": _get_stat_save_add(s.id),
                "param_bits": _get_stat_param_bits(s.id),
            })

    item = {
        "code": code,
        "uid": uid, "ilvl": ilvl, "qb": qb,
        "defense": defense,
        "durability": du, "max_durability": max_du,
        "bit_offset": bit,
        "bit_length": end_pos - jm_off*8 - bit,
        "socket_count": socket_count,
        "stats": stats,
        "mode": header.mode, "equip_loc": header.equip_loc,
        "px": header.px, "py": header.py, "pg": header.pg,
        "unique_id": uniq_id, "set_id": set_id,
        "runeword_id": rw_id,
        "flags": header.flags,
        "is_runeword": bool(header.flags & (1 << 26)),
        "name_zh": "",
    }
    items.append(item)
    bit = end_pos - jm_off * 8

json.dump(items, sys.stdout, ensure_ascii=False, indent=1)
