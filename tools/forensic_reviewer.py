"""
tools/forensic_reviewer.py — Non-Certifying Desktop WebView Diagnostic Utility

WARNING: NON-CERTIFYING DIAGNOSTIC TOOLING ONLY.
This script performs ad-hoc developer inspection of the QtWebEngine WebView.
It does NOT verify native OS desktop reality, HWND bounds, or physical actionability,
and MUST NEVER be used to claim desktop UI certification or a desktop PASS.
All authoritative desktop verification MUST use tools/verify_desktop_ui.py backed
by the desktop-webview-reviewer capability.
"""

import asyncio
import json
import os
import sys
import time
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

# Resolve desktop-webview-reviewer repository location
REPO_CANDIDATE = Path(r"C:\Users\Suraj\.gemini\antigravity\scratch\desktop-webview-reviewer")
if REPO_CANDIDATE.exists():
    sys.path.insert(0, str(REPO_CANDIDATE))

from core.models import Target, VerificationLevel
from core.session import CDPSession, MultiTargetSessionManager
from core.actions import WebviewActions
from core.assertions import WebviewAssertions
from core.evidence import EvidenceCollector


async def main():
    print("=" * 70)
    print("StudyLab Diagnostic Utility (NON-CERTIFYING / DEVELOPER DEBUG ONLY)")
    print("For authoritative desktop certification, use: python tools/verify_desktop_ui.py")
    print("=" * 70)

    mgr = MultiTargetSessionManager(host="127.0.0.1", port=9222, engine="qtwebengine")
    targets = mgr.list_targets()
    print(f"Found {len(targets)} active targets on port 9222:")
    for i, t in enumerate(targets):
        print(f"  [{i}] ID: {t.id} | Title: '{t.title}' | URL: {t.url}")

    main_target = next((t for t in targets if "main webview" in t.title.lower()), None)
    if not main_target:
        print("ERROR: main webview not found!")
        sys.exit(1)

    print(f"\nAttaching to main webview: {main_target.id} ({main_target.websocket_endpoint})...")
    session = await mgr.switch_target(main_target)
    actions = WebviewActions(session)
    assertions = WebviewAssertions(session)
    collector = EvidenceCollector(session)

    # 1. Inspect initial state
    title = await session.evaluate_js("document.title")
    url = await session.evaluate_js("window.location.href")
    body_text = await session.evaluate_js("document.body.innerText")
    print(f"Current main webview URL: {url}")
    print(f"Current main webview body snippet:\n{body_text[:300]}\n")

    # Check if we are in deckBrowser, overview, or review
    is_deckbrowser = await session.evaluate_js("document.getElementById('deckbrowser') !== null || document.querySelector('.deck') !== null")
    is_overview = await session.evaluate_js("document.getElementById('study') !== null")
    is_reviewer = await session.evaluate_js("document.getElementById('qa') !== null || document.getElementById('procedural-card') !== null")

    print(f"State Detection: deckBrowser={is_deckbrowser}, overview={is_overview}, reviewer={is_reviewer}")

    if is_overview:
        print("Clicking #study button to enter reviewer...")
        await actions.click("#study")
        await asyncio.sleep(1.5)
    elif is_deckbrowser:
        print("Selecting Math deck...")
        await actions.click("a.deck")
        await asyncio.sleep(1.0)
        print("Clicking #study button...")
        await actions.click("#study")
        await asyncio.sleep(1.5)

    # Re-inspect reviewer
    current_body = await session.evaluate_js("document.body.innerText")
    print("\n--- Reviewer DOM Snapshot ---")
    print(current_body[:400])

    is_proc = await session.evaluate_js("document.getElementById('procedural-card') !== null")
    print(f"\nIs Procedural Card currently active: {is_proc}")

    # Capture initial screenshot
    os.makedirs("artifacts_qa", exist_ok=True)
    initial_ss = "artifacts_qa/initial_reviewer_state.png"
    await collector.capture_screenshot_file(initial_ss)
    print(f"Captured initial screenshot to {initial_ss}")
    print("\n[DIAGNOSTIC COMPLETE] Result: NON_CERTIFYING_DIAGNOSTIC")
    print("NOTE: This diagnostic output does not provide desktop certification.")

    await session.close()
    await mgr.close_all()


if __name__ == "__main__":
    asyncio.run(main())
