"""
generate_procedural_apkg.py — StudyLab Phase 12 Fixture Generator

Generates a deterministic Anki package (Procedural_StudyLab_Fixture.apkg)
containing real StudyLab Procedural Anchor cards.

The notetype MUST be named exactly "StudyLab Procedural Anchor" — this is
the string checked in rslib/src/notetype/render.rs:122 to intercept the card
and route it through render_procedural_anchor() -> ProceduralService ->
ProceduralReviewer.

Phase 26B Note:
This generator now supports `content_ref`. In production, APKGs must NOT embed 
massive payloads. Instead, they embed `content_ref` (e.g. `item-math-001`). 
Before reviewing such an APKG, the user MUST sync/import the associated StudyLab JSON 
content into the local Procedural Database. Otherwise, the engine will fail to resolve.

Schema IDs used are the production constants from
rslib/procedural/src/problems/catalog.rs:
  - SCHEMA_SUCCESSIVE_PERCENTAGE  = "successive_percentage"    (Mathematics)
  - SCHEMA_REASONING_SEATING      = "reasoning_seating_linear" (Reasoning)

Usage:
  python generate_procedural_apkg.py [output_path]
  python generate_procedural_apkg.py Procedural_StudyLab_Fixture.apkg
"""
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

import hashlib
import json
import os
import random
import re
import sqlite3
import string
import sys
import tempfile
import time
import zipfile

# ---------------------------------------------------------------------------
# Schema IDs — mirrors catalog.rs constants exactly
# ---------------------------------------------------------------------------
SCHEMA_MATH = "successive_percentage"        # SCHEMA_SUCCESSIVE_PERCENTAGE
SCHEMA_REASONING = "reasoning_seating_linear" # SCHEMA_REASONING_SEATING

# Notetype trigger name — must match render.rs:122 exactly
NOTETYPE_NAME = "StudyLab Procedural Anchor"
DECK_NAME = "StudyLab Procedural Fixture"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _gen_guid() -> str:
    chars = string.ascii_letters + string.digits + "!#$%&()*+,-./:;<=>?@[]^_`{|}~"
    return "".join(random.choice(chars) for _ in range(10))

def _field_checksum(s: str) -> int:
    clean = re.sub(r"<[^>]+>", "", s).strip()
    return int(hashlib.sha1(clean.encode("utf-8")).hexdigest()[:8], 16)

def _make_anchor_json(
    schema_id: str,
    seed: int | None = None,
    difficulty_override: float | None = None,
    inline_contract: dict | None = None,
) -> str:
    """Return the JSON string for the ProceduralPayload field."""
    anchor: dict = {"proc_schema": schema_id}
    if seed is not None:
        anchor["seed_mode"] = {"fixed": seed}
    if difficulty_override is not None:
        anchor["difficulty_override"] = difficulty_override
    if inline_contract is not None:
        anchor["inline_contract"] = inline_contract
    return json.dumps(anchor)

# ---------------------------------------------------------------------------
# Sample Rich Declarative Contracts (Phase 36B Content Factory)
# ---------------------------------------------------------------------------

def make_math_linear_eq_contract() -> dict:
    return {
        "contract": {
            "family_id": "family.math.algebra.linear_equations",
            "skill_id": "algebra.linear_equations",
            "domain": "mathematics",
            "default_schema": "schema.algebra.linear_equations.v1",
            "capability": "declarative",
            "min_difficulty": 1.0,
            "max_difficulty": 5.0,
            "supported_variants": ["two_step_basic"],
            "variant_categories": ["parameter"],
            "target_latency_model": {1: 25000, 2: 35000, 3: 50000},
            "structural_tags": ["algebra", "linear"],
            "decision_points": ["isolate_variable"],
            "error_categories": ["sign_error"],
            "prerequisites": [],
            "provenance": None,
            "metadata": {},
        },
        "archetypes": [
            {
                "archetype_id": "linear_eq.two_step_basic",
                "difficulty_level": 1,
                "variant_category": "parameter",
                "variant_name": "two_step_basic",
                "parameters": [
                    {"name": "a", "domain": {"type": "integer_range", "min": 2, "max": 8, "step": None, "non_zero": None}},
                    {"name": "x", "domain": {"type": "integer_range", "min": 1, "max": 12, "step": None, "non_zero": None}},
                    {"name": "b", "domain": {"type": "integer_range", "min": 1, "max": 15, "step": None, "non_zero": None}},
                    {"name": "c", "domain": {"type": "derived_linear", "a_param": "a", "x_param": "x", "b_param": "b"}},
                ],
                "constraints": [],
                "prompt_template": "Solve for \\(x\\):\n\n\\[ {a}x + {b} = {c} \\]",
                "answer_derivation": {
                    "type": "linear_two_step",
                    "c_param": "c",
                    "b_param": "b",
                    "a_param": "a",
                },
                "answer_formatted_template": "{answer}",
                "solution_template": "Subtract {b} from both sides: {a}x = {c_minus_b}, then divide by {a}: x = {answer}.",
                "step_nodes": [
                    {
                        "id": "step_isolate",
                        "step_type": "equation_rearrangement",
                        "label": "Isolate term",
                        "description_template": "Subtract {b} from both sides",
                        "expected_expression_template": "{a}x = {c_minus_b}",
                        "alternate_templates": [],
                        "hint_principle": "Inverse operation",
                        "hint_operation": "Subtract {b}",
                        "hint_intermediate": "{a}x = {c_minus_b}",
                    }
                ],
                "target_time_ms": 25000,
            }
        ],
    }

# ---------------------------------------------------------------------------
# Main builder
# ---------------------------------------------------------------------------

def create_procedural_apkg(output_path: str) -> None:
    now_ms = int(time.time() * 1000)
    now_s = int(time.time())
    deck_id = now_ms
    model_id = now_ms + 1

    css = """\
.card {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 16px;
    color: #1e293b;
    background: #f8fafc;
    padding: 20px;
}
.nightMode .card { color: #f1f5f9; background: #0f172a; }
"""
    # Minimal template - render_procedural_anchor() intercepts before this runs.
    q_tmpl = "{{ProceduralPayload}}"
    a_tmpl = q_tmpl

    model = {
        str(model_id): {
            "id": model_id,
            "name": NOTETYPE_NAME,
            "type": 0,
            "mod": now_s,
            "usn": -1,
            "sortf": 0,
            "did": deck_id,
            "tmpls": [{
                "name": "Procedural Card",
                "ord": 0,
                "qfmt": q_tmpl,
                "afmt": a_tmpl,
                "bqfmt": "",
                "bafmt": "",
                "did": None,
                "bfont": "",
                "bsize": 0,
            }],
            "flds": [
                {
                    "name": "ProceduralPayload",
                    "ord": 0,
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 14,
                    "description": "JSON anchor for procedural engine",
                    "plainText": True,
                    "collapsed": False,
                    "excludeFromSearch": False,
                    "media": [],
                },
                {
                    "name": "TopicTitle",
                    "ord": 1,
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 14,
                    "description": "",
                    "plainText": True,
                    "collapsed": False,
                    "excludeFromSearch": False,
                    "media": [],
                },
                {
                    "name": "Domain",
                    "ord": 2,
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 14,
                    "description": "",
                    "plainText": True,
                    "collapsed": False,
                    "excludeFromSearch": False,
                    "media": [],
                },
                {
                    "name": "Provenance",
                    "ord": 3,
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 14,
                    "description": "",
                    "plainText": True,
                    "collapsed": False,
                    "excludeFromSearch": False,
                    "media": [],
                }
            ],
            "css": css,
            "latexPre": "\\documentclass[12pt]{article}\n\\special{papersize=3in,5in}\n\\usepackage[utf8]{inputenc}\n\\usepackage{amssymb,amsmath}\n\\pagestyle{empty}\n\\setlength{\\parindent}{0in}\n\\begin{document}\n",
            "latexPost": "\\end{document}",
            "latexsvg": False,
            "req": [[0, "all", [0]]],
        }
    }

    decks = {
        "1": {
            "id": 1, "mod": now_s, "name": "Default", "usn": 0,
            "collapsed": False, "browserCollapsed": False, "desc": "",
            "dyn": 0, "conf": 1, "extendNew": 0, "extendRev": 0,
            "lrnToday": [0, 0], "revToday": [0, 0], "newToday": [0, 0], "timeToday": [0, 0],
        },
        str(deck_id): {
            "id": deck_id, "mod": now_s, "name": DECK_NAME, "usn": -1,
            "collapsed": False, "browserCollapsed": False,
            "desc": "Phase 36B fixture: StudyLab Universal Content Factory Procedural Anchor cards.",
            "dyn": 0, "conf": 1, "extendNew": 0, "extendRev": 0,
            "lrnToday": [0, 0], "revToday": [0, 0], "newToday": [0, 0], "timeToday": [0, 0],
        },
    }

    dconf = {
        "1": {
            "id": 1, "mod": 0, "name": "Default", "usn": 0,
            "maxTaken": 60, "autoplay": True, "timer": 0, "replayq": True,
            "new": {"bury": False, "delays": [1.0, 10.0], "initialFactor": 2500, "ints": [1, 4, 0], "order": 1, "perDay": 20},
            "rev": {"bury": False, "ease4": 1.3, "ivlFct": 1.0, "maxIvl": 36500, "perDay": 200, "hardFactor": 1.2},
            "lapse": {"delays": [10.0], "leechAction": 1, "leechFails": 8, "minInt": 1, "mult": 0.0},
            "dyn": False,
        }
    }

    conf = {
        "nextPos": 1, "estTimes": True, "activeDecks": [1],
        "sortType": "noteFld", "timeLim": 0, "sortBackwards": False,
        "addToCur": True, "curDeck": 1, "curModel": str(model_id), "collapseTime": 1200,
    }

    # Card definitions: (label, schema_id, seed, diff_override, inline_contract, tags)
    cards_data = [
        ("Math: Successive Percentage (Legacy proc_schema)", SCHEMA_MATH, 42, None, None, "StudyLab Math Fixture"),
        ("Reasoning: Linear Seating Arrangement (Legacy proc_schema)", SCHEMA_REASONING, None, None, None, "StudyLab Reasoning Fixture"),
        ("Math: Content Ref Path", SCHEMA_MATH, None, 2.0, None, "StudyLab Math Fixture ContentRef"),
        ("Math: Rich Declarative Contract (Linear Equations)", "schema.algebra.linear_equations.v1", 101, 1.0, make_math_linear_eq_contract(), "StudyLab Rich Contract Math"),
    ]

    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "collection.anki2")
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()

        cur.executescript("""\
CREATE TABLE col (id integer primary key, crt integer not null, mod integer not null,
    scm integer not null, ver integer not null, dty integer not null, usn integer not null,
    ls integer not null, conf text not null, models text not null, decks text not null,
    dconf text not null, tags text not null);
CREATE TABLE notes (id integer primary key, guid text not null, mid integer not null,
    mod integer not null, usn integer not null, tags text not null, flds text not null,
    sfld text not null, csum integer not null, flags integer not null, data text not null);
CREATE TABLE cards (id integer primary key, nid integer not null, did integer not null,
    ord integer not null, mod integer not null, usn integer not null, type integer not null,
    queue integer not null, due integer not null, ivl integer not null, factor integer not null,
    reps integer not null, lapses integer not null, left integer not null, odue integer not null,
    odid integer not null, flags integer not null, data text not null);
CREATE TABLE revlog (id integer primary key, cid integer not null, usn integer not null,
    ease integer not null, ivl integer not null, lastIvl integer not null,
    factor integer not null, time integer not null, type integer not null);
CREATE TABLE graves (usn integer not null, oid integer not null, type integer not null);
CREATE INDEX ix_notes_usn on notes (usn);
CREATE INDEX ix_cards_usn on cards (usn);
CREATE INDEX ix_revlog_usn on revlog (usn);
CREATE INDEX ix_cards_nid on cards (nid);
CREATE INDEX ix_cards_sched on cards (did, queue, due);
CREATE INDEX ix_revlog_cid on revlog (cid);
CREATE INDEX ix_notes_csum on notes (csum);
""")

        cur.execute("INSERT INTO col VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)", (
            1, now_s, now_ms, now_ms, 11, 0, 0, 0,
            json.dumps(conf), json.dumps(model),
            json.dumps(decks), json.dumps(dconf), json.dumps({}),
        ))

        note_id = now_ms + 100
        card_id = now_ms + 2000
        due = 1

        for label, schema_id, seed, diff_override, inline_contract, tags_str in cards_data:
            payload = _make_anchor_json(
                schema_id,
                seed=seed,
                difficulty_override=diff_override,
                inline_contract=inline_contract,
            )
            nid = note_id; note_id += 1
            guid = _gen_guid()
            flds = "\x1f".join([payload, label, "mathematics", "{}"])
            sfld = payload
            csum = _field_checksum(payload)
            tags = " " + tags_str.replace(" ", "_") + " "

            cur.execute("INSERT INTO notes VALUES (?,?,?,?,?,?,?,?,?,?,?)",
                        (nid, guid, model_id, now_s, -1, tags, flds, sfld, csum, 0, ""))

            cid = card_id; card_id += 1
            cur.execute("INSERT INTO cards VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                        (cid, nid, deck_id, 0, now_s, -1, 0, 0, due, 0, 2500, 0, 0, 0, 0, 0, 0, ""))
            due += 1

            print(f"  [card] {label}")
            print(f"         payload={payload}")

        conn.commit()
        conn.close()

        media_path = os.path.join(tmpdir, "media")
        with open(media_path, "w", encoding="utf-8") as f:
            f.write("{}")

        abs_out = os.path.abspath(output_path)
        parent = os.path.dirname(abs_out)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with zipfile.ZipFile(abs_out, "w", zipfile.ZIP_DEFLATED) as zf:
            zf.write(db_path, "collection.anki2")
            zf.write(media_path, "media")

    print(f"\n[OK] Generated {abs_out}  ({len(cards_data)} cards)")
    print(f"     Notetype : {NOTETYPE_NAME!r}")
    print(f"     Math     : {SCHEMA_MATH!r}")
    print(f"     Reasoning: {SCHEMA_REASONING!r}")


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "Procedural_StudyLab_Fixture.apkg"
    create_procedural_apkg(out)
