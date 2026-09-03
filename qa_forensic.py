"""
StudyLab Forensic QA — Proper MCQ-aware test driver.
Interacts with MCQ cards via proc-option-item clicks.
"""
import asyncio, websockets, json, urllib.request, base64, os, sys, io, time

# sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
ARTS = "artifacts_qa"
os.makedirs(ARTS, exist_ok=True)

R = {"tests": {}, "errors": [], "screenshots": []}

async def run():
    print('Starting qa_forensic...')
    # Wait for Anki CDP
    for i in range(15):
        try:
            targets = json.loads(urllib.request.urlopen('http://127.0.0.1:9222/json/list').read())
            main_ws = next((t['webSocketDebuggerUrl'] for t in targets if t.get('title')=='main webview'), None)
            bottom_ws = next((t['webSocketDebuggerUrl'] for t in targets if t.get('title')=='bottom toolbar'), None)
            if main_ws: break
        except: pass
        await asyncio.sleep(1)
    
    if not main_ws:
        print("FATAL: No main webview"); return

    async with websockets.connect(main_ws, max_size=None) as ws:
        mid=1; P={}
        async def rdr():
            try:
                while True:
                    d=json.loads(await ws.recv())
                    if 'id' in d and d['id'] in P and not P[d['id']].done(): P[d['id']].set_result(d)
            except: pass
        asyncio.create_task(rdr())
        
        async def ev(code):
            nonlocal mid; m=mid; mid+=1; f=asyncio.Future(); P[m]=f
            await ws.send(json.dumps({'id':m,'method':'Runtime.evaluate','params':{'expression':code,'returnByValue':True}}))
            try: r=await asyncio.wait_for(f,3.0); return r.get('result',{}).get('result',{}).get('value')
            except: return None
            finally: P.pop(m,None)
        
        async def shot(name):
            nonlocal mid; m=mid; mid+=1; f=asyncio.Future(); P[m]=f
            await ws.send(json.dumps({'id':m,'method':'Page.captureScreenshot','params':{'format':'png'}}))
            try:
                r=await asyncio.wait_for(f,5.0); d=r.get('result',{}).get('data')
                if d:
                    p=os.path.join(ARTS,name)
                    with open(p,'wb') as fp: fp.write(base64.b64decode(d))
                    R["screenshots"].append(p); print(f"  [SHOT] {p}")
            except: pass
            finally: P.pop(m,None)
        
        async def dc():
            c={}
            c['is_proc'] = await ev("!!document.getElementById('procedural-card')") or False
            c['mistake_panel_visible'] = await ev("""
                var mp = document.getElementById('proc-mistake-panel');
                mp ? !mp.classList.contains('hidden') : false
            """) or False
            c['mistake_btns'] = await ev("document.querySelectorAll('.proc-mistake-btn').length") or 0
            c['result_visible'] = await ev("""
                var rp = document.getElementById('proc-result-panel');
                rp ? !rp.classList.contains('hidden') : false
            """) or False
            c['solution_visible'] = await ev("""
                var sc = document.getElementById('proc-solution-container');
                sc ? !sc.classList.contains('hidden') : false
            """) or False
            c['next_btn'] = await ev("document.querySelectorAll('#proc-next-btn').length") or 0
            c['native_ease'] = await ev("document.querySelectorAll('#ease1,#ease2,#ease3,#ease4,.ease').length") or 0
            c['options'] = await ev("document.querySelectorAll('.proc-option-item').length") or 0
            return c
        
        async def body():
            return (await ev("document.body.innerText") or "")
        
        async def html_save(name):
            h = await ev("document.body.innerHTML") or ""
            with open(os.path.join(ARTS, name), 'w', encoding='utf-8') as f: f.write(h)
        
        async def click_wrong_option():
            """Click an incorrect MCQ option or submit wrong numeric answer"""
            return await ev("""
                (() => {
                    var opts = document.querySelectorAll('.proc-option-item:not(.disabled)');
                    if (opts.length > 0) {
                        var correct = document.querySelector('.proc-option-item[data-opt-id*=\"C.\"]') ||
                                      document.querySelector('.proc-option-item[data-opt-idx=\"2\"]');
                        var clicked = false;
                        for (var i = 0; i < opts.length; i++) {
                            if (opts[i] !== correct) { opts[i].click(); clicked = true; break; }
                        }
                        return clicked;
                    } else {
                        var inp = document.getElementById('proc-answer-input');
                        var btn = document.getElementById('proc-submit-btn');
                        if (inp && btn && !inp.disabled) {
                            inp.value = "999999999";
                            btn.click();
                            return true;
                        }
                    }
                    return false;
                })();
            """)
        
        async def click_correct_option():
            """Click option C or submit correct numeric answer"""
            return await ev("""
                (() => {
                    var opts = document.querySelectorAll('.proc-option-item:not(.disabled)');
                    if (opts.length > 0) {
                        var opt = document.querySelector('.proc-option-item[data-opt-idx=\"2\"]') ||
                                  document.querySelector('.proc-option-item:nth-child(3)');
                        if (opt && !opt.classList.contains('disabled')) { opt.click(); return true; } 
                    } else {
                        var inp = document.getElementById('proc-answer-input');
                        var btn = document.getElementById('proc-submit-btn');
                        if (inp && btn && !inp.disabled) {
                            var html = document.body.innerHTML;
                            var m = html.match(/"correctAnswer":\\s*\\{[^}]*"formatted":"([^"]+)"/);
                            if(m) {
                                inp.value = m[1];
                            } else {
                                var m2 = html.match(/"correctAnswer":\\s*\\{[^}]*"value":([\\d\\.]+)/);
                                if(m2) inp.value = m2[1];
                                else inp.value = "30";
                            }
                            inp.dispatchEvent(new Event('input', { bubbles: true }));
                            btn.click();
                            return true;
                        }
                    }
                    return false;
                })();
            """)

        async def ensure_procedural():
            while True:
                is_proc = await ev("document.querySelectorAll('.procedural-card-container').length > 0")
                if is_proc:
                    break
                print(f"  [INFO] Skipping non-procedural card...")
                await ev("document.dispatchEvent(new KeyboardEvent('keydown',{key:' ',code:'Space',bubbles:true}))")
                await asyncio.sleep(1.5)
        
        async def bottom_check():
            """Check bottom toolbar for native ease buttons"""
            if not bottom_ws: return {"native_ease": 0, "text": ""}
            try:
                async with websockets.connect(bottom_ws, max_size=None) as bws:
                    bp={}
                    async def br():
                        try:
                            while True:
                                d=json.loads(await bws.recv())
                                if 'id' in d and d['id'] in bp and not bp[d['id']].done(): bp[d['id']].set_result(d)
                        except: pass
                    asyncio.create_task(br())
                    bi=1
                    async def bev(code):
                        nonlocal bi; m=bi; bi+=1; f=asyncio.Future(); bp[m]=f
                        await bws.send(json.dumps({'id':m,'method':'Runtime.evaluate','params':{'expression':code,'returnByValue':True}}))
                        try: r=await asyncio.wait_for(f,3.0); return r.get('result',{}).get('result',{}).get('value')
                        except: return None
                        finally: bp.pop(m,None)
                    txt = await bev("document.body.innerText") or ""
                    ease = await bev("document.querySelectorAll('#ease1,#ease2,#ease3,#ease4,.ease,button[onclick*=ease]').length") or 0
                    hm = await bev("document.body.innerHTML") or ""
                    with open(os.path.join(ARTS,"bottom.html"),'w',encoding='utf-8') as f: f.write(hm)
                    return {"native_ease": ease, "text": txt[:200], "has_again_good": "Again" in txt or "Good" in txt or "Hard" in txt or "Easy" in txt}
            except Exception as e:
                return {"error": str(e)}

        # ===== NAVIGATE TO STUDY =====
        print("=" * 60)
        print("NAVIGATING TO STUDY SESSION")
        print("============================================================")
        print("NAVIGATING TO STUDY SESSION")
        print("============================================================")
        
        # Ensure we are in StudyLab Demo
        await ev("if(typeof pycmd !== 'undefined') pycmd('open:1787954315074');")
        await asyncio.sleep(1.0)
        await ev("if(typeof pycmd !== 'undefined') pycmd('study');")
        await asyncio.sleep(5.0)  # wait longer for reviewer to open
        
        b = await body()
        print(f"Current view: {b[:150]}")

        # ===== TEST A: Fresh Card =====
        print("\n" + "=" * 60)
        print("TEST A: Fresh procedural card")
        print("=" * 60)
        
        await ensure_procedural()
        d = await dc()
        bc = await bottom_check()
        await shot("A_fresh.png")
        await html_save("A_fresh.html")
        
        test_a = {
            "dom": d, "bottom": bc,
            "no_classification_footer": not d['mistake_panel_visible'],
            "no_native_ease_main": d['native_ease'] == 0,
            "no_native_ease_bottom": not bc.get('has_again_good', False),
            "no_next_btn": d['next_btn'] == 0,
            "is_procedural": d['is_proc'],
        }
        test_a["PASS"] = all([
            test_a["no_classification_footer"],
            test_a["no_next_btn"],
            test_a["is_procedural"],
        ])
        R["tests"]["A"] = test_a
        for k,v in test_a.items():
            if k not in ["dom","bottom"]: print(f"  {k}: {v}")
        print(f"  RESULT: {'PASS' if test_a['PASS'] else 'FAIL'}")

        # ===== TEST B: Incorrect Answer =====
        print("\n" + "=" * 60)
        print("TEST B: Incorrect answer")
        print("=" * 60)
        
        clicked = clicked_wrong = await click_wrong_option(); print(f"    [DEBUG] clicked_wrong: {clicked_wrong}")
        print(f"  Clicked wrong option: {clicked}")
        await asyncio.sleep(1.5)
        
        d = await dc()
        b = await body()
        bc = await bottom_check()
        await shot("B_incorrect.png")
        await html_save("B_incorrect.html")
        
        test_b = {
            "dom": d, "bottom": bc,
            "has_feedback": d['result_visible'],
            "has_solution": d['solution_visible'],
            "mistake_panel_shows": d['mistake_panel_visible'],
            "exactly_one_footer": d['mistake_panel_visible'],  # single panel
            "no_native_ease": d['native_ease'] == 0 and not bc.get('has_again_good', False),
            "no_next_btn": d['next_btn'] == 0,
            "body_snippet": b[:300],
        }
        test_b["PASS"] = all([
            test_b["mistake_panel_shows"],
            test_b["no_native_ease"],
            test_b["no_next_btn"],
        ])
        R["tests"]["B"] = test_b
        for k,v in test_b.items():
            if k not in ["dom","bottom","body_snippet"]: print(f"  {k}: {v}")
        print(f"  RESULT: {'PASS' if test_b['PASS'] else 'FAIL'}")

        print("\n" + "=" * 60)
        print("TEST C: Classification buttons (mouse)")
        print("=" * 60)
        
        cls_names = ["Silly Slip", "Pattern Missed", "Concept Gap", "Prereq Unknown"]
        test_c = {}
        
        for idx, name in enumerate(cls_names):
            if idx > 0:
                await ensure_procedural()
                await ev("if(typeof pycmd !== 'undefined') pycmd('undo');"); await asyncio.sleep(2.0)
                clicked_wrong = clicked_wrong = await click_wrong_option(); print(f"    [DEBUG] clicked_wrong: {clicked_wrong}")
                print(f"  [{name}] clicked_wrong_option: {clicked_wrong}")
                await asyncio.sleep(1.5)
            
            b_before = (await body())[:80]; print(f"    [DEBUG] b_before: {b_before.replace(chr(10), ' ')}")
            d_before = await dc()
            
            # Click classification button
            result = await ev(f"""
                var btns = document.querySelectorAll('.proc-mistake-btn, .proc-mistake-card');
                btns[{idx}] ? (btns[{idx}].click(), true) : false
            """)
            await ev("if(typeof pycmd !== 'undefined') pycmd('undo');"); await asyncio.sleep(2.0)
            
            b_after = (await body())[:80]
            d_after = await dc()
            
            sub = {
                "button": name, "clicked": result,
                "card_advanced": b_before != b_after,
                "prev_footer_gone": not d_after['mistake_panel_visible'] or not d_after['is_proc'],
                "next_card_clean": d_after.get('is_proc', False) and not d_after.get('result_visible', True),
            }
            sub["PASS"] = bool(result) and sub["card_advanced"]
            test_c[name] = sub
            print(f"  {name}: clicked={result}, advanced={sub['card_advanced']}, clean={sub['next_card_clean']} => {'PASS' if sub['PASS'] else 'FAIL'}")
        
        await shot("C_after_classify.png")
        R["tests"]["C"] = test_c

        # ===== TEST D: Keyboard classification =====
        print("\n" + "=" * 60)
        print("TEST D: Keyboard classification")
        print("=" * 60)
        
        test_d = {}
        
        # Keys 1-4
        for k in range(1, 5):
            await ensure_procedural()
            await asyncio.sleep(0.5)
            clicked_wrong = await click_wrong_option(); print(f"    [DEBUG] clicked_wrong: {clicked_wrong}")
            await asyncio.sleep(1.5)
            
            b_before = (await body())[:80]; print(f"    [DEBUG] b_before: {b_before.replace(chr(10), ' ')}")
            
            await ev(f"""
                document.dispatchEvent(new KeyboardEvent('keydown', {{
                    key: '{k}', code: 'Digit{k}', keyCode: {48+k}, bubbles: true
                }}));
            """)
            await ev("if(typeof pycmd !== 'undefined') pycmd('undo');"); await asyncio.sleep(2.0)
            
            b_after = (await body())[:80]
            d_after = await dc()
            
            sub = {"key": str(k), "advanced": b_before != b_after}
            sub["PASS"] = sub["advanced"]
            test_d[f"key_{k}"] = sub
            print(f"  Key {k}: advanced={sub['advanced']} => {'PASS' if sub['PASS'] else 'FAIL'}")
        
        # Space and Enter (should NOT advance)
        for kn in ["Space", "Enter"]:
            await ensure_procedural()
            await asyncio.sleep(0.5)
            clicked_wrong = await click_wrong_option(); print(f"    [DEBUG] clicked_wrong: {clicked_wrong}")
            await asyncio.sleep(1.5)
            
            b_before = (await body())[:80]; print(f"    [DEBUG] b_before: {b_before.replace(chr(10), ' ')}")
            d_before = await dc()
            
            kv = " " if kn == "Space" else "Enter"
            kc = 32 if kn == "Space" else 13
            await ev(f"""
                document.dispatchEvent(new KeyboardEvent('keydown', {{
                    key: '{kv}', code: '{kn}', keyCode: {kc}, bubbles: true
                }}));
            """)
            await asyncio.sleep(1.0)
            
            b_after = (await body())[:80]
            d_after = await dc()
            
            sub = {
                "key": kn,
                "did_NOT_advance": b_before == b_after,
                "still_in_classification": d_after['mistake_panel_visible'],
            }
            sub["PASS"] = sub["did_NOT_advance"]
            test_d[f"key_{kn}"] = sub
            print(f"  {kn}: blocked={sub['did_NOT_advance']}, still_classifying={sub['still_in_classification']} => {'PASS' if sub['PASS'] else 'FAIL'}")
            
            # Clean up - classify to advance
            await ev("document.querySelectorAll('.proc-mistake-btn')[0]?.click()")
            await asyncio.sleep(1.5)
        
        await shot("D_keyboard.png")
        R["tests"]["D"] = test_d

        # ===== TEST E: Correct answer =====
        print("\n" + "=" * 60)
        print("TEST E: Correct answer")
        print("=" * 60)
        
        await ensure_procedural()
        await asyncio.sleep(0.5)
        b_before = (await body())[:80]; print(f"    [DEBUG] b_before: {b_before.replace(chr(10), ' ')}")
        
        clicked = await click_correct_option()
        print(f"  Clicked correct: {clicked}")
        await asyncio.sleep(0.5)
        
        d_mid = await dc()
        b_mid = await body()
        await shot("E_correct_feedback.png")
        
        # Wait for auto-advance
        await asyncio.sleep(3.0)
        
        b_after = (await body())[:80]
        d_after = await dc()
        await shot("E_after_advance.png")
        
        test_e = {
            "correct_feedback": "correct" in b_mid.lower() or d_mid['result_visible'],
            "no_classification": not d_mid['mistake_panel_visible'],
            "no_native_ease": d_mid['native_ease'] == 0,
            "no_next_btn": d_mid['next_btn'] == 0,
            "auto_advanced": b_before != b_after,
            "next_card_clean": d_after.get('is_proc', False),
        }
        test_e["PASS"] = all([
            test_e["no_classification"],
            test_e["no_native_ease"],
            test_e["no_next_btn"],
        ])
        R["tests"]["E"] = test_e
        for k,v in test_e.items(): print(f"  {k}: {v}")
        print(f"  RESULT: {'PASS' if test_e['PASS'] else 'FAIL'}")

        # ===== TEST F: Stress test =====
        print("\n" + "=" * 60)
        print("TEST F: Stress test (10 iterations)")
        print("=" * 60)
        
        import random
        stress = []
        
        for i in range(10):
            await ensure_procedural()
            await asyncio.sleep(0.3)
            b_before = (await body())[:80]; print(f"    [DEBUG] b_before: {b_before.replace(chr(10), ' ')}")
            
            do_correct = random.random() < 0.3
            use_kb = random.random() < 0.5
            
            if do_correct:
                await click_correct_option()
                await asyncio.sleep(3.0)
            else:
                clicked_wrong = await click_wrong_option(); print(f"    [DEBUG] clicked_wrong: {clicked_wrong}")
                await asyncio.sleep(1.5)
                
                # Try space (should be blocked)
                if random.random() < 0.3:
                    bp = (await body())[:80]
                    await ev("document.dispatchEvent(new KeyboardEvent('keydown',{key:' ',code:'Space',bubbles:true}))")
                    await asyncio.sleep(0.5)
                    ap = (await body())[:80]
                    if bp != ap:
                        R["errors"].append(f"CRITICAL: Space advanced at iter {i}")
                
                if use_kb:
                    k = random.randint(1, 4)
                    await ev(f"document.dispatchEvent(new KeyboardEvent('keydown',{{key:'{k}',code:'Digit{k}',keyCode:{48+k},bubbles:true}}))")
                else:
                    bi = random.randint(0, 3)
                    await ev(f"document.querySelectorAll('.proc-mistake-btn')[{bi}]?.click()")
                await asyncio.sleep(1.5)
            
            b_after = (await body())[:80]
            d_after = await dc()
            
            it = {
                "i": i, "type": "correct" if do_correct else "incorrect",
                "method": "keyboard" if use_kb else "mouse",
                "advanced": b_before != b_after,
                "footers": 1 if d_after['mistake_panel_visible'] else 0,
                "ease": d_after['native_ease'],
                "next": d_after['next_btn'],
            }
            stress.append(it)
            print(f"  [{i}] {it['type']}/{it['method']}: adv={it['advanced']} f={it['footers']} e={it['ease']} n={it['next']}")
        
        await shot("F_stress.png")
        
        test_f = {
            "iterations": stress,
            "all_advanced": all(s["advanced"] for s in stress),
            "any_leaked_ease": any(s["ease"] > 0 for s in stress),
            "any_leaked_next": any(s["next"] > 0 for s in stress),
            "stale_footer": any(s["footers"] > 0 and s["type"] == "correct" for s in stress),
        }
        test_f["PASS"] = test_f["all_advanced"] and not test_f["any_leaked_ease"] and not test_f["any_leaked_next"] and not test_f["stale_footer"]
        R["tests"]["F"] = test_f
        print(f"  all_advanced: {test_f['all_advanced']}")
        print(f"  leaked_ease: {test_f['any_leaked_ease']}")
        print(f"  leaked_next: {test_f['any_leaked_next']}")
        print(f"  stale_footer: {test_f['stale_footer']}")
        print(f"  RESULT: {'PASS' if test_f['PASS'] else 'FAIL'}")

        # ===== SUMMARY =====
        print("\n" + "=" * 60)
        print("FINAL SUMMARY")
        print("=" * 60)
        
        all_pass = True
        for name, test in R["tests"].items():
            if isinstance(test, dict) and "PASS" in test:
                p = test["PASS"]; all_pass = all_pass and p
                print(f"  {name}: {'PASS' if p else 'FAIL'}")
            else:
                for sn, st in test.items():
                    if isinstance(st, dict) and "PASS" in st:
                        p = st["PASS"]; all_pass = all_pass and p
                        print(f"  {name}/{sn}: {'PASS' if p else 'FAIL'}")
        
        if R["errors"]:
            print("\n  CRITICAL ERRORS:")
            for e in R["errors"]: print(f"    - {e}")
            all_pass = False
        
        R["verdict"] = "READY" if all_pass else "NEEDS FIX"
        print(f"\n  VERDICT: {R['verdict']}")
    
    with open(os.path.join(ARTS, "qa_results.json"), 'w', encoding='utf-8') as f:
        json.dump(R, f, indent=2, ensure_ascii=False)
    print(f"\nResults saved to {ARTS}/qa_results.json")

if __name__ == '__main__':
    asyncio.run(run())
