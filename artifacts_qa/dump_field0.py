import sqlite3
import json

dst = r"C:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\col_copy.anki2"
conn = sqlite3.connect(dst)
cur = conn.cursor()

cur.execute("select flds from notes where id = 1787659067010")
row = cur.fetchone()
fld0 = row[0].split("\x1f")[0]
print("Field 0 full string:")
print(fld0)
print("\nJSON parse test in Python:")
try:
    data = json.loads(fld0)
    print("Valid JSON! Keys:", list(data.keys()))
    print("proc_schema:", data.get("proc_schema"))
    print("inline_contract keys:", list(data.get("inline_contract", {}).keys()) if data.get("inline_contract") else None)
except Exception as e:
    print("Invalid JSON:", e)

conn.close()
