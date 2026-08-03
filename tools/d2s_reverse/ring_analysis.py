#!/usr/bin/env python3
import struct

PATH = r"C:\Users\wang\Saved Games\Diablo II Resurrected\mods\CycleofImmortals\开心图书馆长.d2s"
data = open(PATH, 'rb').read()
jm_items = data[0x398:0x498]

class BR:
    def __init__(self,buf,bit=0): self.buf=buf;self.pos=bit
    def r1(self): v=(self.buf[self.pos>>3]>>(self.pos&7))&1;self.pos+=1;return v
    def r(self,n):
        v=0
        for i in range(n):
            if self.pos>>3<len(self.buf): v|=((self.buf[self.pos>>3]>>(self.pos&7)&1)<<i)
            self.pos+=1
        return v
    def skip(self,n): self.pos+=n
    def tell(self): return self.pos

HC=[('0',223,8),('1',31,7),('2',12,6),('3',91,7),('4',95,8),('5',104,8),
    ('6',123,7),('7',30,5),('8',8,6),('9',14,5),(' ',1,2),('a',15,5),
    ('b',10,4),('c',2,5),('d',35,6),('e',3,6),('f',50,6),('g',11,5),
    ('h',24,5),('i',63,7),('j',232,9),('k',18,6),('l',23,5),('m',22,5),
    ('n',44,6),('o',127,7),('p',19,5),('q',155,8),('r',7,5),('s',4,4),
    ('t',6,5),('u',16,5),('v',59,7),('w',0,5),('x',28,5),('y',40,7),('z',27,8)]
class HN:
    def __init__(self): self.ch=None;self.k=[None,None]
hroot=HN()
for ch,val,bits in HC:
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
        while n.ch is None:
            bit=r.r1()
            if n.k[bit] is None: break
            n=n.k[bit]
        if n.ch: s.append(n.ch)
    return ''.join(s).strip()

r=BR(jm_items,1368)
start=1368

def t(msg):
    pos=r.tell(); consumed=pos-start
    print(f'  +{consumed:3d}b ({pos:4d}): {msg}')

t('start')
f=r.r(32); t(f'flags=0x{f:08X} id={(f>>4)&1} smp={(f>>21)&1}')
ver=r.r(3); t(f'ver={ver}')
loc=r.r(3); t(f'loc_id={loc}')
slot=r.r(4); t(f'slot={slot}')
x=r.r(4); t(f'x={x}')
y=r.r(4); t(f'y={y}')
pg=r.r(3); t(f'page={pg}')
code=huff(r); t(f'code="{code}"')
ns=r.r(3); t(f'ns={ns}')
uid=r.r(32); t(f'uid={uid}')
ilvl=r.r(7); t(f'ilvl={ilvl}')
qb=r.r(4); t(f'quality={qb}')
hg=r.r(1); t(f'has_gfx={hg}')
if hg: gfx=r.r(3); t(f'gfx_id={gfx}')
hc=r.r(1); t(f'has_class={hc}')

# Magic 22-bit: 11+11 split
pf=r.r(11); sf=r.r(11); t(f'Magic: prefix={pf} suffix={sf}')

# Body fields before stat list
rw=(f>>26)&1; pn=(f>>24)&1
if rw: r.skip(16); t('runeword(16b)')
if pn:
    for _ in range(16):
        c=r.r(8)
        if c==0: break
    t('personalized')

# ring: not armor/weapon/shield → no mirror, no durability
r.skip(1); t('v105 pad(1b)')  # always present
sk=(f>>11)&1
if sk: r.skip(4); t('socket(4b)')
if qb==5: r.skip(5); t('set(5b)')

# Stat list with CORRECT bit widths
print()
t('=== Stat 列表(按正确位宽) ===')
STAT_BITS = {
    0:('strength',8,32), 1:('energy',7,32), 2:('dexterity',7,32),
    3:('vitality',7,32), 7:('maxhp',9,32), 9:('maxmana',8,32),
    16:('item_armor_percent',9,0), 19:('tohit',10,0),
    35:('magic_damage_reduction',6,0),
    36:('damageresist',9,200), 37:('magicresist',9,200),
    39:('fireresist',9,200), 41:('lightresist',9,200),
    43:('coldresist',9,200), 45:('poisonresist',9,200),
    93:('IAS',7,20), 96:('FRW',7,20), 105:('FCR',7,20),
    107:('item_singleskill',3,0), 127:('allskills',3,0),
}

while True:
    if r.tell()+9>len(jm_items)*8: break
    sid=r.r(9)
    if sid==0x1FF: t('0x1FF TERMINATOR'); break
    if sid in STAT_BITS:
        name,bits,add=STAT_BITS[sid]
        raw=r.r(bits)
        disp=raw-add
        t(f'sid={sid:3d} {name:25s} raw={raw:4d} display={disp}')
    else:
        r.skip(10)
        t(f'sid={sid:3d} {"(UNKNOWN)":25s} +10b')
