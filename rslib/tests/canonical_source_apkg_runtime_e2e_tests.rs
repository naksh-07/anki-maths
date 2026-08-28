// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::path::Path;
use tempfile::{tempdir, TempDir};
use anki_proto::import_export::ImportAnkiPackageOptions;
use anki::prelude::*;
use anki::collection::CollectionBuilder;
use anki::template::RenderedNode;
use anki::search::SortMode;
use procedural::anchor::source::SourceQuestion;
use procedural::practice::{PracticeAttempt, SchemaPracticeObject};
use procedural::core::AttemptId;
use procedural::skills::Skill;
use procedural::problems::ProblemFamily;

fn open_fs_test_collection(name: &str) -> (Collection, TempDir) {
    let tempdir = tempdir().unwrap();
    let dir = tempdir.path();
    let col = CollectionBuilder::new(dir.join(format!("{name}.anki2")))
        .with_desktop_media_paths()
        .build()
        .unwrap();
    (col, tempdir)
}

fn get_first_card_for_note(col: &mut Collection, note_id: NoteId) -> Card {
    let card_ids = col.search_cards(&format!("nid:{note_id}"), SortMode::NoOrder).unwrap();
    assert!(!card_ids.is_empty(), "Note {note_id} must have at least one card");
    col.storage.get_card(card_ids[0]).unwrap().unwrap()
}

#[test]
fn test_e2e_canonical_source_apkg_import_and_reconciliation() {
    let (mut col, _dir) = open_fs_test_collection("e2e_source_import");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    assert!(apkg_path.exists(), "APKG fixture must exist at {:?}", apkg_path);

    // 1. Import the canonical APKG
    let res = col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default());
    assert!(res.is_ok(), "Importing canonical APKG must succeed: {:?}", res.err());

    // 2. Verify notes imported into collection.anki2
    let source_notes = col.search_notes_unordered("note:\"StudyLab Source*\"").unwrap();
    assert_eq!(source_notes.len(), 5, "Expected 5 StudyLab Source notes in collection");

    // 3. Verify automatic reconciliation into collection.procedural (practice_items)
    let service = col.procedural_service().unwrap();
    let store = service.store();

    for nid in &source_notes {
        let note = col.storage.get_note(*nid).unwrap().unwrap();
        let item_id = SourceQuestion::stable_id_from_guid(&note.guid);
        let practice_item = store.get_practice_item(&item_id).unwrap();
        assert!(practice_item.is_some(), "PracticeItem for note GUID {} must exist in database", note.guid);
        
        let item = practice_item.unwrap();
        assert_eq!(item.id, item_id);
        assert!(!item.prompt.is_empty());
        assert!(item.difficulty >= 1.0 && item.difficulty <= 5.0);
    }
}

#[test]
fn test_e2e_mcq_reviewer_render_and_modality() {
    let (mut col, _dir) = open_fs_test_collection("e2e_mcq_render");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();

    // Find the train platform MCQ note
    let note_ids = col.search_notes_unordered("note:\"StudyLab Source*\" train").unwrap();
    assert_eq!(note_ids.len(), 1);
    let note = col.storage.get_note(note_ids[0]).unwrap().unwrap();
    let nt = col.get_notetype(note.notetype_id).unwrap().unwrap();
    let card = get_first_card_for_note(&mut col, note.id);
    let template = &nt.templates[0];

    // Render the card through Anki's rendering engine
    let output = col.render_card(&note, &card, &nt, template, false, false).unwrap();
    let html = match &output.qnodes[0] {
        RenderedNode::Text { text } => text,
        _ => panic!("Expected text render node"),
    };

    // Verify MCQ Modality & Elements
    assert!(html.contains("data-object-type=\"mcq\""), "Modality must be explicitly mcq");
    assert!(html.contains("train traveling at 72 km/h"), "Prompt must be present");
    assert!(html.contains("150m"), "Option 150m must be present");
    assert!(html.contains("200m"), "Option 200m must be present");
    assert!(html.contains("300m"), "Option 300m must be present");
    assert!(html.contains("350m"), "Option 350m must be present");
    assert!(html.contains("proc-option-group"), "Option group must be present");
    assert!(html.contains("proc-option-item"), "Option items must be present");
    
    // Strict Modality Purity: MCQ must NOT contain numeric input textboxes
    assert!(!html.contains("id=\"proc-answer-input\""), "MCQ must not contain numerical answer textbox");
    assert!(!html.contains("Quick Solve"), "MCQ must not contain Quick Solve tab");

    // Verify Provenance Badge
    assert!(html.contains("PYQ: RRB ALP 2024 · Shift 1"), "Must render authenticated PYQ badge");
}

#[test]
fn test_e2e_numerical_reviewer_render_and_modality() {
    let (mut col, _dir) = open_fs_test_collection("e2e_num_render");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();

    // Find the Newton's Laws numerical note
    let note_ids = col.search_notes_unordered("note:\"StudyLab Source*\" force").unwrap();
    assert_eq!(note_ids.len(), 1);
    let note = col.storage.get_note(note_ids[0]).unwrap().unwrap();
    let nt = col.get_notetype(note.notetype_id).unwrap().unwrap();
    let card = get_first_card_for_note(&mut col, note.id);
    let template = &nt.templates[0];

    // Render card
    let output = col.render_card(&note, &card, &nt, template, false, false).unwrap();
    let html = match &output.qnodes[0] {
        RenderedNode::Text { text } => text,
        _ => panic!("Expected text render node"),
    };

    // Verify Numerical Modality & Elements
    assert!(html.contains("Calculate the force (in Newtons)"), "Prompt must be present");
    assert!(html.contains("id=\"proc-answer-input\""), "Numerical question must provide answer input textbox");
    assert!(html.contains("id=\"proc-submit-btn\""), "Numerical question must provide submit button");
    
    // Numerical question must NOT render MCQ options
    assert!(!html.contains("proc-option-group"), "Numerical question must not render option group");
    assert!(!html.contains("proc-option-item"), "Numerical question must not render option items");
}

#[test]
fn test_e2e_optional_fields_absence_cleanliness() {
    let (mut col, _dir) = open_fs_test_collection("e2e_optional_fields");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();

    // Find solar system note with optional fields omitted
    let note_ids = col.search_notes_unordered("note:\"StudyLab Source*\" planet").unwrap();
    assert_eq!(note_ids.len(), 1);
    let note = col.storage.get_note(note_ids[0]).unwrap().unwrap();
    let nt = col.get_notetype(note.notetype_id).unwrap().unwrap();
    let card = get_first_card_for_note(&mut col, note.id);
    let template = &nt.templates[0];

    let output = col.render_card(&note, &card, &nt, template, false, false).unwrap();
    let html = match &output.qnodes[0] {
        RenderedNode::Text { text } => text,
        _ => panic!("Expected text render node"),
    };

    assert!(html.contains("Which planet in the solar system"), "Prompt rendered");
    assert!(html.contains("Mercury"), "Options rendered");
    // Verify no placeholder strings in rendered markup
    assert!(!html.contains("undefined"), "Must not contain 'undefined'");
    assert!(!html.contains("TODO"), "Must not contain 'TODO'");
    assert!(!html.contains("{{Hint}}"), "Must not contain unrendered mustache tag");
    assert!(!html.contains("{{Solution}}"), "Must not contain unrendered mustache tag");
}

#[test]
fn test_e2e_media_reference_survival() {
    let (mut col, _dir) = open_fs_test_collection("e2e_media");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();

    // Verify media file was unpacked into media folder
    let target_media_file = col.media_folder().join("studylab_diagram.png");
    assert!(target_media_file.exists(), "Media file 'studylab_diagram.png' must exist in collection media folder");

    // Find geometric diagram card
    let note_ids = col.search_notes_unordered("note:\"StudyLab Source*\" diagram").unwrap();
    assert_eq!(note_ids.len(), 1);
    let note = col.storage.get_note(note_ids[0]).unwrap().unwrap();
    let nt = col.get_notetype(note.notetype_id).unwrap().unwrap();
    let card = get_first_card_for_note(&mut col, note.id);
    let template = &nt.templates[0];

    let output = col.render_card(&note, &card, &nt, template, false, false).unwrap();
    let html = match &output.qnodes[0] {
        RenderedNode::Text { text } => text,
        _ => panic!("Expected text render node"),
    };

    assert!(html.contains("studylab_diagram.png"), "Rendered HTML must reference media file");
}

#[test]
fn test_e2e_repeated_import_reconciliation_lifecycle() {
    let (mut col, _dir) = open_fs_test_collection("e2e_reconciliation");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");

    // First import: 5 new items
    let rep1 = col.reconcile_source_questions().unwrap();
    assert_eq!(rep1.new_count, 0); // Collection empty before import

    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();
    let rep_import = col.reconcile_source_questions().unwrap();
    // Notes are already reconciled during import_apkg, so re-running reconcile returns unchanged
    assert_eq!(rep_import.unchanged_count, 5);
    assert_eq!(rep_import.new_count, 0);
    assert_eq!(rep_import.updated_count, 0);
    assert_eq!(rep_import.archived_count, 0);

    // Second import of identical APKG: 5 unchanged
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();
    let rep2 = col.reconcile_source_questions().unwrap();
    assert_eq!(rep2.unchanged_count, 5);
    assert_eq!(rep2.new_count, 0);
    assert_eq!(rep2.updated_count, 0);

    // Modify a note in Anki collection
    let note_ids = col.search_notes_unordered("note:\"StudyLab Source*\" train").unwrap();
    let mut note = col.storage.get_note(note_ids[0]).unwrap().unwrap();
    note.set_field(0, "Updated prompt text for train problem").unwrap();
    col.update_note(&mut note).unwrap();

    let rep3 = col.reconcile_source_questions().unwrap();
    assert_eq!(rep3.updated_count, 1, "Expected 1 updated question after modifying note");
    assert_eq!(rep3.unchanged_count, 4);

    // Delete a note from Anki collection
    col.remove_notes(&[note.id]).unwrap();
    let rep4 = col.reconcile_source_questions().unwrap();
    assert_eq!(rep4.archived_count, 1, "Expected 1 archived question after deleting note");
}

#[test]
fn test_e2e_learner_state_firewall_isolation() {
    let (mut col, _dir) = open_fs_test_collection("e2e_firewall");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();

    let note_ids = col.search_notes_unordered("note:\"StudyLab Source*\" train").unwrap();
    let note = col.storage.get_note(note_ids[0]).unwrap().unwrap();
    let item_id = SourceQuestion::stable_id_from_guid(&note.guid);

    let service = col.procedural_service().unwrap();
    let store = service.store();

    let item_before = store.get_practice_item(&item_id).unwrap().unwrap();
    let prompt_before = item_before.prompt.clone();
    let diff_before = item_before.difficulty;
    let prov_before = item_before.provenance.clone();
    let qtype_before = item_before.question_type.clone();

    // Insert skill, problem family, schema and problem instance before recording attempt
    let skill = Skill::new(
        item_before.skill_id.clone(),
        item_before.domain.clone(),
        "Test Skill",
        "Test Description",
    );
    store.insert_skill(&skill).unwrap();

    let family = ProblemFamily::new(
        item_before.problem_family_id.clone(),
        item_before.skill_id.clone(),
        item_before.domain.clone(),
        "Test Family",
        "templates/test.html",
    );
    store.insert_problem_family(&family).unwrap();

    let schema = SchemaPracticeObject::new(
        item_before.schema_id.clone(),
        item_before.skill_id.clone(),
        item_before.problem_family_id.clone(),
        "Test Schema",
        "Test Schema Description",
    );
    store.insert_schema(&schema).unwrap();

    let problem_instance = item_before.clone().into_problem_instance();
    store.insert_problem_instance(&problem_instance).unwrap();

    // Learner solves and records attempts into procedural storage
    let attempt = PracticeAttempt::new(
        AttemptId::new("att_001"),
        problem_instance.id.clone(),
        item_before.schema_id.clone(),
        item_before.skill_id.clone(),
        serde_json::json!("300m"),
        true,
        1.0,
        12000,
    );

    store.insert_practice_attempt(&attempt).unwrap();

    // Hard Invariant Check: Source Question fields in practice_items MUST REMAIN STATIC
    let item_after = store.get_practice_item(&item_id).unwrap().unwrap();
    assert_eq!(item_after.prompt, prompt_before, "Prompt must not mutate from learner attempts");
    assert_eq!(item_after.difficulty, diff_before, "Difficulty must not mutate from learner attempts");
    assert_eq!(item_after.provenance, prov_before, "Provenance must not mutate from learner attempts");
    assert_eq!(item_after.question_type, qtype_before, "QuestionType must not mutate from learner attempts");
}

#[test]
fn test_e2e_standard_anki_regression_and_isolation() {
    let (mut col, _dir) = open_fs_test_collection("e2e_anki_regression");

    // 1. Add standard Basic card
    let basic_nt = col.get_notetype_by_name("Basic").unwrap().unwrap().as_ref().clone();
    let mut basic_note = basic_nt.new_note();
    basic_note.set_field(0, "Capital of Japan?").unwrap();
    basic_note.set_field(1, "Tokyo").unwrap();
    col.add_note(&mut basic_note, DeckId(1)).unwrap();

    // 2. Add Cloze card
    let cloze_nt = col.get_notetype_by_name("Cloze").unwrap().unwrap().as_ref().clone();
    let mut cloze_note = cloze_nt.new_note();
    cloze_note.set_field(0, "The {{c1::mitochondria}} is the powerhouse of the cell.").unwrap();
    col.add_note(&mut cloze_note, DeckId(1)).unwrap();

    // 3. Add Custom non-StudyLab note
    let mut custom_nt = basic_nt.clone();
    custom_nt.id = NotetypeId(0);
    custom_nt.name = "Custom Language Note".into();
    col.add_notetype(&mut custom_nt, true).unwrap();
    let mut custom_note = custom_nt.new_note();
    custom_note.set_field(0, "Bonjour").unwrap();
    custom_note.set_field(1, "Hello").unwrap();
    col.add_note(&mut custom_note, DeckId(1)).unwrap();

    // 4. Import StudyLab Source APKG
    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();

    // Render Basic Card: must NOT use StudyLab engine
    let basic_card = get_first_card_for_note(&mut col, basic_note.id);
    let basic_out = col.render_card(&basic_note, &basic_card, &basic_nt, &basic_nt.templates[0], false, false).unwrap();
    let basic_html = match &basic_out.qnodes[0] { RenderedNode::Text { text } => text, _ => "" };
    assert!(basic_html.contains("Capital of Japan?"));
    assert!(!basic_html.contains("procedural-card"), "Basic card must not contain procedural container");

    // Render Cloze Card: must NOT use StudyLab engine
    let cloze_card = get_first_card_for_note(&mut col, cloze_note.id);
    let cloze_out = col.render_card(&cloze_note, &cloze_card, &cloze_nt, &cloze_nt.templates[0], false, false).unwrap();
    let cloze_html = match &cloze_out.qnodes[0] { RenderedNode::Text { text } => text, _ => "" };
    assert!(cloze_html.contains("powerhouse of the cell"));
    assert!(!cloze_html.contains("procedural-card"), "Cloze card must not contain procedural container");

    // Render Custom Card: must NOT use StudyLab engine
    let custom_card = get_first_card_for_note(&mut col, custom_note.id);
    let custom_out = col.render_card(&custom_note, &custom_card, &custom_nt, &custom_nt.templates[0], false, false).unwrap();
    let custom_html = match &custom_out.qnodes[0] { RenderedNode::Text { text } => text, _ => "" };
    assert!(custom_html.contains("Bonjour"));
    assert!(!custom_html.contains("procedural-card"), "Custom card must not contain procedural container");

    // Render StudyLab Source Card: MUST use StudyLab engine
    let source_notes = col.search_notes_unordered("note:\"StudyLab Source*\" train").unwrap();
    let source_note = col.storage.get_note(source_notes[0]).unwrap().unwrap();
    let source_nt = col.get_notetype(source_note.notetype_id).unwrap().unwrap();
    let source_card = get_first_card_for_note(&mut col, source_note.id);
    let source_out = col.render_card(&source_note, &source_card, &source_nt, &source_nt.templates[0], false, false).unwrap();
    let source_html = match &source_out.qnodes[0] { RenderedNode::Text { text } => text, _ => "" };
    assert!(source_html.contains("procedural-card"), "StudyLab Source note must render procedural container");
}

#[test]
fn test_e2e_adversarial_malformed_note_render_error_handling() {
    let (mut col, _dir) = open_fs_test_collection("e2e_malformed_render");

    // 1. Import base APKG to register "StudyLab Source" notetype
    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();

    let source_nt = col.get_notetype_by_name("StudyLab Source").unwrap().unwrap().as_ref().clone();

    // 2. Create a malformed note: QuestionType is "invalid_type"
    let mut malformed_note = source_nt.new_note();
    malformed_note.set_field(0, "What is the speed of light?").unwrap(); // Prompt
    malformed_note.set_field(2, "3e8").unwrap(); // CorrectAnswer
    malformed_note.set_field(12, "invalid_unsupported_type").unwrap(); // QuestionType
    col.add_note(&mut malformed_note, DeckId(1)).unwrap();

    let card = get_first_card_for_note(&mut col, malformed_note.id);
    let output = col.render_card(&malformed_note, &card, &source_nt, &source_nt.templates[0], false, false).unwrap();
    let html = match &output.qnodes[0] { RenderedNode::Text { text } => text, _ => "" };

    // Must render error container safely without throwing runtime panic
    assert!(html.contains("class='proc-error'"), "Malformed note must render error container");
    assert!(html.contains("Source Engine Error"), "Must display Source Engine Error title");
    assert!(html.contains("Invalid QuestionType"), "Must state reason for failure");
}

#[test]
fn test_e2e_adversarial_learning_support_propagation() {
    let (mut col, _dir) = open_fs_test_collection("e2e_learning_support");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts_qa/canonical_source_test_fixture.apkg");
    col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default()).unwrap();

    // Find train question which has Hint, Solution, Steps, Explanation
    let note_ids = col.search_notes_unordered("note:\"StudyLab Source*\" train").unwrap();
    let note = col.storage.get_note(note_ids[0]).unwrap().unwrap();
    let item_id = SourceQuestion::stable_id_from_guid(&note.guid);
    assert_eq!(item_id.as_str(), format!("pi_src_{}", note.guid));

    let service = col.procedural_service().unwrap();
    let session = service.resolve_source_target(&note.guid, None).unwrap();

    // Verify metadata propagation into instance and session
    let instance = &session.instance;
    let meta = instance.metadata.as_object().unwrap();
    assert_eq!(meta.get("hint").and_then(|v| v.as_str()), Some("Convert speed from km/h to m/s by multiplying by 5/18."));
    assert_eq!(meta.get("solution").and_then(|v| v.as_str()), Some("Speed = 72 * (5/18) = 20 m/s. Total distance = 20 * 25 = 500m. Length of train = 500 - 200 = 300m."));
    assert_eq!(meta.get("explanation").and_then(|v| v.as_str()), Some("Total distance covered while crossing platform is length of train plus length of platform."));
    assert_eq!(meta.get("exam").and_then(|v| v.as_str()), Some("RRB ALP"));
    assert_eq!(meta.get("year").and_then(|v| v.as_i64()), Some(2024));
    assert_eq!(meta.get("source_question_id").and_then(|v| v.as_str()), Some("RRB_ALP_2024_S1_Q42"));

    // Verify rendered HTML has no template placeholders or undefined tokens
    let nt = col.get_notetype(note.notetype_id).unwrap().unwrap();
    let card = get_first_card_for_note(&mut col, note.id);
    let output = col.render_card(&note, &card, &nt, &nt.templates[0], false, false).unwrap();
    let html = match &output.qnodes[0] { RenderedNode::Text { text } => text, _ => "" };

    assert!(!html.contains("undefined"));
    assert!(!html.contains("TODO"));
    assert!(!html.contains("{{Hint}}"));
    assert!(!html.contains("{{Solution}}"));
    assert!(!html.contains("{{Steps}}"));
    assert!(!html.contains("{{Explanation}}"));
}

#[test]
fn test_e2e_studylab_demo_apkg_100_notes_all_subjects() {
    let (mut col, _dir) = open_fs_test_collection("e2e_studylab_demo_100");

    let apkg_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../demo/output/studylab-demo-v1.0.apkg");
    assert!(apkg_path.exists(), "Demo APKG fixture must exist at {:?}", apkg_path);

    // 1. Import the Demo APKG
    let res = col.import_apkg(&apkg_path, ImportAnkiPackageOptions::default());
    assert!(res.is_ok(), "Importing demo APKG must succeed: {:?}", res.err());

    // 2. Verify exactly 100 notes imported into collection.anki2
    let source_notes = col.search_notes_unordered("note:\"StudyLab Source*\"").unwrap();
    assert_eq!(source_notes.len(), 100, "Expected 100 StudyLab Source notes in collection");

    // 3. Verify reconciliation of all 100 notes into collection.procedural
    let service = col.procedural_service().unwrap();
    let store = service.store();

    let mut subjects_found = std::collections::HashSet::new();
    for nid in &source_notes {
        let note = col.storage.get_note(*nid).unwrap().unwrap();
        let item_id = SourceQuestion::stable_id_from_guid(&note.guid);
        let practice_item = store.get_practice_item(&item_id).unwrap();
        assert!(practice_item.is_some(), "PracticeItem for note GUID {} must exist in store", note.guid);
        
        let item = practice_item.unwrap();
        subjects_found.insert(item.domain.as_str().to_string());
        assert!(!item.prompt.is_empty());
        assert!(item.difficulty >= 1.0 && item.difficulty <= 5.0);
    }

    // 4. Verify all 4 domains represented in store
    assert!(subjects_found.contains("mathematics"), "Mathematics domain must be represented");
    assert!(subjects_found.contains("physics"), "Physics domain must be represented");
    assert!(subjects_found.contains("chemistry"), "Chemistry domain must be represented");
    assert!(subjects_found.contains("reasoning"), "Reasoning domain must be represented");

    // 5. Test rendering of a sample note (Math remainder problem)
    let math_notes = col.search_notes_unordered("note:\"StudyLab Source*\" remainder").unwrap();
    assert!(!math_notes.is_empty(), "Math remainder note must exist");
    let math_note = col.storage.get_note(math_notes[0]).unwrap().unwrap();
    let math_nt = col.get_notetype(math_note.notetype_id).unwrap().unwrap();
    let math_card = get_first_card_for_note(&mut col, math_note.id);
    let math_out = col.render_card(&math_note, &math_card, &math_nt, &math_nt.templates[0], false, false).unwrap();
    let math_html = match &math_out.qnodes[0] { RenderedNode::Text { text } => text, _ => "" };
    assert!(math_html.contains("procedural-card"), "Must render procedural card container");
    assert!(math_html.contains("remainder"), "Must render prompt content");

    // 6. Test rendering of media diagram note
    let graph_notes = col.search_notes_unordered("note:\"StudyLab Source*\" graph").unwrap();
    assert!(!graph_notes.is_empty(), "Graph note must exist");
    let graph_note = col.storage.get_note(graph_notes[0]).unwrap().unwrap();
    let graph_card = get_first_card_for_note(&mut col, graph_note.id);
    let graph_out = col.render_card(&graph_note, &graph_card, &math_nt, &math_nt.templates[0], false, false).unwrap();
    let graph_html = match &graph_out.qnodes[0] { RenderedNode::Text { text } => text, _ => "" };
    assert!(graph_html.contains("studylab_demo_motion_graph.png"), "Must render image tag");
}


