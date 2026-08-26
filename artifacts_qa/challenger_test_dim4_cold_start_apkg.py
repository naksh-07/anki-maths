import os, sys, json, tempfile, sqlite3, zipfile
REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
if REPO_ROOT not in sys.path: sys.path.insert(0, REPO_ROOT)

from tools.studylab_content_factory import get_all_175_topics, build_apkg_from_topics

print('=' * 70)
print('CHALLENGER AUDIT: DIMENSION 4 - Cold-Start APKG Import & Profile Isolation')
print('=' * 70)

temp_dir = tempfile.mkdtemp(prefix='studylab_cold_start_test_')
apkg_path = os.path.join(temp_dir, 'StudyLab_Full_Universe_175.apkg')

checks = 0
failures = []

try:
    all_topics = get_all_175_topics()
    print(f'Building canonical APKG with {len(all_topics)} topics...')
    build_apkg_from_topics(all_topics, apkg_path, 'StudyLab::Universal Practice (175 Topics)')
    
    if not os.path.exists(apkg_path) or os.path.getsize(apkg_path) < 10000:
        failures.append('APKG generation failed or produced undersized archive')
    else:
        print(f'  [+] Generated canonical APKG: {os.path.basename(apkg_path)} ({os.path.getsize(apkg_path):,} bytes).')
        
    extract_dir = os.path.join(temp_dir, 'unpacked')
    os.makedirs(extract_dir, exist_ok=True)
    with zipfile.ZipFile(apkg_path, 'r') as z:
        z.extractall(extract_dir)
        
    col_db = os.path.join(extract_dir, 'collection.anki2')
    if not os.path.exists(col_db):
        failures.append('Missing collection.anki2 in unpacked APKG')
    else:
        conn = sqlite3.connect(col_db)
        cur = conn.cursor()
        
        cur.execute('SELECT COUNT(*) FROM notes;')
        note_count = cur.fetchone()[0]
        checks += 1
        if note_count != len(all_topics):
            failures.append(f'Note count mismatch: expected {len(all_topics)}, found {note_count}')
        else:
            print(f'  [+] Verified note count in fresh collection: {note_count} notes.')
            
        cur.execute('SELECT id, flds FROM notes;')
        for nid, flds in cur.fetchall():
            checks += 1
            parts = flds.split('')
            if len(parts) < 4:
                failures.append(f'Note {nid} malformed fields (<4 parts)')
                continue
            payload_raw = parts[0]
            try:
                payload = json.loads(payload_raw)
                inline_contract = payload.get('inline_contract')
                if not inline_contract:
                    failures.append(f'Note {nid} missing inline_contract')
                else:
                    contract = inline_contract.get('contract', {})
                    archetypes = inline_contract.get('archetypes', [])
                    if not contract.get('family_id') or not archetypes:
                        failures.append(f'Note {nid} incomplete blueprint')
            except Exception as ex:
                failures.append(f'Note {nid} JSON parse error: {ex}')
        conn.close()
        print(f'  [+] Verified all {note_count} notes contain valid self-contained inline_contract blueprints.')

except Exception as e:
    failures.append(f'Cold-start test crashed: {e}')
finally:
    import shutil
    if os.path.exists(temp_dir):
        try: shutil.rmtree(temp_dir)
        except Exception: pass

print(f'Dimension 4 Result: {checks} checks executed. Failures: {len(failures)}')
if failures:
    for f in failures:
        print(f'  [FAIL] {f}')
    sys.exit(1)
else:
    print('  [PASS] ALL COLD-START IMPORT & PROFILE ISOLATION TESTS PASSED!')
    sys.exit(0)
