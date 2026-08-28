#!/usr/bin/env python3
"""
StudyLab Demo APKG Generator v1.0
Constructs a canonical StudyLab Source APKG (studylab-demo-v1.0.apkg) strictly conforming to
StudyLab Source APKG Technical Contract v1.0.
"""

import os
import sys
import json
import time
import zipfile
import sqlite3
import hashlib
import re
import shutil
from datetime import datetime, timezone

NOTETYPE_NAME = "StudyLab Source"
SCHEMA_VERSION = "1.0"
PACKAGE_ID = "studylab-demo-source-v1.0"
SOURCE_CORE_VERSION = "1.0.0"

# Canonical 20 fields in exact authoritative order
CANONICAL_FIELDS = [
    {"name": "Prompt", "ord": 0},
    {"name": "Options", "ord": 1},
    {"name": "CorrectAnswer", "ord": 2},
    {"name": "Hint", "ord": 3},
    {"name": "Solution", "ord": 4},
    {"name": "Steps", "ord": 5},
    {"name": "Explanation", "ord": 6},
    {"name": "Subject", "ord": 7},
    {"name": "Chapter", "ord": 8},
    {"name": "Topic", "ord": 9},
    {"name": "Skill", "ord": 10},
    {"name": "ProblemType", "ord": 11},
    {"name": "QuestionType", "ord": 12},
    {"name": "Difficulty", "ord": 13},
    {"name": "Source", "ord": 14},
    {"name": "Exam", "ord": 15},
    {"name": "Year", "ord": 16},
    {"name": "Shift", "ord": 17},
    {"name": "Paper", "ord": 18},
    {"name": "SourceQuestionID", "ord": 19},
]

SUBJECT_CHAPTER_CONFIG = [
    {
        "subject": "Mathematics",
        "chapter": "Number System",
        "source_file": "demo/sources/mathematics/number_system.json",
        "deck_id": 1700000001001,
        "deck_name": "StudyLab Demo::Mathematics::Number System"
    },
    {
        "subject": "Physics",
        "chapter": "Motion",
        "source_file": "demo/sources/physics/motion.json",
        "deck_id": 1700000001002,
        "deck_name": "StudyLab Demo::Physics::Motion"
    },
    {
        "subject": "Chemistry",
        "chapter": "Mole Concept",
        "source_file": "demo/sources/chemistry/mole_concept.json",
        "deck_id": 1700000001003,
        "deck_name": "StudyLab Demo::Chemistry::Mole Concept"
    },
    {
        "subject": "Reasoning",
        "chapter": "Seating Arrangement",
        "source_file": "demo/sources/reasoning/seating_arrangement.json",
        "deck_id": 1700000001004,
        "deck_name": "StudyLab Demo::Reasoning::Seating Arrangement"
    }
]

def field_checksum(s: str) -> int:
    clean = re.sub(r'<[^>]+>', '', s).strip()
    return int(hashlib.sha1(clean.encode('utf-8')).hexdigest()[:8], 16)

def deterministic_guid(source_id: str, prompt: str) -> str:
    h = hashlib.sha256(f"{source_id}:{prompt}".encode('utf-8')).hexdigest()
    return f"sl_{h[:10]}"

def load_sources_and_audit():
    candidate_questions = []
    accepted_questions = []
    rejected_questions = []
    seen_ids = set()
    seen_prompts = set()

    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    for cfg in SUBJECT_CHAPTER_CONFIG:
        path = os.path.join(base_dir, cfg["source_file"])
        if not os.path.exists(path):
            raise FileNotFoundError(f"Source fixture not found: {path}")
        
        with open(path, "r", encoding="utf-8") as f:
            items = json.load(f)
            
        for q in items:
            candidate_questions.append(q)
            sqid = q.get("SourceQuestionID", "").strip()
            prompt = q.get("Prompt", "").strip()
            qtype = q.get("QuestionType", "").strip().lower()
            ans = q.get("CorrectAnswer", "").strip()

            if not sqid:
                rejected_questions.append({"item": q, "reason": "missing_source_question_id"})
                continue
            if sqid in seen_ids:
                rejected_questions.append({"item": q, "reason": f"duplicate_source_question_id_{sqid}"})
                continue
            if not prompt:
                rejected_questions.append({"item": q, "reason": "empty_prompt"})
                continue
            prompt_norm = re.sub(r'\s+', ' ', prompt.lower())
            if prompt_norm in seen_prompts:
                rejected_questions.append({"item": q, "reason": "duplicate_prompt"})
                continue
            if qtype not in ["mcq", "numerical"]:
                rejected_questions.append({"item": q, "reason": f"unsupported_question_type_{qtype}"})
                continue
            if not ans:
                rejected_questions.append({"item": q, "reason": "missing_correct_answer"})
                continue
            
            # MCQ options check
            if qtype == "mcq":
                opts = q.get("Options", [])
                if not isinstance(opts, list) or len(opts) < 2:
                    rejected_questions.append({"item": q, "reason": "invalid_mcq_options"})
                    continue

            # Passed audit
            seen_ids.add(sqid)
            seen_prompts.add(prompt_norm)
            q["_deck_id"] = cfg["deck_id"]
            q["_deck_name"] = cfg["deck_name"]
            accepted_questions.append(q)

    audit_report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "candidate_questions": len(candidate_questions),
        "accepted_questions": len(accepted_questions),
        "rejected_questions": len(rejected_questions),
        "rejection_reasons": [r["reason"] for r in rejected_questions]
    }

    report_dir = os.path.join(base_dir, "demo", "reports")
    os.makedirs(report_dir, exist_ok=True)
    with open(os.path.join(report_dir, "demo_distinctness_audit.json"), "w", encoding="utf-8") as f:
        json.dump(audit_report, f, indent=2)

    return accepted_questions, audit_report

def generate_demo_apkg(output_path: str = None):
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if output_path is None:
        output_path = os.path.join(base_dir, "demo", "output", "studylab-demo-v1.0.apkg")
    
    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)

    questions, audit = load_sources_and_audit()
    print(f"[AUDIT] Candidates: {audit['candidate_questions']}, Accepted: {audit['accepted_questions']}, Rejected: {audit['rejected_questions']}")

    now_dt = datetime.now(timezone.utc)
    now_iso = now_dt.isoformat()
    now_s = int(now_dt.timestamp())
    now_ms = int(now_dt.timestamp() * 1000)

    root_deck_id = 1700000001000
    model_id = 1700000000001

    import tempfile
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "collection.anki2")
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()

        cur.executescript("""
        CREATE TABLE col (
            id              integer primary key,
            crt             integer not null,
            mod             integer not null,
            scm             integer not null,
            ver             integer not null,
            dty             integer not null,
            usn             integer not null,
            ls              integer not null,
            conf            text not null,
            models          text not null,
            decks           text not null,
            dconf           text not null,
            tags            text not null
        );

        CREATE TABLE notes (
            id              integer primary key,
            guid            text not null,
            mid             integer not null,
            mod             integer not null,
            usn             integer not null,
            tags            text not null,
            flds            text not null,
            sfld            text not null,
            csum            integer not null,
            flags           integer not null,
            data            text not null
        );

        CREATE TABLE cards (
            id              integer primary key,
            nid             integer not null,
            did             integer not null,
            ord             integer not null,
            mod             integer not null,
            usn             integer not null,
            type            integer not null,
            queue           integer not null,
            due             integer not null,
            ivl             integer not null,
            factor          integer not null,
            reps            integer not null,
            lapses          integer not null,
            left            integer not null,
            odue            integer not null,
            odid            integer not null,
            flags           integer not null,
            data            text not null
        );

        CREATE TABLE revlog (
            id              integer primary key,
            cid             integer not null,
            usn             integer not null,
            ease            integer not null,
            ivl             integer not null,
            lastIvl         integer not null,
            factor          integer not null,
            time            integer not null,
            type            integer not null
        );

        CREATE TABLE graves (
            usn             integer not null,
            oid             integer not null,
            type            integer not null
        );

        CREATE INDEX ix_notes_usn on notes (usn);
        CREATE INDEX ix_cards_usn on cards (usn);
        CREATE INDEX ix_revlog_usn on revlog (usn);
        CREATE INDEX ix_cards_nid on cards (nid);
        CREATE INDEX ix_cards_sched on cards (did, queue, due);
        """)

        # Model definition with exact 20 canonical fields
        model = {
            "id": model_id,
            "name": NOTETYPE_NAME,
            "type": 0,
            "mod": now_s,
            "usn": -1,
            "sortf": 0,
            "did": root_deck_id,
            "tmpls": [
                {
                    "name": "StudyLab Canonical Source Card",
                    "ord": 0,
                    "qfmt": "<div class='studylab-source-card'><div class='prompt'>{{Prompt}}</div>{{#Options}}<div class='options'>{{Options}}</div>{{/Options}}</div>",
                    "afmt": "{{FrontSide}}<hr id='answer'><div class='correct-answer'><strong>Correct Answer:</strong> {{CorrectAnswer}}</div>{{#Hint}}<div class='hint'><strong>Hint:</strong> {{Hint}}</div>{{/Hint}}{{#Solution}}<div class='solution'><strong>Solution:</strong> {{Solution}}</div>{{/Solution}}{{#Steps}}<div class='steps'><strong>Steps:</strong> {{Steps}}</div>{{/Steps}}{{#Explanation}}<div class='explanation'><strong>Explanation:</strong> {{Explanation}}</div>{{/Explanation}}",
                    "bqfmt": "",
                    "bafmt": "",
                    "did": None,
                }
            ],
            "flds": [
                {
                    "name": f["name"],
                    "ord": f["ord"],
                    "sticky": False,
                    "rtl": False,
                    "font": "Arial",
                    "size": 20,
                    "description": "",
                    "plainText": False,
                    "collapsed": False,
                    "excludeFromSearch": False,
                    "media": []
                } for f in CANONICAL_FIELDS
            ],
            "css": ".card { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 16px; color: #1e293b; background-color: #f8fafc; padding: 16px; line-height: 1.5; }\n.prompt { font-size: 18px; font-weight: 600; margin-bottom: 12px; }\n.options { margin-top: 12px; padding: 10px; background: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px; white-space: pre-wrap; }\n.correct-answer { color: #16a34a; font-weight: 600; margin-top: 12px; }\n.hint, .solution, .steps, .explanation { margin-top: 10px; font-size: 14px; color: #475569; }",
            "latexPre": "\\documentclass[12pt]{article}\n\\special{papersize=3in,5in}\n\\usepackage[utf8]{inputenc}\n\\usepackage{amssymb,amsmath}\n\\pagestyle{empty}\n\\setlength{\\parindent}{0in}\n\\begin{document}\n",
            "latexPost": "\\end{document}",
            "latexsvg": False,
            "req": [[0, "all", [0]]]
        }

        # Deck hierarchy
        decks = {
            "1": {
                "id": 1,
                "mod": now_s,
                "name": "Default",
                "usn": 0,
                "collapsed": False,
                "browserCollapsed": False,
                "desc": "",
                "dyn": 0,
                "conf": 1,
                "extendNew": 0,
                "extendRev": 0,
                "lrnToday": [0, 0],
                "revToday": [0, 0],
                "newToday": [0, 0],
                "timeToday": [0, 0]
            },
            str(root_deck_id): {
                "id": root_deck_id,
                "mod": now_s,
                "name": "StudyLab Demo",
                "usn": -1,
                "collapsed": False,
                "browserCollapsed": False,
                "desc": "StudyLab Demo APKG Universe v1.0",
                "dyn": 0,
                "conf": 1,
                "extendNew": 0,
                "extendRev": 0,
                "lrnToday": [0, 0],
                "revToday": [0, 0],
                "newToday": [0, 0],
                "timeToday": [0, 0]
            }
        }

        for cfg in SUBJECT_CHAPTER_CONFIG:
            did = cfg["deck_id"]
            dname = cfg["deck_name"]
            decks[str(did)] = {
                "id": did,
                "mod": now_s,
                "name": dname,
                "usn": -1,
                "collapsed": False,
                "browserCollapsed": False,
                "desc": f"StudyLab Demo Subject: {cfg['subject']} - {cfg['chapter']}",
                "dyn": 0,
                "conf": 1,
                "extendNew": 0,
                "extendRev": 0,
                "lrnToday": [0, 0],
                "revToday": [0, 0],
                "newToday": [0, 0],
                "timeToday": [0, 0]
            }

        dconf = {
            "1": {
                "id": 1,
                "mod": 0,
                "name": "Default",
                "usn": 0,
                "maxTaken": 60,
                "autoplay": True,
                "timer": 0,
                "replayq": True,
                "new": {"bury": False, "delays": [1, 10], "initialFactor": 2500, "ints": [1, 4, 0], "order": 1, "perDay": 20},
                "rev": {"bury": False, "ease4": 1.3, "fuzz": 0.05, "ivlFct": 1, "maxIvl": 36500, "perDay": 200, "hardFactor": 1.2},
                "lapse": {"delays": [10], "leechAction": 0, "leechFails": 8, "minInt": 1, "mult": 0}
            }
        }

        conf = {
            "nextPos": 1,
            "estTimes": True,
            "activeDecks": [1],
            "sortType": "noteFld",
            "timeLim": 0,
            "sortBackwards": False,
            "addToCur": True,
            "curDeck": root_deck_id,
            "curModel": str(model_id),
            "collapseTime": 1200,
            "studylab_package_meta": {
                "SchemaVersion": SCHEMA_VERSION,
                "PackageID": PACKAGE_ID,
                "GeneratedAt": now_iso,
                "SourceCoreVersion": SOURCE_CORE_VERSION
            }
        }

        models_dict = {str(model_id): model}

        cur.execute(
            "INSERT INTO col VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                1,
                now_s,
                now_ms,
                now_ms,
                11,
                0,
                0,
                0,
                json.dumps(conf),
                json.dumps(models_dict),
                json.dumps(decks),
                json.dumps(dconf),
                json.dumps({})
            )
        )

        # Media handling
        media_src_dir = os.path.join(base_dir, "demo", "media")
        media_assets = [
            ("studylab_demo_motion_graph.png", os.path.join(media_src_dir, "studylab_demo_motion_graph.png")),
            ("studylab_demo_seating_diagram.png", os.path.join(media_src_dir, "studylab_demo_seating_diagram.png"))
        ]

        media_map = {}
        for idx, (filename, filepath) in enumerate(media_assets):
            if os.path.exists(filepath):
                media_map[str(idx)] = filename

        # Insert notes and cards
        for i, q in enumerate(questions):
            note_id = 1700000100000 + i
            card_id = 1700000200000 + i
            sqid = q.get("SourceQuestionID", f"Q_{i+1}")
            guid = deterministic_guid(sqid, q.get("Prompt", ""))

            # Format Options field: canonical newline separated choices
            raw_options = q.get("Options", "")
            if isinstance(raw_options, list):
                if raw_options:
                    options_str = "\n".join(str(opt) for opt in raw_options)
                else:
                    options_str = ""
            else:
                options_str = str(raw_options)

            # Format Steps field: JSON string if list
            raw_steps = q.get("Steps", "")
            if isinstance(raw_steps, list):
                steps_str = json.dumps(raw_steps) if raw_steps else ""
            else:
                steps_str = str(raw_steps)

            field_dict = {
                "Prompt": str(q.get("Prompt", "")),
                "Options": options_str,
                "CorrectAnswer": str(q.get("CorrectAnswer", "")),
                "Hint": str(q.get("Hint", "")),
                "Solution": str(q.get("Solution", "")),
                "Steps": steps_str,
                "Explanation": str(q.get("Explanation", "")),
                "Subject": str(q.get("Subject", "")),
                "Chapter": str(q.get("Chapter", "")),
                "Topic": str(q.get("Topic", "")),
                "Skill": str(q.get("Skill", "")),
                "ProblemType": str(q.get("ProblemType", "")),
                "QuestionType": str(q.get("QuestionType", "mcq")),
                "Difficulty": str(q.get("Difficulty", "")),
                "Source": str(q.get("Source", "")),
                "Exam": str(q.get("Exam", "")),
                "Year": str(q.get("Year", "")),
                "Shift": str(q.get("Shift", "")),
                "Paper": str(q.get("Paper", "")),
                "SourceQuestionID": sqid,
            }

            field_values = [field_dict[f["name"]] for f in CANONICAL_FIELDS]
            flds_str = "\x1f".join(field_values)
            sfld = field_values[0]
            csum = field_checksum(sfld)

            target_deck_id = q.get("_deck_id", root_deck_id)

            cur.execute(
                "INSERT INTO notes VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    note_id,
                    guid,
                    model_id,
                    now_s,
                    -1,
                    f"studylab_canonical_source {q.get('Subject', '').lower()}",
                    flds_str,
                    sfld,
                    csum,
                    0,
                    ""
                )
            )

            cur.execute(
                "INSERT INTO cards VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    card_id,
                    note_id,
                    target_deck_id,
                    0,
                    now_s,
                    -1,
                    0,
                    0,
                    i + 1,
                    0,
                    2500,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    ""
                )
            )

        conn.commit()
        conn.close()

        # Archive into .apkg zip
        with zipfile.ZipFile(output_path, "w", zipfile.ZIP_DEFLATED) as zf:
            zf.write(db_path, "collection.anki2")
            zf.writestr("media", json.dumps(media_map))
            for idx_str, filename in media_map.items():
                src = os.path.join(media_src_dir, filename)
                zf.write(src, idx_str)

    print(f"[SUCCESS] Generated StudyLab Demo APKG at {output_path} ({len(questions)} notes)")

    # Also mirror to dist/apkgs and artifacts_qa
    dist_path = os.path.join(base_dir, "dist", "apkgs", "studylab-demo-v1.0.apkg")
    qa_path = os.path.join(base_dir, "artifacts_qa", "canonical_source_test_fixture.apkg")
    os.makedirs(os.path.dirname(dist_path), exist_ok=True)
    os.makedirs(os.path.dirname(qa_path), exist_ok=True)
    shutil.copyfile(output_path, dist_path)
    shutil.copyfile(output_path, qa_path)
    print(f"[SYNC] Mirrored fixture to {dist_path} and {qa_path}")

    return output_path

if __name__ == "__main__":
    out_arg = sys.argv[1] if len(sys.argv) > 1 else None
    generate_demo_apkg(out_arg)
