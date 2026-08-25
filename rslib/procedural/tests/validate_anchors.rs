use procedural::anchor::ProceduralCardAnchor;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn test_all_generated_anchors() {
    let file = File::open("../../all_anchors.jsonl").unwrap();
    let reader = BufReader::new(file);
    let mut failures = 0;
    
    for (idx, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        match ProceduralCardAnchor::from_json_str_strict(&line) {
            Ok(Some(_)) => {}
            Ok(None) => {
                println!("Row {}: Did not match anchor format", idx);
                failures += 1;
            }
            Err(e) => {
                println!("Row {}: {}", idx, e);
                failures += 1;
            }
        }
    }
    
    assert_eq!(failures, 0, "Found {} malformed anchors", failures);
}
