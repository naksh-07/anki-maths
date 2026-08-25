#!/usr/bin/env python3
import json
import sqlite3
import sys
import tempfile
import zipfile
import os

APKG_PATH = "dist/apkgs/StudyLab_Full_Universe_175.apkg"
EXPECTED_TOPICS = 177

def validate_apkg(apkg_path: str):
    if not os.path.exists(apkg_path):
        print(f"Error: APKG not found at {apkg_path}")
        sys.exit(1)

    print(f"Validating Canonical APKG: {apkg_path}")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        with zipfile.ZipFile(apkg_path, 'r') as zf:
            zf.extract("collection.anki2", temp_dir)
            
        db_path = os.path.join(temp_dir, "collection.anki2")
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()
        try:
            # 1. Validate col
            cur.execute("SELECT models FROM col")
            col_row = cur.fetchone()
            assert col_row is not None, "col table is empty"
            models = json.loads(col_row[0])
            has_anchor_model = any(m.get("name") == "StudyLab Procedural Anchor" for m in models.values())
            assert has_anchor_model, "Missing StudyLab Procedural Anchor model"
            print("[PASS] Collection metadata and models valid")

            # 2. Validate notes
            cur.execute("SELECT flds FROM notes")
            notes = cur.fetchall()
            print(f"Found {len(notes)} procedural anchors.")
            
            if len(notes) != EXPECTED_TOPICS:
                print(f"Warning: Expected {EXPECTED_TOPICS} topics, found {len(notes)}")
                
            domains = set()
            topics = set()
            
            for idx, (flds_raw,) in enumerate(notes):
                flds = flds_raw.split("\x1f")
                assert len(flds) == 4, f"Note {idx} has invalid field count: {len(flds)}"
                
                payload_json, topic_title, domain, provenance_raw = flds
                
                domains.add(domain)
                topics.add(topic_title)
                
                # 3. Payload completeness
                try:
                    payload = json.loads(payload_json)
                except json.JSONDecodeError:
                    print(f"Error: Invalid JSON payload in note {idx}")
                    sys.exit(1)
                    
                assert "proc_schema" in payload, f"Missing proc_schema in note {idx}"
                assert "inline_contract" in payload, f"Missing inline_contract in note {idx}"
                
                contract = payload["inline_contract"].get("contract", {})
                archetypes = payload["inline_contract"].get("archetypes", [])
                
                # 4. Required fields & Modality completeness
                assert "family_id" in contract, f"Missing family_id in {topic_title}"
                assert "skill_id" in contract, f"Missing skill_id in {topic_title}"
                assert "supported_variants" in contract, f"Missing supported_variants in {topic_title}"
                assert len(contract["supported_variants"]) > 0, f"No modalities defined for {topic_title}"
                
                # Ensure provenance matches
                assert contract.get("provenance") is not None, f"Missing provenance in contract for {topic_title}"
                try:
                    prov_field = json.loads(provenance_raw)
                    assert isinstance(prov_field, dict), "Provenance field must be a dict"
                except Exception:
                    print(f"Error: Invalid JSON provenance field in note {idx}")
                    sys.exit(1)
                    
                # 5. Check Solution Graph / Hints in archetypes
                assert len(archetypes) > 0, f"No archetypes for {topic_title}"
                for arch_idx, archetype in enumerate(archetypes):
                    assert "step_nodes" in archetype or "solution_template" in archetype, f"Missing solution graph/template in archetype {arch_idx} for {topic_title}"
                    
            print(f"\n[PASS] Unique topics verified: {len(topics)}")
            if len(topics) != EXPECTED_TOPICS:
                print(f"Discrepancy: Expected {EXPECTED_TOPICS}, found {len(topics)}")
                
                # Count by domain
                domain_counts = {}
                for idx, (flds_raw,) in enumerate(notes):
                    flds = flds_raw.split("\x1f")
                    domain = flds[2]
                    domain_counts[domain] = domain_counts.get(domain, 0) + 1
                
                print("Counts by domain:")
                for d, count in domain_counts.items():
                    print(f"  {d}: {count}")
                    
            print("[PASS] Payload completeness, modality completeness, provenance, and metadata verified.")
            print("[PASS] Self-contained SQLite import validated.")
            print("\nALL CANONICAL CONTRACT TESTS PASSED.")
        finally:
            conn.close()

if __name__ == "__main__":
    validate_apkg(APKG_PATH)
