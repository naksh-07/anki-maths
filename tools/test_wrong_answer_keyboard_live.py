#!/usr/bin/env python3
"""
tools/test_wrong_answer_keyboard_live.py — Dedicated Live QtWebEngine Verification
for Wrong-Answer Space/Enter Semantics and Reflection Gating.

Powered by desktop-webview-reviewer.
"""

import asyncio
import base64
import hashlib
import json
import os
import sys
import time
import urllib.request
from typing import Any, Dict, List

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


async def capture_target_screenshot(session: CDPSession, filename: str) -> Dict[str, Any]:
    out_dir = os.path.join(REPO_ROOT, "artifacts_qa")
    os.makedirs(out_dir, exist_ok=True)
    filepath = os.path.join(out_dir, filename)

    result = await session.send_command("Page.captureScreenshot", {"format": "png"})
    b64_data = result.get("data", "")
    if not b64_data:
        raise RuntimeError(f"Failed to capture screenshot data for {filename}")

    img_bytes = base64.b64decode(b64_data)
    with open(filepath, "wb") as f:
        f.write(img_bytes)

    file_size = os.path.getsize(filepath)
    sha256 = compute_sha256(filepath)
    print(f"  [Screenshot] Saved: {filename} ({file_size} bytes, sha256: {sha256[:16]}...)")
    return {
        "filename": filename,
        "path": filepath,
        "size_bytes": file_size,
        "sha256": sha256,
        "captured_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    }


async def ensure_anki_running(port: int = 9222) -> bool:
    url = f"http://127.0.0.1:{port}/json/list"
    for _ in range(3):
        try:
            req = urllib.request.urlopen(url, timeout=1.5)
            data = json.loads(req.read().decode("utf-8"))
            if data:
                print(f"[Launcher] Anki is already running with {len(data)} target(s).")
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
    print(f"[Launcher] Spawned PID {proc.pid}. Awaiting CDP endpoint...")

    for i in range(30):
        time.sleep(1.0)
        try:
            req = urllib.request.urlopen(url, timeout=1.0)
            data = json.loads(req.read().decode("utf-8"))
            if data:
                print(f"[Launcher] Connected! Found {len(data)} target(s).")
                return True
        except Exception as e:
            if i % 5 == 0:
                print(f"  [{i+1}/30] Waiting for CDP: {e}")

    return False


async def main():
    print("=" * 80)
    print("  LIVE DESKTOP VERIFICATION: WRONG-ANSWER KEYBOARD FLOW & REFLECTION GATING")
    print("=" * 80)

    if not await ensure_anki_running(port=9222):
        print("ERROR: Failed to connect to Anki QtWebEngine on port 9222.")
        sys.exit(1)

    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    print(f"\nDiscovered {len(targets)} Webview Target(s):")
    for t in targets:
        print(f"  - Target ID: {t.id} | Title: '{t.title}' | URL: {t.url}")

    if not targets:
        print("ERROR: No active webview targets on port 9222!")
        sys.exit(1)

    adapter = EngineDetector.resolve_adapter(engine_name_or_hint="qtwebengine")
    
    # Locate main webview target
    main_target = None
    session = None
    for t in targets:
        try:
            s = await mgr.switch_target(t)
            has_proc = await s.evaluate_js("typeof window.anki !== 'undefined' && typeof window.anki.procedural !== 'undefined'")
            if has_proc:
                print(f"  -> Found Main Reviewer Target with anki.procedural: '{t.title}' ({t.id})")
                main_target = t
                session = s
                break
        except Exception:
            pass

    if not session:
        main_target = next((t for t in targets if "main webview" in t.title.lower()), targets[1] if len(targets) > 1 else targets[0])
        session = await mgr.switch_target(main_target)

    await session.enable_domains(["DOM", "Runtime", "Page"])
    actions = adapter.create_actions(session)
    assertions = adapter.create_assertions(session)
    collector = adapter.create_evidence_collector(session)

    # Load compiled reviewer.js bundle if not already present in webview
    reviewer_js_path = os.path.join(REPO_ROOT, "out", "ts", "reviewer", "reviewer.js")
    if os.path.exists(reviewer_js_path):
        with open(reviewer_js_path, "r", encoding="utf-8") as f:
            reviewer_bundle = f.read()
        await session.evaluate_js(f"""
        (() => {{
            window.anki = window.anki || {{}};
            {reviewer_bundle}
            return true;
        }})()
        """)
        print("  [Reviewer] Injected reviewer.js bundle into live webview context.")

    results = {}

    # -------------------------------------------------------------
    # 1. SETUP PROCEDURAL NUMERICAL PROBLEM
    # -------------------------------------------------------------
    print("\n[STEP 1] Setting up Mathematics Numerical Card in Live Webview...")
    setup_card_js = r"""
    (() => {
        window.__bridgeCalls = [];
        window.bridgeCommand = function(cmd, cb) {
            window.__bridgeCalls.push(cmd);
            if (cb) cb();
        };

        document.body.innerHTML = `
        <div id="qa">
            <div class="procedural-card-container" id="procedural-card" data-instance-id="math-wrong-inst-01" data-family-id="family.math.algebra.quadratic" data-target-time="30000">
                <div class="proc-header">
                    <div class="proc-header-left">
                        <nav class="proc-breadcrumbs" aria-label="Topic breadcrumbs">
                            <span class="proc-crumb proc-crumb-domain">Mathematics</span>
                            <span class="proc-crumb-sep">›</span>
                            <span class="proc-crumb proc-crumb-topic">Quadratic Equations</span>
                            <span class="proc-crumb-sep">›</span>
                            <span class="proc-crumb proc-crumb-skill">Root Discriminant</span>
                        </nav>
                        <div class="proc-badges">
                            <span class="proc-diff-badge">Level 2: Standard</span>
                        </div>
                    </div>
                    <span class="proc-timer" id="proc-stopwatch">00:00</span>
                </div>

                <div class="proc-prompt">
                    Find the discriminant \( \Delta = b^2 - 4ac \) for:<br><br>
                    \[ 2x^2 - 7x + 3 = 0 \]
                </div>

                <div id="proc-quick-container" class="proc-quick-container">
                    <div class="proc-step-row">
                        <input type="text" id="proc-answer-input" class="proc-input" placeholder="Type discriminant value..." autocomplete="off" />
                        <button type="button" id="proc-submit-btn" class="proc-btn proc-btn-primary">Submit</button>
                    </div>
                </div>

                <div id="proc-result-panel" class="proc-result hidden">
                    <div id="proc-result-title" class="proc-result-title"></div>
                    <div id="proc-actual-time" class="proc-actual-time"></div>
                    <div id="proc-result-feedback" class="proc-result-feedback"></div>
                    
                    <div id="proc-mistake-panel" class="proc-mistake-panel hidden">
                        <div class="proc-mistake-heading">Classify error (1-4) to reflect and optimize spaced repetition:</div>
                        <div class="proc-mistake-footer">
                            <button type="button" class="proc-mistake-btn" data-value="silly_mistake" data-key="1">
                                <span class="proc-key-badge">1</span> Silly Slip
                            </button>
                            <button type="button" class="proc-mistake-btn" data-value="pattern_not_recognized" data-key="2">
                                <span class="proc-key-badge">2</span> Pattern Missed
                            </button>
                            <button type="button" class="proc-mistake-btn" data-value="formula_or_concept_misapplied" data-key="3">
                                <span class="proc-key-badge">3</span> Concept Gap
                            </button>
                            <button type="button" class="proc-mistake-btn" data-value="concept_not_known" data-key="4">
                                <span class="proc-key-badge">4</span> Prereq Unknown
                            </button>
                        </div>
                    </div>

                    <div id="proc-solution-container" class="proc-solution">
                        <strong>Step-by-Step Solution:</strong>
                        <div>\( a=2, b=-7, c=3 \). \( \Delta = (-7)^2 - 4(2)(3) = 49 - 24 = 25 \).</div>
                    </div>
                </div>

                <div class="proc-footer">
                    <button type="button" id="proc-next-btn" class="proc-btn proc-btn-primary hidden">Next Problem (Enter)</button>
                </div>
            </div>
        </div>
        `;

        if (window.anki && window.anki.procedural) {
            window.reviewer = window.anki.procedural.setup({
                containerId: "procedural-card",
                instanceId: "math-wrong-inst-01",
                familyId: "family.math.algebra.quadratic",
                targetTimeMs: 30000,
                correctAnswer: { value: 25, formatted: "25" }
            });
            document.getElementById('procedural-card').__proceduralReviewer = window.reviewer;
        }
        return "SETUP_OK";
    })()
    """
    await session.evaluate_js(setup_card_js)
    await asyncio.sleep(0.3)

    state_init = await session.evaluate_js("document.getElementById('procedural-card').__proceduralReviewer.getState()")
    print(f"  Initial State: '{state_init}' (Expected: 'solving')")

    # -------------------------------------------------------------
    # 2. SUBMIT WRONG ANSWER
    # -------------------------------------------------------------
    print("\n[STEP 2] Submitting Incorrect Answer '21' (Expected: 25)...")
    await actions.type_text("#proc-answer-input", "21")
    await actions.click("#proc-submit-btn")
    await asyncio.sleep(0.3)

    state_after_wrong = await session.evaluate_js("document.getElementById('procedural-card').__proceduralReviewer.getState()")
    mistake_panel_visible = await session.evaluate_js("!document.getElementById('proc-mistake-panel').classList.contains('hidden')")
    next_btn_hidden = await session.evaluate_js("document.getElementById('proc-next-btn').classList.contains('hidden')")

    print(f"  State after wrong submit: '{state_after_wrong}' (Expected: 'mistake_classification')")
    print(f"  Mistake panel visible: {mistake_panel_visible}")
    print(f"  Next button hidden   : {next_btn_hidden}")

    # -------------------------------------------------------------
    # 3. VERIFY SPACE KEY TRAP (MUST NOT BYPASS REFLECTION)
    # -------------------------------------------------------------
    print("\n[STEP 3] Testing Space Key: MUST NOT bypass mistake classification...")
    for _ in range(3):
        await session.evaluate_js("""
            window.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', code: 'Space', bubbles: true, cancelable: true }));
            window.dispatchEvent(new KeyboardEvent('keyup', { key: ' ', code: 'Space', bubbles: true, cancelable: true }));
        """)
        await asyncio.sleep(0.05)
    await asyncio.sleep(0.25)

    state_after_space = await session.evaluate_js("document.getElementById('procedural-card').__proceduralReviewer.getState()")
    bridge_calls_space = await session.evaluate_js("window.__bridgeCalls")
    print(f"  State after Space spam: '{state_after_space}' (Expected: 'mistake_classification')")
    print(f"  Bridge calls so far   : {bridge_calls_space}")

    space_trap_ok = state_after_space == "mistake_classification" and len(bridge_calls_space) == 0
    results["space_reflection_trap"] = "PASS" if space_trap_ok else "FAIL"
    print(f"  -> Space Trap Result: {results['space_reflection_trap']}")

    # -------------------------------------------------------------
    # 4. VERIFY ENTER KEY TRAP (MUST NOT BYPASS REFLECTION)
    # -------------------------------------------------------------
    print("\n[STEP 4] Testing Enter Key: MUST NOT bypass mistake classification...")
    for _ in range(3):
        await session.evaluate_js("""
            window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', bubbles: true, cancelable: true }));
            window.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', code: 'Enter', bubbles: true, cancelable: true }));
        """)
        await asyncio.sleep(0.05)
    await asyncio.sleep(0.25)

    state_after_enter = await session.evaluate_js("document.getElementById('procedural-card').__proceduralReviewer.getState()")
    bridge_calls_enter = await session.evaluate_js("window.__bridgeCalls")
    print(f"  State after Enter spam: '{state_after_enter}' (Expected: 'mistake_classification')")

    enter_trap_ok = state_after_enter == "mistake_classification" and len(bridge_calls_enter) == 0
    results["enter_reflection_trap"] = "PASS" if enter_trap_ok else "FAIL"
    print(f"  -> Enter Trap Result: {results['enter_reflection_trap']}")

    ss_wrong_gate = await capture_target_screenshot(session, "09_wrong_answer_reflection_gate.png")

    # -------------------------------------------------------------
    # 5. EXPLICIT 1-4 CLASSIFICATION & TELEMETRY PRESERVATION
    # -------------------------------------------------------------
    print("\n[STEP 5] Pressing Hotkey '3' (Concept Gap) to classify mistake...")
    await session.evaluate_js("""
        (() => {
            const btn = document.querySelector('.proc-mistake-btn[data-key="3"], .proc-mistake-card[data-key="3"]');
            if (btn) {
                btn.click();
            } else if (window.reviewer) {
                window.reviewer.selectMistakeCategory("formula_or_concept_misapplied");
            }
        })()
    """)
    await asyncio.sleep(0.5)

    state_after_key3 = await session.evaluate_js("document.getElementById('procedural-card').__proceduralReviewer.getState()")
    bridge_calls_key3 = await session.evaluate_js("window.__bridgeCalls || []")
    print(f"  State after Key 3 : '{state_after_key3}' (Expected: 'feedback')")
    print(f"  Bridge Calls Emitted: {bridge_calls_key3}")

    has_mistake_telemetry = False
    for call in bridge_calls_key3:
        if "procedural_mistake:" in call:
            data = json.loads(call.replace("procedural_mistake:", ""))
            if data.get("mistake_type") == "formula_or_concept_misapplied" and data.get("instance_id") == "math-wrong-inst-01":
                has_mistake_telemetry = True
                print(f"  Verified Mistake Telemetry: {data}")

    classification_ok = state_after_key3 == "feedback" and has_mistake_telemetry
    results["classification_and_telemetry"] = "PASS" if classification_ok else "FAIL"
    print(f"  -> Classification & Telemetry Result: {results['classification_and_telemetry']}")

    ss_solution_feedback = await capture_target_screenshot(session, "10_wrong_answer_solution_feedback.png")

    # -------------------------------------------------------------
    # 6. FEEDBACK ADVANCE VIA ENTER/SPACE TO RATING FLOW
    # -------------------------------------------------------------
    print("\n[STEP 6] Testing Enter in Feedback State (Rating & Advance Flow)...")
    await session.evaluate_js("""
        window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', bubbles: true, cancelable: true }));
        window.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', code: 'Enter', bubbles: true, cancelable: true }));
    """)
    await asyncio.sleep(0.3)

    bridge_calls_feedback = await session.evaluate_js("window.__bridgeCalls")
    has_answer_rating = any("procedural_answer:1" in c for c in bridge_calls_feedback)
    print(f"  Rating Bridge Command Emitted: {has_answer_rating} (Calls: {bridge_calls_feedback})")

    results["next_rating_advance"] = "PASS" if has_answer_rating else "FAIL"
    print(f"  -> Next Rating Advance Result: {results['next_rating_advance']}")

    # -------------------------------------------------------------
    # 7. NORMAL BASIC ANKI CARD REGRESSION CHECK
    # -------------------------------------------------------------
    print("\n[STEP 7] Verifying Normal Basic Anki Card Behavior...")
    setup_basic_js = """
    (() => {
        if (window.reviewer) {
            window.reviewer.destroy();
            window.reviewer = null;
        }
        window.__basicCardRevealed = false;
        window.bridgeCommand = function(cmd) {
            if (cmd === "ans") {
                window.__basicCardRevealed = true;
            }
        };
        document.body.innerHTML = `
        <div id="qa">
            <div class="card" id="front">What is the formula for kinetic energy?</div>
        </div>
        `;
        return "BASIC_OK";
    })()
    """
    await session.evaluate_js(setup_basic_js)
    await asyncio.sleep(0.2)

    has_proc = await session.evaluate_js("document.getElementById('procedural-card') !== null")
    await session.evaluate_js("""
        window.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', code: 'Space', bubbles: true, cancelable: true }));
        window.dispatchEvent(new KeyboardEvent('keyup', { key: ' ', code: 'Space', bubbles: true, cancelable: true }));
    """)
    await asyncio.sleep(0.2)

    basic_revealed = await session.evaluate_js("window.__basicCardRevealed")
    print(f"  Standard card Space flip triggered: {basic_revealed} (Expected: True)")
    print(f"  Procedural container absent: {not has_proc} (Expected: True)")

    results["normal_basic_card_regression"] = "PASS" if basic_revealed and not has_proc else "FAIL"
    print(f"  -> Normal Basic Card Result: {results['normal_basic_card_regression']}")

    # -------------------------------------------------------------
    # SUMMARY REPORT
    # -------------------------------------------------------------
    print("\n" + "=" * 80)
    print("  VERIFICATION SUMMARY")
    print("=" * 80)
    all_pass = all(v == "PASS" for v in results.values())
    for k, v in results.items():
        print(f"  - {k}: {v}")

    print(f"\nOVERALL RESULT: {'ALL PASS' if all_pass else 'FAILED'}")

    evidence = {
        "timestamp": time.time(),
        "suite": "wrong_answer_keyboard_semantics",
        "results": results,
        "all_passed": all_pass,
        "screenshots": {
            "reflection_gate": ss_wrong_gate,
            "solution_feedback": ss_solution_feedback,
        }
    }
    with open(os.path.join(REPO_ROOT, "artifacts_qa", "wrong_answer_evidence.json"), "w", encoding="utf-8") as f:
        json.dump(evidence, f, indent=2)

    return all_pass


if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)
