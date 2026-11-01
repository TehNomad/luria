//! Plan §6: the worked example in `docs/specs/cli.md` is a hand-written
//! expectation. Run the executable on the page's request block and compare
//! its document with the page's result block, field by field.
//!
//! The page states floats at the verification tables' precision, while
//! luria prints them in full (plan §3.5), so those fields — the Tms, but not
//! the annealing temperature, which is a whole number of degrees — are
//! compared within a tolerance; every other field must be exactly equal.

use luria_cli::json::{parse_json, Json};
use luria_core::identity::sha256_hex;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// The lines strictly between the ```` ```json ```` fence after `marker`
/// and the next closing fence, newline-joined with one trailing newline.
fn block(page: &str, marker: &str) -> String {
    let mut lines = page.lines();
    lines
        .by_ref()
        .find(|line| line.trim() == marker)
        .unwrap_or_else(|| panic!("docs/specs/cli.md: marker {} is missing", marker));
    match lines.next() {
        Some("```json") => {}
        _ => panic!("docs/specs/cli.md: no ```json fence follows {}", marker),
    }
    let mut body = Vec::new();
    for line in lines {
        if line == "```" {
            let mut text = body.join("\n");
            text.push('\n');
            return text;
        }
        body.push(line);
    }
    panic!(
        "docs/specs/cli.md: the block after {} has no closing fence",
        marker
    )
}

fn tolerance(path: &str) -> f64 {
    let last = path.rsplit('.').next().unwrap_or(path);
    match last {
        "celsius" | "gcPercent" => 0.005,
        "enthalpyKcal" | "entropyCal" => 0.05,
        _ => 0.0,
    }
}

/// Record every difference between `actual` and `expected` under `path`.
fn compare(path: &str, actual: &Json, expected: &Json, differences: &mut Vec<String>) {
    match (actual, expected) {
        (Json::Object(actual), Json::Object(expected)) => {
            let actual_keys: Vec<&String> = actual.iter().map(|(key, _)| key).collect();
            let expected_keys: Vec<&String> = expected.iter().map(|(key, _)| key).collect();
            for key in &expected_keys {
                if !actual_keys.contains(key) {
                    differences.push(format!("{}: missing key {:?}", path, key));
                }
            }
            for key in &actual_keys {
                if !expected_keys.contains(key) {
                    differences.push(format!("{}: extra key {:?}", path, key));
                }
            }
            if actual_keys != expected_keys {
                differences.push(format!(
                    "{}: key order {:?}, expected {:?}",
                    path, actual_keys, expected_keys
                ));
            }
            for (key, value) in expected {
                if let Some((_, found)) = actual.iter().find(|(other, _)| other == key) {
                    compare(&format!("{}.{}", path, key), found, value, differences);
                }
            }
        }
        (Json::Array(actual), Json::Array(expected)) => {
            if actual.len() != expected.len() {
                differences.push(format!(
                    "{}: length {}, expected {}",
                    path,
                    actual.len(),
                    expected.len()
                ));
                return;
            }
            for (index, (found, value)) in actual.iter().zip(expected).enumerate() {
                compare(&format!("{}[{}]", path, index), found, value, differences);
            }
        }
        (Json::Number(actual), Json::Number(expected)) => {
            let integers =
                (actual.is_i64() || actual.is_u64()) && (expected.is_i64() || expected.is_u64());
            if integers {
                if actual != expected {
                    differences.push(format!("{}: {}, expected {}", path, actual, expected));
                }
            } else {
                let found = actual.as_f64().unwrap();
                let value = expected.as_f64().unwrap();
                if (found - value).abs() > tolerance(path) {
                    differences.push(format!(
                        "{}: {}, expected {} within {}",
                        path,
                        actual,
                        expected,
                        tolerance(path)
                    ));
                }
            }
        }
        (Json::String(actual), Json::String(expected)) if actual == expected => {}
        (Json::Bool(actual), Json::Bool(expected)) if actual == expected => {}
        (Json::Null, Json::Null) => {}
        _ => differences.push(format!("{}: {:?}, expected {:?}", path, actual, expected)),
    }
}

#[test]
fn the_documented_example_is_what_luria_prints() {
    let page = fs::read_to_string("../../docs/specs/cli.md").unwrap();
    let request = block(&page, "<!-- luria:request -->");
    let result = block(&page, "<!-- luria:result -->");

    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("documented-example");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    fs::write(dir.join("d404n.json"), &request).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(["q5", "design", "d404n.json"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", text);

    let mut differences = Vec::new();
    compare(
        "$",
        &parse_json(&text).unwrap(),
        &parse_json(&result).unwrap(),
        &mut differences,
    );
    assert!(
        differences.is_empty(),
        "luria and docs/specs/cli.md disagree:\n{}",
        differences.join("\n")
    );

    let document: Value = serde_json::from_str(&text).unwrap();
    let documented: Value = serde_json::from_str(&result).unwrap();
    let digest = sha256_hex(request.as_bytes());
    assert_eq!(document["provenance"]["requestSha256"], digest.as_str());
    assert_eq!(documented["provenance"]["requestSha256"], digest.as_str());

    let version = Command::new(env!("CARGO_BIN_EXE_luria"))
        .arg("version")
        .output()
        .unwrap();
    assert_eq!(version.status.code(), Some(0));
    let version: Value = serde_json::from_slice(&version.stdout).unwrap();
    assert_eq!(version["algorithms"], document["provenance"]["algorithms"]);
    assert!(version["algorithms"].is_array());

    assert_eq!(document["experimentalValidation"], "not_performed");
    assert_eq!(document["state"], "verified");
}
