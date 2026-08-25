import ctypes
from ctypes import wintypes
import sys
sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")
from core.window_forensics import WindowForensicsEngine

user32 = ctypes.windll.user32
WINSTAENUMPROCA = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.LPCSTR, wintypes.LPARAM)
DESKTOPENUMPROCA = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.LPCSTR, wintypes.LPARAM)
DESKTOPENUMPROC = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

anki_pids = {12852, 10776, 16728, 15376, 15560, 24936}
found_windows = []

stations = []
def wincb(name, lparam):
    stations.append(name.decode('utf-8', 'ignore'))
    return True
user32.EnumWindowStationsA(WINSTAENUMPROCA(wincb), 0)

for st_name in stations:
    hwinsta = user32.OpenWindowStationA(st_name.encode('ascii'), False, 0x37F)
    if not hwinsta:
        continue
    user32.SetProcessWindowStation(hwinsta)
    desktops = []
    def dcb(dname, lparam):
        desktops.append(dname.decode('utf-8', 'ignore'))
        return True
    user32.EnumDesktopsA(hwinsta, DESKTOPENUMPROCA(dcb), 0)
    for dname in desktops:
        hdesk = user32.OpenDesktopA(dname.encode('ascii'), 0, False, 0x1FF)
        if not hdesk:
            continue
        def wcb(hwnd, lparam):
            pid_var = wintypes.DWORD()
            user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid_var))
            info = WindowForensicsEngine.inspect_hwnd(hwnd)
            if pid_var.value in anki_pids or 'anki' in (info.get('title') or '').lower() or 'studylab' in (info.get('title') or '').lower():
                found_windows.append((st_name, dname, info))
            return True
        user32.EnumDesktopWindows(hdesk, DESKTOPENUMPROC(wcb), 0)
        user32.CloseDesktop(hdesk)
    user32.CloseWindowStation(hwinsta)

print(f"Total Anki windows found across all desktops: {len(found_windows)}")
for st, d, w in found_windows:
    print(f"[{st}\\{d}] HWND={w['hwnd']}, PID={w['pid']}, Title='{w['title']}', Class='{w['class_name']}', Visible={w['is_visible']}, Geom={w['geometry']}")
