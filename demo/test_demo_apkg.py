#!/usr/bin/env python3
"""
Automated Test Suite for StudyLab Demo APKG
Validates compliance with StudyLab Source APKG Technical Contract v1.0.
"""

import os
import sys
import json
import sqlite3
import tempfile
import zipfile
import unittest
import re
from datetime import datetime

EXPECTED_NOTETYPE = "StudyLab Source"
EXPECTED_CANONICAL_FIELDS = [
    "Prompt",
    "Options",
    "CorrectAnswer",
    "Hint",
    "Solution",
    "Steps",
    "Explanation",
    "Subject",
    "Chapter",
    "Topic",
    "Skill",
    "ProblemType",
    "QuestionType",
    "Difficulty",
    "Source",
    "Exam",
    "Year",
    "Shift",
    "Paper",
    "SourceQuestionID",
]

EXPECTED_SUBJECTS = {
    "mathematics": "Mathematics",
    "physics": "Physics",
    "chemistry": "Chemistry",
    "reasoning": "Reasoning"
}

class TestStudyLabDemoApkg(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        cls.apkg_path = os.path.join(cls.base_dir, "demo", "output", "studylab-demo-v1.0.apkg")
        
        # If output apkg does not exist, run generator first
        if not os.path.exists(cls.apkg_path):
            from demo.generate_demo_apkg import generate_demo_apkg
            generate_demo_apkg(cls.apkg_path)

        cls.temp_dir = tempfile.TemporaryDirectory()
        with zipfile.ZipFile(cls.apkg_path, "r") as zf:
            zf.extractall(cls.temp_dir.name)
            cls.zip_files = zf.namelist()

        cls.media_map = {}
        if "media" in cls.zip_files:
            with open(os.path.join(cls.temp_dir.name, "media"), "r", encoding="utf-8") as f:
                cls.media_map = json.load(f)

        db_path = os.path.join(cls.temp_dir.name, "collection.anki2")
        cls.conn = sqlite3.connect(db_path)
        cls.cur = cls.conn.cursor()

        cls.cur.execute("SELECT conf, models, decks FROM col")
        col_row = cls.cur.fetchone()
        assert col_row is not None, "col table is empty"
        cls.conf = json.loads(col_row[0])
        cls.models = json.loads(col_row[1])
        cls.decks = json.loads(col_row[2])

        cls.cur.execute("SELECT id, guid, mid, flds, sfld, csum FROM notes")
        cls.raw_notes = cls.cur.fetchall()

    @classmethod
    def tearDownClass(cls):
        cls.conn.close()
        cls.temp_dir.cleanup()

    def test_01_canonical_note_type_and_fields_order(self):
        """Verify StudyLab Source note type exists with exact 20 canonical fields in order."""
        source_models = [m for m in self.models.values() if m.get("name", "").startswith(EXPECTED_NOTETYPE)]
        self.assertTrue(len(source_models) > 0, f"Missing '{EXPECTED_NOTETYPE}' note model")
        model = source_models[0]
        field_names = [f["name"] for f in model.get("flds", [])]
        self.assertEqual(field_names, EXPECTED_CANONICAL_FIELDS, "Canonical field order mismatch")

    def test_02_total_question_count_and_subject_distribution(self):
        """Verify 100 total questions and exactly 25 questions per subject across 4 required subjects."""
        self.assertGreaterEqual(len(self.raw_notes), 80, "Must contain at least 80 notes")
        self.assertLessEqual(len(self.raw_notes), 120, "Must contain at most 120 notes")
        self.assertEqual(len(self.raw_notes), 100, "Demo package should contain exactly 100 notes")

        subject_counts = {}
        chapter_map = {}
        for _, _, mid, flds_raw, _, _ in self.raw_notes:
            model = self.models.get(str(mid))
            field_names = [f["name"] for f in model.get("flds", [])]
            field_values = flds_raw.split("\x1f")
            field_map = dict(zip(field_names, field_values))
            
            subj = field_map.get("Subject", "").strip().lower()
            canonical_subj = EXPECTED_SUBJECTS.get(subj, subj)
            subject_counts[canonical_subj] = subject_counts.get(canonical_subj, 0) + 1
            chapter_map.setdefault(canonical_subj, set()).add(field_map.get("Chapter", "").strip())

        for exp_subj in EXPECTED_SUBJECTS.values():
            self.assertIn(exp_subj, subject_counts, f"Subject '{exp_subj}' must be present")
            self.assertEqual(subject_counts[exp_subj], 25, f"Subject '{exp_subj}' must have exactly 25 questions")
            self.assertGreaterEqual(len(chapter_map.get(exp_subj, set())), 1, f"Subject '{exp_subj}' must have at least 1 chapter")

    def test_03_required_fields_and_modalities(self):
        """Verify all mandatory fields are non-empty and QuestionTypes are closed/valid."""
        for _, guid, mid, flds_raw, _, _ in self.raw_notes:
            model = self.models.get(str(mid))
            field_names = [f["name"] for f in model.get("flds", [])]
            field_values = flds_raw.split("\x1f")
            field_map = dict(zip(field_names, field_values))

            prompt = field_map.get("Prompt", "").strip()
            correct_answer = field_map.get("CorrectAnswer", "").strip()
            subject = field_map.get("Subject", "").strip()
            chapter = field_map.get("Chapter", "").strip()
            topic = field_map.get("Topic", "").strip()
            qtype = field_map.get("QuestionType", "").strip().lower()

            self.assertTrue(prompt, f"Note {guid} missing Prompt")
            self.assertTrue(correct_answer, f"Note {guid} missing CorrectAnswer")
            self.assertTrue(subject, f"Note {guid} missing Subject")
            self.assertTrue(chapter, f"Note {guid} missing Chapter")
            self.assertTrue(topic, f"Note {guid} missing Topic")
            self.assertIn(qtype, ["mcq", "numerical"], f"Note {guid} invalid QuestionType '{qtype}'")

            if qtype == "mcq":
                opts_raw = field_map.get("Options", "").strip()
                self.assertTrue(opts_raw, f"MCQ note {guid} missing Options")
                if opts_raw.startswith("["):
                    opts = json.loads(opts_raw)
                else:
                    opts = [l.strip() for l in opts_raw.splitlines() if l.strip()]
                self.assertGreaterEqual(len(opts), 2, f"MCQ note {guid} must have at least 2 options")

            if qtype == "numerical":
                try:
                    float(correct_answer)
                except ValueError:
                    self.fail(f"Numerical note {guid} CorrectAnswer '{correct_answer}' is not a valid float")

    def test_04_source_question_id_uniqueness_and_determinism(self):
        """Verify SourceQuestionIDs are unique, non-empty, and adhere to deterministic naming."""
        seen_sqids = set()
        seen_prompts = set()

        for _, guid, mid, flds_raw, _, _ in self.raw_notes:
            model = self.models.get(str(mid))
            field_names = [f["name"] for f in model.get("flds", [])]
            field_values = flds_raw.split("\x1f")
            field_map = dict(zip(field_names, field_values))

            sqid = field_map.get("SourceQuestionID", "").strip()
            prompt = field_map.get("Prompt", "").strip().lower()

            self.assertTrue(sqid, f"Note {guid} missing SourceQuestionID")
            self.assertNotIn(sqid, seen_sqids, f"Duplicate SourceQuestionID '{sqid}'")
            seen_sqids.add(sqid)

            self.assertNotIn(prompt, seen_prompts, f"Duplicate prompt text on note {guid}")
            seen_prompts.add(prompt)

            # Prefix format check
            self.assertTrue(
                sqid.startswith("DEMO-MATH-") or sqid.startswith("DEMO-PHY-") or 
                sqid.startswith("DEMO-CHEM-") or sqid.startswith("DEMO-REAS-"),
                f"SourceQuestionID '{sqid}' must follow deterministic prefix convention"
            )

    def test_05_media_assets_integrity(self):
        """Verify all referenced images in prompts exist in media bundle."""
        inv_media = {v: k for k, v in self.media_map.items()}
        for _, guid, mid, flds_raw, _, _ in self.raw_notes:
            model = self.models.get(str(mid))
            field_names = [f["name"] for f in model.get("flds", [])]
            field_values = flds_raw.split("\x1f")
            field_map = dict(zip(field_names, field_values))

            prompt = field_map.get("Prompt", "")
            img_matches = re.findall(r'<img[^>]+src=["\']([^"\']+)["\']', prompt)
            for img_src in img_matches:
                self.assertIn(img_src, inv_media, f"Note {guid}: Referenced image '{img_src}' not in media manifest")
                idx_name = inv_media[img_src]
                self.assertIn(idx_name, self.zip_files, f"Note {guid}: Image index '{idx_name}' missing from zip archive")

    def test_06_package_metadata(self):
        """Verify package metadata exposed in collection config."""
        pkg_meta = self.conf.get("studylab_package_meta", {})
        self.assertEqual(pkg_meta.get("SchemaVersion"), "1.0")
        self.assertEqual(pkg_meta.get("PackageID"), "studylab-demo-source-v1.0")
        self.assertTrue(pkg_meta.get("GeneratedAt"))
        self.assertEqual(pkg_meta.get("SourceCoreVersion"), "1.0.0")

    def test_07_distinctness_audit_file(self):
        """Verify candidate distinctness audit report exists and records 100 accepted questions."""
        report_path = os.path.join(self.base_dir, "demo", "reports", "demo_distinctness_audit.json")
        self.assertTrue(os.path.exists(report_path), "Missing demo_distinctness_audit.json")
        with open(report_path, "r", encoding="utf-8") as f:
            audit = json.load(f)
        self.assertEqual(audit.get("candidate_questions"), 100)
        self.assertEqual(audit.get("accepted_questions"), 100)
        self.assertEqual(audit.get("rejected_questions"), 0)

if __name__ == "__main__":
    unittest.main()
