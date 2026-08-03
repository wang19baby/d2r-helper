#!/usr/bin/env python3
"""扫描 D2R dump, 找 LEA rip-relative 指令引用关键 D2S/D2I 字符串

D2R 用 64-bit LEA + rip-relative 访问 .rdata 字符串:
  48 8D <ModRM> <disp32>    # REX.W + LEA + mod=00,rm=101 (rip) + 32-bit displacement
  ModRM: 0x05 (rax), 0x0D (rcx), 0x15 (rdx), 0x1D (rbx),
         0x25 (rsp), 0x2D (rbp), 0x35 (rsi), 0x3D (rdi)
  disp32 = target_addr - (instr_addr + 7)

dump imagebase = 0x7ff7ec580000
.text = 0x7ff7ec581000..0x7ff7edb58000 (code)
.rdata = 0x7ff7edb59250..0x7ff7edf06000 (strings)
"""
import struct, sys
from pathlib import Path

DUMP_PATH = r"D:\personal\games\Diablo II Resurrected\aggr_80152_155655_MAP_0x22BB7710000_0x2796000.bin"
# 真实 imagebase (从字符串 VA - file_off 反推,与 IDA 显示的 0x7ff7ec580000 差 0xC00)
IMAGEBASE = 0x7FF7EC580C00
TEXT_START_VA = 0x7FF7EC581000
TEXT_END_VA   = 0x7FF7EDB58000
TEXT_FILE_OFF = TEXT_START_VA - IMAGEBASE   # 0x400
TEXT_FILE_END = TEXT_END_VA   - IMAGEBASE   # 0x15D6400

# 关键字符串地址
TARGETS = {
    0x7ff7edcadd40: "D2I: 'Unsupported save version in inventory stream'",
    0x7ff7edcaec30: "D2S: 'Unsupported save version'",
    0x7ff7edcadbd0: "PLAYERSAVE: 'IsPlayerExit'",
    0x7ff7edc9a230: "itemCode (Huffman 4-char)",
    0x7ff7edca31e8: "RarePrefix",
    0x7ff7edca3218: "MagicPrefix",
    0x7ff7edca3e60: "ItemStatCost",
    0x7ff7edc9a290: "equipmentItemCode",
    0x7ff7edc9e120: "strItemStatThrowDamageRange",
    0x7ff7edcaa2a0: "defaultItemCodeCol3",
    0x7ff7edcaa2c0: "defaultItemCodeCol4",
    0x7ff7edcaa338: "defaultItemCodeCol1",
    0x7ff7edcaa368: "defaultItemCodeCol2",
}

# LEA mod=00 rm=101 (RIP)
LEA_MODRM = {0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x35, 0x3D}

def main():
    p = Path(DUMP_PATH)
    sz = p.stat().st_size
    print(f"Dump: {p.name}  ({sz/1024/1024:.1f} MB)")
    print(f"Imagebase: {IMAGEBASE:#x}")
    print(f".text:     {TEXT_FILE_OFF:#x}..{TEXT_FILE_END:#x}  ({TEXT_FILE_END-TEXT_FILE_OFF/1024/1024:.1f} MB)")
    print()

    with open(p, 'rb') as f:
        data = f.read()

    target_set = set(TARGETS.keys())
    found = []
    lea_total = 0

    # 扫 .text 段
    i = TEXT_FILE_OFF
    end = TEXT_FILE_END - 7
    while i < end:
        if data[i] != 0x48:            # REX.W
            i += 1; continue
        if data[i+1] != 0x8D:          # LEA
            i += 1; continue
        modrm = data[i+2]
        if modrm not in LEA_MODRM:
            i += 1; continue
        disp = struct.unpack_from('<i', data, i+3)[0]
        va_instr = IMAGEBASE + i
        target = (va_instr + 7 + disp) & 0xFFFFFFFFFFFFFFFF
        lea_total += 1
        if target in target_set:
            found.append((va_instr, target, TARGETS[target]))
        i += 7  # LEA + disp32 = 7 bytes, 但若对齐 prefix 不同则可能错位;保守用 1

    print(f"共扫描 {lea_total} 条 LEA rip-relative")
    print(f"命中 {len(found)} 处目标引用:")
    for va, tgt, name in found:
        print(f"  {va:#x}  -> {tgt:#x}  [{name}]")

    # 按目标分组排序
    found.sort(key=lambda x: (x[1], x[0]))
    print("\n按目标分组:")
    cur_tgt = None
    for va, tgt, name in found:
        if tgt != cur_tgt:
            print(f"\n  {tgt:#x}  [{name}]")
            cur_tgt = tgt
        print(f"    LEA @ {va:#x}  (file_off={va-IMAGEBASE:#x})")

if __name__ == '__main__':
    main()