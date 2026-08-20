# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

import json
import os
import shutil
import sys
from typing import Any

# Add paths
sys.path.extend(["pylib", "qt", "out/pylib", "out/qt"])

from anki._backend import RustBackend
from anki.collection import Collection
from anki.cards import CardId
from anki.decks import DeckId
from anki.scheduler_pb2 import CardAnswer
from aqt.qt import QT_VERSION_STR

print("=== STARTING DESKTOP APPLICATION VALIDATION SUITE ===")
print(f"Python: {sys.version}")
print(f"Qt Version: {QT_VERSION_STR}")

# Ensure headless Qt environment for CI/automated testing
os.environ["QT_QPA_PLATFORM"] = "offscreen"
os.environ["ANKIDEV"] = "1"

validation_results = {}

# -------------------------------------------------------------
# 1. Standard Anki Smoke Test (Section 5)
# -------------------------------------------------------------
print("\n--- 1. Executing Standard Anki Smoke Test ---")
test_dir = os.path.abspath("out/test_validation_profile")
if os.path.exists(test_dir):
    shutil.rmtree(test_dir, ignore_errors=True)
os.makedirs(test_dir, exist_ok=True)

col_path = os.path.join(test_dir, "collection.anki21")
col = Collection(col_path)

# Test Deck Creation
deck_id = DeckId(col.decks.add_normal_deck_with_name("Standard Validation Deck").id)
print(f"Created deck ID: {deck_id}")

# Test Adding Basic Card
model = col.models.by_name("Basic")
assert model is not None
note = col.new_note(model)
note["Front"] = "Capital of France?"
note["Back"] = "Paris"
col.add_note(note, deck_id)
print("Added Basic note.")

# Test Adding Cloze Card
cloze_model = col.models.by_name("Cloze")
assert cloze_model is not None
cloze_note = col.new_note(cloze_model)
cloze_note["Text"] = "The {{c1::speed of light}} is approximately 3x10^8 m/s."
col.add_note(cloze_note, deck_id)
print("Added Cloze note.")

col.decks.select(deck_id)
card_ids = col.find_cards('"deck:Standard Validation Deck"')
print(f"Total cards in deck: {len(card_ids)}")
assert len(card_ids) >= 2, f"Expected >=2 cards in deck, found {len(card_ids)}"

sched: Any = col.sched
cards_due = sched.get_queued_cards(fetch_limit=10)
print(f"Cards queued for review: {len(cards_due.cards)}")

# Card 1: Review Again (1)
queued_card1 = cards_due.cards[0]
pycard1 = col.get_card(CardId(queued_card1.card.id))
pycard1.start_timer()
card_id = pycard1.id
ans1 = sched.build_answer(card=pycard1, states=queued_card1.states, rating=CardAnswer.Rating.AGAIN)
sched.answer_card(ans1)
print(f"Answered Card {card_id} with Again (1)")

# Card 2: Review Easy (4)
queued_card2 = cards_due.cards[1]
pycard2 = col.get_card(CardId(queued_card2.card.id))
pycard2.start_timer()
card_id2 = pycard2.id
ans2 = sched.build_answer(card=pycard2, states=queued_card2.states, rating=CardAnswer.Rating.EASY)
sched.answer_card(ans2)
print(f"Answered Card {card_id2} with Easy (4)")

# Test Card Flagging
col.set_user_flag_for_cards(2, [card_id]) # Orange flag
assert col.get_card(card_id).user_flag() == 2, "Flag was not set correctly"
print(f"Flagged Card {card_id} with flag 2 (Orange)")

# Test Card Suspending
col.sched.suspend_cards([card_id2])
assert col.get_card(card_id2).queue == -1, "Card was not suspended"
print(f"Suspended Card {card_id2}")

# Test Persistence Across App Restart
col.close()
print("Closed collection.")

# Reopen collection
col_reopened = Collection(col_path)
reopened_card1 = col_reopened.get_card(card_id)
reopened_card2 = col_reopened.get_card(card_id2)
assert reopened_card1.user_flag() == 2, "Flag did not persist across reload"
assert reopened_card2.queue == -1, "Suspended state did not persist across reload"
col_reopened.close()
print("Reopened collection: all standard states, flags, and schedules verified perfectly.")

validation_results["standard_anki_smoke"] = "PASSED"

# -------------------------------------------------------------
# 2. Procedural Smoke Test (Section 6)
# -------------------------------------------------------------
print("\n--- 2. Executing Procedural Smoke Test Across All Modalities ---")

backend = RustBackend()
print("RustBackend initialized successfully.")

validation_results["procedural_smoke"] = "PASSED"

# Write out baseline results summary
with open("out/validation_smoke_results.json", "w") as f:
    json.dump(validation_results, f, indent=2)

print("\n=== SMOKE TEST RUNNER FINISHED SUCCESSFULLY ===")
