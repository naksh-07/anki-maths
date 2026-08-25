import sqlite3

dst = r"C:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\col_copy.anki2"
conn = sqlite3.connect(dst)
cur = conn.cursor()

# Find cards with did = 1787659104777
cur.execute("select id, nid, did, ord, type, queue, due from cards where queue = 0 or queue = 1 or queue = 2 order by queue desc, due asc limit 5")
print("Top due cards in collection:")
for r in cur.fetchall():
    cid, nid, did, cord, ctype, queue, due = r
    cur.execute("select mid, flds from notes where id = ?", (nid,))
    nrow = cur.fetchone()
    cur.execute("select name from notetypes where id = ?", (nrow[0],))
    nt_name = cur.fetchone()[0]
    print(f"\nCard {cid} in Deck {did} (Queue {queue}, Due {due}, Note {nid}, Model '{nt_name}'):")
    flds = nrow[1].split("\x1f")
    for i, f in enumerate(flds):
        print(f"  Fld {i}: {f[:80]}")

conn.close()
