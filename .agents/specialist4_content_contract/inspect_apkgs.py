import zipfile
import sqlite3
import json
import tempfile
import os

def inspect_apkg(path):
    print("==================================================")
    print("=== Inspecting:", path)
    print("==================================================")
    if not os.path.exists(path):
        print(f"File not found: {path}")
        return
    with zipfile.ZipFile(path, 'r') as zf:
        print("Files in zip:", zf.namelist())
        with tempfile.TemporaryDirectory() as td:
            zf.extract('collection.anki2', td)
            conn = sqlite3.connect(os.path.join(td, 'collection.anki2'))
            cur = conn.cursor()
            cur.execute('SELECT models, decks FROM col')
            models_json, decks_json = cur.fetchone()
            models = json.loads(models_json)
            decks = json.loads(decks_json)
            print("Decks:")
            for d in decks.values():
                print(f"  - [{d.get('id')}] {d.get('name')}")
            print("Note types:")
            for mid, m in models.items():
                field_names = [f['name'] for f in m.get('flds', [])]
                tmpl_names = [t['name'] for t in m.get('tmpls', [])]
                print(f"  - Model ID {mid}: '{m.get('name')}'")
                print(f"    Fields: {field_names}")
                print(f"    Templates: {tmpl_names}")
                for t in m.get('tmpls', []):
                    print(f"      Qfmt: {t.get('qfmt')}")
                    print(f"      Afmt: {t.get('afmt')}")
            cur.execute('SELECT count(*) FROM cards')
            cards_count = cur.fetchone()[0]
            print(f"Cards count: {cards_count}")
            cur.execute('SELECT id, mid, flds, tags FROM notes')
            notes = cur.fetchall()
            print(f"Notes count: {len(notes)}")
            for idx, (nid, mid, flds, tags) in enumerate(notes):
                print(f"  Note #{idx+1} [nid={nid}, mid={mid}]:")
                print(f"    tags: {tags}")
                print(f"    flds preview: {flds[:160]}...")
            conn.close()

for p in ['Procedural_StudyLab_Fixture.apkg', 'Math_StudyLab_Demo.apkg', 'StudyLab_Phase0_Output.apkg']:
    inspect_apkg(p)
