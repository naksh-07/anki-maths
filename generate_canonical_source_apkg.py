#!/usr/bin/env python3
"""
StudyLab Canonical Source APKG Generator
Creates a deterministic canonical APKG fixture adhering strictly to StudyLab-Source-APKG-Contract(1).txt
with MCQ, Numerical, optional-field, full-provenance, and media cases.
"""

import os
import sys
import time
import json
import zipfile
import sqlite3
import hashlib
import random
import string
import tempfile
import re

NOTETYPE_NAME = "StudyLab Source"

def gen_guid():
    chars = string.ascii_letters + string.digits
    return "".join(random.choice(chars) for _ in range(10))

def field_checksum(s):
    clean = re.sub(r'<[^>]+>', '', s).strip()
    return int(hashlib.sha1(clean.encode('utf-8')).hexdigest()[:8], 16)

CANONICAL_FIELDS = [
    {"name": "Prompt", "ord": 0},
    {"name": "QuestionType", "ord": 1},
    {"name": "Options", "ord": 2},
    {"name": "CorrectAnswer", "ord": 3},
    {"name": "Difficulty", "ord": 4},
    {"name": "Subject", "ord": 5},
    {"name": "Chapter", "ord": 6},
    {"name": "Topic", "ord": 7},
    {"name": "Skill", "ord": 8},
    {"name": "ProblemType", "ord": 9},
    {"name": "Hint", "ord": 10},
    {"name": "Solution", "ord": 11},
    {"name": "Steps", "ord": 12},
    {"name": "Explanation", "ord": 13},
    {"name": "Source", "ord": 14},
    {"name": "Exam", "ord": 15},
    {"name": "Year", "ord": 16},
    {"name": "Shift", "ord": 17},
    {"name": "Paper", "ord": 18},
    {"name": "SourceQuestionID", "ord": 19},
]

SAMPLE_QUESTIONS = [
    {
        "guid": "src_mcq_0001",
        "Prompt": "If a train traveling at 72 km/h crosses a 200m platform in 25 seconds, what is the length of the train?",
        "QuestionType": "mcq",
        "Options": json.dumps(["150m", "200m", "300m", "350m"]),
        "CorrectAnswer": "300m",
        "Difficulty": "2.5",
        "Subject": "mathematics",
        "Chapter": "Arithmetic",
        "Topic": "Time Speed Distance",
        "Skill": "math.arithmetic.train_platform",
        "ProblemType": "standard",
        "Hint": "Convert speed from km/h to m/s by multiplying by 5/18.",
        "Solution": "Speed = 72 * (5/18) = 20 m/s. Total distance = 20 * 25 = 500m. Length of train = 500 - 200 = 300m.",
        "Steps": json.dumps(["Convert 72 km/h to 20 m/s", "Calculate total distance = 20 * 25 = 500m", "Subtract platform length: 500 - 200 = 300m"]),
        "Explanation": "Total distance covered while crossing platform is length of train plus length of platform.",
        "Source": "Official PYQ Corpus",
        "Exam": "RRB ALP",
        "Year": "2024",
        "Shift": "Shift 1",
        "Paper": "CBT-1",
        "SourceQuestionID": "RRB_ALP_2024_S1_Q42",
    },
    {
        "guid": "src_num_0002",
        "Prompt": "Calculate the force (in Newtons) required to accelerate a 5 kg mass at 8.5 m/s^2.",
        "QuestionType": "numerical",
        "Options": "",
        "CorrectAnswer": "42.5",
        "Difficulty": "2.0",
        "Subject": "physics",
        "Chapter": "Dynamics",
        "Topic": "Newton's Laws",
        "Skill": "physics.dynamics.f_ma",
        "ProblemType": "standard",
        "Hint": "Apply Newton's Second Law: F = m * a.",
        "Solution": "F = 5 kg * 8.5 m/s^2 = 42.5 N.",
        "Steps": json.dumps(["Identify mass m = 5 kg and acceleration a = 8.5 m/s^2", "Compute F = 5 * 8.5 = 42.5"]),
        "Explanation": "Direct application of F = ma.",
        "Source": "Physics Mastery",
        "Exam": "JEE Main",
        "Year": "2023",
        "Shift": "Morning",
        "Paper": "Paper 1",
        "SourceQuestionID": "JEE_2023_M_PHY_12",
    },
    {
        "guid": "src_opt_0003",
        "Prompt": "Which planet in the solar system is closest to the Sun?",
        "QuestionType": "mcq",
        "Options": json.dumps(["Venus", "Mercury", "Earth", "Mars"]),
        "CorrectAnswer": "Mercury",
        "Difficulty": "1.0",
        "Subject": "reasoning",
        "Chapter": "General Knowledge",
        "Topic": "Solar System",
        "Skill": "",
        "ProblemType": "",
        "Hint": "",
        "Solution": "",
        "Steps": "",
        "Explanation": "",
        "Source": "",
        "Exam": "",
        "Year": "",
        "Shift": "",
        "Paper": "",
        "SourceQuestionID": "",
    },
    {
        "guid": "src_med_0004",
        "Prompt": "Identify the angle marked in the geometric diagram: <img src=\"studylab_diagram.png\">",
        "QuestionType": "mcq",
        "Options": json.dumps(["30°", "45°", "60°", "90°"]),
        "CorrectAnswer": "60°",
        "Difficulty": "2.0",
        "Subject": "mathematics",
        "Chapter": "Geometry",
        "Topic": "Triangles",
        "Skill": "math.geometry.triangles_angles",
        "ProblemType": "diagram_read",
        "Hint": "The sum of interior angles of an equilateral triangle is 180°.",
        "Solution": "Each angle in an equilateral triangle is 180 / 3 = 60°.",
        "Steps": json.dumps(["Observe triangle is equilateral", "180 / 3 = 60°"]),
        "Explanation": "Diagram shows equilateral triangle with equal sides.",
        "Source": "Geometry Foundation",
        "Exam": "SSC CGL",
        "Year": "2024",
        "Shift": "Shift 2",
        "Paper": "Tier 1",
        "SourceQuestionID": "SSC_2024_T1_GEO_05",
    },
    {
        "guid": "src_num_0005",
        "Prompt": "Find the value of x such that 3x + 15 = 45.",
        "QuestionType": "numerical",
        "Options": "",
        "CorrectAnswer": "10.0",
        "Difficulty": "1.5",
        "Subject": "mathematics",
        "Chapter": "Algebra",
        "Topic": "Linear Equations",
        "Skill": "math.algebra.linear_one_variable",
        "ProblemType": "standard",
        "Hint": "Subtract 15 from both sides, then divide by 3.",
        "Solution": "3x = 45 - 15 = 30 -> x = 10.",
        "Steps": json.dumps(["3x = 30", "x = 10"]),
        "Explanation": "Standard two-step linear equation.",
        "Source": "",
        "Exam": "",
        "Year": "",
        "Shift": "",
        "Paper": "",
        "SourceQuestionID": "",
    }
]

def generate_canonical_source_apkg(output_path: str, deck_name: str = "StudyLab Canonical Source Deck"):
    now_ms = int(time.time() * 1000)
    now_s = int(time.time())
    deck_id = 1700000000000
    model_id = 1700000000001

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)

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

        model = {
            "id": model_id,
            "name": NOTETYPE_NAME,
            "type": 0,
            "mod": now_s,
            "usn": -1,
            "sortf": 0,
            "did": deck_id,
            "tmpls": [
                {
                    "name": "Canonical Source Card",
                    "ord": 0,
                    "qfmt": "<div style='padding:20px;font-family:sans-serif;color:#6366f1'>Loading StudyLab Source Card...</div>{{Prompt}}",
                    "afmt": "{{CorrectAnswer}}",
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
            "css": ".card { font-family: sans-serif; font-size: 16px; color: #1e293b; background-color: #f8fafc; }",
            "latexPre": "\\documentclass[12pt]{article}\n\\special{papersize=3in,5in}\n\\usepackage[utf8]{inputenc}\n\\usepackage{amssymb,amsmath}\n\\pagestyle{empty}\n\\setlength{\\parindent}{0in}\n\\begin{document}\n",
            "latexPost": "\\end{document}",
            "latexsvg": False,
            "req": [[0, "all", [0]]]
        }

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
            str(deck_id): {
                "id": deck_id,
                "mod": now_s,
                "name": deck_name,
                "usn": -1,
                "collapsed": False,
                "browserCollapsed": False,
                "desc": "StudyLab Canonical Source Deck for End-to-End Testing",
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
            "curDeck": 1,
            "curModel": str(model_id),
            "collapseTime": 1200
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

        # 1x1 transparent PNG / simple sample diagram for media test
        dummy_png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4\x00\x00\x00\rIDATx\x9cc`\x00\x00\x00\x02\x00\x01H\xaf\xa4q\x00\x00\x00\x00IEND\xaeB`\x82"
        media_file_path = os.path.join(tmpdir, "studylab_diagram.png")
        with open(media_file_path, "wb") as f:
            f.write(dummy_png)

        media_map = {"0": "studylab_diagram.png"}

        for i, q in enumerate(SAMPLE_QUESTIONS):
            note_id = 1700000010000 + i
            card_id = 1700000020000 + i
            guid = q.get("guid") or gen_guid()
            
            field_values = [str(q.get(f["name"], "")) for f in CANONICAL_FIELDS]
            flds_str = "\x1f".join(field_values)
            sfld = field_values[0]
            csum = field_checksum(sfld)

            cur.execute(
                "INSERT INTO notes VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    note_id,
                    guid,
                    model_id,
                    now_s,
                    -1,
                    "studylab_canonical_source",
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
                    deck_id,
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

        with zipfile.ZipFile(output_path, "w", zipfile.ZIP_DEFLATED) as zf:
            zf.write(db_path, "collection.anki2")
            zf.writestr("media", json.dumps(media_map))
            zf.write(media_file_path, "0")

    print(f"[SUCCESS] Generated Canonical Source APKG at {output_path} ({len(SAMPLE_QUESTIONS)} questions)")

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "artifacts_qa/canonical_source_test_fixture.apkg"
    generate_canonical_source_apkg(out)
