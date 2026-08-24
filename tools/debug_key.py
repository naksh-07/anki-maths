import asyncio
import os
import sys
import time

sys.stdout.reconfigure(encoding="utf-8")
sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")

from core.models import Target
from core.session import MultiTargetSessionManager

async def debug_key():
    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    main_target = next((t for t in targets if "main webview" in t.title.lower()), None)
    session = await mgr.switch_target(main_target)

    # Let's check what event listeners receive keydown
    await session.evaluate_js("""
        window.__events = [];
        window.addEventListener('keydown', (e) => {
            window.__events.push({ key: e.key, code: e.code, target: e.target.tagName });
        });
    """)

    print("Dispatching key '1'...")
    await session.dispatch_key_event("keyDown", "1", "1")
    await session.dispatch_key_event("keyUp", "1")
    await asyncio.sleep(0.2)

    evs = await session.evaluate_js("window.__events")
    print("Recorded events on window:", evs)

    await session.close()
    await mgr.close_all()

if __name__ == "__main__":
    asyncio.run(debug_key())
