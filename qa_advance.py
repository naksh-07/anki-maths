import io,sys,asyncio,websockets,json,urllib.request
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

async def main():
    targets = json.loads(urllib.request.urlopen('http://127.0.0.1:9222/json/list').read())
    ws_url = next(t['webSocketDebuggerUrl'] for t in targets if t.get('title')=='main webview')
    async with websockets.connect(ws_url, max_size=None) as ws:
        mid=1; pending={}
        async def reader():
            try:
                while True:
                    m=await ws.recv(); d=json.loads(m)
                    if 'id' in d and d['id'] in pending and not pending[d['id']].done():
                        pending[d['id']].set_result(d)
            except: pass
        asyncio.create_task(reader())
        async def ev(code):
            nonlocal mid; m=mid; mid+=1; f=asyncio.Future(); pending[m]=f
            await ws.send(json.dumps({'id':m,'method':'Runtime.evaluate','params':{'expression':code,'returnByValue':True}}))
            try: r=await asyncio.wait_for(f,3.0); return r.get('result',{}).get('result',{}).get('value')
            except: return None
            finally: pending.pop(m,None)
        
        # Force advance by calling pycmd('ease1') to force Anki to show next card
        r = await ev("pycmd('ease1')")
        print('Sent ease1:', r)
        await asyncio.sleep(2)
        body = await ev('document.body.innerText')
        print('After advance:', (body or '')[:200])

asyncio.run(main())
