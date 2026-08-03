#!/usr/bin/env python3
"""Scan JM section for valid items using known item codes."""
import struct, re

data = open(r'C:\Users\wang\Saved Games\Diablo II Resurrected\mods\CycleofImmortals\开心图书馆长.d2s', 'rb').read()
jm = data[0x394:0x498]

with open(r'src-tauri/src/protocol/d2i/legacy/game_items.rs', encoding='utf-8') as f:
    all_codes = set(re.findall(r'\(\"(\w+)\"', f.read()))

class BR:
    def __init__(self,buf,bit=0): self.buf=buf;self.pos=bit
    def r1(self): v=(self.buf[self.pos>>3]>>(self.pos&7))&1;self.pos+=1;return v
    def r(self,n):
        v=0
        for i in range(n):
            if self.pos>>3<len(self.buf): v|=((self.buf[self.pos>>3]>>(self.pos&7)&1)<<i)
            self.pos+=1
        return v

HC = [
    ('0',223,8),('1',31,7),('2',12,6),('3',91,7),('4',95,8),('5',104,8),
    ('6',123,7),('7',30,5),('8',8,6),('9',14,5),(' ',1,2),('a',15,5),
    ('b',10,4),('c',2,5),('d',35,6),('e',3,6),('f',50,6),('g',11,5),
    ('h',24,5),('i',63,7),('j',232,9),('k',18,6),('l',23,5),('m',22,5),
    ('n',44,6),('o',127,7),('p',19,5),('q',155,8),('r',7,5),('s',4,4),
    ('t',6,5),('u',16,5),('v',59,7),('w',0,5),('x',28,5),('y',40,7),('z',27,8),
]
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

prev=0
found=[]
for start in range(32,len(jm)*8-100):
    r=BR(data,0x394*8+start)
    f=r.r(32)
    if not ((f>>4)&1): continue
    if not ((f>>4)&1): continue
    v=r.r(3)
    if v<3 or v>8: continue
    l=r.r(3)
    if l>2: continue
    x=r.r(4);y=r.r(4);pg=r.r(3)
    code = huff(r)
    if code in all_codes:
        gap=start-prev if prev else 0
        prev=start
        found.append((start,code,l,x,y,pg,gap))

for start,code,l,x,y,pg,gap in found:
    print(f'@{start:4d} +{gap:3d}b  \"{code:4s}\"  loc={l} ({x:2d},{y:2d}) pg={pg}')
print(f'\nTotal: {len(found)} items')
