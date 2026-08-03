#!/usr/bin/env python3
"""
Bit-level diagnostics for 开心图书馆长.d2s
Traces every field in the item bitstream for ALL backpack items,
showing raw hex + consumed bits to identify misalignment.
"""
import struct
from pathlib import Path

PATH = r"D:\work_space\personal_workspace\d2r\开心图书馆长.d2s"
data = open(PATH, 'rb').read()
SZ = len(data)

class BR:
    """LSB-first bit reader, mirroring Rust BitReader behavior"""
    def __init__(self, buf, bit=0):
        self.buf = buf
        self.pos = bit
    
    def r(self, n):
        """Read n bits (LSB first)"""
        v = 0
        for i in range(n):
            if self.pos < len(self.buf) * 8:
                v |= ((self.buf[self.pos // 8] >> (self.pos % 8)) & 1) << i
            self.pos += 1
        return v
    
    def tell(self): return self.pos
    
    def skip(self, n): self.pos += n
    
    def hex_remaining(self, n_bytes=16):
        """Show hex of remaining bytes from current bit position"""
        byte_pos = self.pos // 8
        bit_in_byte = self.pos % 8
        available = min(n_bytes, len(self.buf) - byte_pos)
        hex_str = ' '.join(f'{self.buf[byte_pos+i]:02x}' for i in range(available))
        return f"@{self.pos}b (0x{byte_pos:04X}.{bit_in_byte}) [{hex_str}]"

# --- Huffman decoder ---
H = {'0':(223,8),'1':(31,7),'2':(12,6),'3':(91,7),'4':(95,8),'5':(104,8),'6':(123,7),
     '7':(30,5),'8':(8,6),'9':(14,5),' ':(1,2),'a':(15,5),'b':(10,4),'c':(2,5),
     'd':(35,6),'e':(3,6),'f':(50,6),'g':(11,5),'h':(24,5),'i':(63,7),'j':(232,9),
     'k':(18,6),'l':(23,5),'m':(22,5),'n':(44,6),'o':(127,7),'p':(19,5),'q':(155,8),
     'r':(7,5),'s':(4,4),'t':(6,5),'u':(16,5),'v':(59,7),'w':(0,5),'x':(28,5),
     'y':(40,7),'z':(27,8)}

class HN:
    def __init__(self): self.ch = None; self.k = [None, None]

hroot = HN()
for ch, (val, bits) in H.items():
    n = hroot
    for i in range(bits - 1, -1, -1):
        b = (val >> i) & 1
        if not n.k[b]:
            n.k[b] = HN()
        n = n.k[b]
    n.ch = ch

def decode_huff(r):
    """Decode Huffman-coded item code"""
    s = ''
    n = hroot
    while len(s) < 6:
        if n is None: break
        if n.ch is not None:
            s += n.ch
            if len(s) >= 3 and s[-1] == ' ':
                break
            n = hroot
            continue
        if n.k is None: break
        b = r.r(1)
        n = n.k[b]
    return s.strip()

def hex_of(data, bit_pos, n_bits):
    """Show hex containing the given bit range"""
    byte_start = bit_pos // 8
    byte_end = (bit_pos + n_bits + 7) // 8
    return ' '.join(f'{data[byte_start+i]:02x}' for i in range(min(byte_end - byte_start + 2, len(data) - byte_start)))

def fmt(s):
    """Shorten long fields"""
    if len(s) > 80: return s[:40] + ' ...'
    return s

# ============================================================
# TRACE: Ring item in backpack (code="rin", x=15, y=0)
# ============================================================
print("=" * 80)
print("开心图书馆长.d2s — 戒指 (rin) 精确位追踪")
print("=" * 80)

# JM section @ 0x394
jm_off = 0x394
jm_data = data[jm_off:0x494]  # next JM @ 0x494
SJM = len(jm_data) * 8

count = struct.unpack_from('<H', data, jm_off + 2)[0]
print(f"\nJM @ 0x{jm_off:04X}, count = {count}, bitstream = {SJM} bits")

# Step 1: Read ALL items and record their boundaries
print("\n--- Step 1: Parse all items, find rin ---")
r = BR(jm_data, 32)  # skip JM header (32 bits)

items = []
for idx in range(count):
    start_bit = r.tell()
    
    flags = r.r(32)
    ver = r.r(3)
    loc = r.r(3)
    slot = r.r(4)
    x = r.r(4)
    y = r.r(4)
    pg = r.r(3)
    
    # Save position before Huffman
    huff_start = r.tell()
    code = decode_huff(r)
    huff_end = r.tell()
    huff_bits = huff_end - huff_start
    
    simple = (flags >> 21) & 1
    
    if not simple:
        ns = r.r(3)  # num sockets
        uid = r.r(32)
        ilvl = r.r(7)
        qb = r.r(4)
        
        # Quality-specific fields
        hg = r.r(1)
        if hg: r.skip(3)
        hc = r.r(1)
        
        match qb:
            case 1 | 3: r.skip(3)
            case 4: pf = r.r(11); sf = r.r(11)
            case 5: sid = r.r(12)
            case 6 | 8: r.skip(16); r.skip(6*12)
            case 7: uid_val = r.r(12)
        
        rw_bit = (flags >> 26) & 1
        pn_bit = (flags >> 24) & 1
        if rw_bit: r.skip(16)
        if pn_bit:
            for _ in range(16):
                c = r.r(8)
                if c == 0: break
        
        is_armor = code in ('cap','xap','buc','uap','xpl','lrg')
        is_weapon = code in ('dgr','sbw','hax','scy','2ax','crs','7s8','wae','xpk','jkn','sst')
        is_shield = code in ('buc','xbl','lrg','gts')
        
        if is_armor or is_shield:
            r.skip(11)  # defense
            r.skip(8+10)  # max_dur + cur_dur
        elif is_weapon:
            md = r.r(8)
            if md > 0: r.skip(10)
            else: r.skip(1)
        
        r.skip(1)  # v105 padding
        
        sk_bit = (flags >> 11) & 1
        if sk_bit: r.skip(4)
        if qb == 5: r.skip(5)
        
        # Stat list - skip until 0x1FF
        stat_start = r.tell()
        stats_read = 0
        while True:
            if r.tell() + 9 > SJM:
                break
            sid = r.r(9)
            if sid == 0x1FF:
                break
            stats_read += 1
            r.skip(10)  # placeholder for param+value
        
        # Align
        r.align()
        end_bit = r.tell()
        bit_len = end_bit - start_bit
    else:
        # Simple item - just 1 bit after code
        end_bit = r.tell() + 1
        r.skip(1)
        r.align()
        bit_len = r.tell() - start_bit
        qb = 0
        ilvl = 0
        uid = 0
        stats_read = 0
    
    loc_names = {0:'Stored',1:'Equipped',2:'Belt',3:'Socket',4:'Cursor'}
    q_names = {0:'?',1:'Low',2:'Norm',3:'Sup',4:'Mag',5:'Set',6:'Rare',7:'Uniq',8:'Craft'}
    
    items.append({
        'idx': idx, 'start': start_bit, 'end': end_bit, 'len': bit_len,
        'code': code, 'loc': loc, 'slot': slot,
        'x': x, 'y': y, 'ilvl': ilvl,
        'quality': q_names.get(qb, '?'),
        'stats': stats_read,
        'simple': simple,
        'pos': f'({x},{y})'
    })

print(f"{'#':>3s} {'Start':>7s} {'End':>7s} {'Len':>5s} {'Code':>5s} {'Loc':>4s} {'Slot':>4s} {'Pos':>8s} {'Q':>5s} {'St':>3s}")
print('-' * 65)
for it in items:
    print(f"{it['idx']:3d} {it['start']:7d} {it['end']:7d} {it['len']:5d} "
          f"{it['code']:>5s} {it['loc']:4d} {it['slot']:4d} "
          f"{it['pos']:>8s} {it['quality']:>5s} {it['stats']:3d}")

# Find ring and tome
for it in items:
    if it['code'] == 'rin':
        print(f"\n>>> 戒指 RING: item[{it['idx']}] @ bit {it['start']} (byte 0x{(it['start']//8)+jm_off:04X}.{it['start']%8})")
        print(f"    ends @ bit {it['end']}, len={it['len']} bits")
        
        # Trace the ring bit by bit
        r = BR(jm_data, it['start'])
        print(f"\n=== 戒指 逐位追踪 (start bit = {it['start']}) ===")
        
        def trace_field(name, bits, r, show_hex=True):
            val = r.r(bits)
            h = hex_of(jm_data, r.tell() - bits, bits) if show_hex else ''
            print(f"  [@{r.tell()-bits:5d}b] {name:30s} = {val:>8d} (0x{val:0{max(bits//4,2)}x})  {h}")
            return val
        
        f = trace_field('flags(32b)', 32, r)
        trace_field('version(3b)', 3, r)
        loc_v = trace_field('location(3b)', 3, r)
        slot_v = trace_field('equipped_slot(4b)', 4, r)
        x_v = trace_field('x(4b)', 4, r)
        y_v = trace_field('y(4b)', 4, r)
        pg_v = trace_field('page(3b)', 3, r)
        
        huff_start = r.tell()
        code_str = decode_huff(r)
        huff_end = r.tell()
        print(f"  [@{huff_start:5d}b] huff_code                   = \"{code_str}\"  ({huff_end-huff_start} bits)")
        
        simple = (f >> 21) & 1
        print(f"  {'>>> SIMPLE ITEM' if simple else '>>> FULL ITEM'}")
        
        if not simple:
            trace_field('num_sockets(3b)', 3, r, True)
            trace_field('uid(32b)', 32, r, True)
            trace_field('item_level(7b)', 7, r, True)
            trace_field('quality(4b)', 4, r, True)
            
            hg_v = trace_field('has_gfx(1b)', 1, r)
            if hg_v: trace_field('gfx_id(3b)', 3, r)
            
            hc_v = trace_field('has_class_specific(1b)', 1, r)
            if hc_v: trace_field('class_specific_id(11b)', 11, r, True)
            
            # Using the SAME quality_bits as Rust code
            qb_raw = 4  # Magic = 4 for a ring
            pf_v = trace_field('magic_prefix(11b)', 11, r, True)
            sf_v = trace_field('magic_suffix(11b)', 11, r, True)
            
            rw_bit = (f >> 26) & 1
            pn_bit = (f >> 24) & 1
            if rw_bit: trace_field('runeword_id(12b)+4b', 16, r, True)
            if pn_bit:
                pers_start = r.tell()
                for _ in range(16):
                    c = r.r(8)
                    if c == 0: break
                print(f"  [@{pers_start:5d}b] personalized_name         = skipped ({r.tell()-pers_start} bits)")
            
            is_armor = code_str in ('cap','xap','buc','uap','xpl','lrg')
            is_weapon = code_str in ('dgr','sbw','hax','scy','2ax','crs','7s8','wae','xpk','jkn','sst')
            is_shield = code_str in ('buc','xbl','lrg','gts')
            
            if is_armor or is_shield:
                trace_field('armor_defense(11b)', 11, r, True)
                trace_field('max_dur(8b)', 8, r, True)
                trace_field('cur_dur(10b)', 10, r, True)
            elif is_weapon:
                md = trace_field('max_dur(8b)', 8, r, True)
                if md > 0: trace_field('cur_dur(10b)', 10, r, True)
                else: trace_field('zero_dur(1b)', 1, r, True)
            else:
                print(f"  [@{r.tell():5d}b] ring: no durability (not armor/weapon/shield)")
            
            # CRITICAL: v105 padding
            trace_field('v105_padding(1b)', 1, r, False)
            
            sk_bit = (f >> 11) & 1
            if sk_bit: trace_field('socketed_count(4b)', 4, r)
            if qb_raw == 5: trace_field('set_plist(5b)', 5, r)
            
            # NOW: stat list - trace every entry
            stat_list_start = r.tell()
            print(f"\n  === STAT LIST @ bit {stat_list_start} ===")
            stat_count = 0
            while True:
                remaining = SJM - r.tell()
                if remaining < 9:
                    print(f"  [END] not enough bits ({remaining} < 9)")
                    break
                sid = r.r(9)
                if sid == 0x1FF:
                    print(f"  [0x1FF TERMINATOR @ {r.tell()-9}]")
                    break
                # Skip value bits (10 placeholder)
                stat_count += 1
                print(f"  stat[{stat_count}] id={sid:3d} (0x{sid:03x}) @ bit {r.tell()-9}")
                r.skip(10)
            
            print(f"  End of stat list: consumed {stat_count} stats")
            print(f"  Total bits used before alignment: {r.tell() - stat_list_start}")
            
            r.align()
            final_pos = r.tell()
            bit_len_actual = final_pos - it['start']
            print(f"\n  Aligned to byte boundary @ bit {final_pos}")
            print(f"  Item total: {bit_len_actual} bits = {bit_len_actual} bytes")
            print(f"  Next item starts @ bit {final_pos}")
    
    if it['code'] == 'tbk':
        print(f"\n>>> 传送书 TOME: item[{it['idx']}] @ bit {it['start']}")

# Verify chain: do items overlap or have gaps?
print(f"\n\n=== 链式验证 ===")
for i in range(len(items)-1):
    curr_end = items[i]['end']
    next_start = items[i+1]['start']
    gap = next_start - curr_end
    overlap = curr_end > next_start
    marker = '*** OVERLAP ***' if overlap else f'gap={gap}b'
    if overlap or gap != 0:
        print(f"  Item[{items[i]['idx']}] end={curr_end} → Item[{items[i+1]['idx']}] start={next_start}: {marker}")

# Show raw hex around the ring's bitstream
ring_item = next((it for it in items if it['code'] == 'rin'), None)
if ring_item:
    start_byte = (ring_item['start'] // 8) + jm_off
    end_byte = (ring_item['end'] + 7) // 8 + jm_off
    print(f"\n=== 戒指原始字节 (0x{start_byte:04X} - 0x{end_byte:04X}) ===")
    for off in range(start_byte, end_byte + 1, 16):
        chunk = data[off:off+16]
        hex_str = ' '.join(f'{b:02x}' for b in chunk)
        print(f"  0x{off:04X}: {hex_str}")
