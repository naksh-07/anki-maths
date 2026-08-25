import sqlite3

dst = r"C:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\col_copy.anki2"
conn = sqlite3.connect(dst)
cur = conn.cursor()

print("=== DECKS ===")
cur.execute("select id, name from decks")
for r in cur.fetchall():
    print(r)

print("\n=== CARDS IN DECK ===")
cur.execute("select id, nid, did, ord, type, queue, due from cards where did = 1787659104777 or did in (select id from decks where name like '%studylab%') limit 10")
for r in cur.fetchall():
    cid, nid, did, cord, ctype, queue, due = r
    cur.execute("select mid, flds from notes where id = ?", (nid,))
    nrow = cur.fetchone()
    cur.execute("select name from notetypes where id = ?", (nrow[0],))
    nt_name = cur.fetchone()[0]
    print(f"Card {cid} (Note {nid}, Model: '{nt_name}'):")
    flds = nrow[1].split("\x1f")
    for i, f in enumerate(flds):
        print(f"  Field {i}: {f[:80]}")

conn.close()
