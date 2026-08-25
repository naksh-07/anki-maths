import ctypes
import os
import sys

user32 = ctypes.windll.user32
hwinsta = user32.OpenWindowStationW("WinSta0", False, 0x37F)
if hwinsta:
    user32.SetProcessWindowStation(hwinsta)
hdesk = user32.OpenDesktopW("Default", 0, False, 0x1FF)
if hdesk:
    user32.SetThreadDesktop(hdesk)

sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")
from core.window_forensics import WindowForensicsEngine

hwnd = 13895330
print("Restoring and setting foreground...")
user32.ShowWindow(hwnd, 9) # SW_RESTORE
WindowForensicsEngine.set_foreground_window(hwnd)

info = WindowForensicsEngine.inspect_hwnd(hwnd)
print("Inspect HWND after restore:", info)

os.makedirs(r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit", exist_ok=True)
out_path = r"c:\Users\Suraj\Documents\Antigravity\Anki-maths\artifacts_qa\audit\anki_native_restored.png"
success, sha, err = WindowForensicsEngine.capture_native_window_screenshot(hwnd, out_path)
print(f"Native capture success: {success}, SHA256: {sha}, Error: {err}")
