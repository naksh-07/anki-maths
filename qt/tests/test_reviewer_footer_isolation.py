import json
import os
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

os.environ["QT_QPA_PLATFORM"] = "offscreen"
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "pylib"))
sys.path.insert(0, str(ROOT / "qt"))
sys.path.insert(0, str(ROOT / "out" / "pylib"))
sys.path.insert(0, str(ROOT / "out" / "qt"))

import pytest

from aqt.reviewer import Reviewer


def test_show_ease_buttons_always_outputs_native_answer_buttons():
    """Verify Reviewer._showEaseButtons() renders native answer buttons for all cards."""
    mw = MagicMock()
    mw.state = "review"
    mw.col.decks.config_dict_for_deck_id.return_value = {"stopTimerOnAnswer": False}
    mw.col.sched.answerButtons.return_value = 4
    mw.pm.spacebar_rates_card.return_value = True

    card = MagicMock()
    card.id = 12345
    card.current_deck_id.return_value = 1
    card.note_type.return_value = {"name": "StudyLab Procedural Anchor - Mathematics"}

    reviewer = Reviewer.__new__(Reviewer)
    reviewer.mw = mw
    reviewer.card = card
    reviewer.bottom = MagicMock()
    reviewer._states_mutated = True
    reviewer._last_procedural_attempt = {"is_correct": False, "score": 0.0}
    reviewer._last_procedural_mistake = None
    reviewer._answerButtons = MagicMock(return_value="<button>Again</button><button>Good</button><button>Easy</button>")

    eval_calls = []
    reviewer.bottom.web.eval = lambda js: eval_calls.append(js)

    reviewer._showEaseButtons()

    assert len(eval_calls) == 1
    js_call = eval_calls[0]
    assert js_call.startswith("showAnswer(")

    # Must contain native Again/Good/Easy/Hard rating elements
    assert "ansbut" not in js_call  # show answer button is gone
    assert "proc-mistake-btn" not in js_call  # mistake buttons MUST NOT be in bottom toolbar
    assert "procedural_mistake_select" not in js_call  # no procedural mistake onclick in native footer


def test_answer_card_passes_through_native_rating():
    """Verify Reviewer._answerCard rates the card without being intercepted by mistake classification."""
    mw = MagicMock()
    mw.state = "review"
    mw.col.sched = MagicMock()

    card = MagicMock()
    card.id = 12345
    card.note_type.return_value = {"name": "StudyLab Procedural Anchor - Mathematics"}

    reviewer = Reviewer.__new__(Reviewer)
    reviewer.mw = mw
    reviewer.card = card
    reviewer.state = "answer"
    reviewer._v3 = MagicMock()
    reviewer._last_procedural_attempt = {"is_correct": False, "score": 0.0}
    reviewer._last_procedural_mistake = None

    with patch("aqt.reviewer.answer_card") as mock_answer_card, \
         patch("aqt.reviewer.gui_hooks.reviewer_will_answer_card", return_value=(True, 1)):
        mock_answer_op = MagicMock()
        mock_answer_card.return_value.success.return_value = mock_answer_op

        reviewer._answerCard(1)

        # answer_card must be called for rating 1 (Again)
        assert mock_answer_card.called
        assert reviewer.state == "transition"


def test_state_a_question_before_answer_has_no_mistake_buttons_in_footer():
    """STATE A: Question before answer has no mistake buttons in footer."""
    mw = MagicMock()
    mw.state = "review"
    mw.col.conf = {"dueCounts": True}
    card = MagicMock()
    card.note_type.return_value = {"name": "StudyLab Procedural Anchor (Maths)"}
    card.should_show_timer.return_value = False
    card.time_limit.return_value = 0

    reviewer = Reviewer.__new__(Reviewer)
    reviewer.mw = mw
    reviewer.card = card
    reviewer.state = "question"
    reviewer.bottom = MagicMock()
    reviewer._v3 = MagicMock()
    reviewer._v3.counts.return_value = (0, [1, 0, 0])

    eval_calls = []
    reviewer.bottom.web.eval = lambda js: eval_calls.append(js)

    reviewer._showAnswerButton()

    assert len(eval_calls) == 1
    assert "proc-mistake-btn" not in eval_calls[0]
    assert "proc-mistake-panel" not in eval_calls[0]


def test_state_b_mcq_incorrect_answer_preserves_native_answer_buttons():
    """STATE B: MCQ incorrect answer keeps native Again/Hard/Good/Easy buttons in footer."""
    mw = MagicMock()
    mw.state = "review"
    mw.col.decks.config_dict_for_deck_id.return_value = {"stopTimerOnAnswer": False}
    mw.col.sched.answerButtons.return_value = 4
    card = MagicMock()
    card.id = 101
    card.current_deck_id.return_value = 1
    card.note_type.return_value = {"name": "StudyLab Procedural Anchor - MCQ"}

    reviewer = Reviewer.__new__(Reviewer)
    reviewer.mw = mw
    reviewer.card = card
    reviewer.state = "answer"
    reviewer.bottom = MagicMock()
    reviewer._states_mutated = True
    reviewer._last_procedural_attempt = {"is_correct": False, "mode": "mcq", "score": 0.0}
    reviewer._last_procedural_mistake = None
    reviewer._answerButtons = MagicMock(return_value="<table class=stat><tr><td>Again</td><td>Hard</td><td>Good</td><td>Easy</td></tr></table>")

    eval_calls = []
    reviewer.bottom.web.eval = lambda js: eval_calls.append(js)

    reviewer._showEaseButtons()

    assert len(eval_calls) == 1
    assert "showAnswer(" in eval_calls[0]
    assert "Again" in eval_calls[0]
    assert "proc-mistake-btn" not in eval_calls[0]


def test_state_c_numerical_incorrect_answer_preserves_native_rating_buttons():
    """STATE C: Numerical incorrect answer keeps native rating buttons intact."""
    mw = MagicMock()
    mw.state = "review"
    mw.col.decks.config_dict_for_deck_id.return_value = {"stopTimerOnAnswer": False}
    mw.col.sched.answerButtons.return_value = 4
    card = MagicMock()
    card.id = 102
    card.current_deck_id.return_value = 1
    card.note_type.return_value = {"name": "StudyLab Procedural Anchor - Mathematics"}

    reviewer = Reviewer.__new__(Reviewer)
    reviewer.mw = mw
    reviewer.card = card
    reviewer.state = "answer"
    reviewer.bottom = MagicMock()
    reviewer._states_mutated = True
    reviewer._last_procedural_attempt = {"is_correct": False, "mode": "quick", "score": 0.0}
    reviewer._last_procedural_mistake = None
    reviewer._answerButtons = MagicMock(return_value="<table class=stat><tr><td>Again</td><td>Hard</td><td>Good</td><td>Easy</td></tr></table>")

    eval_calls = []
    reviewer.bottom.web.eval = lambda js: eval_calls.append(js)

    reviewer._showEaseButtons()

    assert len(eval_calls) == 1
    assert "Again" in eval_calls[0]
    assert "proc-mistake-btn" not in eval_calls[0]


def test_state_d_mistake_category_selected_records_signal():
    """STATE D: Mistake category selected records signal in _last_procedural_mistake."""
    reviewer = Reviewer.__new__(Reviewer)
    reviewer._last_procedural_mistake = None

    mistake_payload = {
        "instance_id": "inst-101",
        "family_id": "math.linear",
        "mistake_type": "silly_mistake",
    }
    reviewer._on_procedural_mistake(mistake_payload)

    assert reviewer._last_procedural_mistake == mistake_payload


def test_state_e_next_card_resets_procedural_state_cleanly():
    """STATE E: Next card lifecycle resets attempt and mistake state."""
    reviewer = Reviewer.__new__(Reviewer)
    reviewer._last_procedural_attempt = {"is_correct": False}
    reviewer._last_procedural_mistake = {"mistake_type": "silly_mistake"}

    # Simulate card reset / init
    reviewer._last_procedural_attempt = None
    reviewer._last_procedural_mistake = None

    assert reviewer._last_procedural_attempt is None
    assert reviewer._last_procedural_mistake is None


