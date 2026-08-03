"""Compare D2RMM mod's itemstatcost.txt against magical_props.rs.
Outputs all mismatches for stats that are Saved=1 in the mod file.
"""
import csv, re

FILE = r"D:\personal\games\Diablo II Resurrected\mods\D2RMM\D2RMM.mpq\data\global\excel\itemstatcost.txt"
TABLE = r"D:\work_space\personal_workspace\d2r\d2r-marketplace-tauri\src-tauri\src\stash\magical_props.rs"

# ---- Parse itemstatcost.txt (only Saved=1 stats) ----
file_stats = {}
with open(FILE, 'r', encoding='utf-8', errors='replace') as f:
    reader = csv.DictReader(f, delimiter='\t')
    for row in reader:
        try:
            sid = int(row['*ID'])
            saved = row.get('Saved', '').strip()
            if saved != '1':
                continue
            sb = int(row['Save Bits']) if row['Save Bits'].strip() else 0
            spb = int(row['Save Param Bits']) if row['Save Param Bits'].strip() else 0
            sa = int(row['Save Add']) if row['Save Add'].strip() else 0
            signed = 1 if row.get('Signed', '').strip() == '1' else 0
            encoding = 4 if row.get('Encode', '').strip() == 'bytime' else 0
            file_stats[sid] = (row['Stat'], sb, spb, sa, signed, encoding)
        except (ValueError, KeyError):
            pass

# ---- Parse magical_props.rs ----
# Build (sb, spb, sa, signed, encoding) keyed by index
table_entries = {}  # sid -> (sb, spb, sa, signed, encoding)
with open(TABLE, 'r', encoding='utf-8') as f:
    for line in f:
        m = re.search(r'// \[(\d+)\]', line)
        if not m:
            continue
        sid = int(m.group(1))
        m2 = re.search(r'MagicProp::new\(\s*(\d+),\s*(\d+),\s*(-?\d+),\s*(\d+),\s*(\d+),\s*(\d+)', line)
        if m2:
            table_entries[sid] = (int(m2.group(1)), int(m2.group(4)), int(m2.group(3)), int(m2.group(5)), int(m2.group(6)))
        elif 'empty()' in line:
            table_entries[sid] = (0, 0, 0, 0, 0)

# ---- Find real mismatches (both Saved=1 AND different) ----
real_diffs = 0
table_only = 0

print("=== Stats where Mod's itemstatcost.txt says Saved=1 AND differs from table ===\n")
for sid in sorted(set(file_stats.keys()) | set(table_entries.keys())):
    f = file_stats.get(sid)
    t = table_entries.get(sid)

    if f is None:
        # Not in mod's file as Saved=1 - skip
        continue

    name, fsb, fspb, fsa, fsig, fenc = f

    if t is None:
        print(f"  ID={sid:>4} ({name:<25}): in file (sb={fsb} spb={fspb}) but MISSING from table!")
        real_diffs += 1
        continue

    tsb, tspb, tsa, tsig, tenc = t

    if (fsb, fspb, fsa, fsig, fenc) != (tsb, tspb, tsa, tsig, tenc):
        real_diffs += 1
        print(f"  ID={sid:>4} ({name:<25}):")
        print(f"       mod file: sb={fsb:>2} spb={fspb} sa={fsa:>4} signed={fsig} enc={fenc}")
        print(f"       table:    sb={tsb:>2} spb={tspb} sa={tsa:>4} signed={tsig} enc={tenc}")

if real_diffs == 0:
    print("  (none - all Saved=1 stats match!)\n")

# Count total Saved=1 stats in file
total_saved1 = len(file_stats)
print(f"\nTotal Saved=1 stats in mod file: {total_saved1}")
print(f"Total table entries: {len(table_entries)}")
print(f"Real differences: {real_diffs}")

# Also generate corrected table entries
if real_diffs > 0:
    print("\n\n=== Corrected entries (from mod file) ===\n")
    for sid in sorted(file_stats.keys()):
        name, sb, spb, sa, signed, enc = file_stats[sid]
        print(f"  MagicProp::new({sb:>2}, 1, {sa:>4}, {spb}, {signed}, {enc}), // [{sid}] {name}")
