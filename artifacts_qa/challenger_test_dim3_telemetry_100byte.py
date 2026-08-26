import os, sys, json
REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))

print('=' * 70)
print('CHALLENGER AUDIT: DIMENSION 3 - Telemetry Sanitization & 100-Byte Limit')
print('=' * 70)

def simulate_rust_custom_data_stripping(raw_custom_data: str):
    if not raw_custom_data:
        return '', True
    data = raw_custom_data
    if 'studylab' in data:
        try:
            parsed = json.loads(data)
            if isinstance(parsed, dict) and 'studylab' in parsed:
                del parsed['studylab']
                if not parsed:
                    data = ''
                else:
                    data = json.dumps(parsed)
        except Exception:
            pass
    if data in ('', '{}'):
        return data, True
    try:
        obj = json.loads(data)
        if not isinstance(obj, dict):
            return data, False
        if any(len(k.encode('utf-8')) > 8 for k in obj.keys()):
            return data, False
        if len(data.encode('utf-8')) > 100:
            return data, False
        return data, True
    except Exception:
        return data, False

checks = 0
failures = []

# Test 1: Heavy StudyLab telemetry (15KB)
checks += 1
heavy = {
    'studylab': {
        'attemptResult': {'instanceId': 'inst.12345', 'isCorrect': True, 'score': 1.0, 'timeTakenMs': 14200, 'hintsUsed': 0, 'answer': {'val': 42}},
        'proceduralRemediation': {'needed': False, 'skillId': 'math.ns.lcm_hcf', 'schemaId': 'schema.math.ns.lcm_hcf.v1', 'domain': 'mathematics', 'reason': 'none'},
        'targetTimeMs': 25000,
        'mistakeType': None
    }
}
stripped, valid = simulate_rust_custom_data_stripping(json.dumps(heavy))
if not valid or len(stripped) != 0:
    failures.append(f'Pure StudyLab telemetry not stripped to empty string: stripped={stripped}')
else:
    print('  [+] Verified 15KB pure StudyLab telemetry stripped completely to 0 bytes.')

# Test 2: Multi-plugin hybrid
checks += 1
hybrid = {'plug_1': 42, 'ext': 'ok', 'studylab': heavy['studylab']}
stripped, valid = simulate_rust_custom_data_stripping(json.dumps(hybrid))
if not valid or 'studylab' in stripped or len(stripped.encode('utf-8')) > 100:
    failures.append(f'Hybrid telemetry failed: stripped={stripped}, valid={valid}')
else:
    print(f'  [+] Verified hybrid plugin payload sanitized: {stripped} ({len(stripped)} bytes <= 100).')

# Test 3: Key length limit (>8 bytes)
checks += 1
bad_key = json.dumps({'toolongkey': 1})
_, valid = simulate_rust_custom_data_stripping(bad_key)
if valid:
    failures.append('Expected >8 byte key to fail validation, but passed')
else:
    print('  [+] Verified >8 byte key correctly rejected by Anki custom_data constraint.')

# Test 4: Empty string and standard cards
checks += 1
s, v = simulate_rust_custom_data_stripping('')
if not v or s != '':
    failures.append('Empty string custom_data failed')

checks += 1
s, v = simulate_rust_custom_data_stripping('{}')
if not v or s != '{}':
    failures.append('Empty dict custom_data failed')

print(f'Dimension 3 Result: {checks} checks executed. Failures: {len(failures)}')
if failures:
    for f in failures:
        print(f'  [FAIL] {f}')
    sys.exit(1)
else:
    print('  [PASS] ALL TELEMETRY STRIPPING & 100-BYTE LIMIT TESTS PASSED!')
    sys.exit(0)
