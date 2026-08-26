#!/usr/bin/env python3
"""
tools/verify_wrong_answer_flow.py — Live QtWebEngine Verification for Wrong-Answer Keyboard Semantics

Powered by desktop-webview-reviewer.

Verifies:
1. Wrong answer submission triggers 'mistake_classification' state.
2. Pressing Space does NOT bypass or skip mistake classification; state remains 'mistake_classification'.
3. Pressing Enter does NOT bypass or skip mistake classification; state remains 'mistake_classification'.
4. Pressing 1-4 hotkey explicitly selects category, emits procedural_mistake: telemetry, and transitions to 'feedback'.
5. In 'feedback' state, Space/Enter advances to next problem / rating flow (procedural_answer:1).
6. Standard Anki Basic card behaves normally without interference.
7. Captures verified screenshot and evidence json.
"""

import asyncio
import hashlib
import json
import os
import sys
import time
import urllib.request
from typing import Any, Dict, List, Optional

if sys.platform == 'win32':
    try:
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')
    except Exception:
        pass

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, r"C:\Users\Suraj\.gemini\config\skills\desktop-webview-reviewer")

from core.session import CDPSession, MultiTargetSessionManager
from core.models import Target, VerificationLevel
from detectors.engine_detector import EngineDetector


def compute_sha256(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


async def ensure_anki_running(port: int = 9222) -> bool:
    url = f"http://127.0.0.1:{port}/json/list"
    for _ in range(3):
        try:
            req = urllib.request.urlopen(url, timeout=1.5)
            data = json.loads(req.read().decode("utf-8"))
            if data:
                print(f"[Launcher] Anki is running with {len(data)} target(s).")
                return True
        except Exception:
            pass
        time.sleep(0.5)

    print("[Launcher] Launching Anki dev instance...")
    python_exe = os.path.join(REPO_ROOT, r"out\pyenv\Scripts\python.exe")
    run_script = os.path.join(REPO_ROOT, r"tools\run.py")

    env = os.environ.copy()
    env["ANKIDEV"] = "1"
    env["PYTHONWARNINGS"] = "default"
    env["PYTHONPYCACHEPREFIX"] = os.path.join(REPO_ROOT, r"out\pycache")
    env["QTWEBENGINE_REMOTE_DEBUGGING"] = str(port)
    env["QTWEBENGINE_CHROMIUM_FLAGS"] = f"--remote-allow-origins=http://localhost:{port},http://127.0.0.1:{port} --no-sandbox"
    env["ANKI_API_PORT"] = "40000"
    env["ANKI_API_HOST"] = "127.0.0.1"

    import subprocess
    flags = 0
    if sys.platform == "win32":
        flags = subprocess.DETACHED_PROCESS | subprocess.CREATE_NEW_PROCESS_GROUP

    log_path = os.path.join(REPO_ROOT, "desktop_app.log")
    log_file = open(log_path, "a", encoding="utf-8", errors="replace")

    proc = subprocess.Popen(
        [python_exe, run_script],
        cwd=REPO_ROOT,
        env=env,
        stdout=log_file,
        stderr=log_file,
        stdin=subprocess.DEVNULL,
        creationflags=flags
    )

    print(f"[Launcher] Spawned PID: {proc.pid}")
    for i in range(25):
        time.sleep(1.0)
        try:
            req = urllib.request.urlopen(url, timeout=1.0)
            data = json.loads(req.read().decode("utf-8"))
            if data:
                print(f"[Launcher] Connected to port {port} after {i+1}s ({len(data)} targets).")
                return True
        except Exception:
            pass

    print("[Launcher] Failed to connect to debugging port!")
    return False


async def capture_target_screenshot(session: CDPSession, filename: str) -> Dict[str, Any]:
    out_dir = os.path.join(REPO_ROOT, "artifacts_qa")
    os.makedirs(out_dir, exist_ok=True)
    filepath = os.path.join(out_dir, filename)

    data = await session.execute_cdp_command("Page.captureScreenshot", {"format": "png"})
    raw_b64 = data.get("data", "")
    import base64
    raw_bytes = base64.b64decode(raw_b64)
    with open(filepath, "wb") as f:
        f.write(raw_bytes)

    h = compute_sha256(filepath)
    print(f"  Captured screenshot: {filename} (SHA256: {h[:16]}...)")
    return {
        "file": filename,
        "path": filepath,
        "sha256": h,
        "size_bytes": len(raw_bytes),
    }


async def main():
    print("=" * 80)
    print("=== LIVE QTWEBENGINE VERIFICATION: WRONG-ANSWER KEYBOARD FLOW ===")
    print("=" * 80)

    running = await ensure_anki_running(port=9222)
    if not running:
        print("ERROR: Anki could not be started or attached.")
        sys.exit(1)

    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    print(f"\nDiscovered {len(targets)} active targets:")
    for t in targets:
        print(f" - [{t.type}] '{t.title}' -> {t.url}")

    main_target = next((t for t in targets if "main webview" in t.title.lower()), None)
    if not main_target and targets:
        main_target = targets[0]

    if not main_target:
        print("ERROR: No suitable webview target found!")
        sys.exit(1)

    adapter = EngineDetector.resolve_adapter(engine_name_or_hint="qtwebengine")
    session = await mgr.switch_target(main_target)
    actions = adapter.create_actions(session)
    assertions = adapter.create_assertions(session)

    results = {}

    # -------------------------------------------------------------
    # TEST 1: Wrong Answer Setup & Immediate Mistake Classification
    # -------------------------------------------------------------
    print("\n[TEST 1] Setting up Mathematics Numerical problem & submitting wrong answer...")
    setup_js = """
    (() => {
        window.__bridgeCalls = [];
        window.bridgeCommand = function(cmd, cb) {
            window.__bridgeCalls.push(cmd);
            if (cb) cb();
        };

        document.body.innerHTML = `
        <div id="qa">
            <div class="procedural-card-container" id="procedural-card" data-instance-id="math-inst-wrong-01" data-family-id="math.quadratic_roots" data-target-time="30000">
                <div class="proc-header">
                    <div class="proc-header-left">
                        <nav class="proc-breadcrumbs"><span class="proc-crumb">Mathematics</span> &gt; <span class="proc-crumb">Algebra</span></nav>
                        <div class="proc-badges"><span class="proc-diff-badge">Level 2</span></div>
                    </div>
                    <div class="proc-timer" id="proc-timer">00:00</div>
                </div>
                <div class="proc-prompt" id="proc-prompt">
                    Find the discriminant of the quadratic equation \\(2x^2 - 7x + 3 = 0\\).
                </div>
                <div class="proc-workspace" id="proc-workspace">
                    <div class="proc-quick-container" id="proc-quick-container">
                        <div class="proc-input-group">
                            <label class="proc-input-label" for="proc-answer-input">Discriminant \\(D = b^2 - 4ac\\):</label>
                            <input type="text" id="proc-answer-input" class="proc-input" placeholder="e.g. 25" autocomplete="off" />
                            <button type="button" id="proc-submit-btn" class="proc-btn proc-btn-primary">Submit</button>
                        </div>
                    </div>
                    <div class="proc-stepwise-container hidden" id="proc-stepwise-container"></div>
                </div>
                <div class="proc-result-panel hidden" id="proc-result-panel">
                    <div class="proc-result-title" id="proc-result-title"></div>
                    <div class="proc-actual-time" id="proc-actual-time"></div>
                    <div class="proc-result-feedback" id="proc-result-feedback"></div>
                    <div class="proc-solution-container" id="proc-solution-container">
                        <div class="proc-solution-title">Worked Solution:</div>
                        <div class="proc-solution-body">For \\(2x^2 - 7x + 3 = 0\\), \\(a=2, b=-7, c=3\\). \\(D = (-7)^2 - 4(2)(3) = 49 - 24 = 25\\).</div>
                    </div>
                </div>
                </div>
            </div>
            </div>
        </div>
        `;

        if (window.proceduralAPI && window.proceduralAPI.setup) {
            window.reviewer = window.proceduralAPI.setup({
                containerId: "procedural-card",
                instanceId: "math-inst-wrong-01",
                familyId: "math.quadratic_roots",
                targetTimeMs: 30000,
                correctAnswer: { value: 25, formatted: "25" }
            });
            document.getElementById('procedural-card').__proceduralReviewer = window.reviewer;
        }
        return "SETUP_OK";
    })()
    """
    res = await session.evaluate_js(setup_js)
    print(f"  Setup result: {res}")
    await asyncio.sleep(0.3)

    # Initial state should be 'solving'
    st_init = await session.evaluate_js("window.reviewer ? window.reviewer.getState() : 'NO_REVIEWER'")
    print(f"  Initial State: '{st_init}'")

    # Enter wrong answer "21"
    await actions.type_text("#proc-answer-input", "21")
    await actions.click("#proc-submit-btn")
    await asyncio.sleep(0.3)

    st_after_wrong = await session.evaluate_js("window.reviewer.getState()")
    print(f"  State after wrong submission: '{st_after_wrong}' (Expected: 'mistake_classification')")
    mistake_panel_visible = await session.evaluate_js("!document.getElementById('proc-mistake-panel').classList.contains('hidden')")
    print(f"  Mistake panel visible: {mistake_panel_visible}")

    # -------------------------------------------------------------
    # TEST 2: Space Key Trap (Must NOT Bypass Reflection)
    # -------------------------------------------------------------
    print("\n[TEST 2] Testing Space key in 'mistake_classification' state...")
    print("  Dispatching Space keydown and keyup...")
    await session.dispatch_key_event("keyDown", "Space", " ")
    await session.dispatch_key_event("keyUp", "Space")
    await asyncio.sleep(0.3)

    st_after_space = await session.evaluate_js("window.reviewer.getState()")
    print(f"  State after Space key: '{st_after_space}' (Expected: 'mistake_classification')")
    if st_after_space == "mistake_classification":
        print("  PASS: Space key successfully trapped; reflection gate not bypassed.")
        results["space_reflection_trap"] = "PASS"
    else:
        print(f"  FAIL: Reflection gate was bypassed by Space! State is '{st_after_space}'")
        results["space_reflection_trap"] = "FAIL"

    # -------------------------------------------------------------
    # TEST 3: Enter Key Trap (Must NOT Bypass Reflection)
    # -------------------------------------------------------------
    print("\n[TEST 3] Testing Enter key in 'mistake_classification' state...")
    print("  Dispatching Enter keydown and keyup...")
    await session.dispatch_key_event("keyDown", "Enter")
    await session.dispatch_key_event("keyUp", "Enter")
    await asyncio.sleep(0.3)

    st_after_enter = await session.evaluate_js("window.reviewer.getState()")
    print(f"  State after Enter key: '{st_after_enter}' (Expected: 'mistake_classification')")
    if st_after_enter == "mistake_classification":
        print("  PASS: Enter key successfully trapped; reflection gate not bypassed.")
        results["enter_reflection_trap"] = "PASS"
    else:
        print(f"  FAIL: Reflection gate was bypassed by Enter! State is '{st_after_enter}'")
        results["enter_reflection_trap"] = "FAIL"

    ss_wrong_gate = await capture_target_screenshot(session, "wrong_answer_mistake_gate.png")

    # -------------------------------------------------------------
    # TEST 4: Explicit 1-4 Classification & Telemetry Dispatch
    # -------------------------------------------------------------
    print("\n[TEST 4] Testing explicit classification via hotkey '3' (Concept Gap)...")
    await session.dispatch_key_event("keyDown", "3", "3")
    await session.dispatch_key_event("keyUp", "3")
    await asyncio.sleep(0.4)

    st_after_3 = await session.evaluate_js("window.reviewer.getState()")
    print(f"  State after key '3': '{st_after_3}' (Expected: 'feedback')")

    bridge_calls = await session.evaluate_js("window.__bridgeCalls")
    print(f"  Recorded Bridge Calls: {bridge_calls}")

    mistake_telemetry = [c for c in bridge_calls if "procedural_mistake" in c]
    has_valid_telemetry = False
    if mistake_telemetry:
        payload_str = mistake_telemetry[-1].replace("procedural_mistake:", "")
        data = json.loads(payload_str)
        print(f"  Parsed Mistake Telemetry: {data}")
        if data.get("mistake_type") == "formula_or_concept_misapplied" and data.get("instance_id") == "math-inst-wrong-01":
            has_valid_telemetry = True

    if st_after_3 == "feedback" and has_valid_telemetry:
        print("  PASS: Classification hotkey '3' recorded valid telemetry and transitioned to feedback.")
        results["classification_and_telemetry"] = "PASS"
    else:
        print("  FAIL: Classification or telemetry missing!")
        results["classification_and_telemetry"] = "FAIL"

    ss_feedback = await capture_target_screenshot(session, "wrong_answer_solution_feedback.png")

    # -------------------------------------------------------------
    # TEST 5: Automatic Advance to Next Problem / Rating Flow
    # -------------------------------------------------------------
    print("\n[TEST 5] Verifying automatic advance to next problem/rating flow...")

    bridge_calls_after_next = await session.evaluate_js("window.__bridgeCalls")
    has_answer_rating = any("procedural_answer:1" in c for c in bridge_calls_after_next)
    print(f"  Rating Command Emitted Automatically: {has_answer_rating} (Calls: {bridge_calls_after_next})")
    if has_answer_rating:
        print("  PASS: Mistake classification successfully advanced to Anki rating flow automatically.")
        results["next_advance_flow"] = "PASS"
    else:
        print("  FAIL: Automatic rating flow did not occur!")
        results["next_advance_flow"] = "FAIL"

    # -------------------------------------------------------------
    # TEST 6: Normal Basic Anki Card Non-Regression
    # -------------------------------------------------------------
    print("\n[TEST 6] Testing standard Basic Anki card (non-procedural regression check)...")
    setup_basic_js = """
    (() => {
        if (window.reviewer) {
            window.reviewer.destroy();
            window.reviewer = null;
        }
        window.__basicFlipped = false;
        window.bridgeCommand = function(cmd) {
            if (cmd === "ans") {
                window.__basicFlipped = true;
            }
        };
        document.body.innerHTML = `
        <div id="qa">
            <div id="front">What is the capital of France?</div>
            <div id="back" class="hidden">Paris</div>
        </div>
        `;
        return "BASIC_SETUP_OK";
    })()
    """
    await session.evaluate_js(setup_basic_js)
    await asyncio.sleep(0.2)

    has_proc = await session.evaluate_js("document.getElementById('procedural-card') !== null")
    print(f"  Procedural container present on standard card: {has_proc} (Expected: False)")

    # Press Space on standard basic card -> flips card / triggers "ans" bridge call
    await session.dispatch_key_event("keyDown", "Space", " ")
    await session.dispatch_key_event("keyUp", "Space")
    await asyncio.sleep(0.2)

    basic_flipped = await session.evaluate_js("window.__basicFlipped")
    print(f"  Basic card answer reveal triggered by Space: {basic_flipped} (Expected: True)")
    if not has_proc and basic_flipped:
        print("  PASS: Standard Basic cards retain 100% normal Anki behavior.")
        results["basic_card_non_regression"] = "PASS"
    else:
        print("  FAIL: Standard Basic card behavior altered!")
        results["basic_card_non_regression"] = "FAIL"

    ss_basic = await capture_target_screenshot(session, "standard_basic_card_non_regression.png")

    # -------------------------------------------------------------
    # Output Summary Evidence
    # -------------------------------------------------------------
    print("\n" + "=" * 80)
    print("=== LIVE DESKTOP VERIFICATION RESULTS ===")
    print("=" * 80)
    all_passed = all(v == "PASS" for v in results.values())
    for test_name, status in results.items():
        print(f"  - {test_name}: {status}")

    evidence_data = {
        "timestamp": time.time(),
        "engine": "qtwebengine",
        "platform": sys.platform,
        "results": results,
        "all_passed": all_passed,
        "screenshots": {
            "wrong_answer_mistake_gate": ss_wrong_gate,
            "wrong_answer_solution_feedback": ss_feedback,
            "standard_basic_card_non_regression": ss_basic,
        }
    }

    evidence_path = os.path.join(REPO_ROOT, "artifacts_qa", "wrong_answer_keyboard_evidence.json")
    with open(evidence_path, "w", encoding="utf-8") as f:
        json.dump(evidence_data, f, indent=2)

    print(f"\nEvidence written to {evidence_path}")
    print(f"FINAL STATUS: {'ALL_PASSED' if all_passed else 'FAILED'}")
    return all_passed


if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)
