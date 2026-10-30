//! 9h §5 test 7: `terminal::save_step`, the part of a save that touches a
//! filesystem, run on names under a fresh directory in `target/tmp`.
//!
//! The three refusals of §4.3 (the opened file, a link to it, an existing other
//! file) are pinned here, where the files are, because `state.rs` never writes
//! (D003). The text a save hands the writer is a constant, so the bytes a file
//! ends up holding are exactly the ones an assertion names.
//!
//! §7's "the opened-file check is removed" row bites in
//! `the_opened_file_itself_is_refused` and `a_link_to_the_opened_file_is_refused`:
//! without that check a save would go over the file the view opened.

use luria_cli::sequence_view::terminal::save_step;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// The text a save hands the writer.
const TEXT: &str = "the text one save writes\n";

/// The bytes the opened file holds before a step.
const OPENED_BYTES: &str = "the record the view opened\n";

/// A fresh directory for one test under `target/tmp`, cleared first so that a
/// repeat run starts where the last one left off.
fn workspace(test: &str) -> PathBuf {
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("save-cli-{test}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// The file the view opened, written with its own bytes, as a path.
fn opened(dir: &Path) -> PathBuf {
    let path: PathBuf = dir.join("pbr322.gb");
    fs::write(&path, OPENED_BYTES).unwrap();
    path
}

/// §4.3: a name nothing holds is written, and the opened file keeps its bytes.
#[test]
fn a_new_name_is_written() {
    let dir: PathBuf = workspace("new_name");
    let opened: PathBuf = opened(&dir);
    let target: PathBuf = dir.join("pbr322.edited.gb");

    let outcome: Result<(), String> = save_step(&opened, &target, TEXT, &mut || -> bool {
        panic!("a name nothing holds asks no question")
    });

    assert!(outcome.is_ok(), "a free name is written: {outcome:?}");
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        TEXT,
        "the text the save handed"
    );
    assert_eq!(
        fs::read_to_string(&opened).unwrap(),
        OPENED_BYTES,
        "the file the view opened is left alone"
    );
}

/// §4.3: an empty name refuses, and nothing is written at all.
#[test]
fn an_empty_name_refuses() {
    let dir: PathBuf = workspace("empty_name");
    let opened: PathBuf = opened(&dir);

    let outcome: Result<(), String> = save_step(&opened, Path::new(""), TEXT, &mut || -> bool {
        panic!("an empty name refuses before any anything is asked")
    });

    assert_eq!(
        outcome.err(),
        Some(String::from("save: an empty name writes nothing")),
        "the empty name is refused by the terminal"
    );
    assert_eq!(
        fs::read_to_string(&opened).unwrap(),
        OPENED_BYTES,
        "and nothing else is written"
    );
}

/// §4.3: the opened file itself is never written, and the message says so.
#[test]
fn the_opened_file_itself_is_refused() {
    let dir: PathBuf = workspace("opened_itself");
    let opened: PathBuf = opened(&dir);

    let outcome: Result<(), String> = save_step(&opened, &opened, TEXT, &mut || -> bool {
        panic!("the opened file is refused before any ask")
    });

    assert_eq!(
        outcome.err(),
        Some(String::from("luria never writes over the file it opened")),
        "the same name as the opened file is refused"
    );
    assert_eq!(
        fs::read_to_string(&opened).unwrap(),
        OPENED_BYTES,
        "and the opened file still holds what it did"
    );
}

/// §4.3, §8: a link to the opened file is refused too, because the two paths
/// canonicalize to the one file, and the link and its target stay as they were.
#[cfg(unix)]
#[test]
fn a_link_to_the_opened_file_is_refused() {
    use std::os::unix::fs::symlink;

    let dir: PathBuf = workspace("opened_link");
    let opened: PathBuf = opened(&dir);
    let link: PathBuf = dir.join("pbr322-alias.gb");
    symlink(&opened, &link).unwrap();

    let outcome: Result<(), String> = save_step(&opened, &link, TEXT, &mut || -> bool {
        panic!("a link to the opened file is refused before any ask")
    });

    assert_eq!(
        outcome.err(),
        Some(String::from("luria never writes over the file it opened")),
        "the link canonicalizes to the opened file"
    );
    assert!(
        fs::symlink_metadata(&link)
            .expect("the link is still there")
            .file_type()
            .is_symlink(),
        "the link itself is unchanged"
    );
    assert_eq!(
        fs::read_to_string(&link).unwrap(),
        OPENED_BYTES,
        "and nothing was written through it"
    );
}

/// §4.3: an existing other file is asked about; a key that is not `y` keeps it.
#[test]
fn an_existing_other_file_is_kept_when_the_answer_is_not_yes() {
    let dir: PathBuf = workspace("keep");
    let opened: PathBuf = opened(&dir);
    let target: PathBuf = dir.join("other.gb");
    fs::write(&target, "the file that was already here\n").unwrap();

    let mut asked: bool = false;
    let mut answer = || {
        asked = true;
        false
    };
    let outcome: Result<(), String> = save_step(&opened, &target, TEXT, &mut answer);

    assert!(asked, "an existing name is asked about");
    let message: String = outcome.expect_err("keeping the file is an error");
    assert!(
        message.contains("exists"),
        "the line names the file: {message}"
    );
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "the file that was already here\n",
        "any key that is not `y` keeps the file"
    );
}

/// §4.3: `y` answers the ask, and the existing file is replaced, with
/// `replace: true`.
#[test]
fn an_existing_other_file_is_replaced_when_the_answer_is_yes() {
    let dir: PathBuf = workspace("replace");
    let opened: PathBuf = opened(&dir);
    let target: PathBuf = dir.join("other.gb");
    fs::write(&target, "the file that was already here\n").unwrap();

    let outcome: Result<(), String> = save_step(&opened, &target, TEXT, &mut || -> bool { true });

    assert!(outcome.is_ok(), "a y answer replaces the file: {outcome:?}");
    assert_eq!(fs::read_to_string(&target).unwrap(), TEXT);
}
