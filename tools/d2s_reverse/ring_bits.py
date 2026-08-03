#!/usr/bin/env python3
"""戒指逐 bit 展开——按字段分组"""
import struct

PATH = r"C:\Users\wang\Saved Games\Diablo II Resurrected\mods\CycleofImmortals\开心图书馆长.d2s"
data = open(PATH, 'rb').read()
jm_items = data[0x398:0x498]

RING_START = 1368  # jm_items 内的 bit 偏移

def read_bits(start, n):
    """返回从 start 开始的 n 个 bit 字符串(LSB顺序)"""
    bits = []
    for i in range(n):
        pos = start + i
        byte_idx = pos // 8
        bit_off = pos % 8
        if byte_idx < len(jm_items):
            val = (jm_items[byte_idx] >> bit_off) & 1
        else:
            val = '?'
        bits.append(str(val))
    return ''.join(bits)

def show_field(name, start, bits, note=''):
    """显示一个字段"""
    b = read_bits(start, bits)
    # 计算整数值（LSB first）
    val = 0
    for i, c in enumerate(b):
        if c == '1':
            val |= 1 << i
    # 显示
    b_rev = b[::-1]  # 从高位到低位显示
    print(f'  {name:15s}  @{start:4d}  {b:8s}  ({val:5d})  {note}  [LSB={b}]')
    return val

# 按 D2S item 格式逐字段展开
print(f'戒指 @ jm_items bit {RING_START}')
print(f'文件偏移: 0x{0x398+RING_START//8:04X}')
print()

pos = RING_START
print(f'─── Compact Header ───')
pos += 32; show_field('flags', pos-32, 32, '0x00800010')
pos += 3;  show_field('ver', pos-3, 3)
pos += 3;  show_field('loc_id', pos-3, 3, '1=Equipped')
pos += 4;  show_field('slot', pos-4, 4, '6=RightFinger')
pos += 4;  show_field('x', pos-4, 4)
pos += 4;  show_field('y', pos-4, 4)
pos += 3;  show_field('page', pos-3, 3, '0=Equipped')

# Huffman code
print(f'  {"huff code":15s}  @{pos:4d}  --Huffman--    "rin"')
# r=5b, i=7b, n=6b, space=2b = 20 bits
huff_bits = {'r':5, 'i':7, 'n':6, ' ':2}
for ch, bits in huff_bits.items():
    b = read_bits(pos, bits)
    print(f'    -> char "{ch}" ({bits}b) bits: {b}')
    pos += bits

pos += 3;  show_field('ns', pos-3, 3, 'socket count')

print()
print(f'─── Body ───')
pos += 32; show_field('uid', pos-32, 32)
pos += 7;  show_field('ilvl', pos-7, 7)
pos += 4;  show_field('quality', pos-4, 4, '4=Magic')
pos += 1;  show_field('has_gfx', pos-1, 1)
pos += 3;  show_field('gfx_id', pos-3, 3)
pos += 1;  show_field('has_class', pos-1, 1)
pos += 11; show_field('magic_prefix', pos-11, 11)
pos += 11; show_field('magic_suffix', pos-11, 11)
pos += 1;  show_field('v105_pad', pos-1, 1)

# 后面就是 stat 列表了
print()
print(f'─── Stat 列表 @{pos} ───')
# 前几个 stat
for si in range(5):
    sid = read_bits(pos, 9)
    sid_val = sum(int(sid[i])<<i for i in range(9))
    if sid_val == 0x1FF:
        print(f'  [终止符 0x1FF] @{pos}')
        break
    # 读 value: 假设 8b (strength)
    val_bits = read_bits(pos+9, 8)
    val = sum(int(val_bits[i])<<i for i in range(8))
    print(f'  stat[{si}] sid={sid_val:3d} @{pos:4d}  {sid}  value={val:3d} (raw)')
    pos += 9 + 8

print()
print(f'─── 后续 raw ───')
# 继续显示未解析的 bit
for i in range(0, 64, 8):
    b = read_bits(pos+i, 8)
    val = sum(int(b[j])<<j for j in range(8))
    print(f'  @{pos+i:4d}  {b}  0x{val:02X}')
