import subprocess
import time

import os

print("Launching Anki dev instance...")
env = os.environ.copy()
env['QTWEBENGINE_REMOTE_DEBUGGING'] = '9222'
anki_proc = subprocess.Popen(['out\\pyenv\\Scripts\\python.exe', 'tools\\run.py'], env=env)
print("Waiting for Anki to start...")
time.sleep(15)

print("Running simulator...")
res = subprocess.run(['out\\pyenv\\Scripts\\python.exe', 'longitudinal_simulator.py'])

print("Killing Anki...")
anki_proc.kill()
