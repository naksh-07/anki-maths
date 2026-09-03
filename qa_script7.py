import asyncio, websockets, json, urllib.request
async def run():
    targets=json.loads(urllib.request.urlopen('http://127.0.0.1:9222/json').read())
    main_ws=next((t['webSocketDebuggerUrl'] for t in targets if t.get('title')=='main webview'), None)
    ws=await websockets.connect(main_ws, max_size=None)
    await ws.send(json.dumps({'id':1,'method':'Log.enable'}))
    await ws.send(json.dumps({'id':2,'method':'Runtime.enable'}))
    while True:
        try: msg=json.loads(await asyncio.wait_for(ws.recv(), 2.0)); print(msg)
        except asyncio.TimeoutError: break
asyncio.run(run())
