import asyncio
import sys
import json
sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")
from core.session import MultiTargetSessionManager

async def test():
    mgr = MultiTargetSessionManager(host='127.0.0.1', port=9222, engine='qtwebengine')
    targets = mgr.list_targets()
    main_target = next((t for t in targets if 'main webview' in t.title.lower()), targets[1])
    session = await mgr.switch_target(main_target)
    
    # Check if window.anki or reviewer state has card info
    info = await session.evaluate_js("""
        ({
            ankiProps: Object.keys(window.anki || {}),
            reviewerProps: Object.keys(window.reviewer || {}),
        })
    """)
    print("Window props:", info)

asyncio.run(test())
