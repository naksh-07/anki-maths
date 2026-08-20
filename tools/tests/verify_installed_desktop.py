# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import os
import sys
import subprocess
import time
from pathlib import Path

def main():
    print("=" * 60)
    print("ANKI STUDYLAB DESKTOP INSTALLER & COEXISTENCE VALIDATION")
    print("=" * 60)

    repo_root = Path(__file__).parent.parent.parent.resolve()
    installer_msi = repo_root / "out" / "installer" / "dist" / "AnkiStudyLab-26.08.1-win-x64.msi"
    installed_dir = repo_root / "out" / "test_install_dir"
    installed_exe = installed_dir / "PFiles64" / "Anki StudyLab" / "AnkiStudyLab.exe"
    installed_packages = installed_dir / "PFiles64" / "Anki StudyLab" / "app_packages"
    test_profile_dir = repo_root / "out" / "test_profile"

    # Step 1: Installer Check
    print(f"\n[1] Checking Windows Installer MSI artifact:")
    print(f"    Path: {installer_msi}")
    assert installer_msi.exists(), f"MSI does not exist: {installer_msi}"
    msi_size = installer_msi.stat().st_size
    print(f"    Size: {msi_size:,} bytes ({msi_size / (1024*1024):.2f} MB)")
    assert msi_size > 100 * 1024 * 1024, "MSI file size appears unexpectedly small"
    print("    -> Installer artifact verified: PASS")

    # Step 2: Administrative Extraction to Test Directory
    print(f"\n[2] Extracting MSI to Test Installation Directory:")
    print(f"    Target Directory: {installed_dir}")
    if installed_dir.exists():
        import shutil
        shutil.rmtree(installed_dir, ignore_errors=True)
    installed_dir.mkdir(parents=True, exist_ok=True)
    
    extract_cmd = f'msiexec /a "{installer_msi}" /qn TARGETDIR="{installed_dir}"'
    res = subprocess.run(extract_cmd, shell=True, capture_output=True, text=True)
    print(f"    msiexec exit code: {res.returncode}")
    assert res.returncode == 0, f"msiexec extraction failed: {res.stderr}"

    # Step 3: Installed Directory & Executable Verification
    print(f"\n[3] Checking Installed Application Directory:")
    print(f"    Installed Exe: {installed_exe}")
    assert installed_exe.exists(), f"AnkiStudyLab.exe not found at {installed_exe}"
    exe_size = installed_exe.stat().st_size
    print(f"    AnkiStudyLab.exe Size: {exe_size:,} bytes")
    assert installed_packages.exists(), f"app_packages not found at {installed_packages}"
    print("    -> Installed executable and packages verified: PASS")

    # Step 4: Coexistence & Isolation Verification
    print(f"\n[4] Verifying Windows Coexistence & Isolation:")
    # Verify installation directory is 'Anki StudyLab' and NOT 'Anki'
    official_dir = installed_dir / "PFiles64" / "Anki"
    assert not official_dir.exists(), f"Unexpected official Anki directory created: {official_dir}"
    print(f"    - Clean separation: No clash with official Anki ProgramFiles dir: PASS")
    
    # Verify default profile directory is AnkiStudyLab
    sys.path.insert(0, str(installed_packages))
    from aqt.profiles import ProfileManager
    default_base = ProfileManager._default_base()
    print(f"    - Default base directory: {default_base}")
    assert "AnkiStudyLab" in default_base, f"Expected AnkiStudyLab in default base, got {default_base}"
    assert not default_base.endswith("Anki2"), f"Expected default base not to be Anki2, got {default_base}"
    print(f"    - AppData isolation: %APPDATA%/AnkiStudyLab isolated from official Anki2: PASS")

    # Step 5: Launch Installed App with Clean Profile
    print(f"\n[5] Launching Installed AnkiStudyLab.exe with clean profile:")
    test_profile_dir.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["QT_QPA_PLATFORM"] = "offscreen"
    env["ANKIDEV"] = "1"
    
    proc = subprocess.Popen(
        [str(installed_exe), "-b", str(test_profile_dir)],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    print(f"    Process launched (PID: {proc.pid}). Waiting for initialization...")
    time.sleep(3)
    is_running = proc.poll() is None
    print(f"    Process running healthy: {is_running}")
    if is_running:
        proc.terminate()
        proc.wait(timeout=5)
        print("    -> Process gracefully terminated after successful launch test: PASS")
    else:
        out, err = proc.communicate()
        print(f"    Stdout: {out.decode('utf-8', errors='ignore')}")
        print(f"    Stderr: {err.decode('utf-8', errors='ignore')}")
        assert proc.returncode == 0, f"Process exited unexpectedly with code {proc.returncode}"

    # Step 6: Validate Standard Anki Reviewer in Clean Profile
    print(f"\n[6] Validating Standard Reviewer in Clean Profile:")
    from anki.collection import Collection
    from anki.cards import Card
    from anki.consts import QUEUE_TYPE_NEW, QUEUE_TYPE_REV, CARD_TYPE_NEW

    col_path = test_profile_dir / "test_col.anki2"
    if col_path.exists():
        col_path.unlink()

    col = Collection(str(col_path))
    try:
        notetype = col.models.by_name("Basic")
        assert notetype is not None, "Basic notetype not found in clean collection"
        note = col.new_note(notetype)
        note["Front"] = "Standard Capital Question: What is the capital of France?"
        note["Back"] = "Paris"
        col.add_note(note, deck_id=1)
        
        card_id = note.card_ids()[0]
        card = col.get_card(card_id)
        assert card.queue == QUEUE_TYPE_NEW, f"Expected card queue to be new, got {card.queue}"
        assert card.type == CARD_TYPE_NEW, f"Expected card type to be new, got {card.type}"
        
        rendered = card.render_output()
        assert "Standard Capital Question" in rendered.question_text
        assert "Paris" in rendered.answer_text
        print(f"    - Card front rendered: OK")
        print(f"    - Card back rendered: OK")

        from anki.scheduler.v3 import CardAnswer
        card.start_timer()
        queued = col.sched.get_queued_cards(fetch_limit=1)
        assert len(queued.cards) > 0, "Expected at least 1 queued card"
        queued_card = queued.cards[0]
        answer = col.sched.build_answer(card=card, states=queued_card.states, rating=CardAnswer.GOOD)
        col.sched.answer_card(answer)
        card_after = col.get_card(card_id)
        assert card_after.reps == 1, f"Expected reps to be 1, got {card_after.reps}"
        print(f"    - Standard Reviewer answer rating & transition: OK (reps={card_after.reps})")
    finally:
        col.close()
    print("    -> Standard Reviewer verified: PASS")

    # Step 7: Validate Procedural Reviewer & Math Engine
    print(f"\n[7] Validating Procedural Reviewer & Math Engine:")
    ui_val_script = repo_root / "tools" / "tests" / "desktop_qt_ui_validation.py"
    res = subprocess.run([sys.executable, str(ui_val_script)], capture_output=True, text=True)
    print(res.stdout)
    assert res.returncode == 0, f"Desktop UI validation failed:\n{res.stderr}"

    print("\n" + "=" * 60)
    print("ALL ANKI STUDYLAB VALIDATION CHECKS COMPLETED: PASS")
    print("=" * 60)

if __name__ == "__main__":
    main()
