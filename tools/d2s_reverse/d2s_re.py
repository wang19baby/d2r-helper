#!/usr/bin/env python3
"""从零开始逆向 .d2s 格式。逐段反序列化。"""

import struct

PATH = r"D:\work_space\personal_workspace\d2r\开心图书馆长.d2s"
data = open(PATH, 'rb').read()
SZ = len(data)

# ── 工具函数 ──
def u32(off): return struct.unpack_from('<I', data, off)[0] if off+4 <= SZ else -1
def u16(off): return struct.unpack_from('<H', data, off)[0] if off+2 <= SZ else -1
def u8(off): return data[off]

# ── LSB-first BitReader ──
class BR:
    def __init__(self, buf, bit_off=0):
        self.buf = buf
        self.pos = bit_off
    def r1(self):
        v = (self.buf[self.pos>>3]>>(self.pos&7))&1
        self.pos+=1; return v
    def r(self, n):
        v=0
        for i in range(n):
            if self.pos>>3 < len(self.buf):
                v |= ((self.buf[self.pos>>3]>>(self.pos&7)&1)<<i)
            self.pos+=1
        return v
    def skip(self, n): self.pos+=n
    def align(self): self.pos=(self.pos+7)&~7
    def tell(self): return self.pos

# ── Huffman 4-char item code ──
H = {'0':(223,8),'1':(31,7),'2':(12,6),'3':(91,7),'4':(95,8),
     '5':(104,8),'6':(123,7),'7':(30,5),'8':(8,6),'9':(14,5),
     ' ':(1,2),'a':(15,5),'b':(10,4),'c':(2,5),'d':(35,6),
     'e':(3,6),'f':(50,6),'g':(11,5),'h':(24,5),'i':(63,7),
     'j':(232,9),'k':(18,6),'l':(23,5),'m':(22,5),'n':(44,6),
     'o':(127,7),'p':(19,5),'q':(155,8),'r':(7,5),'s':(4,4),
     't':(6,5),'u':(16,5),'v':(59,7),'w':(0,5),'x':(28,5),
     'y':(40,7),'z':(27,8)}
class HN:
    def __init__(self): self.ch=None; self.k=[None,None]
hroot=HN()
for ch,(val,bits) in H.items():
    n=hroot
    for b in range(bits):
        bit=(val>>b)&1
        if not n.k[bit]: n.k[bit]=HN()
        n=n.k[bit]
    n.ch=ch
def huff(r):
    s=[]
    for _ in range(4):
        n=hroot
        while n.ch is None and r.pos>>3<len(r.buf):
            bit=r.r1()
            if n.k[bit] is None: break
            n=n.k[bit]
        if n.ch: s.append(n.ch)
    return ''.join(s).strip()

QUAL = {1:'Low',2:'Normal',3:'Superior',4:'Magic',5:'Set',6:'Rare',7:'Unique',8:'Crafted'}
SM = {0:'Stored',1:'Equipped',2:'Belt'}
PM = {0:'Equipped',1:'Backpack',5:'MyStash'}
SL = {1:'Helm',2:'Neck',3:'Torso',4:'RHand',5:'LHand',
      6:'RFinger',7:'LFinger',8:'Waist',9:'Feet',10:'Hands'}

# ─────────────────────────────────────
print(f"{'='*60}")
print(f"d2s 逆向工程: {PATH}")
print(f"大小: {SZ} 字节")
print(f"{'='*60}")

# 1. HEADER
print(f"\n{'─'*60}")
print("1. HEADER")
print(f"{'─'*60}")
print(f"  0000  {data[0:4].hex():12s}    Magic = 55 AA 55 AA")
print(f"  0004  {SZ:>5d} B            Version = {u32(4)}")
print(f"  0008  {u32(8):>5d} B            FileSize = {u32(8)}")
print(f"  000C  0x{u32(12):08X}       Checksum")
print(f"  0010  {u32(16):>5d}             ActiveWeapon = {u32(16)}")
print(f"  0014  0x{u8(0x14):02x}                     Status")
print(f"  0018  {u8(0x18):>3d}                  Class = {u8(0x18)} (Warlock)")
print(f"  001B  {u8(0x1B):>3d}                  Level = {u8(0x1B)}")

# 2. modified layout items (0xF8~0x12A)
print(f"\n{'─'*60}")
print("2. 魔改 12B stride items @ 0xF8")
print(f"{'─'*60}")
cnt = u16(0xF8)
print(f"  count = {cnt}")
for i in range(4):
    off = 0xFB + i*12
    if off+12 > SZ: break
    c3 = data[off:off+3].decode('ascii',errors='replace')
    il = u8(off+4)
    qb = u8(off+5)
    raw = u16(off+6)
    print(f"  [{i}] @0x{off:04X}: code={c3:4s} ilvl={il:3d} quality={qb}({QUAL.get(qb,'?')}) raw={raw}")

# 3. NAME
print(f"\n{'─'*60}")
print("3. 角色名 @ 0x12B")
print(f"{'─'*60}")
nb = data[0x12B:0x13A]
nm = nb.split(b'\x00')[0].decode('utf-8',errors='replace')
print(f"  \"{nm}\"")

# ─────────────────────────────────────
# 4. gf → Attributes
# ─────────────────────────────────────
print(f"\n{'─'*60}")
print("4. gf @ 0x341 — Attributes (角色属性)")
print(f"{'─'*60}")

A = [
    (0,'strength',10,0),(1,'energy',10,0),(2,'dexterity',10,0),
    (3,'vitality',10,0),(4,'stat_points',10,0),(5,'new_skills',8,0),
    (6,'hitpoints',21,1),(7,'max_hp',21,1),(8,'mana',21,1),
    (9,'max_mana',21,1),(10,'stamina',21,1),(11,'max_stamina',21,1),
    (12,'level',7,0),(13,'experience',32,0),(14,'gold',25,0),(15,'gold_bank',25,0),
]
A_MAP = {a[0]:a for a in A}

r = BR(data, 0x341*8 + 16)  # skip "gf"
while True:
    if r.tell() + 9 > SZ*8: break
    sid = r.r(9)
    if sid == 0x1FF:
        print(f"  [终止符 0x1FF @ bit {r.tell()}]")
        break
    if sid in A_MAP:
        _,name,bits,q8 = A_MAP[sid]
        val = r.r(bits)
        if q8:
            print(f"  {name:15s} = {val:>8d} (raw) = {val/256:.1f}")
        else:
            print(f"  {name:15s} = {val:>8d}")
    else:
        v10 = r.r(10)
        print(f"  [未知 sid={sid} val={v10}]")
print(f"  段结束 @ bit {r.tell()} (字节 {(r.tell()+7)//8})", end="")
print(f"  <- next bit = 0x{(r.tell()+7)//8:X}")

# ─────────────────────────────────────
# 5. if → Skills
# ─────────────────────────────────────
print(f"\n{'─'*60}")
print("5. if @ 0x374 — Skills (技能)")
print(f"{'─'*60}")

# if 段: 0x374..0x393 (32B), 之后就是 JM 0x394
# 格式: "if" + u32 header + skill data
if_end = 0x394  # next marker
r = BR(data, (0x374+2)*8)  # skip "if"
hdr4 = r.r(32)
print(f"  if header (u32) = {hdr4}")

skills = []
while r.tell() < if_end*8 - 17:  # stop before JM
    sid = r.r(9)
    if sid == 0x1FF: print(f"  [终止符 @ {r.tell()}]"); break
    lvl = r.r(8)
    skills.append((sid, lvl))

if skills:
    print(f"  {len(skills)} 个技能:")
    for i,(sid,lvl) in enumerate(skills):
        print(f"  [{i:2d}] skill_id={sid:3d} level={lvl:3d}")
else:
    print(f"  (全 0，新角色无技能点)")
print(f"  段结束 @ 0x{r.tell()//8:X}")

# ─────────────────────────────────────
# 6. JM → Items
# ─────────────────────────────────────
print(f"\n{'─'*60}")
print("6. JM @ 0x394 — Items (物品)")
print(f"{'─'*60}")

jm_pos = 0x394
jm_count = u16(jm_pos+2)
print(f"  声明: {jm_count} 个物品")

# Find end of JM section (next marker)
jm_end = SZ
for mk in (b'JM', b'jf', b'kf'):
    p = data.find(mk, jm_pos+4)
    if p > 0 and p < jm_end: jm_end = p

r = BR(data, (jm_pos+4)*8)  # after "JM" + count
items = []
for idx in range(jm_count):
    start_bit = r.tell()
    if start_bit + 32 > SZ*8: print(f"  [{idx}] 截断"); break
    
    # flags
    f = r.r(32)
    idf = (f>>4)&1; sck = (f>>11)&1; nwf = (f>>13)&1; ear = (f>>16)&1
    stf = (f>>17)&1; smp = (f>>21)&1; eth = (f>>22)&1; pn = (f>>24)&1; rw = (f>>26)&1
    
    # compact
    ver   = r.r(3)
    loc   = r.r(3)
    eslot = r.r(4)
    x     = r.r(4)
    y     = r.r(4)
    pg    = r.r(3)
    code  = huff(r)
    ns    = r.r(3)
    hdr_end = r.tell()
    
    fls = []
    if idf: fls.append('ID')
    if sck: fls.append('SK')
    if smp: fls.append('SIMPLE')
    if eth: fls.append('ETH')
    if rw: fls.append('RW')
    if pn: fls.append('PN')
    
    print(f"\n  [{idx}] {code} @ 0x{(jm_pos+4+start_bit//8):04X}+{start_bit%8}b")
    print(f"    flags=0x{f:08X} ({','.join(fls) if fls else 'none'})")
    print(f"    loc={loc}({SM.get(loc,'?')}) slot={eslot}({SL.get(eslot,'-')}) pos=({x},{y}) pg={pg}({PM.get(pg,'?')}) ns={ns}")
    
    if not smp:
        uid = r.r(32); ilvl = r.r(7); qb = r.r(4)
        print(f"    uid={uid} ilvl={ilvl} quality={qb}({QUAL.get(qb,'?')})")
        
        # 跳过 body 细节，跳到 item 末尾
        hg = r.r(1)
        if hg: r.skip(3)
        hc = r.r(1)
        
        # quality 分支
        if qb in (1,3): r.skip(3)
        elif qb == 4: pf=r.r(11); sf=r.r(11); print(f"    Magic: prefix={pf} suffix={sf}")
        elif qb == 5: sid=r.r(12); print(f"    Set ID={sid}")
        elif qb in (6,8):
            r1=r.r(8); r2=r.r(8)
            afs=[]
            for _ in range(6):
                if r.r(1): afs.append(r.r(11))
            print(f"    Rare: name={r1},{r2} affixes={len(afs)}")
        elif qb == 7: uniq=r.r(12); print(f"    Unique ID={uniq}")
        
        if rw: r.skip(12+4)
        if pn:
            for _ in range(16):
                c=r.r(8)
                if c==0: break
        
        # Durability (粗略)
        if code in ('dgr','buc','cap','xap','hax','sbw','scy','crs','2ax'):
            md=r.r(8)
            if md>0: r.skip(10)
            else: r.skip(1)
        
        if sck: r.skip(4)
        if qb == 5: r.skip(5)
        
        # Stat list
        st = []
        limit=0
        while r.tell()+9 <= jm_end*8 and limit<100:
            sid=r.r(9); limit+=1
            if sid==0x1FF: break
            r.skip(10)  # 近似
            st.append(sid)
        if st: print(f"    stat IDs: {len(st)} entries before 0x1FF")
    
    r.align()
    items.append(code)

print(f"\n  摘要:")
for i,c in enumerate(items):
    print(f"  [{i}] {c}")
print(f"  共 {len(items)} 个物品 @ {(r.tell()+7)//8} 字节")

# ─────────────────────────────────────
# 7. jf/kf (Waypoints/Quests)
# ─────────────────────────────────────
print(f"\n{'─'*60}")
print("7. jf @ 0x498, kf @ 0x49A — Waypoints & Quests")
print(f"{'─'*60}")
print(f"  jf marker @ 0x{data.find(b'jf'):04X}, kf marker @ 0x{data.find(b'kf'):04X}")
print(f"  (新角色，无小站/任务数据)")

print(f"\n{'='*60}")
print(f"END. 文件结束 @ 0x{SZ:X} ({SZ} bytes)")
