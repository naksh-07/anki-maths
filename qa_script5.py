import asyncio, websockets, json, urllib.request
async def run():
    targets=json.loads(urllib.request.urlopen('http://127.0.0.1:9222/json').read())
    main_ws=next((t['webSocketDebuggerUrl'] for t in targets if t.get('title')=='main webview'), None)
    ws=await websockets.connect(main_ws, max_size=None)
    js='''(() => { return document.body.innerHTML; })();'''
    await ws.send(json.dumps({'id':1,'method':'Runtime.evaluate','params':{'expression':js,'returnByValue':True}}))
    res = await ws.recv()
    data = json.loads(res)
    with open('current_card_5.html', 'w', encoding='utf-8') as f: f.write(data['result']['result']['value'])
asyncio.run(run())
