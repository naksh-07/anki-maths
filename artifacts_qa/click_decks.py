import asyncio
import ctypes
import json
import os
import sys

# Ensure WinSta0/Default desktop
user32 = ctypes.windll.user32
hwinsta = user32.OpenWindowStationW("WinSta0", False, 0x37F)
if hwinsta:
    user32.SetProcessWindowStation(hwinsta)
hdesk = user32.OpenDesktopW("Default", 0, False, 0x1FF)
if hdesk:
    user32.SetThreadDesktop(hdesk)

sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")
from core.session import MultiTargetSessionManager
from core.window_forensics import WindowForensicsEngine

async def click_decks():
    hwnd = 13895330
    user32.ShowWindow(hwnd, 9)
    WindowForensicsEngine.set_foreground_window(hwnd)

    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    print("Discovered targets:")
    for t in targets:
        print(f"  {t.id} : '{t.title}' -> {t.url}")

    top_target = next((t for t in targets if "top toolbar" in t.title.lower()), None)
    if top_target:
        top_session = await mgr.switch_target(top_target)
        await top_session.enable_domains(["DOM", "Runtime"])
        buttons = await top_session.evaluate_js("""
            Array.from(document.querySelectorAll('a, button')).map(b => ({
                text: b.innerText,
                id: b.id,
                onclick: b.getAttribute('onclick'),
                href: b.getAttribute('href')
            }))
        """)
        print("Top toolbar buttons:", buttons)
        
        # Click Decks
        print("Clicking Decks link in top toolbar...")
        await top_session.evaluate_js("""
            const deckBtn = Array.from(document.querySelectorAll('a, button')).find(b => b.innerText.includes('Decks'));
            if (deckBtn) { deckBtn.click(); }
        """)
        await asyncio.sleep(1.5)

    # Check main webview
    main_target = next((t for t in targets if "main webview" in t.title.lower()), targets[1])
    main_session = await mgr.switch_target(main_target)
    await main_session.enable_domains(["DOM", "Runtime", "Page"])
    main_info = await main_session.evaluate_js("""
        ({
            url: window.location.href,
            bodyText: document.body.innerText.substring(0, 200),
            hasDeckbrowser: !!document.getElementById('deckbrowser') || !!document.querySelector('.deck')
        })
    """)
    print("Main webview after clicking Decks:", main_info)

    # Capture screenshots
    os.makedirs(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit", exist_ok=True)
    WindowForensicsEngine.capture_native_window_screenshot(hwnd, r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\after_click_decks_native.png")
    print("Saved after_click_decks_native.png")

asyncio.run(click_decks())
