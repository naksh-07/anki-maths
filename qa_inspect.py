import asyncio
import json
import urllib.request
import websockets
import base64
import sys

async def cdp_send(ws, method, params=None):
    msg_id = getattr(cdp_send, "msg_id", 1)
    setattr(cdp_send, "msg_id", msg_id + 1)
    req = {"id": msg_id, "method": method}
    if params:
        req["params"] = params
    print(f"Sending: {req}")
    await ws.send(json.dumps(req))
    return msg_id

async def main():
    print("Fetching targets...")
    try:
        req = urllib.request.urlopen('http://127.0.0.1:9222/json')
        targets = json.loads(req.read())
    except Exception as e:
        print(f"Failed to fetch targets: {e}")
        return

    main_ws = None
    for t in targets:
        if 'legacyPageData' in t.get('url', '') and t.get('title') == 'main webview':
            main_ws = t.get('webSocketDebuggerUrl')
            
    if not main_ws:
        print("No main webview found")
        return

    print(f"Connecting to {main_ws}...")
    try:
        async with websockets.connect(main_ws, max_size=None, open_timeout=5) as ws:
            print("Connected.")
            responses = {}
            
            async def listener():
                try:
                    while True:
                        msg = await ws.recv()
                        data = json.loads(msg)
                        if 'id' in data:
                            responses[data['id']] = data.get('result', data.get('error', {}))
                except Exception as e:
                    print(f"Listener error: {e}")
            
            lt = asyncio.create_task(listener())
            
            async def call(method, params=None):
                msg_id = await cdp_send(ws, method, params)
                for _ in range(50):
                    if msg_id in responses:
                        return responses.pop(msg_id)
                    await asyncio.sleep(0.1)
                print(f"Timeout waiting for {method}")
                return None

            print("Capturing screenshot...")
            res = await call('Page.captureScreenshot', {'format': 'png'})
            if res and 'data' in res:
                with open('qa_screenshot_current.png', 'wb') as f:
                    f.write(base64.b64decode(res['data']))
                print("Screenshot saved.")
            else:
                print("Screenshot failed:", res)

            print("Getting DOM...")
            res = await call('Runtime.evaluate', {'expression': 'document.body.innerHTML'})
            if res and 'result' in res:
                with open('qa_dom.html', 'w', encoding='utf-8') as f:
                    f.write(res['result'].get('value', ''))
                print("DOM saved.")
            else:
                print("DOM failed:", res)
                
            lt.cancel()
    except Exception as e:
        print(f"Connection error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
