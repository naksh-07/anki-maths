import asyncio, websockets, json, urllib.request
async def run():
    targets=json.loads(urllib.request.urlopen('http://127.0.0.1:9222/json').read())
    main_ws=next((t['webSocketDebuggerUrl'] for t in targets if t.get('title')=='main webview'), None)
    ws=await websockets.connect(main_ws, max_size=None)
    js='''(() => { return document.body.id || document.body.className; })();'''
    await ws.send(json.dumps({'id':1,'method':'Runtime.evaluate','params':{'expression':js,'returnByValue':True}}))
    res = await ws.recv()
    print(res)
asyncio.run(run())
