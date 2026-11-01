---
type: Task Contract
title: "Milestone 5g2: --write-edited"
description: "q5 design prints its result but writes no sequence text: plan §3.5 keeps the edited construct out of the JSON so a 64-member expansion of a plasmid does not produce a multi-megabyte document."
status: deprecated
tags: [milestone-5]
---
# Milestone 5g2: `--write-edited`

Status: **Implemented 2026-10-17** by the local worker, across two rounds.
Review found that a result name holding a path separator could write
outside `DIR`, a gap in this contract rather than the code. §4.1, step 1a,
§5.3 and one test were added, and the worker implemented them. Still
untested: the `WriteFailure::Io` path (exit 1 with a stderr note).
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.2, §3.5 and §5
(5g2). Follows 5g1 (the wiring).

## 1. What and why

`q5 design` prints its result but writes no sequence text: plan §3.5 keeps
the edited construct out of the JSON so a 64-member expansion of a plasmid
does not produce a multi-megabyte document. `--write-edited DIR` is how the
constructs get out, one FASTA file per member. 5g1 refuses the flag; this
contract implements it.

## 2. Requirements this contract applies

- **Plan §3.2:** with `--write-edited DIR`, each edited construct is written
  as FASTA at `DIR/<result-name>.fa`. An existing file is never overwritten.
- **G8 (D041, owner 2026-10-17):** `DIR` is created if it does not exist,
  parents included. An existing file is still never overwritten: that is
  `output_exists` and exit 4, with nothing written.
- **Plan §3.6:** `output_exists` is one of the CLI's own error codes, with
  category `MalformedInput`.
- **Plan §3.5, D012:** the result document is deterministic and holds no
  absolute paths. So neither the message nor the `details` of
  `output_exists` may name a directory: only the file's own name.
- **D007:** a sequence's identifier is the caller's, not the file's. The
  written header is the result name the caller asked for.

## 3. Rendering a FASTA record (in `crates/core/src/fasta.rs`)

The reader is in core, so its inverse belongs beside it, and the round trip
is then testable in one place.

```rust
/// Write one FASTA record: a header line of `>` and `name`, then the
/// sequence's letters, 60 to a line. Every line ends in `\n`, including the
/// last, so the text is a complete file.
///
/// The letters are the sequence's own canonical upper-case bases; nothing is
/// re-derived and no description is invented. 60 is this project's choice,
/// stated here because the committed fixtures use no single width (one is
/// 79, another is one long line) and `read_fasta` accepts any.
pub fn render_fasta(name: &Identifier, sequence: &Sequence) -> String;
```

A sequence is never empty (`normalize` refuses that), so the text always has
at least a header line and one base line.

## 4. Writing the files (in `crates/cli/src/write.rs`, new)

Add `pub mod write;` to `lib.rs`, after `pub mod run;`.

```rust
/// Why no edited construct was written.
#[derive(Clone, Debug, PartialEq)]
pub enum WriteFailure {
    /// A file that would be written already exists. Nothing is written, not
    /// even the files whose names are free (G8). The `String` is the file's
    /// own name, never its directory (D012).
    Exists(String),
    /// The directory could not be created, or a file could not be written.
    /// The `String` is the operating system's message.
    Io(String),
    /// A result name holds a path separator (`/` or `\`), so `<name>.fa`
    /// would not be a file inside the directory. Nothing is created. The
    /// `String` is the result name. (Planner amendment, 2026-10-17, §4.1.)
    NotAFileName(String),
}

/// Write each member's edited construct as FASTA into `directory`, one file
/// per artifact at `directory/<name>.fa`, and return the paths written in
/// the order given.
///
/// Nothing is created when `artifacts` is empty: a run that edited nothing
/// leaves no directory behind. Otherwise the directory is created first
/// (G8), then **every** path is checked, and only then is anything written,
/// so a refusal leaves the directory as it was.
pub fn write_edited(
    directory: &Path,
    artifacts: &[(Identifier, Sequence)],
) -> Result<Vec<PathBuf>, WriteFailure>;
```

Steps, in order:
1. `artifacts` empty: `Ok(vec![])`, and `directory` is not created.
1a. Any artifact's name contains `/` or `\`: `WriteFailure::NotAFileName`
   with the first such name, in artifact order. This comes **before**
   step 2, so nothing is created (§4.1).
2. `fs::create_dir_all(directory)`: on failure, `WriteFailure::Io` with the
   error's `to_string()`. An existing directory is not a failure.
3. Build each path as `directory.join(format!("{}.fa", name.value()))`.
4. For each path, `path.try_exists()`: `Ok(true)` is
   `WriteFailure::Exists(file_name)`, where `file_name` is
   `format!("{}.fa", name.value())`. An `Err` is `WriteFailure::Io`. Check
   every path before writing any file.
5. Write each file with `fs::write(path, render_fasta(name, sequence))`: on
   failure, `WriteFailure::Io`. Files already written are left as they are;
   step 4 has already established that none of them existed before.

### 4.1 Result names are file names (planner amendment, 2026-10-17)

Found in review of the first implementation: an `Identifier` may hold any
printable ASCII, so a request that named its edit `../escaped` wrote
`escaped.fa` beside `DIR` rather than inside it. A later milestone has a
language model write requests, so a request must not be able to choose where
luria writes. Step 1a refuses any name holding `/` or `\`, before anything
is created. That is enough: with no separator, `<name>.fa` is one path
component, and because it ends in `.fa` it is never `.` or `..`.

## 5. The run (in `crates/cli/src/run.rs`)

`run`'s signature does not change; 5g1's tests keep calling it.

```rust
/// Run one request file through core and report it, writing each edited
/// construct into `write_edited` when one was given (plan §3.2, G8).
pub fn run_with_output(request_path: &Path, write_edited: Option<&Path>) -> Outcome;

/// Run one request file through core and report it, writing nothing.
pub fn run(request_path: &Path) -> Outcome;
```

- `run(path)` is `run_with_output(path, None)`.
- The three path helpers each collect the artifacts they produced, as
  `Vec<(Identifier, Sequence)>` in member order:
  - a single edit: one pair, the member's name and `execution.edited()`'s
    value. The artifact exists whenever the edit was applied, so it is
    collected on the design-failure path too (G7): a construct that cannot
    be ordered as a primer pair is still a construct.
  - a set or an expansion: one pair per `execution.results()`, each
    `(result.identifier(), result.edited())`.
  - a clarification or a failure: none.
- After the outcome is built, and only when a directory was given, call
  `write_edited`. Then:
  - `Ok(_)`: the outcome is unchanged.
  - `Err(WriteFailure::Exists(name))`: the document becomes
    `failed_after(&prepared, view)` and the status
    `ExitStatus::MalformedInput`, where `view` is an `ErrorView` with
    `code: "output_exists"`, `rule: None`,
    `category: category_name(ErrorCategory::MalformedInput)`,
    `message: format!("will not overwrite the existing file '{}'", name)`,
    `path: None` and
    `details: Details(vec![("file", DetailValue::Text(name))])`.
    Nothing was written.
  - `Err(WriteFailure::Io(message))`: the document is **unchanged** — the
    run itself succeeded and the result is still true — and the status
    becomes `ExitStatus::Internal` (plan §3.7: an I/O failure is exit 1, and
    the document still goes to stdout). The message reaches the user through
    `main.rs`, so `Outcome` gains one field:

```rust
pub struct Outcome {
    pub document: ResultDocument,
    pub status: ExitStatus,
    /// A line for stderr, when something outside the document failed. The
    /// document is still the run's answer.
    pub note: Option<String>,
}
```

`note` is `None` on every other path. 5g1's tests construct no `Outcome`, so
adding the field breaks none of them.

### 5.1 `RequestErrorCode` (in `crates/cli/src/request.rs`)

Add two variants, `OutputExists`, whose `code()` is `"output_exists"`, and
`OutputNameUnusable`, whose `code()` is `"output_name_unusable"`, each with
a doc comment naming 5g2. Neither is produced by `parse_request`: they exist
so the CLI's codes are one list (plan §3.6). Change nothing else in that
file.

### 5.3 `NotAFileName` in `run.rs`

`Err(WriteFailure::NotAFileName(name))` is handled like `Exists`:
`failed_after(&prepared, view)` and `ExitStatus::MalformedInput`, where
`view` has `code: "output_name_unusable"`, `rule: None`, category
`malformed_input`,
`message: format!("the result name '{}' cannot be a file name: it contains a path separator", name)`,
`path: None` and `details: Details(vec![("name", DetailValue::Text(name))])`.

### 5.2 `main.rs`

- Pass the parsed `write_edited` through: the `Q5Design` arm calls
  `run_with_output(request, write_edited.as_deref())`.
- Print the document as now, then, when `outcome.note` is `Some(text)`,
  write `format!("luria: {}\n", text)` to stderr with the existing
  `write_err`. A failed stderr write changes nothing.
- Delete `WRITE_EDITED_UNAVAILABLE` and `write_edited_unavailable`.

## 6. Tests

### 6.1 In `crates/core/src/fasta.rs` (planner-written; append to its `mod tests`)

```rust
    // 60 bases a line, every line terminated, and the record reads back as
    // the sequence it was written from. The digest is sha256 of the 65-base
    // canonical text, computed independently:
    // python3 -c "import hashlib;print(hashlib.sha256((b'ACGT'*16+b'A')).hexdigest())"
    #[test]
    fn a_rendered_record_reads_back_as_the_same_sequence() {
        let name: Identifier = Identifier::create("edited-1".to_string()).unwrap();
        let text: String = "ACGT".repeat(16) + "A";
        let sequence: Sequence = normalize(
            name.clone(),
            text.clone(),
            SequenceTopology::Circular,
        )
        .unwrap();
        assert_eq!(sequence.length(), 65);
        assert_eq!(
            sequence.content_hash().hex_digest(),
            "8f9f1ddb55b44538520ac5adbac06717fc0840702a9dc2728d1f99ec99911a5a"
        );

        let rendered: String = render_fasta(&name, &sequence);
        assert_eq!(
            rendered,
            ">edited-1\n\
             ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT\n\
             ACGTA\n"
        );

        // The round trip: the same letters, length, topology and digest. The
        // name is the caller's, as `read_fasta` requires (D007).
        let record: FastaRecord =
            read_fasta(name.clone(), rendered, SequenceTopology::Circular).unwrap();
        let read_back: Sequence = record.sequence();
        assert_eq!(record.header(), "edited-1");
        assert_eq!(read_back.letters(), sequence.letters());
        assert_eq!(read_back.length(), 65);
        assert_eq!(read_back.topology(), SequenceTopology::Circular);
        assert_eq!(
            read_back.content_hash().hex_digest(),
            sequence.content_hash().hex_digest()
        );
    }

    // A sequence shorter than one line is one line, still terminated.
    #[test]
    fn a_short_record_is_one_line() {
        let name: Identifier = Identifier::create("t".to_string()).unwrap();
        let sequence: Sequence =
            normalize(name.clone(), "acgt".to_string(), SequenceTopology::Linear).unwrap();
        assert_eq!(render_fasta(&name, &sequence), ">t\nACGT\n");
    }

    // Exactly one line's worth is one line, with no empty second line.
    #[test]
    fn sixty_bases_are_one_line() {
        let name: Identifier = Identifier::create("t".to_string()).unwrap();
        let text: String = "A".repeat(60);
        let sequence: Sequence =
            normalize(name.clone(), text.clone(), SequenceTopology::Linear).unwrap();
        let rendered: String = render_fasta(&name, &sequence);
        assert_eq!(rendered, format!(">t\n{}\n", text));
        assert_eq!(rendered.lines().count(), 2);
    }
```

If `normalize`, `Identifier`, `FastaRecord`, `Sequence` or `SequenceTopology`
is not already in that `mod tests`'s scope, add the `use` it needs and
nothing else.

### 6.2 `crates/cli/tests/write_edited.rs` (planner-written; add verbatim)

`cargo fmt` may reflow it; do not change its content. The expected digests
are independently derived, not taken from luria: `D404N`'s is the
`sdmfinalseq.fa` row of `tests/fixtures/fasta/expected.tsv`, the construct
NEBaseChanger produced; `A20G`'s is sha256 of the SID-1 cDNA with base 20
changed from `A` to `G`, computed with Python's `hashlib`.

```rust
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
    assert!(!json.contains(first_line), "no sequence text in the document");
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
        "the result name '../escaped' cannot be a file name: it contains a path separator"
    );
    assert_eq!(
        serde_json::to_string(&error.details).unwrap(),
        r#"{"name":"../escaped"}"#
    );
    assert!(!out.exists(), "nothing is created");
    assert!(!dir.join("escaped.fa").exists(), "nothing escaped DIR");
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
```

### 6.3 Proving the rules are load-bearing (AGENTS.md)

Break each rule, run the named test, confirm it fails, restore.

| Rule | Where | Test that must fail when it is broken |
| --- | --- | --- |
| 60 bases a line (use 70) | `fasta.rs` | `a_rendered_record_reads_back_as_the_same_sequence` |
| the last line is terminated (drop the final `\n`) | `fasta.rs` | `a_short_record_is_one_line` |
| every path is checked before any is written (check and write in one loop) | `write.rs` | `one_existing_name_refuses_the_whole_set` |
| an existing file is refused (overwrite it) | `write.rs` | `an_existing_file_is_refused_and_nothing_is_written` |
| nothing is written when there are no artifacts (create the directory anyway) | `write.rs` | `a_clarification_writes_nothing_and_creates_no_directory` |
| an undesigned member's construct is still collected (skip it) | `run.rs` | `an_undesignable_edit_still_writes_its_construct` |
| the refusal names the file, not the path (use the whole path) | `run.rs` | `an_existing_file_is_refused_and_nothing_is_written` |
| a name with a path separator is refused (drop step 1a) | `write.rs` | `a_result_name_with_a_path_separator_is_refused` |

## 7. Do not

- Edit files other than `crates/core/src/fasta.rs`, `crates/cli/src/write.rs`
  (new), `crates/cli/src/run.rs`, `crates/cli/src/request.rs` (one variant),
  `crates/cli/src/lib.rs` (one `pub mod` line), `crates/cli/src/main.rs` and
  `crates/cli/tests/write_edited.rs` (new).
- Change any existing test. 5g1's `crates/cli/tests/run.rs` must keep
  passing unchanged, including its `write_edited_is_refused_in_this_build`
  test — **delete that one test only**, because this contract is what makes
  it false, and say so in the report.
- Write a GenBank file, write anything when no directory was given, or put
  sequence text in the JSON document.
- Re-derive a sequence, a digest or a design. `render_fasta` reads the
  sequence's own letters.
- Add dependencies.

## 8. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 9. Handoff

Report the changed and new files, the exact commands and results, the §6.3
results, and anything that did not match this contract. If an expected
digest or text disagrees with what the code produces, report the test, the
expected value and the actual one rather than changing the test.
