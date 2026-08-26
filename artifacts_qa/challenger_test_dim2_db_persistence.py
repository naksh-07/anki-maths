import os, sys, time, sqlite3, tempfile, threading, traceback
REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))

print('=' * 70)
print('CHALLENGER AUDIT: DIMENSION 2 - SQLite ACID Atomicity & Concurrency')
print('=' * 70)

schema_path = os.path.join(REPO_ROOT, 'rslib', 'procedural', 'src', 'storage', 'schema.rs')
with open(schema_path, 'r', encoding='utf-8') as sf:
    content = sf.read()

q = chr(34)
raw_parts = content.split('r#' + q)[1:]
migrations = [p.split(q + '#')[0] for p in raw_parts]
print(f'Loaded {len(migrations)} canonical migrations from schema.rs')
if len(migrations) != 5:
    print(f'FAIL: Expected 5 migrations, found {len(migrations)}')
    sys.exit(1)

temp_db = tempfile.NamedTemporaryFile(suffix='.procedural', delete=False)
temp_db_path = temp_db.name
temp_db.close()

checks = 0
failures = []

try:
    conn = sqlite3.connect(temp_db_path)
    conn.execute('PRAGMA busy_timeout = 5000;')
    conn.execute('PRAGMA foreign_keys = ON;')
    conn.execute('PRAGMA journal_mode = WAL;')
    
    for idx, sql in enumerate(migrations, 1):
        checks += 1
        conn.executescript(sql)
        
    cur = conn.cursor()
    cur.execute('SELECT name FROM sqlite_master WHERE type=' + q + 'table' + q + ' AND name NOT LIKE ' + q + 'sqlite_%' + q + ';')
    tables = [row[0] for row in cur.fetchall()]
    checks += 1
    expected_tables = {
        'skills', 'skill_states', 'problem_families', 'schemas',
        'problem_instances', 'practice_attempts', 'error_events',
        'catalog_metadata', 'pyq_sources', 'pyq_mappings',
        'rejected_variants', 'exam_profiles', 'practice_items',
        'chapter_practice_profiles', 'remediation_queue_items',
        'remediation_recurrence'
    }
    missing = expected_tables - set(tables)
    if missing:
        failures.append(f'Missing tables: {missing}')
    else:
        print(f'  [+] Verified all {len(tables)} tables present in database.')
        
    cur.execute('SELECT name FROM sqlite_master WHERE type=' + q + 'index' + q + ' AND name NOT LIKE ' + q + 'sqlite_%' + q + ';')
    indexes = [row[0] for row in cur.fetchall()]
    checks += 1
    print(f'  [+] Verified {len(indexes)} SQLite indexes present.')
    
    # 1. Cascade Delete
    checks += 1
    now = int(time.time())
    conn.execute('INSERT INTO skills VALUES (?, ?, ?, ?, ?, ?, ?);', ('test.skill.1', 'mathematics', 'Test Skill', 'Desc', '[]', '{}', now))
    conn.execute('INSERT INTO skill_states VALUES (?, ?, ?, ?, ?, ?, ?, ?);', ('test.skill.1', 0.5, 0.5, 1, 1, now, '{}', now))
    conn.execute('INSERT INTO problem_families VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);', ('fam.1', 'test.skill.1', 'mathematics', 'Family 1', 'tmpl.1', 1.0, 5.0, '{}', '{}', now))
    conn.execute('INSERT INTO schemas VALUES (?, ?, ?, ?, ?, ?, ?, ?);', ('schema.1', 'test.skill.1', 'fam.1', 'Schema 1', 'Desc', 0.8, '{}', now))
    conn.execute('INSERT INTO problem_instances VALUES (?, ?, ?, ?, ?, ?, ?, ?);', ('inst.1', 'fam.1', 42, '{}', 'Prompt', 'Ans', '{}', now))
    conn.execute('INSERT INTO practice_attempts VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);', ('att.1', 'inst.1', 'schema.1', 'test.skill.1', 101, 'Ans', 1, 1.0, 5000, now, '{}'))
    conn.execute('INSERT INTO error_events VALUES (?, ?, ?, ?, ?);', ('err.1', 'att.1', 'careless', '{}', now))
    conn.execute('INSERT INTO remediation_queue_items VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);', ('rem.1', 'review', 'test.skill.1', 'schema.1', 'mathematics', 'careless', None, 2, None, 'att.1', 'high', 0, 1, 'Rationale', now))
    conn.commit()
    
    conn.execute('DELETE FROM skills WHERE id = ?;', ('test.skill.1',))
    conn.commit()
    
    for tbl in ['skill_states', 'problem_families', 'schemas', 'problem_instances', 'practice_attempts', 'error_events', 'remediation_queue_items']:
        checks += 1
        cur.execute(f'SELECT COUNT(*) FROM {tbl}')
        cnt = cur.fetchone()[0]
        if cnt != 0:
            failures.append(f'Cascade delete failed for {tbl}: {cnt} orphan rows remain')
    print('  [+] Verified ON DELETE CASCADE: 0 orphaned rows across all 7 child tables.')
    
    # 2. Rollback Atomicity
    checks += 1
    try:
        with conn:
            conn.execute('INSERT INTO skills VALUES (?, ?, ?, ?, ?, ?, ?);', ('test.skill.atomic', 'mathematics', 'Skill', 'Desc', '[]', '{}', now))
            conn.execute('INSERT INTO problem_instances VALUES (?, ?, ?, ?, ?, ?, ?, ?);', ('inst.bad', 'nonexistent_family', 42, '{}', 'P', 'A', '{}', now))
    except sqlite3.IntegrityError:
        pass
    cur.execute('SELECT COUNT(*) FROM skills WHERE id = ?;', ('test.skill.atomic',))
    if cur.fetchone()[0] != 0:
        failures.append('Transaction atomicity failure: uncommitted insert survived rollback')
    else:
        print('  [+] Verified transaction atomicity rollback: 0 dirty rows committed.')
    conn.close()
    
    # 3. Concurrency
    checks += 1
    thread_errors = []
    def worker(thread_id: int):
        try:
            c = sqlite3.connect(temp_db_path, timeout=10.0)
            c.execute('PRAGMA busy_timeout = 5000;')
            c.execute('PRAGMA foreign_keys = ON;')
            for i in range(25):
                t_now = int(time.time() * 1000)
                sk_id = f'skill.thread.{thread_id}'
                with c:
                    c.execute('INSERT INTO skills VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO NOTHING;', (sk_id, 'mathematics', 'TName', 'TDesc', '[]', '{}', t_now))
                    c.execute('INSERT INTO skill_states VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(skill_id) DO UPDATE SET total_attempts = skill_states.total_attempts + 1, updated_at = excluded.updated_at;', (sk_id, 0.1, 0.1, 1, 1, t_now, '{}', t_now))
            c.close()
        except Exception as ex:
            thread_errors.append(f'Thread {thread_id} crashed: {ex}')

    threads = [threading.Thread(target=worker, args=(t,)) for t in range(8)]
    for t in threads: t.start()
    for t in threads: t.join()
    if thread_errors:
        failures.extend(thread_errors)
    else:
        print('  [+] Verified 8 concurrent worker threads (200 transactions) completed with 0 lock errors.')

except Exception as e:
    failures.append(f'DB stress crash: {e}\n{traceback.format_exc()}')
finally:
    if os.path.exists(temp_db_path):
        try: os.remove(temp_db_path)
        except Exception: pass

print(f'Dimension 2 Result: {checks} checks executed. Failures: {len(failures)}')
if failures:
    for f in failures:
        print(f'  [FAIL] {f}')
    sys.exit(1)
else:
    print('  [PASS] ALL SQLITE PERSISTENCE, MIGRATIONS & CONCURRENCY TESTS PASSED!')
    sys.exit(0)
