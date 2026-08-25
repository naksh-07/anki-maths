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

async def reload_and_check():
    hwnd = 13895330
    user32.ShowWindow(hwnd, 9)
    WindowForensicsEngine.set_foreground_window(hwnd)

    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    main_target = next((t for t in targets if "main webview" in t.title.lower()), targets[1])
    session = await mgr.switch_target(main_target)
    await session.enable_domains(["DOM", "Runtime", "Page"])

    # Reload page
    print("Reloading main webview...")
    await session.evaluate_js("window.location.reload()")
    await asyncio.sleep(2.0)

    # Check DOM
    card_info = await session.evaluate_js("""
        (() => {
            const procCard = document.getElementById('procedural-card');
            const errorDiv = document.querySelector('.proc-error');
            return {
                hasProceduralCard: !!procCard,
                hasError: !!errorDiv,
                errorText: errorDiv ? errorDiv.innerText : null,
                objectType: procCard ? procCard.getAttribute('data-object-type') : null,
                domain: document.querySelector('.proc-domain-badge') ? document.querySelector('.proc-domain-badge').innerText : null,
                breadcrumbs: document.querySelector('.proc-breadcrumbs') ? document.querySelector('.proc-breadcrumbs').innerText : null,
                prompt: document.querySelector('.proc-prompt') ? document.querySelector('.proc-prompt').innerText : null,
                hasQuickContainer: !!document.getElementById('proc-quick-container'),
                hasStepwiseContainer: !!document.getElementById('proc-stepwise-container'),
                hasMcqContainer: !!document.querySelector('.proc-option-group'),
                quickInputPlaceholder: document.getElementById('proc-answer-input') ? document.getElementById('proc-answer-input').placeholder : null,
                optionsCount: document.querySelectorAll('.proc-option-item').length,
                hasResultPanel: !!document.getElementById('proc-result-panel'),
                isResultVisible: document.getElementById('proc-result-panel') && !document.getElementById('proc-result-panel').classList.contains('hidden'),
                hasMistakePanel: !!document.getElementById('proc-mistake-panel'),
                isMistakeVisible: document.getElementById('proc-mistake-panel') && !document.getElementById('proc-mistake-panel').classList.contains('hidden')
            };
        })()
    """)
    print("\nCard Info after rebuild & reload:\n", json.dumps(card_info, indent=2))

    # Capture screenshots
    os.makedirs(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit", exist_ok=True)
    cdp_bytes = await session.capture_screenshot()
    with open(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\reloaded_card_cdp.png", "wb") as f:
        f.write(cdp_bytes)

    WindowForensicsEngine.capture_native_window_screenshot(hwnd, r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\reloaded_card_native.png")
    print("Saved reloaded_card screenshots.")

asyncio.run(reload_and_check())
