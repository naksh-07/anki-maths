import asyncio
import sys
sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")
from core.session import MultiTargetSessionManager

async def test():
    mgr = MultiTargetSessionManager(host='127.0.0.1', port=9222, engine='qtwebengine')
    targets = mgr.list_targets()
    main_target = next((t for t in targets if 'main webview' in t.title.lower()), targets[1])
    session = await mgr.switch_target(main_target)
    html = await session.evaluate_js("""
        (() => {
            const qa = document.getElementById('qa');
            return qa ? qa.innerHTML : document.body.innerHTML;
        })()
    """)
    print("QA HTML:")
    print(html)
    
    # Check what scripts or fields are in the page
    scripts = await session.evaluate_js("""
        Array.from(document.querySelectorAll('script')).map(s => s.src || s.innerText.substring(0, 100))
    """)
    print("\nScripts in page:", scripts)

asyncio.run(test())
