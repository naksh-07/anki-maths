import sqlite3
import json

dst = r"C:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\col_copy.anki2"
conn = sqlite3.connect(dst)
cur = conn.cursor()

cur.execute("SELECT name FROM sqlite_master WHERE type='table'")
tables = cur.fetchall()
print("Tables:", [t[0] for t in tables])

print("\n=== NOTETYPES ===")
cur.execute("select id, name, config from notetypes")
for row in cur.fetchall():
    nt_id, name, config = row
    print(f"\nModel {nt_id}: '{name}'")
    # check fields for this notetype
    cur.execute("select ord, name from fields where ntid = ? order by ord", (nt_id,))
    fields = cur.fetchall()
    print("  Fields:", fields)
    cur.execute("select ord, name, config from templates where ntid = ? order by ord", (nt_id,))
    templates = cur.fetchall()
    for t in templates:
        print(f"  Template {t[0]}: '{t[1]}'")

print("\n=== SAMPLE NOTES ===")
cur.execute("select id, mid, flds from notes limit 10")
for row in cur.fetchall():
    nid, mid, flds = row
    print(f"\nNote {nid} (Model {mid}):")
    fld_list = flds.split("\x1f")
    for i, f in enumerate(fld_list):
        print(f"  Field {i}: {f[:120]}")

conn.close()
