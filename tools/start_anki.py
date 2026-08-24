import os
import sys
import subprocess
import time
import urllib.request
import json

app_dir = r"c:\Users\Suraj\Documents\Antigravity\Anki-maths"
python_exe = os.path.join(app_dir, r"out\pyenv\Scripts\python.exe")
run_script = os.path.join(app_dir, r"tools\run.py")

env = os.environ.copy()
env["ANKIDEV"] = "1"
env["PYTHONWARNINGS"] = "default"
env["PYTHONPYCACHEPREFIX"] = os.path.join(app_dir, r"out\pycache")
env["QTWEBENGINE_REMOTE_DEBUGGING"] = "9222"
env["QTWEBENGINE_CHROMIUM_FLAGS"] = "--remote-allow-origins=http://localhost:9222,http://127.0.0.1:9222,https://chrome-devtools-frontend.appspot.com --no-sandbox"
env["ANKI_API_PORT"] = "40000"
env["ANKI_API_HOST"] = "127.0.0.1"

DETACHED_PROCESS = 0x00000008
CREATE_NEW_PROCESS_GROUP = 0x00000200

flags = 0
if sys.platform == "win32":
    flags = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP

log_file = open(os.path.join(app_dir, "desktop_app.log"), "a", encoding="utf-8", errors="replace")

proc = subprocess.Popen(
    [python_exe, run_script],
    cwd=app_dir,
    env=env,
    stdout=log_file,
    stderr=log_file,
    stdin=subprocess.DEVNULL,
    creationflags=flags
)

print(f"Launched Anki detached PID: {proc.pid}")
with open(os.path.join(app_dir, "desktop_app.pid"), "w") as f:
    f.write(str(proc.pid))

with open(os.path.join(app_dir, "desktop_ownership.json"), "w") as f:
    json.dump({
        "pid": proc.pid,
        "create_time": time.time(),
        "launched_by_reviewer": True,
        "command": [python_exe, run_script],
        "port": 9222
    }, f, indent=2)

print("Waiting for debugging port 9222...")
connected = False
for i in range(25):
    time.sleep(1.0)
    try:
        req = urllib.request.urlopen("http://127.0.0.1:9222/json/list", timeout=1.0)
        data = json.loads(req.read().decode("utf-8"))
        print(f"Connected! Found {len(data)} target(s):")
        for t in data:
            print(f"  - [{t.get('type')}] '{t.get('title')}' -> {t.get('url')}")
        connected = True
        break
    except Exception as e:
        print(f"  [{i+1}/25] Waiting: {e}")

if not connected:
    print("FAILED to connect to port 9222!")
    sys.exit(1)
else:
    print("Anki dev instance is running and CDP endpoint is ready.")
