#!/usr/bin/env python3
"""从零逆向 d2s：逐段解码 gf(属性) if(技能) JM(物品)"""

import struct

PATH = r"D:\work_space\personal_workspace\d2r\开心图书馆长.d2s"
data = open(PATH, 'rb').read()

# ── BitReader (LSB-first per byte) ──
class BR:
    def __init__(self, buf, bit_off=0):
        self.buf = buf
        self.pos = bit_off
    def r1(self):
        b = (self.buf[self.pos >> 3] >> (self.pos & 7)) & 1
        self.pos += 1
        return b
    def r(self, n):
        v = 0
        for i in range(n):
            if self.pos >> 3 < len(self.buf):
                v |= (self.buf[self.pos >> 3] >> (self.pos & 7) & 1) << i
            self.pos += 1
        return v
    def r8(self, n=8): return self.r(n)
    def r16(self, n=16): return self.r(n)
    def r32(self, n=32): return self.r(n)
    def skip(self, n): self.pos += n
    def align(self): self.pos = (self.pos + 7) & ~7
    def tell(self): return self.pos

# ── Huffman decode for item 4-char codes ──
# 37 chars: value=LSB-first tree path, bits=path length
HUFF = {
    '0':(223,8),'1':(31,7),'2':(12,6),'3':(91,7),'4':(95,8),
    '5':(104,8),'6':(123,7),'7':(30,5),'8':(8,6),'9':(14,5),
    ' ':(1,2),'a':(15,5),'b':(10,4),'c':(2,5),'d':(35,6),
    'e':(3,6),'f':(50,6),'g':(11,5),'h':(24,5),'i':(63,7),
    'j':(232,9),'k':(18,6),'l':(23,5),'m':(22,5),'n':(44,6),
    'o':(127,7),'p':(19,5),'q':(155,8),'r':(7,5),'s':(4,4),
    't':(6,5),'u':(16,5),'v':(59,7),'w':(0,5),'x':(28,5),
    'y':(40,7),'z':(27,8),
}
# Build trie
class HN:
    def __init__(self):
        self.ch = None
        self.kids = [None, None]
hroot = HN()
for ch, (val, bits) in HUFF.items():
    n = hroot
    for b in range(bits):
        bit = (val >> b) & 1
        if not n.kids[bit]:
            n.kids[bit] = HN()
        n = n.kids[bit]
    n.ch = ch

def decode_huff(r):
    s = []
    for _ in range(4):
        n = hroot
        while n.ch is None and r.pos >> 3 < len(r.buf):
            bit = r.r1()
            if n.kids[bit] is None:
                break
            n = n.kids[bit]
        if n.ch: s.append(n.ch)
    return ''.join(s).strip()

# ─────────────────────────────────────
# SECTION 1: gf (Attributes)
# ─────────────────────────────────────
def parse_gf(off):
    print(f"{'='*60}")
    print(f"gf段 @ 0x{off:X} — 角色属性")
    print(f"{'='*60}")
    
    marker = data[off:off+2]
    assert marker == b'gf', f"Expected gf at 0x{off:X}, got {marker}"
    
    r = BR(data, off*8 + 16)  # skip "gf"
    
    # Attribute names and their bit widths
    attr_defs = [
        (0, "strength", 10), (1, "energy", 10), (2, "dexterity", 10),
        (3, "vitality", 10), (4, "stat_points", 10), (5, "new_skills", 8),
        (6, "hitpoints", 21), (7, "max_hp", 21), (8, "mana", 21),
        (9, "max_mana", 21), (10, "stamina", 21), (11, "max_stamina", 21),
        (12, "level", 7), (13, "experience", 32), (14, "gold", 25),
        (15, "gold_bank", 25),
    ]
    
    attrs = {}
    while r.tell() + 9 <= len(data)*8:
        sid = r.r16(9)
        if sid == 0x1FF:
            print(f"  [终止符 0x1FF @ bit {r.tell()}]")
            break
        if sid <= 15:
            _, name, bits = attr_defs[sid]
            val = r.r32(bits)
            # Q8 stats (HP/Mana/Stamina) display = val / 256
            if sid in (6,7,8,9,10,11):
                display = val / 256
                print(f"  {name:15s} = {val:>10d} (raw) = {display:.1f} (display)")
            else:
                print(f"  {name:15s} = {val:>10d}")
            attrs[sid] = val
        else:
            val = r.r32(10)
            print(f"  [未知 sid={sid}, val={val}]")
    
    print(f"  段结束 @ bit {r.tell()} (=字节 {(r.tell()+7)//8})")
    return attrs

# ─────────────────────────────────────
# SECTION 2: if (Skills)
# ─────────────────────────────────────
def parse_if(off):
    print(f"\n{'='*60}")
    print(f"if段 @ 0x{off:X} — 技能")
    print(f"{'='*60}")
    
    marker = data[off:off+2]
    assert marker == b'if', f"Expected if at 0x{off:X}, got {marker}"
    
    r = BR(data, (off+2)*8)  # after "if"
    
    # First 4 bytes = count/header?
    hdr = r.r32(32)
    print(f"  if header (u32) = {hdr}")
    
    skills = []
    max_skills = 60
    for i in range(max_skills):
        if r.tell() + 9 > len(data)*8:
            break
        sid = r.r16(9)
        if sid == 0x1FF:
            print(f"  [终止符 0x1FF @ bit {r.tell()}]")
            break
        if r.tell() + 8 > len(data)*8:
            break
        lvl = r.r16(8)
        skills.append((sid, lvl))
        print(f"  skill[{i:2d}] id={sid:3d} level={lvl}")
    
    print(f"  段结束 @ bit {r.tell()} (=字节 {(r.tell()+7)//8})")
    return skills

# ─────────────────────────────────────
# SECTION 3: JM (Items)
# ─────────────────────────────────────
QUAL = {1:'Low',2:'Normal',3:'Superior',4:'Magic',5:'Set',6:'Rare',7:'Unique',8:'Crafted'}

def parse_jm(off):
    print(f"\n{'='*60}")
    print(f"JM段 @ 0x{off:X} — 物品")
    print(f"{'='*60}")
    
    assert data[off:off+2] == b'JM', f"Expected JM at 0x{off:X}"
    
    count = struct.unpack_from('<H', data, off+2)[0]
    print(f"  声明物品数: {count}")
    
    r = BR(data, (off+4)*8)  # after JM+count
    
    items = []
    for idx in range(count):
        start_bit = r.tell()
        
        if start_bit + 32 > len(data)*8:
            print(f"  [{idx}] 截断 (剩余不足32位)")
            break
        
        # 1) Flags (32b)
        flags = r.r32(32)
        identified = (flags >> 4) & 1
        socketed  = (flags >> 11) & 1
        is_new    = (flags >> 13) & 1
        is_ear    = (flags >> 16) & 1
        starter   = (flags >> 17) & 1
        simple    = (flags >> 21) & 1
        ethereal  = (flags >> 22) & 1
        personal  = (flags >> 24) & 1
        runeword  = (flags >> 26) & 1
        
        # 2) Compact header
        ver   = r.r8(3)
        loc_id = r.r8(3)    # D2S: 1=Equipped, 2=Belt, 0/3=Stored
        e_slot = r.r8(4)    # Equipped slot (1-12)
        x     = r.r8(4)
        y     = r.r8(4)
        page  = r.r8(3)     # 0=Equipped, 1=Backpack, 5=MyStash
        
        # 3) Huffman code
        code = decode_huff(r)
        
        # 4) Socket count
        ns = r.r8(3)
        
        hdr_end = r.tell()
        
        # Print basic info
        slot_names = {1:'Helm',2:'Neck',3:'Torso',4:'RightHand',5:'LeftHand',
                      6:'RightFinger',7:'LeftFinger',8:'Waist',9:'Feet',10:'Hands'}
        mode_str = {0:'Stored',1:'Equipped',2:'Belt'}.get(loc_id, f'?({loc_id})')
        page_str = {0:'Equipped',1:'Backpack',5:'MyStash'}.get(page, f'Mod({page})')
        
        flags_str = []
        if identified: flags_str.append('ID')
        if socketed: flags_str.append('SK')
        if simple: flags_str.append('SIMPLE')
        if ethereal: flags_str.append('ETH')
        if runeword: flags_str.append('RW')
        if personal: flags_str.append('PN')
        
        print(f"\n  --- [{idx}] {code} @ bit {start_bit} ({start_bit//8}B into JM) ---")
        print(f"  flags=0x{flags:08X} ({','.join(flags_str) if flags_str else 'none'})")
        print(f"  ver={ver} loc_id={loc_id}({mode_str}) slot={e_slot}({slot_names.get(e_slot,'-')})")
        print(f"  pos=({x},{y}) page={page}({page_str}) ns={ns}")
        print(f"  header={hdr_end-start_bit} bits")
        
        # For non-simple items, try to read more
        if not simple:
            # uid + ilvl + quality
            uid = r.r32(32)
            ilvl = r.r8(7)
            qb = r.r8(4)
            qname = QUAL.get(qb, f'?({qb})')
            print(f"  uid={uid} ilvl={ilvl} quality={qname}")
            
            has_gfx = r.r8(1)
            if has_gfx: gfx = r.r8(3)
            has_class = r.r8(1)
            
            # quality-specific fields
            if qb in (1,3):   # Low/Superior
                r.skip(3)
            elif qb == 4:      # Magic
                pref = r.r16(11)
                suff = r.r16(11)
                print(f"  Magic: prefix={pref} suffix={suff}")
            elif qb == 5:      # Set
                set_id = r.r16(12)
                print(f"  Set ID={set_id}")
            elif qb in (6,8):  # Rare/Crafted
                r1 = r.r8(8); r2 = r.r8(8)
                affs = []
                for _ in range(6):
                    if r.r8(1): affs.append(r.r16(11))
                print(f"  Rare: name={r1},{r2} affixes={affs}")
            elif qb == 7:      # Unique
                uniq = r.r16(12)
                print(f"  Unique ID={uniq}")
            
            # Runeword
            if runeword:
                rw_id = r.r16(12); r.skip(4)
                print(f"  Runeword ID={rw_id}")
            
            # Personalized name
            if personal:
                pn = ''
                for _ in range(16):
                    c = r.r8(8)
                    if c == 0: break
                    pn += chr(c)
                print(f"  Personalized: '{pn}'")
            
            # Durability (for armor/weapon/shield)
            cat_a = code in ('cap','xap','buc','uap','xpl')  # rough check
            cat_w = code in ('dgr','sbw','hax','scy','2ax','crs')
            cat_s = code in ('buc','xbl','lrg','gts')
            if cat_a or cat_w or cat_s:
                md = r.r8(8)
                if md > 0:
                    cd = r.r16(10)
                    md_disp = md // 2
                    cd_disp = cd // 2
                    print(f"  dur: {cd_disp}/{md_disp}")
                else:
                    r.skip(1)
            
            # Socket info
            if socketed: r.skip(4)
            if qb == 5: r.skip(5)  # set bitmap
            
            # Stat list
            stats = []
            while r.tell() + 9 <= len(data)*8:
                sid = r.r16(9)
                if sid == 0x1FF:
                    break
                # Try to read value (varies by stat)
                r.skip(10)  # approximate
                stats.append(sid)
            
            if stats:
                print(f"  stat IDs before 0x1FF: {len(stats)} entries ({stats[:5]}...)")
            else:
                print(f"  (no stats)")
        
        r.align()
        items.append({
            'code': code, 'start_bit': start_bit, 'loc_id': loc_id,
            'x': x, 'y': y, 'page': page, 'simple': simple
        })
    
    print(f"\n  段结束 @ bit {r.tell()} (=字节 {(r.tell()+7)//8})")
    return items

# ── Find markers ──
def find(pat):
    pos = data.find(pat)
    return pos if pos >= 0 else None

gf_off = find(b'gf')
if_off = find(b'if')
jm_off = find(b'JM')

print(f"gf @ 0x{gf_off:X}" if gf_off else "gf NOT FOUND")
print(f"if @ 0x{if_off:X}" if if_off else "if NOT FOUND")
print(f"JM @ 0x{jm_off:X}" if jm_off else "JM NOT FOUND")

if gf_off: attrs = parse_gf(gf_off)
if if_off: skills = parse_if(if_off)
if jm_off: items = parse_jm(jm_off)
