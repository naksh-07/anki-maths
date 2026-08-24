@echo off
cd /d "c:\Users\Suraj\Documents\Antigravity\Anki-maths"
set ANKIDEV=1
set PYTHONWARNINGS=default
set PYTHONPYCACHEPREFIX=out\pycache
set QTWEBENGINE_REMOTE_DEBUGGING=9222
set QTWEBENGINE_CHROMIUM_FLAGS=--remote-allow-origins=http://localhost:9222,http://127.0.0.1:9222,https://chrome-devtools-frontend.appspot.com --no-sandbox
set ANKI_API_PORT=40000
set ANKI_API_HOST=127.0.0.1
set ANKI_SINGLE_INSTANCE_KEY=ankistudylab_dev_temp_1867120219
out\pyenv\Scripts\python.exe tools\run.py
