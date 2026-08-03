# D2I Parser Refactor Plan

## Current State (after 48/48 item fix)

Files:
- `jm_reader.rs` — 496 lines, 13 functions (6 unused), 10 forward-scan instances
- `stat_list.rs` — 149 lines, guard logic mixed in
- `stat.rs` — 375 lines, save_bits=0→9 fix
- `dump_stash.rs` — 36 lines CLI wrapper

## Issues

### 1. Dead Code (6 functions)
```
u8_to_mode, u8_to_location, u8_to_page         — unused, duplicate of Item enum methods
read_type_fields, parse_stat_list, read_all_stat_lists — unused, from old Python port
scan_ext_alignment, scan_valid_code              — unused, replaced by micro-scan
read_chest_stackable_amount                      — unused, replaced by (y<<4|x)
```

### 2. Forward Scan Duplication (10 instances)
Each scan reimplements the same pattern:
- Create `BitReader` at probe position
- Read 32-bit flags + 3-bit version
- Check `count_ones() <= N` && `identified bit`
- Read Huffman code at +53 bits

Sites (jm_reader.rs):
1. Compact item path: `for skip in 1..=max_skip` (line ~254)
2. Non-compact body: `for byte_skip in 0..max_scan` (line ~290)
3. Non-compact body fallback: `for byte_skip in 0..max_scan` (line ~313)
4. Err handler: `for skip in 1..=limit` (line ~412)
5. Filter rejection: `for skip in 1..=max_scan` (line ~408)
6-10. Non-compact body + filter + Err (variations with different thresholds)

### 3. try_parse_one — 160 lines, two personalities
- Compact and non-compact paths in one function
- Each path has its own error recovery logic
- Non-compact path has 12 sub-steps embedded inline

### 4. parse_jm_page — orchestrator + scanner
- Validates items
- Runs micro-scans on error
- Checks ALL_ITEMS
- Pushes to output
Too many responsibilities.

### 5. make_item — 18+ positional parameters
```rust
fn make_item(flags_raw, ver, mode, loc, px, py, pg, code, uid, ilvl,
    quality, stat_lists, num_sockets, amount, cur_dur, max_dur, start_bit, bit_len)
```
Callers must remember all positions — error-prone.

### 6. Guard logic — hardcoded in StatList::read
```rust
const MAX_KNOWN_ID: u16 = 419;
const MAX_STATS: u32 = 100;
const MAX_SAME_STAT: u32 = 8;
const MAX_UNKNOWN_CONSEC: u32 = 3;
const MAX_CORE_CONSEC: u32 = 3;
```
These should be configurable per-parse context (item stats vs character stats have different rules).

### 7. No tests for new logic
- `test_empty_jm` and `test_no_jm_magic` — trivial, don't test real parsing
- No fixtures for mod stash files

---

## Refactor Plan

### Phase 1: Extract Scan Logic (Session 1-2)

**Goal**: Replace 10 duplicated forward scans with one reusable function.

```rust
/// Result of scanning forward for the next item header.
struct ScanResult {
    position: usize,    // bit offset of found header
    flags: u32,
    version: u8,
    code: String,
}

/// Scan forward from `start` for a valid item header.
/// Returns Some(ScanResult) on first match, None otherwise.
fn scan_next_item(
    payload: &[u8],
    start: usize,
    config: ScanConfig,
) -> Option<ScanResult>;
```

`ScanConfig` parametrizes:
- max_scan_bytes (was 32-512)
- max_flag_bits (was 6-10)
- accept_version (was ==5 or <=5)
- code_length_check (was >=1 or ==3)
- require_identified (was true)

All 10 scan sites → single function.

### Phase 2: Extract Parsing Pipeline (Session 2-3)

**Goal**: Split try_parse_one into focused stages.

```rust
enum ItemBody {
    Compact(CompactBody),
    NonCompact(NonCompactBody),
}

struct CompactBody {
    amount: u32,
    has_realm_data: bool,
    realm_data: Vec<u8>,
}

struct NonCompactBody {
    uid: u32,
    ilvl: u8,
    quality: ItemQuality,
    stats: Vec<StatList>,
    amount: u32,
}

// Stage 1: Read the common header
fn parse_item_header(reader: &mut BitReader) -> ParseResult<ItemHeader>;

// Stage 2: Read the body based on compact flag
fn parse_compact_body(reader: &mut BitReader, header: &ItemHeader, is_stackable: bool) -> CompactBody;
fn parse_noncompact_body(reader: &mut BitReader, header: &ItemHeader, payload: &[u8], is_stackable: bool) -> NonCompactBody;

// Stage 3: Assemble into ParsedItem
fn build_item(header: ItemHeader, body: ItemBody, start: usize) -> ParsedItem;
```

Benefits:
- Each stage independently testable
- Body types are explicit
- No more 160-line monolith

### Phase 3: Encapsulate Guards (Session 3)

**Goal**: Make stat guards configurable, not hardcoded.

```rust
struct StatReadConfig {
    max_stats: u32,           // 100 for items, unlimited for character
    max_same_stat: u32,       // 8
    max_unknown_consec: u32,  // 3
    max_core_consec: u32,     // 3
    default_save_bits: u8,    // 9 (matching Python)
    known_id_max: u16,        // 419
    core_stats: &'static [u16], // [4,5,6,8,10,12,13,14,15]
}

impl StatList {
    pub fn read(reader: &mut BitReader, table: &StatTable) -> ParseResult<Self>;
    pub fn read_with_config(reader: &mut BitReader, table: &StatTable, config: StatReadConfig) -> ParseResult<Self>;
}
```

Default `StatReadConfig::default()` = item-appropriate values.
Character parsing uses a different config.

### Phase 4: Builder Pattern for make_item (Session 3)

```rust
let item = ParsedItemBuilder::new()
    .flags(flags_raw)
    .code(code)
    .position(px, py, pg)
    .quality(quality)
    .amount(amount)
    .bit_range(start, reader.offset())
    .build();
```

### Phase 5: Tests (Session 4)

- Port Python reference (`cli_construct.py --d2i --bits`) test cases
- Fixture-based: read known d2i → verify item count + amounts
- Test each scan stage in isolation
- Test guard behavior with crafted stat streams

### Phase 6: Cleanup (Session 4)

- Remove 6 dead functions
- Move skip_string_7bit to core/encoding
- Rename `parse_jm_page` → `parse_item_stream` (clearer)
- Add doc comments to all public functions
- Remove read_chest_stackable_amount (dead)

---

## Dependency Graph

```
Phase 1 (scan_next_item) ← no deps, can start immediately
Phase 2 (stages)          ← depends on Phase 1 (scan becomes a stage)
Phase 3 (guards)          ← independent, but touches StatList::read
Phase 4 (builder)         ← depends on Phase 2 (replaces make_item callers)
Phase 5 (tests)           ← can start alongside Phase 2
Phase 6 (cleanup)         ← depends on Phase 2-4 (removes dead code)
```

## Summary

| Metric | Before | After |
|--------|--------|-------|
| jm_reader.rs lines | 496 | ~300 |
| try_parse_one lines | 160 | ~60 |
| parse_jm_page lines | 80 | ~50 |
| Forward scan sites | 10 | 1 |
| Unused functions | 6 | 0 |
| Test coverage | 2 trivial tests | 20+ tests |
| make_item params | 18 positional | builder pattern |
| Guard config | hardcoded | configurable |