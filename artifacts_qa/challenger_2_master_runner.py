#!/usr/bin/env python3
import os, sys, subprocess, json, time

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))

tests = [
    ('DIMENSION_1_MATH_CAS', 'artifacts_qa/challenger_test_dim1_math_cas.py'),
    ('DIMENSION_2_DB_PERSISTENCE', 'artifacts_qa/challenger_test_dim2_db_persistence.py'),
    ('DIMENSION_3_TELEMETRY_100BYTE', 'artifacts_qa/challenger_test_dim3_telemetry_100byte.py'),
    ('DIMENSION_4_COLD_START_APKG', 'artifacts_qa/challenger_test_dim4_cold_start_apkg.py'),
]

report = {
    'audit_timestamp': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
    'auditor': 'Challenger 2 (Backend & Database Adversarial Verifier)',
    'verdict': 'APPROVE',
    'dimensions': {}
}

all_passed = True
print('=' * 80)
print('CHALLENGER 2: RELEASE CANDIDATE ADVERSARIAL MASTER AUDIT')
print('=' * 80)

for name, script in tests:
    script_path = os.path.join(REPO_ROOT, script)
    t0 = time.time()
    res = subprocess.run([sys.executable, script_path], capture_output=True, text=True)
    dt = time.time() - t0
    passed = (res.returncode == 0)
    if not passed:
        all_passed = False
    report['dimensions'][name] = {
        'passed': passed,
        'duration_seconds': round(dt, 3),
        'stdout': res.stdout.strip(),
        'stderr': res.stderr.strip()
    }
    status = '[PASS]' if passed else '[FAIL]'
    print(f'  {status} {name} ({dt:.2f}s)')
    if not passed:
        print(f'     Error: {res.stderr.strip()}')

final_verdict = 'APPROVE' if all_passed else 'REJECT'
report['verdict'] = final_verdict
out_json = os.path.join(REPO_ROOT, 'artifacts_qa', 'challenger_2_audit_report.json')
with open(out_json, 'w', encoding='utf-8') as f:
    json.dump(report, f, indent=2)

print('=' * 80)
print(f'FINAL CHALLENGER 2 VERDICT: {final_verdict}')
print(f'Audit report written to: {out_json}')
print('=' * 80)
sys.exit(0 if all_passed else 1)
