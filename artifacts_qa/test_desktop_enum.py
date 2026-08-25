import ctypes
from ctypes import wintypes
import json
import sys

user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32

WINSTA_ALL = 0x37F
DESKTOP_ALL = 0x1FF

# Try to connect to WinSta0\Default
hwinsta = user32.OpenWindowStationW("WinSta0", False, WINSTA_ALL)
print(f"OpenWindowStation WinSta0: {hwinsta}")
if hwinsta:
    res = user32.SetProcessWindowStation(hwinsta)
    print(f"SetProcessWindowStation: {res}")
    
hdesk = user32.OpenDesktopW("Default", 0, False, DESKTOP_ALL)
print(f"OpenDesktop Default: {hdesk}")
if hdesk:
    res = user32.SetThreadDesktop(hdesk)
    print(f"SetThreadDesktop: {res}")

# Now test EnumDesktopWindows
DESKTOPENUMPROC = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
windows = []

def cb(hwnd, lparam):
    length = user32.GetWindowTextLengthW(hwnd)
    buf = ctypes.create_unicode_buffer(length + 1)
    user32.GetWindowTextW(hwnd, buf, length + 1)
    title = buf.value
    
    cls_buf = ctypes.create_unicode_buffer(256)
    user32.GetClassNameW(hwnd, cls_buf, 256)
    cls_name = cls_buf.value
    
    pid = wintypes.DWORD()
    user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
    
    vis = bool(user32.IsWindowVisible(hwnd))
    
    windows.append({'hwnd': hwnd, 'pid': pid.value, 'title': title, 'class': cls_name, 'vis': vis})
    return True

if hdesk:
    user32.EnumDesktopWindows(hdesk, DESKTOPENUMPROC(cb), 0)
else:
    user32.EnumWindows(DESKTOPENUMPROC(cb), 0)

print(f"Total desktop windows: {len(windows)}")
for w in windows:
    if w['vis'] and (w['title'] or w['pid'] in [24628, 24896]):
        print(f"HWND {w['hwnd']} | PID {w['pid']} | Class: {w['class']} | Title: {w['title']}")
