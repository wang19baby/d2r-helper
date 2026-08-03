#!/usr/bin/env python3
import struct

data = open(r'C:\Users\wang\Saved Games\Diablo II Resurrected\mods\CycleofImmortals\开心图书馆长.d2s', 'rb').read()
jm_data = data[0x394:0x498]

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
    def align(self): self.pos=(self.pos+7)&~7
    def tell(self): return self.pos

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

for item_idx, start_bit in [(0,32),(1,112),(2,192),(3,272)]:
    r = BR(jm_data, start_bit)
    s = r.tell()
    
    f = r.r(32); print(f'[item {item_idx}] start@{start_bit}b flags=0x{f:08X}')
    print(f'  +{r.tell()-s:2d}  ver={r.r(3)}')
    print(f'  +{r.tell()-s:2d}  loc={r.r(3)}')
    print(f'  +{r.tell()-s:2d}  slot={r.r(4)}')
    print(f'  +{r.tell()-s:2d}  x={r.r(4)} y={r.r(4)}')
    print(f'  +{r.tell()-s:2d}  pg={r.r(3)}')
    pos_before_huff = r.tell()
    code = huff(r)
    print(f'  +{r.tell()-s:2d}  huff="{code}" (huff consumed {r.tell()-pos_before_huff}b)')
    print(f'  +{r.tell()-s:2d}  ns={r.r(3)}')
    
    # Simple: cs bit + optional amount
    cs = r.r1()
    amt = 0
    if cs: amt = r.r(8)
    print(f'  +{r.tell()-s:2d}  cs={cs} amount={amt}')
    
    before_align = r.tell()
    r.align()
    after = r.tell()
    align_pad = after - before_align
    print(f'  +{r.tell()-s:2d}  align(+{align_pad}b padding)')
    print(f'  Total: {after-s}b = {after-start_bit-32}b from prev end')
    print()
