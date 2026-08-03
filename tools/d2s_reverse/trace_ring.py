#!/usr/bin/env python3
"""
Bit-level trace for 开心图书馆长.d2s
Focus on ring and tome items.
Huffman decoder fixed to LSB-first (matching Rust).
"""
import struct

PATH = r"D:\work_space\personal_workspace\d2r\开心图书馆长.d2s"
data = open(PATH, 'rb').read()

# ── LSB-first BitReader (EXACTLY matches Rust bitio::BitReader) ──
class BR:
    def __init__(self, buf, bit=0):
        self.buf = buf
        self.pos = bit
    
    def r(self, n):
        v = 0
        for i in range(n):
            if self.pos < len(self.buf) * 8:
                v |= ((self.buf[self.pos // 8] >> (self.pos % 8)) & 1) << i
            self.pos += 1
        return v
    
    def tell(self): return self.pos
    def skip(self, n): self.pos += n
    def align(self): self.pos = (self.pos + 7) & ~7
    
    def peek(self, n):
        v = 0
        for i in range(n):
            p = self.pos + i
            if p < len(self.buf) * 8:
                v |= ((self.buf[p // 8] >> (p % 8)) & 1) << i
        return v

# ── Huffman: LSB-first tree construction ──
# Same data as Rust HUFFMAN_LOOKUP: (char, value, bits)
HUFFMAN = [
    ('0',223,8),('1',31,7),('2',12,6),('3',91,7),('4',95,8),('5',104,8),
    ('6',123,7),('7',30,5),('8',8,6),('9',14,5),(' ',1,2),('a',15,5),
    ('b',10,4),('c',2,5),('d',35,6),('e',3,6),('f',50,6),('g',11,5),
    ('h',24,5),('i',63,7),('j',232,9),('k',18,6),('l',23,5),('m',22,5),
    ('n',44,6),('o',127,7),('p',19,5),('q',155,8),('r',7,5),('s',4,4),
    ('t',6,5),('u',16,5),('v',59,7),('w',0,5),('x',28,5),('y',40,7),('z',27,8),
]

# Build tree: nodes[idx] = [child0, child1] or None if leaf
# Rust's tree: vector of DecodeNode { children: [u16; 2], ch: Option<char> }
# LSB-first: for b in 0..bits: direction = (value >> b) & 1
nodes = [{'children': [0, 0], 'ch': None}]  # root at index 0

for ch, value, bits in HUFFMAN:
    idx = 0
    for b in range(bits):
        direction = (value >> b) & 1
        if b == bits - 1:
            # Last bit → leaf
            child_idx = nodes[idx]['children'][direction]
            if child_idx == 0:
                child_idx = len(nodes)
                nodes.append({'children': [0, 0], 'ch': ch})
                nodes[idx]['children'][direction] = child_idx
            else:
                nodes[child_idx]['ch'] = ch
            idx = child_idx
        else:
            child_idx = nodes[idx]['children'][direction]
            if child_idx == 0:
                child_idx = len(nodes)
                nodes.append({'children': [0, 0], 'ch': None})
                nodes[idx]['children'][direction] = child_idx
            idx = child_idx

def decode_huff(r):
    """Read 4-char item code via Huffman tree (LSB-first)"""
    result = []
    for _ in range(4):
        idx = 0
        while True:
            node = nodes[idx]
            if node['ch'] is not None:
                result.append(node['ch'])
                break
            bit = r.r(1)
            idx = node['children'][bit]
    return ''.join(result).strip()

# ── ARMOR/WEAPON/SHIELD sets ──
ARMOR_SET = {'cap','xap','buc','uap','xpl','lrg'}
WEAPON_SET = {'dgr','sbw','hax','scy','2ax','crs','7s8','wae','xpk','jkn','sst','bwn'}
SHIELD_SET = {'buc','xbl','lrg','gts'}

QUAL_NAMES = {0:'None',1:'Low',2:'Norm',3:'Sup',4:'Mag',5:'Set',6:'Rare',7:'Uniq',8:'Craft'}

# ══════════════════════════════════════════════════════════
# SCAN ALL JM ITEMS
# ══════════════════════════════════════════════════════════
jm_off = 0x394
jm_data = data[jm_off:0x494]
SJM = len(jm_data) * 8

count = struct.unpack_from('<H', data, jm_off + 2)[0]
print(f"JM @ 0x{jm_off:04X}, count={count}, bits={SJM}")

r = BR(jm_data, 32)  # skip JM header
items = []

for idx in range(count):
    if r.tell() + 32 > SJM:
        print(f"\nItem[{idx}]: STOP — not enough bits ({SJM - r.tell()} remaining)")
        break
    
    start = r.tell()
    flags = r.r(32)
    ver = r.r(3)
    loc = r.r(3)
    slot = r.r(4)
    x = r.r(4)
    y = r.r(4)
    pg = r.r(3)
    
    # Save position before code read
    huff_start = r.tell()
    code = decode_huff(r)
    huff_bits = r.tell() - huff_start
    
    simple = (flags >> 21) & 1
    identified = (flags >> 20) & 1
    runeword = (flags >> 26) & 1
    personalized = (flags >> 24) & 1
    ethereal = (flags >> 22) & 1
    socketed_flag = (flags >> 11) & 1
    
    if simple:
        ns = r.r(1)
        # simple items only
        r.align()
        end = r.tell()
        items.append({
            'start': start, 'end': end, 'code': code, 'simple': True,
            'loc': loc, 'slot': slot, 'x': x, 'y': y,
            'quality': 'Simple', 'ilvl': 0
        })
        continue
    
    ns = r.r(3)
    uid = r.r(32)
    ilvl = r.r(7)
    qb = r.r(4)
    
    # quality-specific fields
    hg = r.r(1)
    if hg: r.skip(3)
    hc = r.r(1)
    
    if qb in (1, 3):
        r.skip(3)
    elif qb == 4:
        pf = r.r(11)
        sf = r.r(11)
    elif qb == 5:
        set_id = r.r(12)
    elif qb in (6, 8):
        r.skip(16)  # 2x8 rare names
        for _ in range(6):
            if r.r(1): r.skip(11)
    elif qb == 7:
        unique_id = r.r(12)
    
    # runeword
    if runeword: r.skip(16)
    # personalized
    if personalized:
        for _ in range(16):
            c = r.r(8)
            if c == 0: break
    
    is_armor = code in ARMOR_SET
    is_weapon = code in WEAPON_SET
    is_shield = code in SHIELD_SET
    
    # defense + durability
    if is_armor or is_shield:
        r.skip(11)   # defense
        md = r.r(8)
        r.skip(10)   # cur_dur
    elif is_weapon:
        md = r.r(8)
        if md > 0: r.skip(10)
        else: r.skip(1)
    else:
        md = 0
    
    r.skip(1)  # v105 padding
    
    if socketed_flag: r.skip(4)
    if qb == 5: r.skip(5)
    
    # Stat list: read until 0x1FF with correct bit widths
    stat_start = r.tell()
    stats = []
    while True:
        if r.tell() + 9 > SJM:
            break
        sid = r.r(9)
        if sid == 0x1FF:
            break
        # Need actual stat table to know correct widths
        # For now: record stat ID only
        stats.append({'id': sid, 'bit_pos': r.tell() - 9})
        # We need the actual stat table widths to skip properly
        # Without it: scan forward for next 0x1FF candidate
        r.skip(10)  # placeholder skip
    
    r.align()
    end = r.tell()
    
    items.append({
        'start': start, 'end': end,
        'code': code, 'simple': False,
        'loc': loc, 'slot': slot,
        'x': x, 'y': y,
        'quality': QUAL_NAMES.get(qb, str(qb)),
        'ilvl': ilvl, 'stats': len(stats),
        'stat_ids': [s['id'] for s in stats],
    })

# ── Print results ──
print(f"\n{'#':>3s} {'Start':>7s} {'End':>7s} {'Bits':>5s} {'Code':>5s} {'L':>2s} {'Sl':>3s} {'(x,y)':>8s} {'Q':>6s} {'St':>3s}")
print('-' * 66)
for it in items:
    qdisp = it['quality']
    print(f"{items.index(it):3d} {it['start']:7d} {it['end']:7d} {it['end']-it['start']:5d} "
          f"{it['code']:>5s} {it['loc']:2d} {it['slot']:3d} "
          f"({it['x']:2d},{it['y']:2d}) {qdisp:>6s} "
          f"{it.get('stats',0):3d}")

# ── RING DEEP TRACE ──
ring = next((it for it in items if it['code'] == 'rin' and it['slot'] in (6,7)), None)
ring = ring or next((it for it in items if it['code'] == 'rin'), None)

if ring:
    print(f"\n{'='*70}")
    print(f"RING trace: item[{items.index(ring)}] @ bit {ring['start']}")
    print(f"{'='*70}")
    
    r = BR(jm_data, ring['start'])
    def tr(name, n): 
        v = r.r(n)
        hex_info = f"0x{r.tell()-n:04X}: " + ' '.join(f'{jm_data[(r.tell()-n)//8+i]:02x}' for i in range((n+7)//8 + 2) if (r.tell()-n)//8+i < len(jm_data))
        print(f"  [{r.tell()-n:5d}b] {name:28s} = {v:>10d} ({v:#06x})")
        return v
    
    f = tr('flags(32b)', 32)
    tr('version(3b)', 3)
    tr('location(3b)', 3)
    sv = tr('slot(4b)', 4)
    tr('x(4b)', 4)
    tr('y(4b)', 4)
    tr('page(3b)', 3)
    
    hs = r.tell()
    code = decode_huff(r)
    print(f"  [{hs:5d}b] huff_code                  = \"{code}\" ({r.tell()-hs} bits)")
    
    tr('num_sockets(3b)', 3)
    tr('uid(32b)', 32)
    tr('ilvl(7b)', 7)
    qb = tr('quality(4b)', 4)
    
    tr('has_gfx(1b)', 1)
    hc = tr('has_class(1b)', 1)
    
    if qb == 4:
        pf = tr('magic_prefix(11b)', 11)
        sf = tr('magic_suffix(11b)', 11)
        print(f"  >>> Magic prefix={pf}, suffix={sf}")
    
    # Check flags for runeword/personalized
    rw_bit = (f >> 26) & 1
    pn_bit = (f >> 24) & 1
    sk_bit = (f >> 11) & 1
    
    if rw_bit: tr('runeword(16b)', 16)
    if pn_bit:
        ps = r.tell()
        for _ in range(16):
            c = r.r(8)
            if c == 0: break
        print(f"  [{ps:5d}b] personalized              = {r.tell()-ps} bits")
    
    is_a = code in ARMOR_SET
    is_w = code in WEAPON_SET
    is_s = code in SHIELD_SET
    
    if is_a or is_s:
        tr('defense(11b)', 11)
        md = tr('max_dur(8b)', 8)
        tr('cur_dur(10b)', 10)
    elif is_w:
        md = tr('max_dur(8b)', 8)
        if md > 0: tr('cur_dur(10b)', 10)
        else: tr('zero_dur(1b)', 1)
    else:
        print(f"  [{r.tell():5d}b] (no durability — not armor/weapon/shield)")
    
    tr('v105_padding(1b)', 1)
    
    if sk_bit: tr('socketed(4b)', 4)
    
    # STAT LIST with actual bit widths!
    print(f"\n  ── STAT LIST @ bit {r.tell()} ──")
    
    # Check: is the first stat ID actually 0x1FF?
    peek_val = r.peek(9)
    print(f"  peek first 9 bits = 0x{peek_val:03x} ({peek_val})")
    
    # Full stat list now using scan_forward approach
    # Read each stat with correct widths from a mini stat table
    known_stats = {
        0: ('strength', 10, 0), 1: ('energy', 10, 0), 2: ('dexterity', 10, 0), 
        3: ('vitality', 10, 0), 4: ('statpts', 10, 0), 5: ('newskills', 8, 0),
        6: ('hitpoints', 22, 0), 7: ('maxhp', 22, 0), 8: ('mana', 22, 0),
        9: ('maxmana', 22, 0), 10: ('stamina', 22, 0), 11: ('maxstamina', 22, 0),
        12: ('level', 10, 0), 13: ('experience', 32, 0), 14: ('gold', 20, 0),
        15: ('goldbank', 20, 0), 16: ('item_armor_percent', 8, 0),
        17: ('item_mindamage', 5, 0), 18: ('item_maxdamage', 5, 0),
        19: ('item_secondary_mindamage', 5, 0), 20: ('item_secondary_maxdamage', 5, 0),
        21: ('item_tohit', 9, 0), 22: ('item_attack_speed', 4, 5),
        23: ('item_skill_attack_speed', 4, 0),
        36: ('item_fastermovevelocity', 6, 0),
        37: ('strength', 10, 0), 38: ('energy', 10, 0),
        39: ('dexterity', 10, 0), 40: ('vitality', 10, 0),
        41: ('item_absorbfire', 4, 0), 42: ('item_absorbfire_percent', 4, 0),
        43: ('item_absorblight', 4, 0), 44: ('item_absorblight_percent', 4, 0),
        45: ('item_absorbmagic', 4, 0), 46: ('item_absorbmagic_percent', 4, 0),
        47: ('item_absorbcold', 4, 0), 48: ('item_absorbcold_percent', 4, 0),
        49: ('item_poisonlengthresist', 4, 0),
        50: ('item_damagepercent', 7, 0), 51: ('item_magicalbonus', 4, 0),
        52: ('item_fire_resist', 4, 0), 53: ('item_light_resist', 4, 0),
        54: ('item_cold_resist', 4, 0), 55: ('item_poison_resist', 4, 0),
        56: ('item_all_resist', 4, 0),
        57: ('item_fasterattackrate', 5, 0),
        59: ('item_fastergethitrate', 5, 0),
        60: ('item_fasterblockrate', 5, 0),
        63: ('item_fastercastrate', 5, 0),
        72: ('item_singleskill', 10, 0),  # param = skill_id
        73: ('item_nonclassskill', 8, 0),
        79: ('item_findgold', 6, 0),
        80: ('item_magicsingle_resist', 12, 0),
        81: ('item_magicmulti_resist', 12, 0),
        83: ('item_rapidexhaust', 4, 0),
        84: ('item_hpregen', 1, 0), 85: ('item_manaregen', 5, 0),
        86: ('item_armor', 7, 0),
        87: ('item_cannotbefrozen', 1, 0),
        88: ('item_staminapercent', 4, 0),
        89: ('item_lifeleech', 4, 0), 90: ('item_manaleech', 4, 0),
        91: ('item_manaperhit', 4, 0), 92: ('item_healafterkill', 4, 0),
        93: ('item_pierce', 5, 0),
        94: ('item_reducedprices', 4, 0),
        95: ('item_nofriction', 4, 0),
        96: ('item_magicfind', 7, 0),
        99: ('item_normaldamagereduction', 4, 0),
        100: ('item_firedamage', 7, 0), 101: ('item_lightdamage', 7, 0),
        102: ('item_magicdamage', 7, 0), 103: ('item_colddamage', 7, 0),
        104: ('item_coldlength', 7, 0),
        105: ('item_extracharges', 10, 0),  # param=skill_id, value=charges
        106: ('item_fastermove', 6, 0),
        107: ('item_poison_damage', 6, 0),
        108: ('item_damage_vs_monster', 8, 0),
        109: ('item_damage_vs_demon', 8, 0),
        110: ('item_damage_vs_undead', 8, 0),
        111: ('item_thorns', 6, 0),
        112: ('item_fire_skill_damage', 5, 0),
        113: ('item_light_skill_damage', 5, 0),
        114: ('item_magic_skill_damage', 5, 0),
        115: ('item_cold_skill_damage', 5, 0),
        116: ('item_poison_skill_damage', 5, 0),
        117: ('item_criticalstrike', 4, 0),
        118: ('item_openwounds', 4, 0),
        119: ('item_kickdamage', 4, 0),
        120: ('item_manaafterkill', 4, 0),
        121: ('item_armor_by_time', 10, 0),
        122: ('item_armorpercent_by_time', 10, 0),
        123: ('item_hpregen_by_time', 10, 0),
        124: ('item_manaregen_by_time', 10, 0),
        125: ('item_attackertakesdamage', 5, 0),
        126: ('item_attacker_takes_light', 5, 0),
        127: ('item_ironmaiden_level', 6, 0),
        128: ('item_lifetap_level', 6, 0),
        129: ('item_thorns_level', 6, 0),
        130: ('item_bone_armor_level', 6, 0),
        131: ('item_bone_wall_level', 6, 0),
        132: ('item_concentration_level', 6, 0),
        133: ('item_decrepify_level', 6, 0),
        134: ('item_battlecommand_level', 6, 0),
        135: ('item_battlecry_level', 6, 0),
        136: ('item_battleorder_level', 6, 0),
        137: ('item_weaken_level', 6, 0),
        138: ('item_dol_level', 6, 0),
        139: ('item_terror_level', 6, 0),
        140: ('item_howl_level', 6, 0),
        141: ('item_taunt_level', 6, 0),
        142: ('item_dimvision_level', 6, 0),
        143: ('item_slow_level', 6, 0),
        144: ('item_firestorm_level', 6, 0),
        145: ('item_twister_level', 6, 0),
        146: ('item_volcano_level', 6, 0),
        147: ('item_armageddon_level', 6, 0),
        148: ('item_battletide_level', 6, 0),
        149: ('item_warmastery_level', 6, 0),
        150: ('item_poisonover_time', 6, 0),
        151: ('item_extra_trap_level', 6, 0),
        152: ('item_extra_tab', 0, 0),
        153: ('item_extra_single_skill', 10, 0),
        188: ('item_addskill_tab', 10, 0),
        195: ('item_death_send', 6, 0),
        196: ('item_death_level', 6, 0),
        197: ('item_death_skill', 6, 0),
        198: ('item_levelup_send', 6, 0),
        199: ('item_levelup_level', 6, 0),
        200: ('item_levelup_skill', 6, 0),
        201: ('item_kill_send', 6, 0), 202: ('item_kill_level', 6, 0),
        203: ('item_kill_skill', 6, 0),
        204: ('item_struck_send', 6, 0), 205: ('item_struck_level', 6, 0),
        206: ('item_struck_skill', 6, 0),
        207: ('item_damaged_send', 6, 0), 208: ('item_damaged_level', 6, 0),
        209: ('item_damaged_skill', 6, 0),
        212: ('item_fire_penetrate', 6, 0),
        213: ('item_light_penetrate', 6, 0),
        214: ('item_magic_penetrate', 6, 0),
        215: ('item_cold_penetrate', 6, 0),
        216: ('item_poison_penetrate', 6, 0),
        237: ('item_extra_low_damage', 5, 0),
        238: ('item_extra_low_attrib', 5, 0),
        239: ('item_extra_charges_2', 6, 0),
        240: ('item_add_class_charges', 8, 0),
    }
    
    n_stats = 0
    while True:
        if r.tell() + 9 > SJM:
            print(f"  [STOP] only {SJM - r.tell()} bits remaining (< 9)")
            break
        
        stat_pos = r.tell()
        sid = r.r(9)
        if sid == 0x1FF:
            print(f"  [0x1FF] terminator @ bit {stat_pos}")
            break
        
        if sid in known_stats:
            name, vb, pb = known_stats[sid]
            print(f"  stat[{n_stats:2d}] id={sid:3d} ({name:30s}) pb={pb} vb={vb}", end='')
            if pb > 0:
                pv = r.r(pb)
                print(f" param={pv}({pv:#x})", end='')
            if vb > 0:
                vv = r.r(vb)
                print(f" value={vv}({vv:#x})", end='')
            print()
        else:
            print(f"  stat[{n_stats:2d}] id={sid:3d} (UNKNOWN)")
            # Skip unknown - assume param+value = ~10 bits
            r.skip(10)
        n_stats += 1
        if n_stats > 20:
            print(f"  [STOP] too many stats")
            break
    
    r.align()
    end = r.tell()
    print(f"\n  Item end @ bit {end}, total = {end - ring['start']} bits = {(end - ring['start'] + 7)//8} bytes")
    print(f"  Expected: ~{ring['end']} bits") 
