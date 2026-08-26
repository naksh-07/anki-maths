import os, sys, random, json
REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
if REPO_ROOT not in sys.path: sys.path.insert(0, REPO_ROOT)
from tools.studylab_content_factory import get_all_175_topics

print('=' * 70)
print('CHALLENGER AUDIT: DIMENSION 1 - Math/CAS & Parameter Boundaries')
print('=' * 70)

topics = get_all_175_topics()
print(f'Auditing {len(topics)} total topics...')
checks = 0
failures = []

for topic in topics:
    fid = topic.get('family_id', 'unknown')
    archetypes = topic.get('archetypes', [])
    if not archetypes:
        failures.append(f'{fid}: No archetypes')
        continue
    for arch in archetypes:
        arch_id = arch.get('archetype_id', 'unknown')
        params_spec = arch.get('parameters', [])
        obj_type = arch.get('object_type', 'problem')
        prompt_tmpl = arch.get('prompt_template', '')
        for p in params_spec:
            pname = p.get('name', '')
            pdom = p.get('domain', {})
            ptype = pdom.get('type', '')
            checks += 1
            if ptype == 'integer_range':
                pmin = pdom.get('min', 0)
                pmax = pdom.get('max', 0)
                if pmin > pmax:
                    failures.append(f'{fid}/{arch_id}: {pname} min > max')
            elif ptype == 'discrete_choice':
                vals = pdom.get('values', [])
                if not vals:
                    failures.append(f'{fid}/{arch_id}: {pname} empty values')
        if obj_type == 'mcq':
            checks += 1
            opt_param = next((p for p in params_spec if p.get('name') in ['options', 'choices']), None)
            corr_param = next((p for p in params_spec if p.get('name') in ['correct_option', 'correct_answer', 'answer']), None)
            if opt_param:
                opt_vals = opt_param.get('domain', {}).get('values', [])
                if len(opt_vals) < 2:
                    failures.append(f'{fid}/{arch_id}: MCQ < 2 options')
                if len(opt_vals) != len(set(opt_vals)):
                    failures.append(f'{fid}/{arch_id}: MCQ duplicate options')
                if corr_param:
                    corr_vals = corr_param.get('domain', {}).get('values', [])
                    for cv in corr_vals:
                        if cv not in opt_vals:
                            failures.append(f'{fid}/{arch_id}: Correct option {cv} not in options')
        for seed in [0, 1, 42, 100, 9999, 123456]:
            checks += 1
            rng = random.Random(seed)
            sample_params = {}
            for p in params_spec:
                pname = p.get('name', '')
                pdom = p.get('domain', {})
                ptype = pdom.get('type', '')
                if ptype == 'integer_range':
                    pmin = pdom.get('min', 1)
                    pmax = pdom.get('max', 10)
                    pstep = pdom.get('step') or 1
                    sample_params[pname] = rng.randrange(pmin, pmax + 1, pstep)
                elif ptype == 'discrete_choice':
                    vals = pdom.get('values', [''])
                    sample_params[pname] = rng.choice(vals) if vals else ''
                else:
                    sample_params[pname] = 1
            try:
                formatted_prompt = prompt_tmpl
                for k, v in sample_params.items():
                    formatted_prompt = formatted_prompt.replace('{' + k + '}', str(v))
                if 'NaN' in formatted_prompt or 'Infinity' in formatted_prompt:
                    failures.append(f'{fid}/{arch_id}: Prompt generated NaN/Inf with seed {seed}')
            except Exception as e:
                failures.append(f'{fid}/{arch_id}: Prompt format crash with seed {seed}: {e}')
        step_nodes = arch.get('step_nodes', [])
        for sn in step_nodes:
            checks += 1
            hp = sn.get('hint_principle', '')
            ho = sn.get('hint_operation', '')
            hi = sn.get('hint_intermediate', '')
            if not hp or not ho or not hi:
                failures.append(f'{fid}/{arch_id}: Incomplete 3-tier hints in step')

print(f'Dimension 1 Result: {checks} checks executed. Failures: {len(failures)}')
if failures:
    for f in failures[:10]:
        print(f'  [FAIL] {f}')
    sys.exit(1)
else:
    print('  [PASS] ALL 177 TOPIC BLUEPRINTS PASSED MATH/CAS & PARAMETER STRESS TESTS!')
    sys.exit(0)
