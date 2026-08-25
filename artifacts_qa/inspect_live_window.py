import asyncio
import ctypes
from ctypes import wintypes
import json
import os
import sys
import psutil

sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")
from core.session import MultiTargetSessionManager
from core.window_forensics import WindowForensicsEngine

async def inspect_live():
    print("=== LIVE ANKI ATTACH DIAGNOSTIC ===")
    
    # 1. Process & Connection check
    anki_pid = None
    for conn in psutil.net_connections(kind='inet'):
        if conn.laddr.port == 9222 and conn.pid:
            anki_pid = conn.pid
            break
            
    print(f"Port 9222 Owning PID: {anki_pid}")
    if anki_pid:
        p = psutil.Process(anki_pid)
        print(f"Process Name: {p.name()}, Exe: {p.exe()}")
        print(f"Cmdline: {' '.join(p.cmdline())}")
        parent = p.parent()
        if parent:
            print(f"Parent PID {parent.pid}: {parent.name()}, Exe: {parent.exe()}")
    
    # 2. Window Forensics
    user32 = ctypes.windll.user32
    WNDENUMPROC = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    
    visible_windows = []
    def cb(hwnd, lparam):
        if user32.IsWindowVisible(hwnd):
            info = WindowForensicsEngine.inspect_hwnd(hwnd)
            if info.get("is_real_gui"):
                visible_windows.append(info)
        return True
        
    user32.EnumWindows(WNDENUMPROC(cb), 0)
    
    print(f"\nVisible Real GUI Windows count: {len(visible_windows)}")
    anki_window = None
    for w in visible_windows:
        # Check if pid matches anki or children/parent, or title/class
        print(f"HWND {w['hwnd']} | PID {w['pid']} | Class: '{w['class_name']}' | Title: '{w['title']}' | Geom: {w['geometry']}")
        if "anki" in w['title'].lower() or "studylab" in w['title'].lower() or "qt" in w['class_name'].lower() or w['pid'] == anki_pid:
            anki_window = w
            
    fg_hwnd = user32.GetForegroundWindow()
    fg_info = WindowForensicsEngine.inspect_hwnd(fg_hwnd)
    print(f"\nForeground HWND: {fg_hwnd} | Title: '{fg_info.get('title')}' | PID: {fg_info.get('pid')}")
    
    # 3. CDP Targets
    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    print(f"\nDiscovered {len(targets)} CDP Targets:")
    for t in targets:
        print(f"  Target ID: {t.id} | Type: {t.type} | Title: '{t.title}' | URL: {t.url}")
        
    # 4. Connect to main webview
    main_target = next((t for t in targets if "main webview" in t.title.lower()), None)
    if not main_target:
        main_target = targets[0]
        
    session = await mgr.switch_target(main_target)
    await session.enable_domains(["DOM", "Runtime", "Page"])
    
    # Inspect DOM
    dom_summary = await session.evaluate_js("""
        ({
            url: window.location.href,
            title: document.title,
            bodyClass: document.body.className,
            qaContent: document.getElementById('qa') ? document.getElementById('qa').innerText.substring(0, 300) : null,
            hasProceduralCard: !!document.getElementById('procedural-card'),
            cardType: document.getElementById('procedural-card') ? document.getElementById('procedural-card').getAttribute('data-object-type') : null,
            fullHtmlLength: document.documentElement.outerHTML.length
        })
    """)
    print(f"\nMain Webview State:\n{json.dumps(dom_summary, indent=2)}")
    
    # Capture initial screenshot
    os.makedirs(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit", exist_ok=True)
    screenshot_bytes = await session.capture_screenshot()
    with open(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\initial_live_cdp.png", "wb") as f:
        f.write(screenshot_bytes)
    print("Saved initial CDP screenshot to artifacts_qa/audit/initial_live_cdp.png")
    
    if anki_window:
        native_bytes = WindowForensicsEngine.capture_native_window_screenshot(anki_window['hwnd'])
        if native_bytes:
            with open(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\initial_live_native.png", "wb") as f:
                f.write(native_bytes)
            print("Saved initial Native HWND screenshot to artifacts_qa/audit/initial_live_native.png")

asyncio.run(inspect_live())
