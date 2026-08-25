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

async def open_and_study():
    hwnd = 13895330
    user32.ShowWindow(hwnd, 9)
    WindowForensicsEngine.set_foreground_window(hwnd)

    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    main_target = next((t for t in targets if "main webview" in t.title.lower()), targets[1])
    session = await mgr.switch_target(main_target)
    await session.enable_domains(["DOM", "Runtime", "Page"])

    # Trigger pycmd to open deck
    print("Opening deck 1787659104777...")
    await session.evaluate_js("pycmd('open:1787659104777')")
    await asyncio.sleep(1.0)

    # Check if study button exists
    overview = await session.evaluate_js("""
        ({
            hasStudy: !!document.getElementById('study'),
            bodyText: document.body.innerText.substring(0, 200)
        })
    """)
    print("Overview state:", overview)

    if overview.get('hasStudy'):
        print("Starting study...")
        await session.evaluate_js("pycmd('study')")
        await asyncio.sleep(1.5)

    # Now inspect reviewer
    card_info = await session.evaluate_js("""
        ({
            hasProceduralCard: !!document.getElementById('procedural-card'),
            cardId: document.getElementById('procedural-card') ? document.getElementById('procedural-card').getAttribute('data-object-type') : null,
            qaHtml: document.getElementById('qa') ? document.getElementById('qa').innerHTML : '',
            prompt: document.querySelector('.proc-prompt') ? document.querySelector('.proc-prompt').innerText : null,
            domain: document.querySelector('.proc-domain-badge') ? document.querySelector('.proc-domain-badge').innerText : null,
            breadcrumbs: document.querySelector('.proc-breadcrumbs') ? document.querySelector('.proc-breadcrumbs').innerText : null,
            optionsCount: document.querySelectorAll('.proc-option-item').length,
            hasInput: !!document.getElementById('proc-answer-input'),
            hasStepwise: !!document.getElementById('proc-stepwise-container'),
            hasQuick: !!document.getElementById('proc-quick-container'),
            resultVisible: document.getElementById('proc-result-panel') && !document.getElementById('proc-result-panel').classList.contains('hidden')
        })
    """)
    print("Reviewer card state:\n", json.dumps({k: v for k, v in card_info.items() if k != 'qaHtml'}, indent=2))

    # Capture screenshots
    os.makedirs(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit", exist_ok=True)
    cdp_bytes = await session.capture_screenshot()
    with open(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\first_study_card_cdp.png", "wb") as f:
        f.write(cdp_bytes)

    WindowForensicsEngine.capture_native_window_screenshot(hwnd, r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\first_study_card_native.png")
    print("Saved first_study_card screenshots.")

asyncio.run(open_and_study())
