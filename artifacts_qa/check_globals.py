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
    
    globals_info = await session.evaluate_js("""
        ({
            hasAnki: typeof window.anki !== 'undefined',
            ankiKeys: window.anki ? Object.keys(window.anki) : [],
            hasProcedural: typeof window.procedural !== 'undefined',
            hasProceduralAPI: typeof window.proceduralAPI !== 'undefined',
            ankiProcedural: window.anki && window.anki.procedural ? Object.keys(window.anki.procedural) : null,
            loadedScripts: Array.from(document.querySelectorAll('script')).map(s => s.src || 'inline')
        })
    """)
    print("Main Webview JavaScript Environment:\n", json.dumps(globals_info, indent=2))

asyncio.run(test())
