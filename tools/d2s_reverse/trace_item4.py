#!/usr/bin/env python3
import struct

data = open(r'C:\Users\wang\Saved Games\Diablo II Resurrected\mods\CycleofImmortals\开心图书馆长.d2s', 'rb').read()
jm_data = data[0x394:0x498]
SJM = len(jm_data)*8

class BR:
    def __init__(self,buf,bit=0): self.buf=buf;self.pos=bit
    def r1(self):
        v=(self.buf[self.pos>>3]>>(self.pos&7))&1
        self.pos+=1
        return v
    def r(self,n):
        v=0
        for i in range(n):
            if self.pos>>3<len(self.buf): v|=((self.buf[self.pos>>3]>>(self.pos&7)&1)<<i)
            self.pos+=1
        return v
    def skip(self,n): self.pos+=n
    def tell(self): return self.pos
    def align(self): self.pos=(self.pos+7)&~7

H={'0':(223,8),'1':(31,7),'2':(12,6),'3':(91,7),'4':(95,8),'5':(104,8),'6':(123,7),'7':(30,5),'8':(8,6),'9':(14,5),' ':(1,2),'a':(15,5),'b':(10,4),'c':(2,5),'d':(35,6),'e':(3,6),'f':(50,6),'g':(11,5),'h':(24,5),'i':(63,7),'j':(232,9),'k':(18,6),'l':(23,5),'m':(22,5),'n':(44,6),'o':(127,7),'p':(19,5),'q':(155,8),'r':(7,5),'s':(4,4),'t':(6,5),'u':(16,5),'v':(59,7),'w':(0,5),'x':(28,5),'y':(40,7),'z':(27,8)}
class HN:
    def __init__(self): self.ch=None;self.k=[None,None]
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
        while n.ch is None:
            bit=r.r1()
            if n.k[bit] is None: break
            n=n.k[bit]
        if n.ch: s.append(n.ch)
    return ''.join(s).strip()

ARMOR={'cap','xap','buc','uap','xpl'}
WEAPON={'dgr','sbw','hax','scy','2ax','crs'}
SHIELD={'buc','xbl','lrg','gts'}

r=BR(jm_data,352)

def t(msg):
    consumed = r.tell() - 352
    print(f'  +{consumed:3d}b (@{r.tell():4d}b body): {msg}')

f=r.r(32); t(f'flags=0x{f:08X}')
ver=r.r(3); t(f'ver={ver}')
loc=r.r(3); t(f'loc_id={loc}')
slot=r.r(4); t(f'slot={slot}')
x=r.r(4);y=r.r(4); t(f'x={x} y={y}')
pg=r.r(3); t(f'page={pg}')
code=huff(r); t(f'code="{code}"')
ns=r.r(3); t(f'ns={ns}')

uid=r.r(32); t(f'uid={uid}')
ilvl=r.r(7); t(f'ilvl={ilvl}')
qb=r.r(4); t(f'quality={qb}')
hg=r.r(1); t(f'has_gfx={hg}')
if hg: r.skip(3); t(f'gfx_id(3b)')
hc=r.r(1); t(f'has_class={hc}')

if qb in (1,3): r.skip(3); t(f'quality-specific(3b)')
elif qb==4: pf=r.r(11); sf=r.r(11); t(f'Magic prefix={pf} suffix={sf}')
elif qb==5: sid=r.r(12); t(f'Set ID={sid}')
elif qb in (6,8):
    r.skip(16)
    for _ in range(6):
        if r.r(1): r.skip(11)
    t(f'Rare/Crafted')
elif qb==7: r.skip(12); t(f'Unique ID')
else: t(f'no quality fields (Normal)')

rw=(f>>26)&1; pn=(f>>24)&1
if rw: r.skip(16); t(f'runeword(16b)')
if pn:
    for _ in range(16):
        c=r.r(8)
        if c==0: break
    t(f'personalized name')

if code in ARMOR or code in SHIELD: r.skip(11); t(f'armor/shield mirror(11b)')
else: t(f'no mirror (not armor/shield)')

if code in ARMOR or code in WEAPON or code in SHIELD:
    md=r.r(8); t(f'max_dur raw={md}')
    if md>0: r.skip(10); t(f'cur_dur(10b)')
    else: r.skip(1); t(f'zero dur(1b)')
else:
    t(f'no durability (not armor/weapon/shield)')

r.skip(1); t(f'post-dur padding(1b)')

sk=(f>>11)&1
if sk: r.skip(4); t(f'socket(4b)')
if qb==5: r.skip(5); t(f'set bitmap(5b)')

if qb>=4:
    while True:
        if r.tell()+9 > SJM: break
        sid=r.r(9)
        if sid==0x1FF: t(f'stat TERMINATOR'); break
        r.skip(10)
else:
    t(f'no stat list (quality {qb} < 4)')

r.align()
t(f'ALIGNED — total body bits')
print(f'\nItem[4] total: {r.tell()-352} bits = {(r.tell()-352+7)//8} bytes')
print(f'Item[5] starts at bit {r.tell()}')
