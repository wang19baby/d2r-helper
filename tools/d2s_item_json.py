#!/usr/bin/env python3
"""Output d2s items as JSON, using Python's canonical parser."""
import json, sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'd2r-zero', 'src'))
from d2r_zero.construct_adapter.jm_parser import _scan_item

path = sys.argv[1]
data = open(path, 'rb').read()
jm_off = data.find(b'JM') + 4
result = []

bit = 0
while bit + 80 <= (len(data) - jm_off) * 8:
    try:
        r = _scan_item(data, jm_off * 8 + bit)
        if r is None: break
        header, code, uid, ilvl, qb, uniq_id, set_id, rw_id, du, max_du, defense, qty, stat_lists, socket_count, mpc, msc, soc_3, book_pg, is_compact, realm_data, end_pos = r
        items = []
        for sl in stat_lists:
            for s in sl.stats:
                items.append({"id": s.id, "value": s.display if hasattr(s, 'display') else s.value, "param": s.param})
        entry = {
            "code": code, "uid": uid, "ilvl": ilvl, "qb": qb,
            "defense": defense, "durability": du, "max_durability": max_du,
            "bit_offset": bit, "bit_length": end_pos - jm_off*8 - bit,
            "socket_count": socket_count, "stats": items,
            "mode": header.mode, "equip_loc": header.equip_loc,
            "px": header.px, "py": header.py, "pg": header.pg,
            "unique_id": uniq_id, "set_id": set_id, "runeword_id": rw_id,
            "flags": header.flags,
        }
        result.append(entry)
        bit = end_pos - jm_off * 8
    except Exception as e:
        bit += 8

json.dump(result, sys.stdout, ensure_ascii=False, indent=1)
