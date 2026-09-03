import io, sys, asyncio, websockets, json, urllib.request
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
            nonlocal mid
            m=mid; mid+=1; f=asyncio.Future(); pending[m]=f
            await ws.send(json.dumps({'id':m,'method':'Runtime.evaluate','params':{'expression':code,'returnByValue':True}}))
            try:
                r=await asyncio.wait_for(f,3.0)
                return r.get('result',{}).get('result',{}).get('value')
            except: return None
            finally: pending.pop(m,None)
        
        # Check the procedural state machine
        checks = [
            ("anki.procedural type", "typeof window.anki && typeof window.anki.procedural"),
            ("procedural keys", "window.anki && window.anki.procedural ? Object.keys(window.anki.procedural).join(',') : 'none'"),
            ("setup type", "window.anki && window.anki.procedural ? typeof window.anki.procedural.setup : 'none'"),
            ("state", "window.anki && window.anki.procedural && window.anki.procedural._sm ? window.anki.procedural._sm.state : 'no _sm'"),
            ("_ctrl keys", "window.anki && window.anki.procedural && window.anki.procedural._ctrl ? Object.keys(window.anki.procedural._ctrl).join(',') : 'no _ctrl'"),
            ("_instance keys", "window.anki && window.anki.procedural && window.anki.procedural._instance ? Object.keys(window.anki.procedural._instance).slice(0,15).join(',') : 'no _instance'"),
        ]
        
        for label, code in checks:
            result = await ev(code)
            print(f"{label}: {result}")
        
        # Try a broader search for the state machine
        more = await ev("""
            var r = [];
            if (window.anki && window.anki.procedural) {
                var p = window.anki.procedural;
                for (var k in p) {
                    r.push(k + ':' + typeof p[k]);
                }
            }
            r.join('; ')
        """)
        print(f"All procedural props: {more}")
        
asyncio.run(main())
