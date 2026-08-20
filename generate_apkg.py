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

def gen_guid():
    chars = string.ascii_letters + string.digits + "!#$%&()*+,-./:;<=>?@[]^_`{|}~"
    return "".join(random.choice(chars) for _ in range(10))

def field_checksum(s):
    clean = re.sub(r'<[^>]+>', '', s).strip()
    return int(hashlib.sha1(clean.encode('utf-8')).hexdigest()[:8], 16)

def create_apkg(output_path, deck_name="Math & Science - StudyLab Demo"):
    now_ms = int(time.time() * 1000)
    now_s = int(time.time())
    deck_id = now_ms
    basic_model_id = now_ms + 1
    cloze_model_id = now_ms + 2

    # Create temporary directory for sqlite db
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "collection.anki2")
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()

        # Create Schema
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
        CREATE INDEX ix_revlog_cid on revlog (cid);
        CREATE INDEX ix_notes_csum on notes (csum);
        """)

        # Decks JSON
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
                "desc": "Rich Math, LaTeX, MathJax, Cloze, and Science flashcards for testing Anki StudyLab.",
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

        # DConf JSON
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
                "new": {
                    "bury": False,
                    "delays": [1.0, 10.0],
                    "initialFactor": 2500,
                    "ints": [1, 4, 0],
                    "order": 1,
                    "perDay": 20
                },
                "rev": {
                    "bury": False,
                    "ease4": 1.3,
                    "ivlFct": 1.0,
                    "maxIvl": 36500,
                    "perDay": 200,
                    "hardFactor": 1.2
                },
                "lapse": {
                    "delays": [10.0],
                    "leechAction": 1,
                    "leechFails": 8,
                    "minInt": 1,
                    "mult": 0.0
                },
                "dyn": False
            }
        }

        css_style = """
.card {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 20px;
    text-align: center;
    color: #1e293b;
    background: linear-gradient(135deg, #f8fafc 0%, #edf2f7 100%);
    padding: 28px;
    border-radius: 12px;
    line-height: 1.6;
}
.nightMode .card {
    color: #f1f5f9;
    background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
}
.formula-box {
    background: rgba(99, 102, 241, 0.08);
    border: 1px solid rgba(99, 102, 241, 0.25);
    border-radius: 10px;
    padding: 16px 20px;
    margin: 14px auto;
    display: inline-block;
    max-width: 90%;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.04);
}
.nightMode .formula-box {
    background: rgba(99, 102, 241, 0.15);
    border-color: rgba(99, 102, 241, 0.4);
}
.badge {
    display: inline-block;
    padding: 4px 12px;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-radius: 20px;
    background: #6366f1;
    color: #ffffff;
    margin-bottom: 12px;
}
.cloze {
    font-weight: 700;
    color: #6366f1;
    background: rgba(99, 102, 241, 0.12);
    padding: 2px 8px;
    border-radius: 6px;
}
.nightMode .cloze {
    color: #818cf8;
    background: rgba(129, 140, 248, 0.2);
}
table.math-table {
    margin: 14px auto;
    border-collapse: collapse;
    text-align: left;
}
table.math-table td {
    padding: 8px 16px;
    border-bottom: 1px solid rgba(148, 163, 184, 0.3);
}
"""

        # Models JSON
        models = {
            str(basic_model_id): {
                "id": basic_model_id,
                "name": "Math & Science (Basic)",
                "type": 0,
                "mod": now_s,
                "usn": -1,
                "sortf": 0,
                "did": deck_id,
                "tmpls": [
                    {
                        "name": "Card 1",
                        "ord": 0,
                        "qfmt": "<div class='badge'>{{Tag}}</div><br>{{Front}}",
                        "afmt": "{{FrontSide}}\n\n<hr id=answer style='border: 0; height: 1px; background: rgba(99,102,241,0.3); margin: 20px 0;'>\n\n{{Back}}",
                        "bqfmt": "",
                        "bafmt": "",
                        "did": None,
                        "bfont": "",
                        "bsize": 0
                    }
                ],
                "flds": [
                    {"name": "Front", "ord": 0, "sticky": False, "rtl": False, "font": "Arial", "size": 20, "description": "", "plainText": False, "collapsed": False, "excludeFromSearch": False, "media": []},
                    {"name": "Back", "ord": 1, "sticky": False, "rtl": False, "font": "Arial", "size": 20, "description": "", "plainText": False, "collapsed": False, "excludeFromSearch": False, "media": []},
                    {"name": "Tag", "ord": 2, "sticky": False, "rtl": False, "font": "Arial", "size": 14, "description": "", "plainText": False, "collapsed": False, "excludeFromSearch": False, "media": []}
                ],
                "css": css_style,
                "latexPre": "\\documentclass[12pt]{article}\n\\special{papersize=3in,5in}\n\\usepackage[utf8]{inputenc}\n\\usepackage{amssymb,amsmath}\n\\pagestyle{empty}\n\\setlength{\\parindent}{0in}\n\\begin{document}\n",
                "latexPost": "\\end{document}",
                "latexsvg": False,
                "req": [[0, "all", [0]]]
            },
            str(cloze_model_id): {
                "id": cloze_model_id,
                "name": "Math & Science (Cloze)",
                "type": 1,
                "mod": now_s,
                "usn": -1,
                "sortf": 0,
                "did": deck_id,
                "tmpls": [
                    {
                        "name": "Cloze",
                        "ord": 0,
                        "qfmt": "<div class='badge'>{{Tag}}</div><br>{{cloze:Text}}",
                        "afmt": "<div class='badge'>{{Tag}}</div><br>{{cloze:Text}}<br><br><div style='font-size:16px; color:#64748b;'>{{Extra}}</div>",
                        "bqfmt": "",
                        "bafmt": "",
                        "did": None,
                        "bfont": "",
                        "bsize": 0
                    }
                ],
                "flds": [
                    {"name": "Text", "ord": 0, "sticky": False, "rtl": False, "font": "Arial", "size": 20, "description": "", "plainText": False, "collapsed": False, "excludeFromSearch": False, "media": []},
                    {"name": "Extra", "ord": 1, "sticky": False, "rtl": False, "font": "Arial", "size": 16, "description": "", "plainText": False, "collapsed": False, "excludeFromSearch": False, "media": []},
                    {"name": "Tag", "ord": 2, "sticky": False, "rtl": False, "font": "Arial", "size": 14, "description": "", "plainText": False, "collapsed": False, "excludeFromSearch": False, "media": []}
                ],
                "css": css_style,
                "latexPre": "\\documentclass[12pt]{article}\n\\special{papersize=3in,5in}\n\\usepackage[utf8]{inputenc}\n\\usepackage{amssymb,amsmath}\n\\pagestyle{empty}\n\\setlength{\\parindent}{0in}\n\\begin{document}\n",
                "latexPost": "\\end{document}",
                "latexsvg": False,
                "req": []
            }
        }

        # Conf JSON
        conf = {
            "nextPos": 1,
            "estTimes": True,
            "activeDecks": [1],
            "sortType": "noteFld",
            "timeLim": 0,
            "sortBackwards": False,
            "addToCur": True,
            "curDeck": 1,
            "curModel": str(basic_model_id),
            "collapseTime": 1200
        }

        # Insert Collection Record
        cur.execute("""
        INSERT INTO col VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            1,
            now_s,
            now_ms,
            now_ms,
            11,
            0,
            0,
            0,
            json.dumps(conf),
            json.dumps(models),
            json.dumps(decks),
            json.dumps(dconf),
            json.dumps({})
        ))

        # Define Sample Cards
        # Basic cards: (Front, Back, Tag)
        basic_cards_data = [
            (
                r"What is the quadratic formula for solving \( ax^2 + bx + c = 0 \) (where \( a \neq 0 \))?",
                r"<div class='formula-box'>\[ x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a} \]</div><br><small style='color:#64748b;'>Discriminant: \( \Delta = b^2 - 4ac \)</small>",
                "Algebra"
            ),
            (
                r"What is <b>Euler's Identity</b> and which 5 fundamental mathematical constants does it relate?",
                r"<div class='formula-box'>\[ e^{i\pi} + 1 = 0 \]</div><br>It connects the five fundamental constants: \( 0, 1, e, i, \pi \).",
                "Complex Numbers"
            ),
            (
                r"State the <b>Fundamental Theorem of Calculus</b> (Part 1 & Part 2):",
                r"<div class='formula-box'><b>Part 1:</b> \[ \frac{d}{dx} \int_{a}^{x} f(t)\,dt = f(x) \]<br><b>Part 2:</b> \[ \int_{a}^{b} f(x)\,dx = F(b) - F(a) \]</div>",
                "Calculus"
            ),
            (
                r"What is the <b>Time-Dependent Schrödinger Equation</b> in Quantum Mechanics?",
                r"<div class='formula-box'>\[ i\hbar \frac{\partial}{\partial t} \Psi(\mathbf{r}, t) = \hat{H} \Psi(\mathbf{r}, t) = \left[ -\frac{\hbar^2}{2m} \nabla^2 + V(\mathbf{r}, t) \right] \Psi(\mathbf{r}, t) \]</div>",
                "Quantum Physics"
            ),
            (
                r"Write <b>Maxwell's Equations</b> in vacuum (differential form):",
                r"""<table class='math-table'>
                <tr><td><b>Gauss's Law:</b></td><td>\( \nabla \cdot \mathbf{E} = \frac{\rho}{\varepsilon_0} \)</td></tr>
                <tr><td><b>Gauss (Magnetism):</b></td><td>\( \nabla \cdot \mathbf{B} = 0 \)</td></tr>
                <tr><td><b>Faraday's Law:</b></td><td>\( \nabla \times \mathbf{E} = -\frac{\partial \mathbf{B}}{\partial t} \)</td></tr>
                <tr><td><b>Ampère-Maxwell:</b></td><td>\( \nabla \times \mathbf{B} = \mu_0 \mathbf{J} + \mu_0 \varepsilon_0 \frac{\partial \mathbf{E}}{\partial t} \)</td></tr>
                </table>""",
                "Electromagnetism"
            ),
            (
                r"What is the continuous <b>Fourier Transform</b> and its Inverse?",
                r"<div class='formula-box'><b>Forward:</b> \[ \hat{f}(\xi) = \int_{-\infty}^{\infty} f(x) e^{-2\pi i x \xi} \, dx \]<br><b>Inverse:</b> \[ f(x) = \int_{-\infty}^{\infty} \hat{f}(\xi) e^{2\pi i x \xi} \, d\xi \]</div>",
                "Signal Processing"
            ),
            (
                r"State <b>Stokes' Theorem</b> and the <b>Divergence (Gauss's) Theorem</b>:",
                r"<div class='formula-box'><b>Stokes' Theorem:</b> \[ \oint_{\partial \Sigma} \mathbf{F} \cdot d\mathbf{r} = \iint_{\Sigma} (\nabla \times \mathbf{F}) \cdot d\mathbf{S} \]<br><b>Divergence Theorem:</b> \[ \oiint_{\partial V} \mathbf{F} \cdot d\mathbf{S} = \iiint_{V} (\nabla \cdot \mathbf{F}) \, dV \]</div>",
                "Vector Calculus"
            ),
            (
                r"What is the <b>Arrhenius Equation</b> for reaction rate constants?",
                r"<div class='formula-box'>\[ k = A e^{-\frac{E_a}{RT}} \]</div><br><span style='font-size:15px; color:#64748b;'>\( k \): rate constant &bull; \( E_a \): activation energy &bull; \( R \): gas constant (\(8.314\,\text{J/mol}\cdot\text{K}\)) &bull; \( T \): absolute temp</span>",
                "Chemistry"
            )
        ]

        # Cloze cards: (Text, Extra, Tag, num_clozes)
        cloze_cards_data = [
            (
                r"The Taylor series expansion of \( e^x \) around \( x = 0 \) is {{c1::<div class='formula-box'>\[ e^x = \sum_{n=0}^{\infty} \frac{x^n}{n!} = 1 + x + \frac{x^2}{2!} + \frac{x^3}{3!} + \cdots \]</div>}}.",
                r"Valid for all \( x \in \mathbb{R} \). Convergence radius \( R = \infty \).",
                "Calculus",
                1
            ),
            (
                r"For a 2×2 matrix \( A = \begin{pmatrix} a & b \\ c & d \end{pmatrix} \), the determinant is \( \det(A) = \) {{c1::\( ad - bc \)}} and its inverse is {{c2::<div class='formula-box'>\[ A^{-1} = \frac{1}{ad - bc} \begin{pmatrix} d & -b \\ -c & a \end{pmatrix} \]</div>}}.",
                r"The inverse exists if and only if \( \det(A) \neq 0 \).",
                "Linear Algebra",
                2
            ),
            (
                r"The Probability Density Function (PDF) of the Normal distribution \( \mathcal{N}(\mu, \sigma^2) \) is {{c1::<div class='formula-box'>\[ f(x) = \frac{1}{\sigma \sqrt{2\pi}} \exp\left( -\frac{(x - \mu)^2}{2\sigma^2} \right) \]</div>}}.",
                r"For standard normal \( Z \sim \mathcal{N}(0, 1) \), \( \mu = 0 \) and \( \sigma = 1 \).",
                "Statistics",
                1
            ),
            (
                r"In Special Relativity, the relativistic energy-momentum relation is {{c1::<div class='formula-box'>\[ E^2 = (pc)^2 + (m_0 c^2)^2 \]</div>}} and for a particle at rest (\( p = 0 \)) it simplifies to {{c2::\( E = m_0 c^2 \)}}.",
                r"Where \( p \) is momentum, \( m_0 \) is rest mass, and \( c \) is speed of light.",
                "Physics",
                2
            )
        ]

        note_id_counter = now_ms + 100
        card_id_counter = now_ms + 2000
        due_counter = 1

        # Insert Basic Notes & Cards
        for front, back, tag in basic_cards_data:
            nid = note_id_counter
            note_id_counter += 1
            guid = gen_guid()
            flds = f"{front}\x1f{back}\x1f{tag}"
            sfld = front
            csum = field_checksum(front)
            tags = f" {tag.replace(' ', '_')} Math_Demo "

            cur.execute("""
            INSERT INTO notes VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (nid, guid, basic_model_id, now_s, -1, tags, flds, sfld, csum, 0, ""))

            cid = card_id_counter
            card_id_counter += 1
            cur.execute("""
            INSERT INTO cards VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (cid, nid, deck_id, 0, now_s, -1, 0, 0, due_counter, 0, 2500, 0, 0, 0, 0, 0, 0, ""))
            due_counter += 1

        # Insert Cloze Notes & Cards
        for text, extra, tag, num_clozes in cloze_cards_data:
            nid = note_id_counter
            note_id_counter += 1
            guid = gen_guid()
            flds = f"{text}\x1f{extra}\x1f{tag}"
            sfld = text
            csum = field_checksum(text)
            tags = f" {tag.replace(' ', '_')} Cloze Math_Demo "

            cur.execute("""
            INSERT INTO notes VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (nid, guid, cloze_model_id, now_s, -1, tags, flds, sfld, csum, 0, ""))

            for cloze_idx in range(num_clozes):
                cid = card_id_counter
                card_id_counter += 1
                cur.execute("""
                INSERT INTO cards VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, (cid, nid, deck_id, cloze_idx, now_s, -1, 0, 0, due_counter, 0, 2500, 0, 0, 0, 0, 0, 0, ""))
                due_counter += 1

        conn.commit()
        conn.close()

        # Write out media file (empty JSON mapping)
        media_path = os.path.join(tmpdir, "media")
        with open(media_path, "w", encoding="utf-8") as f:
            f.write("{}")

        # Create the .apkg zip file
        os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
        with zipfile.ZipFile(output_path, "w", zipfile.ZIP_DEFLATED) as zf:
            zf.write(db_path, "collection.anki2")
            zf.write(media_path, "media")

    print(f"[OK] Generated {output_path} with {due_counter - 1} cards!")

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "Math_StudyLab_Demo.apkg"
    create_apkg(out)
