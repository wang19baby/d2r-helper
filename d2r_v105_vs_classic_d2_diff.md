# D2R V105 RotW vs Classic D2 — Format Differences Summary

Based on analysis of `krisives/d2s-format` (classic D2 spec) and `non-npc/D2R-Save-Editor` (D2R Infernal Edition), plus direct binary reverse-engineering of `开心邪帝.d2s`.

## 1. Overall Layout — CLASSIC vs D2R V105

| Region | Classic D2 (v96-v99) | D2R V105 (RotW) |
|--------|---------------------|------------------|
| Magic | `AA AA` (2B, 0xAA55) | `55 AA 55 AA` (4B, 0xAA55AA55) |
| Version | 96 | 105 (0x69) |
| Character | 0x14-0x14E (335 bytes) | 0x10-0x192 (387 bytes) |
| Quests | 0x14F (335) | 0x193 (403) |
| Waypoints | 0x279 (633) | 0x2BD (701) |
| NPCs | 0x2CA (714) | 0x30D (781) |
| Attributes | 0x2FD (765) | 0x341 (833) |
| Items (JM) | after attributes | after attributes |

D2R V105 has a **larger character section** (387 vs 335 bytes) with new fields for Resurrected menu appearance, RotW mode marker, and expanded unknown regions. Offsets for quests/waypoints/NPCs all shifted.

## 2. Item Header Bit Layout

| Bits | Classic D2 (d2s-format) | D2R V105 (actual file) | Diff |
|------|-------------------------|------------------------|------|
| 20-20 | Identified (1) | Identified (1) | Same |
| 27-27 | Socketed (1) | Socketed (1) | Same |
| 32-32 | Ear (1) | Ear (1) | Same |
| 33-33 | Starter Gear (1) | Starter Gear (1) | Same |
| 37-37 | Compact (1) | Simple flag (1) | Same |
| 38-38 | Ethereal (1) | Ethereal (1) | Same |
| 40-40 | Personalized (1) | Personalized (1) | Same |
| 42-42 | Runeword (1) | Runeword (1) | Same |
| 58-60 | Parent (3) | Parent (3) | Same |
| 61-64 | Equipped (4) | Equipped (4) | Same |
| 65-68 | Column (4) | Column (4) | Same |
| 69-71 | Row (3) | Row (3) | Same |
| 73-75 | Stash (3) | Stash (3) | Same |
| **76-79** | **unused (4)** | **Code index low 4 (4)** | **CHANGED** |
| **80-103** | **Type code ASCII (24)** | **Code index high 6 + reserved (24)** | **CHANGED** |
| 104-111 | padding (8) | (8) | Possibly same |

**Key change:** Classic D2 stores the 3-letter item type code as **24-bit ASCII** at bits 80-103. D2R V105 stores a **10-bit index** at bits 76-85 into a sorted code table. This means bits 76-79 changed from "unused" to "low 4 bits of code index".

## 3. Property List Stat Bit Widths

ItemStatCost.txt column comparison:

| Column | Classic D2 Name | D2R V105 Name | Purpose |
|--------|----------------|---------------|---------|
| Stat ID | `ID` | `*ID` | 9-bit key in property list |
| Width | `Save Bits` | `Save Bits` (still exists) | Bit width of stat value |
| Add | `Save Add` | `Save Add` (still exists) | Offset for signed values |
| Param Bits | `Save Param Bits` | `Save Param Bits` (still exists) | Extra parameter |
| Encode | `Encode` | `Encode` | Encoding type |
| **NEW** | - | **`Send Bits`** | **D2R uses this for item properties?** |
| **NEW** | - | **`Send Param Bits`** | **D2R param width** |

The `Send Bits` column is NEW in D2R and has LARGER values (11-32 bits) vs `Save Bits` (7-10 bits). 

**Current determination: INCONCLUSIVE.** Neither `Save Bits` nor `Send Bits` alone produces correct item boundaries for all 104 items. The truth is likely:
- A **hybrid**: `Save Bits` for classic legacy stats, `Send Bits` for D2R-new stats
- Or: `Save Bits` for the main property list, `Send Bits` for the runeword property list

## 4. Unknown Item Codes

104 items declared, but only ~819 codes in our table. Items with code indices >= 819 have codes not found in the mod's txt files. These may be:
- Vanilla D2R items excluded by the mod
- RotW expansion exclusive items
- Quest items or internal-use items

## 5. Recommended Implementation Path for Editor

```
1. Parse header + character section → name, class, level, stats, skills
2. Parse quests, waypoints, NPCs (fixed format)
3. Parse attributes (9-bit stat stream starting at 'gf' marker)
4. Parse items:
   a. Find 'JM' marker
   b. Read item count (LE uint16 at JM+2)
   c. For each item:
      - Read 112-bit header (use bit positions from section 2)
      - Decode type code: read bits 76-85 as 10-bit index into sorted code table
      - If simple (bit 37=1): advance 112 bits, done
      - If advanced: read extended data per _skip_advanced_item logic
      - Read property list: 9-bit stat_ids with variable values
   d. Need to determine correct stat bit widths (Save vs Send column)
5. Write back: modify bits, recalculate checksum
```

## 6. Code Table Population

For complete editor support, load codes from:
- `armor.txt`: `code`, `ubercode`, `ultracode`, `normcode`
- `weapons.txt`: same columns  
- `misc.txt`: `code` only
- Plus unique items codes from `uniqueitems.txt`
- Plus set item codes from `setitems.txt` / `sets.txt`
