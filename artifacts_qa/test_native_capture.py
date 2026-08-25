import ctypes
from ctypes import wintypes
import os
import sys

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32

# Open interactive desktop
hwinsta = user32.OpenWindowStationW("WinSta0", False, 0x37F)
if hwinsta:
    user32.SetProcessWindowStation(hwinsta)
hdesk = user32.OpenDesktopW("Default", 0, False, 0x1FF)
if hdesk:
    user32.SetThreadDesktop(hdesk)

sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")
from core.window_forensics import WindowForensicsEngine

hwnd = 13895330
info = WindowForensicsEngine.inspect_hwnd(hwnd)
print("Inspect HWND 13895330:", info)

# Capture native screenshot
native_bytes = WindowForensicsEngine.capture_native_window_screenshot(hwnd)
print(f"Captured native screenshot bytes: {len(native_bytes) if native_bytes else 0}")
if native_bytes:
    os.makedirs(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit", exist_ok=True)
    with open(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\anki_native_proof.png", "wb") as f:
        f.write(native_bytes)
    print("Saved anki_native_proof.png")
