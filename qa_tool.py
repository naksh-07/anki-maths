import asyncio
import websockets
import json
import urllib.request
import time
import base64

async def run_qa():
    print("Starting QA script...", flush=True)
    req = urllib.request.urlopen('http://127.0.0.1:9222/json/list')
    targets = json.loads(req.read())
    main_ws = None
    for t in targets:
        if t.get('type') == 'page' and t.get('title') == 'main webview':
            main_ws = t.get('webSocketDebuggerUrl')
            
    if not main_ws:
        print("No main webview found", flush=True)
        return

    print("Found ws:", main_ws, flush=True)
    async with websockets.connect(main_ws, max_size=None) as ws:
        msg_id = 1
        eval_requests = {}
        screenshot_requests = {}
        
        async def reader():
            try:
                while True:
                    msg = await ws.recv()
                    data = json.loads(msg)
                    if 'id' in data:
                        if data['id'] in eval_requests:
                            if not eval_requests[data['id']].done():
                                eval_requests[data['id']].set_result(data)
                        elif data['id'] in screenshot_requests:
                            if not screenshot_requests[data['id']].done():
                                screenshot_requests[data['id']].set_result(data)
            except Exception as e:
                pass
                
        reader_task = asyncio.create_task(reader())
        
        async def eval_js(code):
            nonlocal msg_id
            m_id = msg_id
            msg_id += 1
            fut = asyncio.Future()
            eval_requests[m_id] = fut
            await ws.send(json.dumps({'id': m_id, 'method': 'Runtime.evaluate', 'params': {'expression': code, 'returnByValue': True}}))
            try:
                res = await asyncio.wait_for(fut, timeout=2.0)
                return res.get('result', {}).get('result', {}).get('value', None)
            except asyncio.TimeoutError:
                return None
            finally:
                if m_id in eval_requests:
                    del eval_requests[m_id]

        async def get_screenshot(name):
            nonlocal msg_id
            m_id = msg_id
            msg_id += 1
            fut = asyncio.Future()
            screenshot_requests[m_id] = fut
            await ws.send(json.dumps({'id': m_id, 'method': 'Page.captureScreenshot', 'params': {'format': 'png'}}))
            try:
                res = await asyncio.wait_for(fut, timeout=5.0)
                data = res.get('result', {}).get('data')
                if data:
                    with open(name, 'wb') as f:
                        f.write(base64.b64decode(data))
                    print(f"Saved {name}", flush=True)
                return True
            except asyncio.TimeoutError:
                return False
            finally:
                if m_id in screenshot_requests:
                    del screenshot_requests[m_id]

        import sys
        if len(sys.argv) > 1:
            cmd = sys.argv[1]
            if cmd == "screenshot":
                await get_screenshot(sys.argv[2])
            elif cmd == "eval":
                res = await eval_js(sys.argv[2])
                with open("eval_res.txt", "w", encoding="utf-8") as f:
                    f.write(str(res))
                print("EVAL RESULT saved to eval_res.txt", flush=True)
            elif cmd == "press":
                await ws.send(json.dumps({
                    'id': msg_id, 
                    'method': 'Input.dispatchKeyEvent', 
                    'params': {'type': 'char', 'text': sys.argv[2]}
                }))
                msg_id += 1
                await asyncio.sleep(0.1)

        reader_task.cancel()

if __name__ == '__main__':
    asyncio.run(run_qa())
