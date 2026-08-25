import asyncio
import ctypes
import json
import os
import sys
import psutil

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

async def study_deck():
    hwnd = 13895330
    user32.ShowWindow(hwnd, 9) # Restore
    WindowForensicsEngine.set_foreground_window(hwnd)

    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    main_target = next((t for t in targets if "main webview" in t.title.lower()), targets[0])
    session = await mgr.switch_target(main_target)
    await session.enable_domains(["DOM", "Runtime", "Page"])

    # Check if we are on deck browser
    is_deckbrowser = await session.evaluate_js("document.getElementById('deckbrowser') !== null || document.querySelector('.deck') !== null")
    print(f"Is deckbrowser: {is_deckbrowser}")
    if is_deckbrowser:
        print("Clicking StudyLab deck...")
        await session.evaluate_js("""
            const deckLink = Array.from(document.querySelectorAll('a, .deck, tr.deck td')).find(el => el.innerText.includes('StudyLab'));
            if (deckLink) { deckLink.click(); }
        """)
        await asyncio.sleep(1.0)

    # Check if we are on overview page
    is_overview = await session.evaluate_js("document.getElementById('study') !== null")
    print(f"Is overview: {is_overview}")
    if is_overview:
        print("Clicking #study button...")
        await session.evaluate_js("document.getElementById('study').click();")
        await asyncio.sleep(1.0)

    # Now let's inspect the reviewer state
    card_info = await session.evaluate_js("""
        ({
            bodyHtml: document.body.innerHTML.substring(0, 500),
            hasProceduralCard: !!document.getElementById('procedural-card'),
            qaText: document.getElementById('qa') ? document.getElementById('qa').innerText.substring(0, 300) : '',
            objectType: document.getElementById('procedural-card') ? document.getElementById('procedural-card').getAttribute('data-object-type') : null,
            domain: document.querySelector('.proc-domain-badge') ? document.querySelector('.proc-domain-badge').innerText : null,
            breadcrumbs: document.querySelector('.proc-breadcrumbs') ? document.querySelector('.proc-breadcrumbs').innerText : null,
            prompt: document.querySelector('.proc-prompt') ? document.querySelector('.proc-prompt').innerText : null,
            hasQuickContainer: !!document.getElementById('proc-quick-container'),
            hasStepwiseContainer: !!document.getElementById('proc-stepwise-container'),
            hasMcqContainer: !!document.querySelector('.proc-option-group'),
            inputs: Array.from(document.querySelectorAll('input, button')).map(el => ({ id: el.id, class: el.className, tag: el.tagName, text: el.innerText.substring(0, 30) }))
        })
    """)
    print("\nCurrent Card State:\n", json.dumps(card_info, indent=2))

    # Capture dual screenshot
    os.makedirs(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit", exist_ok=True)
    cdp_bytes = await session.capture_screenshot()
    with open(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\real_card_1_cdp.png", "wb") as f:
        f.write(cdp_bytes)
        
    WindowForensicsEngine.capture_native_window_screenshot(hwnd, r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\real_card_1_native.png")
    print("Saved real_card_1 screenshots.")

asyncio.run(study_deck())
