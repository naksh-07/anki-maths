import os
import sys
import json
from pathlib import Path

os.environ["QT_QPA_PLATFORM"] = "offscreen"
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "pylib"))
sys.path.insert(0, str(ROOT / "qt"))
sys.path.insert(0, str(ROOT / "out" / "pylib"))
sys.path.insert(0, str(ROOT / "out" / "qt"))

from anki.collection import Collection
from anki.cards import CardId
from anki.scheduler_pb2 import CardAnswer
from anki.import_export_pb2 import ImportAnkiPackageRequest, ImportAnkiPackageOptions

def main():
    print("=== STARTING FULL RUNTIME VALIDATION (HEADLESS) ===")
    
    test_dir = str(ROOT / "out" / "test_validation_profile_phase13")
    import shutil
    shutil.rmtree(test_dir, ignore_errors=True)
    os.makedirs(test_dir, exist_ok=True)
    
    import zipfile
    pkg_path = str(ROOT / "Procedural_StudyLab_Fixture.apkg")
    
    with zipfile.ZipFile(pkg_path, 'r') as z:
        z.extract("collection.anki2", test_dir)
        
    col_path = os.path.join(test_dir, "collection.anki2")
    col = Collection(col_path)
    
    # Upgrade to v3 scheduler because genanki generates legacy v1 collections
    if type(col.sched).__name__ == "DummyScheduler":
        try:
            col.upgrade_to_v2_scheduler()
        except:
            pass
        col.set_v3_scheduler(True)
    
    print("Fixture collection opened directly.")
    
    # Get deck
    deck_name = "StudyLab Procedural Fixture"
    deck_id = col.decks.id(deck_name)
    assert deck_id is not None
    
    col.decks.select(deck_id)
    all_cards = col.find_cards(f'"deck:{deck_name}"')
    print(f"Total cards in '{deck_name}': {len(all_cards)}")
    
    cards_due = col.sched.get_queued_cards(fetch_limit=10)
    print("Cards queued:", len(cards_due.cards))
    if len(cards_due.cards) == 0 and len(all_cards) > 0:
        # Force the card into the queue by making it a review due today
        card = col.get_card(all_cards[0])
        card.type = 1 # Review
        card.queue = 1 # Review
        card.due = 0 # Due today
        col.update_card(card)
        cards_due = col.sched.get_queued_cards(fetch_limit=10)
    
    queued_card = cards_due.cards[0]
    pycard = col.get_card(CardId(queued_card.card.id))
    pycard.start_timer()
    
    # Mock the Javascript `mutateNextCardStates` behavior by injecting the customData manually into the states proto
    import copy
    states = copy.deepcopy(queued_card.states)
    telemetry = {
        "mistakeType": "silly_mistake",
        "mode": "mathematics",
        "proceduralPerformance": {"score": 0.0},
        "proceduralRemediation": {"needed": True, "reason": "silly_mistake", "domain": "mathematics", "skillId": "math_multiplication", "schemaId": "2x2"},
        "attemptResult": {"isCorrect": False, "targetTimeMs": 30000, "timeTakenMs": 15000, "instanceId": "test-inst", "answer": "99"}
    }
    
    # Embed telemetry in the `again` state exactly like `procedural.ts` does
    custom_data_str = states.again.custom_data
    custom_data = json.loads(custom_data_str) if custom_data_str else {}
    custom_data["studylab"] = telemetry
    states.again.custom_data = json.dumps(custom_data)
    
    # Prepare DB schema requirements
    proc_db_path = os.path.join(test_dir, "collection.procedural")
    import sqlite3
    proc_conn = sqlite3.connect(proc_db_path)
    proc_cursor = proc_conn.cursor()
    
    # Read schema.rs and execute to create tables
    with open("rslib/procedural/src/storage/schema.rs", "r", encoding="utf8") as f:
        schema_sql = f.read()
    
    import re
    # Extract CREATE TABLE statements
    statements = re.findall(r'CREATE TABLE IF NOT EXISTS .*?\);', schema_sql, re.DOTALL)
    for stmt in statements:
        proc_cursor.execute(stmt)
        
    # Insert required skills, schemas, instances
    print("Checking DB contents before answering...")
    
    # 3. Setup Backend DB with the specific schema/skill expected by the Fixture
    import sqlite3
    proc_db_path = os.path.join(test_dir, "collection.procedural")
    proc_conn = sqlite3.connect(proc_db_path)
    proc_cursor = proc_conn.cursor()
    
    # Insert required skills, schemas, instances
    proc_cursor.execute("INSERT OR IGNORE INTO skills (id, domain, name, description, prerequisites, metadata, created_at) VALUES ('percentage.successive', 'mathematics', 'Percentage Successive', '', '[]', '{}', 0)")
    proc_cursor.execute("INSERT OR IGNORE INTO problem_families (id, skill_id, domain, name, template_ref, min_difficulty, max_difficulty, parameters_schema, metadata, created_at) VALUES ('family.math.percentage.successive', 'percentage.successive', 'mathematics', 'Percentage Successive Family', '', 1.0, 10.0, '{}', '{}', 0)")
    proc_cursor.execute("INSERT OR IGNORE INTO schemas (id, skill_id, problem_family_id, title, description, target_mastery, config, created_at) VALUES ('successive_percentage', 'percentage.successive', 'family.math.percentage.successive', 'Successive Percentage', '', 1.0, '{}', 0)")
    proc_cursor.execute("INSERT OR IGNORE INTO problem_instances (id, family_id, seed, parameters, rendered_prompt, correct_answer, metadata, created_at) VALUES ('test-inst', 'family.math.percentage.successive', 1, '{}', '', '99', '{}', 0)")
    proc_conn.commit()
    
    print("skills:", proc_cursor.execute("select id from skills").fetchall())
    print("families:", proc_cursor.execute("select id from problem_families").fetchall())
    print("schemas:", proc_cursor.execute("select id from schemas").fetchall())
    print("instances:", proc_cursor.execute("select id from problem_instances").fetchall())
    
    proc_conn.close()
    
    # --- WRONG ANSWER FLOW ---
    print("Simulating answer...")
    
    states = copy.deepcopy(queued_card.states)
    telemetry = {
        "mistakeType": "silly_mistake",
        "mode": "mathematics",
        "proceduralPerformance": {"score": 0.0},
        "attemptResult": {"isCorrect": False, "targetTimeMs": 45000, "timeTakenMs": 15000, "instanceId": "test-inst", "answer": "99"},
        "proceduralRemediation": {"needed": True, "domain": "mathematics", "skillId": "percentage.successive", "schemaId": "successive_percentage", "reason": "silly_mistake"}
    }
    # Check learner profile in backend!
    print("Checking database for persisted telemetry...")
    
    import sqlite3
    proc_db_path = os.path.join(test_dir, "collection.procedural")
    
    # Embed telemetry in the `again` state exactly like `procedural.ts` does
    custom_data_str = states.again.custom_data
    custom_data = json.loads(custom_data_str) if custom_data_str else {}
    custom_data["studylab"] = telemetry
    states.again.custom_data = json.dumps(custom_data)
    
    # Answer AGAIN
    ans = col.sched.build_answer(card=pycard, states=states, rating=CardAnswer.Rating.AGAIN)
    col.sched.answer_card(ans)
    
    # --- CORRECT ANSWER FLOW ---
    # We will NOT answer test-inst-2 yet! We want to render it and see the remediation!
    # Instead, we just check the backend for the FIRST attempt.
    
    # Check learner profile in backend!
    proc_conn = sqlite3.connect(proc_db_path)
    proc_cursor = proc_conn.cursor()
    
    attempts = proc_cursor.execute("select * from practice_attempts order by attempted_at asc").fetchall()
    print(f"Attempts found: {len(attempts)}")
    assert len(attempts) == 1, "Only first practice attempt should be recorded!"
    
    attempt1 = attempts[0]
    
    print(f"Recorded Attempt 1 (Wrong): {attempt1}")
    assert attempt1[6] == 0, "is_correct should be false (0)"
    assert attempt1[8] == 15000, "response_time_ms mismatch"
    
    # Check Error events
    errors = proc_cursor.execute("select * from error_events").fetchall()
    print(f"Error events found: {len(errors)}")
    assert len(errors) == 1, "Error event was not recorded for the wrong answer!"
    
    
    # Check Remediation Queue implicitly by fetching next card
    queued_cards2 = col.sched.get_queued_cards(fetch_limit=1, intraday_learning_only=False)
    assert len(queued_cards2.cards) > 0, "Next card should be queued!"
    next_card = queued_cards2.cards[0]
    
    # Render the next card to verify remediation payload injection
    from anki.cards import Card
    card_obj = col.get_card(next_card.card.id)
    html = card_obj.question(reload=True)
    print(f"Rendered next card HTML length: {len(html)}")
    
    with open("debug_rendered.html", "w", encoding="utf-8") as f:
        f.write(html)
        
    # Verify the transparency banner is present
    assert "proc-transparency-banner" in html, "Remediation transparency banner missing from next card HTML!"
    assert "remediation_message" in html, "remediation_message missing from JS payload!"
    
    # Check Skill States
    states = proc_cursor.execute("select * from skills").fetchall()
    print("Skills found:", states)
    
    # Wait, the table is skill_states!
    states = proc_cursor.execute("select * from skill_states where skill_id = 'percentage.successive'").fetchall()
    print(f"Skill states found: {len(states)}")
    assert len(states) == 1, "Skill state was not created or persisted!"
    state = states[0]
    print(f"Recorded Skill State: {state}")
    # state should have recent_attempts as json list
    
    tables = proc_cursor.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()
    print("Tables in procedural.db:", tables)
    
    proc_conn.close()
    col.close()
    print("=== SUCCESS: Full Backend Runtime Validated! ===")

if __name__ == "__main__":
    main()
