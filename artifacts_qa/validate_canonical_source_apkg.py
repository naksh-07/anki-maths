#!/usr/bin/env python3
"""
StudyLab Canonical Source APKG Contract QA Validator
Validates that an APKG containing 'StudyLab Source' notes strictly conforms to
the canonical StudyLab Source APKG contract specification.
"""
import json
import sqlite3
import sys
import tempfile
import zipfile
import os

def validate_canonical_source_apkg(apkg_path: str):
    if not os.path.exists(apkg_path):
        print(f"Error: APKG not found at {apkg_path}")
        sys.exit(1)

    print(f"Validating Canonical Source APKG: {apkg_path}")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        with zipfile.ZipFile(apkg_path, 'r') as zf:
            zf.extract("collection.anki2", temp_dir)
            
        db_path = os.path.join(temp_dir, "collection.anki2")
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()
        try:
            # 1. Inspect notetypes / models
            cur.execute("SELECT models FROM col")
            col_row = cur.fetchone()
            assert col_row is not None, "col table is empty"
            models = json.loads(col_row[0])
            
            source_models = {mid: m for mid, m in models.items() if m.get("name", "").startswith("StudyLab Source")}
            print(f"[PASS] Identified {len(source_models)} 'StudyLab Source*' notetype(s)")

            # 2. Inspect notes
            cur.execute("SELECT mid, flds FROM notes")
            notes = cur.fetchall()
            print(f"Total notes in deck: {len(notes)}")

            validated_count = 0
            for idx, (mid, flds_raw) in enumerate(notes):
                model = models.get(str(mid)) or models.get(mid)
                if not model or not model.get("name", "").startswith("StudyLab Source"):
                    continue

                field_names = [f["name"] for f in model.get("flds", [])]
                field_values = flds_raw.split("\x1f")
                field_map = dict(zip(field_names, field_values))

                # Required field checks
                prompt = field_map.get("Prompt") or field_map.get("Question") or field_map.get("Front")
                assert prompt and prompt.strip(), f"Note {idx}: Missing required 'Prompt' field"

                raw_qtype = field_map.get("QuestionType") or field_map.get("Type") or field_map.get("LearningObjectType")
                assert raw_qtype and raw_qtype.strip(), f"Note {idx}: Missing required 'QuestionType' field"

                qtype_norm = raw_qtype.strip().lower()
                assert qtype_norm in ["mcq", "multiple_choice", "multiplechoice", "numerical", "numeric"], \
                    f"Note {idx}: Invalid QuestionType '{raw_qtype}'"

                correct_answer = field_map.get("CorrectAnswer") or field_map.get("Answer") or field_map.get("Back")
                assert correct_answer and correct_answer.strip(), f"Note {idx}: Missing required 'CorrectAnswer' field"

                # MCQ validation
                if qtype_norm in ["mcq", "multiple_choice", "multiplechoice"]:
                    options_raw = field_map.get("Options")
                    assert options_raw and options_raw.strip(), f"Note {idx}: MCQ note missing 'Options' field"
                    trimmed_opts = options_raw.strip()
                    if trimmed_opts.startswith("["):
                        try:
                            options = json.loads(trimmed_opts)
                        except Exception:
                            options = [l.strip() for l in trimmed_opts.splitlines() if l.strip()]
                    else:
                        options = [l.strip() for l in trimmed_opts.splitlines() if l.strip()]
                    assert len(options) >= 2, f"Note {idx}: MCQ requires at least 2 options, found {len(options)}"

                # Numerical validation
                if qtype_norm in ["numerical", "numeric"]:
                    try:
                        float(correct_answer.strip())
                    except ValueError:
                        raise AssertionError(f"Note {idx}: Numerical CorrectAnswer '{correct_answer}' is not a valid float")

                # Difficulty validation
                if "Difficulty" in field_map and field_map["Difficulty"].strip():
                    try:
                        diff = float(field_map["Difficulty"].strip())
                        assert 1.0 <= diff <= 5.0, f"Note {idx}: Difficulty {diff} out of bounds [1.0, 5.0]"
                    except ValueError:
                        raise AssertionError(f"Note {idx}: Difficulty '{field_map['Difficulty']}' is not a valid float")

                validated_count += 1

            print(f"[PASS] Successfully validated {validated_count} Canonical Source questions against contract.")
            print("\nALL CANONICAL SOURCE CONTRACT CHECKS PASSED.")
        finally:
            conn.close()

if __name__ == "__main__":
    if len(sys.argv) > 1:
        validate_canonical_source_apkg(sys.argv[1])
    else:
        print("Usage: python validate_canonical_source_apkg.py <path_to_apkg>")
