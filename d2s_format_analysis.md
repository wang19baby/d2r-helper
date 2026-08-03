# D2R V105 RotW Save File Format Analysis

## File: 开心邪帝.d2s (4126 bytes)

---

## 1. File Header (16 bytes)

| Offset | Size | Field | Value |
|--------|------|-------|-------|
| 0x00 | 4 | Magic | `55 AA 55 AA` |
| 0x04 | 4 | Version | 105 (0x69) |
| 0x08 | 4 | File Size | 4126 |
| 0x0C | 4 | Checksum | Adler-32 |

## 2. Section Layout (V105 RotW)

| Offset | End | Section | Size | Description |
|--------|-----|---------|------|-------------|
| 0x000 | 0x00F | Header | 16B | Magic, Version, Size, Checksum |
| 0x010 | 0x192 | Character | 387B | Stats, Skills, Name, Appearance |
| 0x193 | 0x2BC | Quests | 298B | Quest completion flags |
| 0x2BD | 0x30C | Waypoints | 80B | Waypoint bitfield (39 bits) |
| 0x30D | 0x340 | NPCs | 52B | NPC interaction flags |
| 0x341 | - | Attributes | var | `gf` marker, stat stream |
| 0x374 | - | Mercenary | 30B | `if` marker, merc stats |
| **0x394** | **0xDD2** | **Items (JM)** | **~2619B** | **All player items** |
| 0xDD3 | 0xDD6 | Empty JM | 4B | `JM 00 00` (end marker) |
| 0xDD7 | 0xFA0 | Journey | var | `jf` section marker |
| 0xFA1 | 0x1005 | Corpse | var | `kf` section marker |
| 0x1006 | 0x101E | Golem | var | `gf` section marker |

## 3. Item Encoding (JM Section at 0x394)

### 3.1 JM Section Header

| Offset | Field | Size | Description |
|--------|-------|------|-------------|
| 0x394 | `JM` marker | 2B | ASCII 'J' 'M' = 0x4A 0x4D |
| 0x396 | Version | 2B | LE uint16 = 0x0068 (104) |
| 0x398 | Item Data | - | Bitstream starts here |

**Note:** `struct.unpack("<H", raw, idx+2)[0]` reads the item count (104). The bitstream at `(idx+4)*8` contains all items.

### 3.2 Item Bitstream Header (112 bits = 14 bytes)

Each item starts with a 112-bit header. Bit offsets relative to item start:

| Bits | Field | Size | Values |
|------|-------|------|--------|
| 20 | Identified | 1 | 0/1 |
| 27 | Socketed | 1 | 0/1 |
| 32 | Ear | 1 | Player ear item flag |
| 33 | Starter Gear | 1 | Starting equipment |
| 37 | Simple Flag | 1 | 0=Advanced, 1=Simple (112 bits only) |
| 38 | Ethereal | 1 | 0/1 |
| 40 | Personalized | 1 | Has custom name |
| 42 | Runeword | 1 | Runeword item (or has runes?) |
| 58-60 | Parent | 3 | 0=Stored, 1=Equipped, 2=Belt, 4=Transit, 6=Socketed |
| 61-64 | Equipped | 4 | Body slot (1=Head..12=Alt.Left) |
| 65-68 | Column | 4 | Grid column in container |
| 69-71 | Row | 3 | Grid row in container |
| 73-75 | Stash | 3 | 1=Inv, 4=Cube, 5=Stash |
| **76-85** | **Code Index** | **10** | **Index into sorted item code table** |

**CRITICAL FINDING:** The type_code is NOT 32-bit ASCII! It's a **10-bit index** into the sorted global item code table (819 codes from mod files).

### 3.3 Body Slots

| Equipped | Slot |
|----------|------|
| 1 | Head |
| 2 | Neck |
| 3 | Torso |
| 4 | Right Hand (Weapon) |
| 5 | Left Hand (Shield) |
| 6 | Right Ring |
| 7 | Left Ring |
| 8 | Waist |
| 9 | Feet |
| 10 | Hands |
| 11 | Alt Right Weapon |
| 12 | Alt Left Weapon |

### 3.4 Simple vs Advanced Items

**Simple items** (potions, scrolls, runes, gems): **exactly 112 bits**. No extra data.

**Advanced items** (weapons, armor, jewelry, charms): Data extends beyond 112 bits.

### 3.5 Advanced Item Data Structure

Starting at bit 111 (after the 112-bit header):

| Field | Bits | Description |
|-------|------|-------------|
| Item ID | 32 | Unique instance identifier |
| Level | 7 | Item level (ilvl) |
| Rarity | 4 | 0=Low, 1=Normal, 2=Superior, 3=Magic, 4=Rare, 5=Set, 6=Unique |
| Multiple Pictures | 1 | Has alternate graphics |
| Multiple Pictures Data | (0 or 4) | Extra data if multiple pictures |
| Class Specific | 1 | Has class restriction |
| Class Specific Data | (0 or 12) | Class/req data if class specific |
| Rarity Affixes | var | 7(Magic) / 21(Rare) / 27(Set/Unique) bits |
| Runeword ID | (0 or 16) | Runeword identifier |
| Personalized Name | var | 7-bit chars until terminator |
| Tome Pages | (0 or 5) | For tome/scroll books |
| Timestamp | 1 | Creation timestamp flag |
| Defense Bonus | (0 or 11) | For armor items |
| Max Durability | (0 or 8) | For armor/weapon items |
| Current Durability | (0 or 9) | If max durability > 0 |
| Quantity | (0 or 9) | For stackable items |
| Socket Count | (0 or 4) | Number of sockets |
| Set Bits | (0 or 5) | Set bonus configuration |
| Property List | var | Item magic properties |
| Set Bonus Lists | (0-5 × var) | Set bonus properties |
| Runeword Properties | (0 or var) | Runeword custom properties |

### 3.6 Property List Encoding

The property list is a sequence of stat entries terminated by `0x1FF` (9-bit sentinel):

```
while True:
    stat_id = read_bits(9)  # Stat identifier
    if stat_id == 0x1FF: break
    value = read_bits(get_stat_width(stat_id))  # Variable width per stat
```

**IMPORTANT:** Stat bit widths come from the `Save Bits` column in ItemStatCost.txt (NOT `Send Bits`!)

For D2R V105 RotW, the column mapping in ItemStatCost.txt is:
- `*ID` → stat ID (not `ID`)
- `Save Bits` → bit width for property value
- `Save Add` → offset for signed values
- `Save Param Bits` → extra parameter bits
- `Encode` → encoding type (1=direct, 2=duration, 3=percentage)

## 4. Parsing Results

**22 of 104 items parsed** before skip_advanced_item exceeds data bounds.

Equipped items found:
- Alt.Left: `g33` (unknown mod code) [ETHEREAL][SOCKETED]
- Neck: `ssp` (Short Spear?) [RUNEWORD][PERSONALIZED]
- R.Ring: `6bs` (unknown mod code) [RUNEWORD][PERSONALIZED]
- Neck: `jl1` (Jewel?) [RUNEWORD][PERSONALIZED]

**82 items unparsed** - these likely contain your target items (rings, amulets, etc.)

## 5. Key Issues to Fix for Full Parser

1. **Advanced item skip drifts**: The `skip_advanced_item` function using classic D2 stat bit widths doesn't exactly match D2R V105 RotW. Items after ~#4 start drifting.

2. **Missing mod item codes**: Some code indices exceed the 819-entry table (indices 852, 880, 897, 913 observed). These are mod-added items.

3. **Property list terminator**: D2R might use a different end marker or format for property lists than the classic `0x1FF`.

4. **Weapon/armor detection**: Must use `code`, `ubercode`, `ultracode`, AND `normcode` from armor.txt/weapons.txt to correctly identify all item categories.

## 6. Recommended Approach for Editor

1. Load ALL item codes from mod txt files (all code columns)
2. Use `Save Bits` for property widths (proven correct for items 0-3)
3. For advanced items that fail to parse: fall back to finding 0x1FF terminator by scanning forward
4. Build item→container mappings from parent/stash/equipped fields
5. For unique/set identification: use the 32-bit item_id + base code to look up uniqueitems.txt/sets.txt
