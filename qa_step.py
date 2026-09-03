"""
StudyLab Forensic QA - Step-by-step CLI
Usage: out\pyenv\Scripts\python.exe qa_step.py <command> [args]
Commands:
  state       - Print current state info
  screenshot <name> - Take a screenshot  
  html <name> - Save innerHTML to file
  click_option <idx> - Click MCQ option (0-based)
  submit      - Click submit button
  classify <idx> - Click classification button (0-based)
  key <key>   - Dispatch keyboard event
  dom_counts  - Count DOM elements
  bottom      - Inspect bottom toolbar
  advance_correct - Submit correct then wait for auto-advance
  advance_wrong   - Submit wrong then show state
"""
import asyncio
import websockets
import json
import urllib.request
import base64
import sys
import os
import io

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

ARTIFACTS = "artifacts_qa"
os.makedirs(ARTIFACTS, exist_ok=True)

async def main():
    targets = json.loads(urllib.request.urlopen('http://127.0.0.1:9222/json/list').read())
    main_ws = next((t['webSocketDebuggerUrl'] for t in targets if t.get('title') == 'main webview'), None)
    bottom_ws = next((t['webSocketDebuggerUrl'] for t in targets if t.get('title') == 'bottom toolbar'), None)
    
    if not main_ws:
        print("ERROR: No main webview"); return

    async with websockets.connect(main_ws, max_size=None) as ws:
        mid = 1; pending = {}
        
        async def reader():
            try:
                while True:
                    msg = await ws.recv()
                    d = json.loads(msg)
                    if 'id' in d and d['id'] in pending and not pending[d['id']].done():
                        pending[d['id']].set_result(d)
            except: pass
        asyncio.create_task(reader())
        
        async def ev(code):
            nonlocal mid
            m = mid; mid += 1; f = asyncio.Future(); pending[m] = f
            await ws.send(json.dumps({'id':m,'method':'Runtime.evaluate','params':{'expression':code,'returnByValue':True}}))
            try:
                r = await asyncio.wait_for(f, 3.0)
                return r.get('result',{}).get('result',{}).get('value')
            except: return None
            finally: pending.pop(m,None)
        
        async def shot(name):
            nonlocal mid
            m = mid; mid += 1; f = asyncio.Future(); pending[m] = f
            await ws.send(json.dumps({'id':m,'method':'Page.captureScreenshot','params':{'format':'png'}}))
            try:
                r = await asyncio.wait_for(f, 5.0)
                d = r.get('result',{}).get('data')
                if d:
                    p = os.path.join(ARTIFACTS, name)
                    with open(p,'wb') as fp: fp.write(base64.b64decode(d))
                    print(f"Saved: {p}")
            except: print("Screenshot timeout")
            finally: pending.pop(m,None)
        
        async def dom_counts():
            c = {}
            c['proc_card'] = await ev("!!document.getElementById('procedural-card')") or False
            c['mistake_panel_visible'] = await ev("!document.getElementById('proc-mistake-panel')?.classList.contains('hidden')") or False
            c['mistake_btns'] = await ev("document.querySelectorAll('.proc-mistake-btn').length") or 0
            c['result_panel_visible'] = await ev("!document.getElementById('proc-result-panel')?.classList.contains('hidden')") or False
            c['solution_visible'] = await ev("!document.getElementById('proc-solution-container')?.classList.contains('hidden')") or False
            c['proc_next_btn'] = await ev("document.querySelectorAll('#proc-next-btn, .proc-next-btn').length") or 0
            c['native_ease'] = await ev("document.querySelectorAll('#ease1, #ease2, #ease3, #ease4, .ease').length") or 0
            c['option_items'] = await ev("document.querySelectorAll('.proc-option-item').length") or 0
            c['interaction_footer'] = await ev("!!document.getElementById('proc-interaction-footer')") or False
            return c
        
        cmd = sys.argv[1] if len(sys.argv) > 1 else "state"
        
        if cmd == "state":
            body = await ev("document.body.innerText")
            dc = await dom_counts()
            print("=== CURRENT STATE ===")
            print(f"Body text (200): {(body or '')[:200]}")
            print(f"DOM counts: {json.dumps(dc, indent=2)}")
            
        elif cmd == "screenshot":
            name = sys.argv[2] if len(sys.argv) > 2 else "screenshot.png"
            await shot(name)
            
        elif cmd == "html":
            name = sys.argv[2] if len(sys.argv) > 2 else "page.html"
            html = await ev("document.body.innerHTML")
            p = os.path.join(ARTIFACTS, name)
            with open(p, 'w', encoding='utf-8') as f: f.write(html or "")
            print(f"Saved HTML: {p} ({len(html or '')} bytes)")
            
        elif cmd == "click_option":
            idx = int(sys.argv[2])
            r = await ev(f"var b=document.querySelectorAll('.proc-option-item'); b[{idx}]?.click(); b.length")
            print(f"Clicked option {idx}, total options: {r}")
            await asyncio.sleep(0.5)
            dc = await dom_counts()
            print(f"DOM after: {json.dumps(dc, indent=2)}")
            
        elif cmd == "submit":
            r = await ev("""
                var b = document.getElementById('proc-submit-btn') 
                     || document.getElementById('proc-check-steps-btn');
                if(b) { b.click(); true; } else { false; }
            """)
            print(f"Submit clicked: {r}")
            await asyncio.sleep(1.0)
            dc = await dom_counts()
            print(f"DOM after submit: {json.dumps(dc, indent=2)}")
            
        elif cmd == "classify":
            idx = int(sys.argv[2])
            body_before = (await ev("document.body.innerText") or "")[:80]
            r = await ev(f"var b=document.querySelectorAll('.proc-mistake-btn'); if(b[{idx}]) {{ b[{idx}].click(); true; }} else {{ false; }}")
            print(f"Classify clicked [{idx}]: {r}")
            await asyncio.sleep(1.5)
            body_after = (await ev("document.body.innerText") or "")[:80]
            dc = await dom_counts()
            print(f"Card advanced: {body_before != body_after}")
            print(f"DOM after: {json.dumps(dc, indent=2)}")
            
        elif cmd == "key":
            key = sys.argv[2]
            body_before = (await ev("document.body.innerText") or "")[:80]
            if key in ["1","2","3","4"]:
                await ev(f"document.dispatchEvent(new KeyboardEvent('keydown', {{key:'{key}',code:'Digit{key}',bubbles:true}}))")
            elif key == "Space":
                await ev("document.dispatchEvent(new KeyboardEvent('keydown', {key:' ',code:'Space',bubbles:true}))")
            elif key == "Enter":
                await ev("document.dispatchEvent(new KeyboardEvent('keydown', {key:'Enter',code:'Enter',bubbles:true}))")
            elif key in ["A","B","C","D","a","b","c","d"]:
                await ev(f"document.dispatchEvent(new KeyboardEvent('keydown', {{key:'{key.lower()}',code:'Key{key.upper()}',bubbles:true}}))")
            await asyncio.sleep(1.0)
            body_after = (await ev("document.body.innerText") or "")[:80]
            dc = await dom_counts()
            print(f"Key '{key}' pressed")
            print(f"Card advanced: {body_before != body_after}")
            print(f"DOM after: {json.dumps(dc, indent=2)}")
            
        elif cmd == "eval":
            code = sys.argv[2]
            r = await ev(code)
            print(f"Eval result: {r}")
            dc = await dom_counts()
            print(json.dumps(dc, indent=2))
            
        elif cmd == "bottom":
            if not bottom_ws:
                print("No bottom toolbar"); return
            async with websockets.connect(bottom_ws, max_size=None) as bws:
                bp = {}
                async def br():
                    try:
                        while True:
                            m = await bws.recv(); d = json.loads(m)
                            if 'id' in d and d['id'] in bp and not bp[d['id']].done():
                                bp[d['id']].set_result(d)
                    except: pass
                asyncio.create_task(br())
                bi = 1
                async def bev(code):
                    nonlocal bi
                    m = bi; bi += 1; f = asyncio.Future(); bp[m] = f
                    await bws.send(json.dumps({'id':m,'method':'Runtime.evaluate','params':{'expression':code,'returnByValue':True}}))
                    try:
                        r = await asyncio.wait_for(f, 3.0)
                        return r.get('result',{}).get('result',{}).get('value')
                    except: return None
                    finally: bp.pop(m,None)
                
                html = await bev("document.body.innerHTML")
                text = await bev("document.body.innerText")
                btns = await bev("document.querySelectorAll('button').length")
                
                with open(os.path.join(ARTIFACTS, "bottom_toolbar.html"), 'w', encoding='utf-8') as f:
                    f.write(html or "")
                print(f"Bottom toolbar text: {(text or '')[:300]}")
                print(f"Bottom toolbar buttons: {btns}")
                print(f"Bottom toolbar HTML saved")
                
                # Screenshot bottom
                bm = bi; bi += 1; bf = asyncio.Future(); bp[bm] = bf
                await bws.send(json.dumps({'id':bm,'method':'Page.captureScreenshot','params':{'format':'png'}}))
                try:
                    r = await asyncio.wait_for(bf, 5.0)
                    d = r.get('result',{}).get('data')
                    if d:
                        p = os.path.join(ARTIFACTS, "bottom_toolbar.png")
                        with open(p,'wb') as fp: fp.write(base64.b64decode(d))
                        print(f"Bottom screenshot: {p}")
                except: pass

if __name__ == '__main__':
    asyncio.run(main())
