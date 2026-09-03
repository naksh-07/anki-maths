import sys
import traceback

print("Script started", flush=True)

try:
    import asyncio
    import websockets
    import json
    import urllib.request
    import time
    import base64
except Exception as e:
    print(f"Import error: {e}", flush=True)
    sys.exit(1)

async def cdp_call(ws, msg_id, method, params=None):
    req = {'id': msg_id, 'method': method}
    if params:
        req['params'] = params
    await ws.send(json.dumps(req))

async def main():
    try:
        req = urllib.request.urlopen('http://127.0.0.1:9222/json')
        targets = json.loads(req.read())
        main_ws = None
        for t in targets:
            if t.get('type') == 'page' and 'legacyPageData' in t.get('url', '') and t.get('title') == 'main webview':
                main_ws = t.get('webSocketDebuggerUrl')
                
        if not main_ws:
            print("No main webview found", flush=True)
            return

        print("Connecting to ws", flush=True)
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
                    print(f"Reader error: {e}")
                    pass
                    
            reader_task = asyncio.create_task(reader())
            
            async def eval_js(code):
                nonlocal msg_id
                m_id = msg_id
                msg_id += 1
                fut = asyncio.Future()
                eval_requests[m_id] = fut
                await cdp_call(ws, m_id, 'Runtime.evaluate', {'expression': code, 'returnByValue': True})
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
                await cdp_call(ws, m_id, 'Page.captureScreenshot', {'format': 'png'})
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

            print("Checking UI state", flush=True)
            
            body = await eval_js("document.body.innerHTML")
            with open('qa_dom.html', 'w', encoding='utf-8') as f:
                f.write(str(body))
                
            await get_screenshot("qa_current.png")
            
            print("Done", flush=True)
            reader_task.cancel()
    except Exception as e:
        print(f"Main error: {e}")
        traceback.print_exc()

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except Exception as e:
        print(f"Run error: {e}")
