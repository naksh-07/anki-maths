import asyncio
import websockets
import json
import urllib.request
import time
import random
import shutil
import sqlite3
import os
import uuid

DB_PATH = r'C:\Users\Suraj\AppData\Roaming\AnkiStudyLab\User 1\collection.anki2'

def get_db_metrics():
    copy_path = f'collection_metric_temp_{uuid.uuid4().hex}.anki2'
    shutil.copy2(DB_PATH, copy_path)
    
    metrics = {}
    try:
        with sqlite3.connect(copy_path) as conn:
            c = conn.cursor()
            
            try:
                c.execute("SELECT COUNT(*) FROM practice_attempts")
                metrics['practice_attempts'] = c.fetchone()[0]
                c.execute("SELECT COUNT(*) FROM error_events")
                metrics['error_events'] = c.fetchone()[0]
                c.execute("SELECT COUNT(*) FROM remediation_queue_items")
                metrics['remediation_items'] = c.fetchone()[0]
                c.execute("SELECT COUNT(*) FROM skill_states")
                metrics['skill_states'] = c.fetchone()[0]
            except sqlite3.OperationalError:
                pass 
                
            c.execute("SELECT COUNT(*) FROM revlog")
            metrics['revlog_entries'] = c.fetchone()[0]
            c.execute("SELECT COUNT(*) FROM cards")
            metrics['total_cards'] = c.fetchone()[0]
            c.execute("SELECT COUNT(*) FROM notes")
            metrics['total_notes'] = c.fetchone()[0]
            
    except Exception as e:
        print(f"Error reading DB: {e}", flush=True)
    finally:
        pass
    
    return metrics

async def run_simulation(iterations=300):
    print("Starting pre-simulation DB metric collection...", flush=True)
    before_metrics = get_db_metrics()
    print("Before metrics:", before_metrics, flush=True)
    
    req = urllib.request.urlopen('http://127.0.0.1:9222/json')
    targets = json.loads(req.read())
    main_ws = None
    for t in targets:
        if t.get('type') == 'page' and 'legacyPageData' in t.get('url', ''):
            main_ws = t.get('webSocketDebuggerUrl')

    if not main_ws:
        print("No main webview found", flush=True)
        return None

    evidence = {
        "iterations_target": iterations,
        "iterations_completed": 0,
        "metrics_before": before_metrics,
        "performance": {},
        "exceptions": [],
        "console_errors": [],
        "bugs": []
    }

    async with websockets.connect(main_ws, max_size=None) as ws:
        msg_id = 1
        
        perf_requests = {}
        eval_requests = {}
        
        async def reader():
            try:
                while True:
                    msg = await ws.recv()
                    data = json.loads(msg)
                    if 'method' in data:
                        if data['method'] == 'Runtime.exceptionThrown':
                            err = data['params']['exceptionDetails']['exception'].get('description', 'Unknown Exception')
                            evidence['exceptions'].append(err)
                            print("Exception:", err, flush=True)
                        elif data['method'] == 'Runtime.consoleAPICalled':
                            if data['params']['type'] == 'error':
                                err = ' '.join(str(a.get('value', a)) for a in data['params']['args'])
                                evidence['console_errors'].append(err)
                    elif 'id' in data:
                        if data['id'] in perf_requests:
                            phase = perf_requests.pop(data['id'])
                            metrics = {m['name']: m['value'] for m in data['result']['metrics']}
                            evidence['performance'][phase] = metrics
                        elif data['id'] in eval_requests:
                            if not eval_requests[data['id']].done():
                                eval_requests[data['id']].set_result(data)
            except Exception as e:
                pass
                
        reader_task = asyncio.create_task(reader())
        
        await ws.send(json.dumps({'id': msg_id, 'method': 'Runtime.enable'}))
        msg_id += 1
        await ws.send(json.dumps({'id': msg_id, 'method': 'Performance.enable'}))
        msg_id += 1
        await ws.send(json.dumps({'id': msg_id, 'method': 'Log.enable'}))
        msg_id += 1

        async def get_perf(phase):
            nonlocal msg_id
            m_id = msg_id
            msg_id += 1
            perf_requests[m_id] = phase
            await ws.send(json.dumps({'id': m_id, 'method': 'Performance.getMetrics'}))
            
        async def eval_js(code):
            nonlocal msg_id
            m_id = msg_id
            msg_id += 1
            fut = asyncio.Future()
            eval_requests[m_id] = fut
            await ws.send(json.dumps({'id': m_id, 'method': 'Runtime.evaluate', 'params': {'expression': code, 'returnByValue': True}}))
            try:
                res = await asyncio.wait_for(fut, timeout=2.0)
                return res
            except asyncio.TimeoutError:
                return {}
            finally:
                if m_id in eval_requests:
                    del eval_requests[m_id]

        await get_perf("START")
        await asyncio.sleep(0.5)
        
        i = 0
        while i < iterations:
            res = await eval_js("!!document.getElementById('procedural-card')")
            is_proc = res.get('result', {}).get('result', {}).get('value', False)
            
            if is_proc:
                if random.random() < 0.2:
                    await eval_js("var t = document.getElementById('tab-stepwise'); if(t) t.click();")
                    await asyncio.sleep(0.1)
                if random.random() < 0.1:
                    await eval_js("var h = document.getElementById('proc-hint-btn'); if(h) h.click();")
                    await asyncio.sleep(0.1)
                    
                correct = random.random() < 0.6
                ans = 'Frank' if correct else str(random.randint(1, 100))
                await eval_js(f"var el = document.getElementById('proc-answer-input'); if(!el) el = document.querySelector('.proc-step-input'); if(el){{el.value = '{ans}'; el.dispatchEvent(new Event('input'));}}")
                await asyncio.sleep(0.1)
                
                await eval_js("var b1=document.getElementById('proc-submit-btn'); var b2=document.getElementById('proc-check-steps-btn'); if(b1) b1.click(); else if(b2) b2.click();")
                await asyncio.sleep(0.3)
                
                if not correct:
                    await eval_js("var btns = document.querySelectorAll('.proc-mistake-btn'); if(btns.length > 0) btns[Math.floor(Math.random() * btns.length)].click();")
                    await asyncio.sleep(0.1)
                    
                await eval_js("var nbtn = document.getElementById('proc-next-btn'); if(nbtn) nbtn.click();")
                await asyncio.sleep(0.3)
            else:
                res_ans = await eval_js("!!document.getElementById('answer')")
                is_ans = res_ans.get('result', {}).get('result', {}).get('value', False)
                if not is_ans:
                    await eval_js("if(typeof pycmd !== 'undefined') pycmd('ans');")
                    await asyncio.sleep(0.2)
                
                ease = random.choice(['ease1', 'ease2', 'ease3', 'ease4'])
                await eval_js(f"if(typeof pycmd !== 'undefined') pycmd('{ease}');")
                await asyncio.sleep(0.3)
                
            i += 1
            if i % 10 == 0:
                print(f"Completed {i}/{iterations} iterations", flush=True)
                
            if i == iterations // 2:
                await get_perf("MIDPOINT")

        await get_perf("END")
        await asyncio.sleep(1.0)
        reader_task.cancel()
        
        evidence['iterations_completed'] = i

    print("Starting post-simulation DB metric collection...", flush=True)
    after_metrics = get_db_metrics()
    print("After metrics:", after_metrics, flush=True)
    evidence['metrics_after'] = after_metrics
    
    with open('evidence.json', 'w') as f:
        json.dump(evidence, f, indent=2)
        
    return evidence

if __name__ == '__main__':
    asyncio.run(run_simulation(1200))
