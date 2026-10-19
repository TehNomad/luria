//! `--write-edited DIR`: one FASTA file per member, the directory created
//! when it is missing (G8), and an existing file never overwritten.
//!
//! The written construct is read back with core's own reader and its digest
//! compared with an independently derived one, so the file's content is
//! checked rather than its bytes.

use luria_cli::exit::ExitStatus;
use luria_cli::run::{run_with_output, Outcome};
use luria_core::fasta::read_fasta;
use luria_core::identity::Identifier;
use luria_core::sequence::SequenceTopology;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The construct NEBaseChanger produced for D404N: the `sdmfinalseq.fa` row
/// of `tests/fixtures/fasta/expected.tsv`.
const D404N_DIGEST: &str = "6ab7d5a6fdd995692ed5cda5079db64c97a70935deb9edfd10fe402628b7ff50";

/// The SID-1 cDNA with base 20 changed from `A` to `G`, by `hashlib`.
const A20G_DIGEST: &str = "96aeab438dba87711aff8db636d1178a8c6a2bf9b82a0f45fcbffec4ffdfd2a8";

fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("write-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        dir.join("J01749.1.gb"),
    )
    .unwrap();
    dir
}

fn write(dir: &Path, request: &Value) -> PathBuf {
    let path = dir.join("request.json");
    fs::write(&path, request.to_string()).unwrap();
    path
}

fn design() -> Value {
    json!({
        "policy": "luriaNative",
        "style": "q5_polymerase",
        "minAnnealingOverlap": 15,
        "placement": "inline",
        "buffer": "q5_reaction"
    })
}

fn sid1_template() -> Value {
    json!({
        "id": "sid-1",
        "fasta": "sid-1_cDNA.fa",
        "topology": "linear",
        "cds": [{
            "id": "sid-1",
            "intervals": [[0, 3153]],
            "strand": "+",
            "completeness": "complete",
            "geneticCode": "standard",
            "numbering": "from-cds-start"
        }]
    })
}

fn d404n() -> Value {
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": sid1_template(),
        "scope": "sid-1",
        "edit": {
            "name": "D404N",
            "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" }
        },
        "design": design()
    })
}

/// The digest of the sequence in a written FASTA file, read back by core.
fn digest_of(path: &Path) -> String {
    let name = Identifier::create("read-back".to_string()).unwrap();
    let text = fs::read_to_string(path).unwrap();
    let record = read_fasta(name, text, SequenceTopology::Linear).unwrap();
    record.sequence().content_hash().hex_digest().to_string()
}

/// Every entry of a directory, by file name, sorted.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

fn ran(test: &str, request: &Value, out: &str) -> (Outcome, PathBuf) {
    let dir = workspace(test);
    let output = dir.join(out);
    let outcome = run_with_output(&write(&dir, request), Some(&output));
    (outcome, output)
}

#[test]
fn the_edited_construct_is_written_as_fasta() {
    let (outcome, out) = ran("d404n", &d404n(), "out");
    assert_eq!(outcome.status, ExitStatus::Success);
    assert_eq!(outcome.note, None);
    assert_eq!(entries(&out), vec!["D404N.fa"]);
    assert_eq!(digest_of(&out.join("D404N.fa")), D404N_DIGEST);

    // The header is the result name, and the file is a complete FASTA text.
    let text = fs::read_to_string(out.join("D404N.fa")).unwrap();
    assert!(text.starts_with(">D404N\n"), "{}", &text[..16]);
    assert!(text.ends_with('\n'));
    // 3153 bases, 60 to a line, is 53 lines, plus the header.
    assert_eq!(text.lines().count(), 1 + 53);

    // The document still carries no sequence text (plan §3.5): not the
    // construct, and not the first line of it either.
    let json = serde_json::to_string(&outcome.document).unwrap();
    let first_line: &str = text.lines().nth(1).unwrap();
    assert_eq!(first_line.len(), 60);
    assert!(
        !json.contains(first_line),
        "no sequence text in the document"
    );
}

// G8: a directory that does not exist is created, parents included.
#[test]
fn a_missing_output_directory_is_created() {
    let (outcome, out) = ran("missing", &d404n(), "a/b/c");
    assert_eq!(outcome.status, ExitStatus::Success);
    assert!(out.is_dir());
    assert_eq!(entries(&out), vec!["D404N.fa"]);
}

#[test]
fn a_variant_set_writes_one_file_per_member() {
    let (outcome, out) = ran(
        "set",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": sid1_template(),
            "scope": "sid-1",
            "variants": {
                "composition": "separate_variants",
                "members": [
                    { "name": "s1", "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" } },
                    { "name": "d1", "deletion": { "interval": [1209, 1212], "from": "GAT" } },
                    { "name": "i1", "insertion": { "point": 1209, "before": "A", "after": "G", "insert": "GAT" } }
                ]
            },
            "design": design()
        }),
        "out",
    );
    assert_eq!(outcome.status, ExitStatus::Success);
    assert_eq!(entries(&out), vec!["d1.fa", "i1.fa", "s1.fa"]);
    // s1 is the same edit as D404N, so it is the same construct.
    assert_eq!(digest_of(&out.join("s1.fa")), D404N_DIGEST);
    // The deletion is three bases shorter, the insertion three longer.
    let name = Identifier::create("x".to_string()).unwrap();
    let deleted = read_fasta(
        name.clone(),
        fs::read_to_string(out.join("d1.fa")).unwrap(),
        SequenceTopology::Linear,
    )
    .unwrap();
    assert_eq!(deleted.sequence().length(), 3150);
    let inserted = read_fasta(
        name,
        fs::read_to_string(out.join("i1.fa")).unwrap(),
        SequenceTopology::Linear,
    )
    .unwrap();
    assert_eq!(inserted.sequence().length(), 3156);
}

// G7 and plan §3.2: the edit was applied, so its construct is written even
// though no primer pair fits it.
#[test]
fn an_undesignable_edit_still_writes_its_construct() {
    let (outcome, out) = ran(
        "undesignable",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": sid1_template(),
            "edit": {
                "name": "A20G",
                "substitution": { "interval": [20, 21], "from": "A", "to": "G" }
            },
            "design": design()
        }),
        "out",
    );
    assert_eq!(outcome.status, ExitStatus::PartialDesign);
    assert_eq!(entries(&out), vec!["A20G.fa"]);
    assert_eq!(digest_of(&out.join("A20G.fa")), A20G_DIGEST);
}

// G8: an existing file is never overwritten, and nothing is written.
#[test]
fn an_existing_file_is_refused_and_nothing_is_written() {
    let dir = workspace("exists");
    let out = dir.join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("D404N.fa"), ">mine\nACGT\n").unwrap();

    let outcome = run_with_output(&write(&dir, &d404n()), Some(&out));
    assert_eq!(outcome.status, ExitStatus::MalformedInput);
    assert_eq!(outcome.status.code(), 4);
    assert_eq!(outcome.document.state, "failed");
    assert_eq!(outcome.document.results.len(), 0);

    let error = outcome.document.error.clone().unwrap();
    assert_eq!(error.code, "output_exists");
    assert_eq!(error.category, "malformed_input");
    assert_eq!(error.path, None);
    assert_eq!(
        error.message,
        "will not overwrite the existing file 'D404N.fa'"
    );
    // D012: the document names the file, never its directory.
    assert_eq!(
        serde_json::to_string(&error.details).unwrap(),
        r#"{"file":"D404N.fa"}"#
    );
    let json = serde_json::to_string(&outcome.document).unwrap();
    assert!(!json.contains(out.to_str().unwrap()), "no absolute path");

    // The file is as it was, and it is the only one.
    assert_eq!(
        fs::read_to_string(out.join("D404N.fa")).unwrap(),
        ">mine\nACGT\n"
    );
    assert_eq!(entries(&out), vec!["D404N.fa"]);
}

// One existing name refuses the whole set: the free names are not written
// either, so a re-run after fixing the directory writes all of them.
#[test]
fn one_existing_name_refuses_the_whole_set() {
    let dir = workspace("exists-set");
    let out = dir.join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("d1.fa"), ">mine\nACGT\n").unwrap();

    let outcome = run_with_output(
        &write(
            &dir,
            &json!({
                "schema": "luria.q5-request/1",
                "coordinates": "zero-based-half-open",
                "template": sid1_template(),
                "scope": "sid-1",
                "variants": {
                    "composition": "separate_variants",
                    "members": [
                        { "name": "s1", "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" } },
                        { "name": "d1", "deletion": { "interval": [1209, 1212], "from": "GAT" } }
                    ]
                },
                "design": design()
            }),
        ),
        Some(&out),
    );
    assert_eq!(outcome.status, ExitStatus::MalformedInput);
    assert_eq!(
        outcome.document.error.clone().unwrap().message,
        "will not overwrite the existing file 'd1.fa'"
    );
    assert_eq!(entries(&out), vec!["d1.fa"], "s1.fa was not written");
}

// D005: a clarification edits nothing, so there is nothing to write and no
// directory is left behind.
#[test]
fn a_clarification_writes_nothing_and_creates_no_directory() {
    let dir = workspace("clarify");
    let out = dir.join("out");
    let outcome = run_with_output(
        &write(
            &dir,
            &json!({
                "schema": "luria.q5-request/1",
                "coordinates": "zero-based-half-open",
                "template": { "id": "pbr322", "genbank": "J01749.1.gb", "cds": ["tet", "bla"] },
                "edit": {
                    "name": "G42",
                    "residue": { "number": 42, "from": "A", "to": "G", "policy": "default" }
                },
                "design": design()
            }),
        ),
        Some(&out),
    );
    assert_eq!(outcome.status, ExitStatus::NeedsClarification);
    assert!(!out.exists(), "no directory for a run that edited nothing");
}

// A failed run writes nothing either.
#[test]
fn a_failed_run_writes_nothing() {
    let dir = workspace("failed");
    let out = dir.join("out");
    let outcome = run_with_output(&dir.join("absent.json"), Some(&out));
    assert_eq!(outcome.status, ExitStatus::MalformedInput);
    assert_eq!(
        outcome.document.error.clone().unwrap().code,
        "request_unreadable"
    );
    assert!(!out.exists());
}

// The flag works through the binary now: 5g1's refusal is gone.
#[test]
fn the_binary_accepts_write_edited() {
    let dir = workspace("binary");
    let out = dir.join("out");
    let path = write(&dir, &d404n());
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args([
            "q5",
            "design",
            path.to_str().unwrap(),
            "--write-edited",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
    let document: Value = serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert_eq!(document["state"], "verified");
    assert_eq!(entries(&out), vec!["D404N.fa"]);
    assert_eq!(digest_of(&out.join("D404N.fa")), D404N_DIGEST);
}

// §3.7: an I/O failure is exit 1, the document still goes to stdout and the
// operating system's message goes to stderr. `afile` is a regular file, so
// creating the output directory `afile/x` cannot succeed; the run itself still
// succeeded, so its document is unchanged.
#[test]
fn an_io_failure_prints_the_document_and_exits_1() {
    let dir = workspace("io-failure");
    fs::write(dir.join("afile"), "not a directory\n").unwrap();
    let path = write(&dir, &d404n());

    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args([
            "q5",
            "design",
            path.to_str().unwrap(),
            "--write-edited",
            dir.join("afile/x").to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "an I/O failure exits 1");
    let document: Value = serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert_eq!(
        document["state"], "verified",
        "the run's own document is still printed"
    );
    let errors: String = String::from_utf8(output.stderr).unwrap();
    assert!(
        errors.starts_with("luria: "),
        "the operating system's message on stderr: {}",
        errors
    );
}

// §4.1: a result name becomes a file name, so one holding a path separator
// is refused before anything is created. A request cannot write outside DIR.
#[test]
fn a_result_name_with_a_path_separator_is_refused() {
    let dir = workspace("separator");
    let out = dir.join("out");
    let mut request = d404n();
    request["edit"]["name"] = json!("../escaped");
    let outcome = run_with_output(&write(&dir, &request), Some(&out));
    assert_eq!(outcome.status, ExitStatus::MalformedInput);
    assert_eq!(outcome.document.state, "failed");
    let error = outcome.document.error.clone().unwrap();
    assert_eq!(error.code, "output_name_unusable");
    assert_eq!(error.path, None);
    assert_eq!(
        error.message,
        "the result name '../escaped' cannot be a file name: it contains a path separator or one of : * ? < > |"
    );
    assert_eq!(
        serde_json::to_string(&error.details).unwrap(),
        r#"{"name":"../escaped"}"#
    );
    assert!(!out.exists(), "nothing is created");
    assert!(!dir.join("escaped.fa").exists(), "nothing escaped DIR");
}

// C-1: a dangling symlink reads as a free name, so the write went through it
// to wherever it points. The check must see the path itself (G8: an existing
// file is never overwritten, and here there is a link rather than nothing).
#[cfg(unix)]
#[test]
fn a_dangling_symlink_is_refused_and_nothing_is_written_through_it() {
    use std::os::unix::fs::symlink;

    let dir = workspace("symlink");
    let out = dir.join("out");
    fs::create_dir_all(&out).unwrap();
    symlink("../elsewhere.fa", out.join("D404N.fa")).unwrap();
    assert!(
        !dir.join("elsewhere.fa").exists(),
        "the link's target is missing"
    );

    let outcome = run_with_output(&write(&dir, &d404n()), Some(&out));
    assert_eq!(outcome.status, ExitStatus::MalformedInput);
    assert_eq!(outcome.status.code(), 4);
    assert_eq!(outcome.document.state, "failed");
    assert_eq!(outcome.document.results.len(), 0);
    let error = outcome.document.error.clone().unwrap();
    assert_eq!(error.code, "output_exists");
    assert_eq!(error.category, "malformed_input");
    assert_eq!(error.path, None);
    assert_eq!(
        error.message,
        "will not overwrite the existing file 'D404N.fa'"
    );
    assert_eq!(
        serde_json::to_string(&error.details).unwrap(),
        r#"{"file":"D404N.fa"}"#
    );

    // The link is still there, still dangling, and nothing was created at its
    // target: the run never wrote through it.
    assert_eq!(
        entries(&out),
        vec!["D404N.fa"],
        "the link itself is unchanged"
    );
    assert!(
        !dir.join("elsewhere.fa").exists(),
        "nothing written outside DIR"
    );
}

// C-2: two result names that differ only in case name the same file on a
// case-insensitive file system, while the document reports both members. The
// later name is refused and nothing at all is created (G8).
#[test]
fn two_names_differing_only_in_case_are_refused() {
    let dir = workspace("case-clash");
    let out = dir.join("out");
    let outcome = run_with_output(
        &write(
            &dir,
            &json!({
                "schema": "luria.q5-request/1",
                "coordinates": "zero-based-half-open",
                "template": sid1_template(),
                "scope": "sid-1",
                "variants": {
                    "composition": "separate_variants",
                    "members": [
                        { "name": "D404N", "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" } },
                        { "name": "d404n", "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" } }
                    ]
                },
                "design": design()
            }),
        ),
        Some(&out),
    );
    assert_eq!(outcome.status, ExitStatus::MalformedInput);
    assert_eq!(outcome.status.code(), 4);
    assert_eq!(outcome.document.state, "failed");
    assert_eq!(outcome.document.results.len(), 0);
    let error = outcome.document.error.clone().unwrap();
    assert_eq!(error.code, "output_name_unusable");
    assert_eq!(error.category, "malformed_input");
    assert_eq!(error.path, None);
    assert_eq!(
        error.message,
        "the result name 'd404n' differs from an earlier one only in case, so their files would collide"
    );
    assert_eq!(
        serde_json::to_string(&error.details).unwrap(),
        r#"{"name":"d404n"}"#
    );

    // The refusal comes before anything is created, so DIR stays absent.
    assert!(!out.exists(), "nothing is created");
}

// C-2: the characters a portable file name may not hold are refused the same
// way as a path separator. `Identifier` accepts every printable ASCII token,
// so each of these reaches `write_edited`, and each name reaches the file-name
// check with nothing created.
#[test]
fn an_unportable_character_in_a_result_name_is_refused() {
    let characters: Vec<char> = [':', '*', '?', '<', '>', '|'].to_vec();
    for (index, character) in characters.iter().enumerate() {
        let name: String = format!("a{}b", character);
        let dir = workspace(&format!("unportable-{}", index));
        let out = dir.join("out");
        let mut request: Value = d404n();
        request["edit"]["name"] = json!(name.clone());

        let outcome = run_with_output(&write(&dir, &request), Some(&out));
        assert_eq!(
            outcome.status,
            ExitStatus::MalformedInput,
            "name {}",
            name.clone()
        );
        assert_eq!(outcome.document.state, "failed", "name {}", name.clone());
        assert_eq!(outcome.document.results.len(), 0, "name {}", name.clone());
        let error = outcome.document.error.clone().unwrap();
        assert_eq!(error.code, "output_name_unusable", "name {}", name.clone());
        assert_eq!(error.category, "malformed_input", "name {}", name.clone());
        assert_eq!(error.path, None, "name {}", name.clone());
        assert_eq!(
            error.message,
            format!(
                "the result name '{}' cannot be a file name: it contains a path separator or one of : * ? < > |",
                name.clone(),
            ),
            "name {}",
            name.clone(),
        );
        assert_eq!(
            serde_json::to_string(&error.details).unwrap(),
            json!({ "name": name.clone() }).to_string(),
            "name {}",
            name.clone(),
        );
        assert!(
            !out.exists(),
            "the directory was not created for name {}",
            name.clone()
        );
    }
}

// The same request twice, into two directories, gives byte-identical files
// (D012). 5g3 proves the same of the document.
#[test]
fn the_written_construct_is_the_same_both_times() {
    let (_, first) = ran("twice-a", &d404n(), "out");
    let (_, second) = ran("twice-b", &d404n(), "out");
    assert_eq!(
        fs::read(first.join("D404N.fa")).unwrap(),
        fs::read(second.join("D404N.fa")).unwrap()
    );
}
