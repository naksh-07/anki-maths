import os
import sys
import json
from pathlib import Path
import pytest
from unittest.mock import MagicMock

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "pylib"))
sys.path.insert(0, str(ROOT / "qt"))
sys.path.insert(0, str(ROOT / "out" / "pylib"))
sys.path.insert(0, str(ROOT / "out" / "qt"))

from aqt.reviewer import Reviewer
from anki.cards import Card


@pytest.fixture
def mock_reviewer(monkeypatch):
    monkeypatch.setattr("aqt.reviewer.ReviewerCardInfo", MagicMock())
    monkeypatch.setattr("aqt.reviewer.PreviousReviewerCardInfo", MagicMock())
    
    # Mock fluent tr calls in aqt.reviewer
    import aqt.reviewer
    monkeypatch.setattr(aqt.reviewer.tr, "_translate", lambda *args, **kwargs: "MockedText")

    mw = MagicMock()
    mw.col.sched_ver.return_value = 3
    mw.col.v3_scheduler.return_value = True
    mw.col.conf = {"dueCounts": True}
    mw.state = "review"

    rev = Reviewer(mw)
    rev.bottom = MagicMock()
    rev.web = MagicMock()
    rev._v3 = MagicMock()
    rev._v3.counts.return_value = (0, [1, 2, 3])
    return rev


def test_p0_a_procedural_card_suppresses_show_answer_button(mock_reviewer):
    """P0-A: Procedural cards must suppress the #ansbut button in the bottom bar."""
    card = MagicMock(spec=Card)
    card.note_type.return_value = {"name": "StudyLab Procedural Anchor (Maths)"}
    card.should_show_timer.return_value = False
    card.time_limit.return_value = 0

    mock_reviewer.card = card
    assert mock_reviewer._is_procedural_card() is True

    mock_reviewer._showAnswerButton()

    # Verify that showQuestion was evaluated on bottom webview
    assert mock_reviewer.bottom.web.eval.called
    eval_arg = mock_reviewer.bottom.web.eval.call_args[0][0]
    
    # Assert #ansbut is NOT present in the bottom HTML
    assert "id=\"ansbut\"" not in eval_arg
    assert "id='ansbut'" not in eval_arg
    assert "Show Answer" not in eval_arg


def test_p0_a_normal_card_renders_show_answer_button(mock_reviewer):
    """P0-A: Normal Basic/Cloze cards MUST continue to render #ansbut."""
    card = MagicMock(spec=Card)
    card.note_type.return_value = {"name": "Basic"}
    card.should_show_timer.return_value = False
    card.time_limit.return_value = 0

    mock_reviewer.card = card
    assert mock_reviewer._is_procedural_card() is False

    mock_reviewer._showAnswerButton()

    assert mock_reviewer.bottom.web.eval.called
    eval_arg = mock_reviewer.bottom.web.eval.call_args[0][0]
    
    # Assert #ansbut IS present in the bottom HTML for standard cards
    assert 'ansbut' in eval_arg
    assert 'pycmd(\\"ans\\")' in eval_arg or 'pycmd("ans")' in eval_arg


def test_p0_a_show_answer_delegates_without_dom_destruction(mock_reviewer):
    """P0-A: _showAnswer on procedural cards does not overwrite main card DOM."""
    card = MagicMock(spec=Card)
    card.note_type.return_value = {"name": "StudyLab Procedural Anchor (Maths)"}
    card.autoplay.return_value = False
    mock_reviewer.card = card
    mock_reviewer._showEaseButtons = MagicMock()

    mock_reviewer._showAnswer()

    # Verify handleNativeShowAnswer was invoked
    assert mock_reviewer.web.eval.called
    eval_arg = mock_reviewer.web.eval.call_args[0][0]
    assert "handleNativeShowAnswer" in eval_arg
    # Verify _showEaseButtons was NOT called on procedural cards (footer isolation)
    assert not mock_reviewer._showEaseButtons.called


def test_p0_b_procedural_answer_link_executes_answercard(mock_reviewer):
    """P0-B: procedural_answer:<ease> executes _answerCard(ease) correctly."""
    mock_reviewer._answerCard = MagicMock()

    mock_reviewer._linkHandler("procedural_answer:3")

    mock_reviewer._answerCard.assert_called_once_with(3)
    assert mock_reviewer.state == "answer"
