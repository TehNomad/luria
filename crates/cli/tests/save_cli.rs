//! 9l §5 tests 11 to 16: `terminal::save_step`, the part of a write that touches
//! a filesystem, run on names under a fresh directory in `target/tmp`.
//!
//! The state writes no file (D003), so what 9l §4.6 and §4.7 add - the opened
//! file's original kept once per session at `<opened>~` before the opened file is
//! written over - is pinned here, where the files are. The text a write hands the
//! writer is a constant, so the bytes a file ends up holding are exactly the ones
//! an assertion names, and the `backed_up` flag the session owns is passed as a
//! local so a test can watch what it says afterwards.
//!
//! §7's rows bite here in three tests: "No backup is made" and "The backup is
//! made on every write, ignoring `backed_up`" in
//! `the_opened_file_is_written_after_a_backup`, and "save_step writes an existing
//! other file when `replace` is false" in `an_existing_other_file_needs_replace`.
//! The link tests pin §4.7's other half: a link that leads to the opened file
//! writes the opened file (test 12), and an opened path that is itself a link
//! writes nothing at all (test 13).

use luria_cli::sequence_view::terminal::save_step;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// The text one write hands the writer.
const TEXT: &str = "the text one write makes\n";

/// A second text, for the write that follows the first.
const TEXT2: &str = "the text the next write makes\n";

/// The bytes the opened file holds before any write.
const ORIGINAL: &str = "the record the view opened\n";

/// A fresh directory for one test under `target/tmp`, cleared first so that a
/// repeat run starts where the last one left off.
fn workspace(test: &str) -> PathBuf {
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("save-cli-{test}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// The file the view opened, holding 9l's `ORIGINAL`, as a path.
fn opened(dir: &Path) -> PathBuf {
    let path: PathBuf = dir.join("pbr322.gb");
    fs::write(&path, ORIGINAL).unwrap();
    path
}

/// §4.7's name for one file's backup: the file name with `~` on the end, in the
/// same directory.
fn backup_of(path: &Path) -> PathBuf {
    let mut name: OsString = path.file_name().unwrap().to_os_string();
    name.push("~");
    path.with_file_name(name)
}

/// The names in `dir` holding a `~`, which is what §4.7 promises about.
fn backups(dir: &Path) -> Vec<OsString> {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| name.to_string_lossy().contains('~'))
        .collect()
}

/// 9l §5 test 11: writing the opened file keeps its original first, once a
/// session. The call reports the backup it made; a second write with the flag the
/// first one set makes none, and the backup still holds what the file did before
/// the session wrote it (§4.6, §4.7 step 2).
#[test]
fn the_opened_file_is_written_after_a_backup() {
    let dir: PathBuf = workspace("opened_backup");
    let a: PathBuf = dir.join("a.gb");
    fs::write(&a, ORIGINAL).unwrap();
    let backup: PathBuf = backup_of(&a);

    let mut backed_up: bool = false;
    let outcome: Result<Option<PathBuf>, String> = save_step(&a, &a, TEXT, false, &mut backed_up);
    assert_eq!(
        outcome,
        Ok(Some(backup.clone())),
        "the opened file is written, and the call says which backup it made"
    );
    assert_eq!(
        fs::read_to_string(&backup).unwrap(),
        ORIGINAL,
        "the backup holds what the opened file held"
    );
    assert_eq!(
        fs::read_to_string(&a).unwrap(),
        TEXT,
        "and the write went to the opened file"
    );
    assert!(backed_up, "the session's flag says the original is kept");

    // The next write of the same session: no second backup, so the one file still
    // names the original rather than what the last write left there (§4.6).
    let outcome: Result<Option<PathBuf>, String> = save_step(&a, &a, TEXT2, false, &mut backed_up);
    assert_eq!(
        outcome,
        Ok(None),
        "a session writes the opened file once only from the original"
    );
    assert_eq!(
        fs::read_to_string(&a).unwrap(),
        TEXT2,
        "the new text is in the opened file"
    );
    assert_eq!(
        fs::read_to_string(&backup).unwrap(),
        ORIGINAL,
        "the backup still holds the original, not what the first write left"
    );
    assert_eq!(
        backups(&dir),
        vec![backup.file_name().unwrap().to_os_string()],
        "and one backup file, not two"
    );
}

/// 9l §5 test 12: a name that is a link to the opened file is the opened file, so
/// the write goes to the file and the link stays a link; the backup is the opened
/// file's own name with `~` (§4.7 step 2).
#[cfg(unix)]
#[test]
fn a_link_to_the_opened_file_writes_the_file() {
    use std::os::unix::fs::symlink;

    let dir: PathBuf = workspace("link_to_opened");
    let a: PathBuf = dir.join("a.gb");
    fs::write(&a, ORIGINAL).unwrap();
    let link: PathBuf = dir.join("link");
    symlink(&a, &link).unwrap();

    let mut backed_up: bool = false;
    let outcome: Result<Option<PathBuf>, String> =
        save_step(&a, &link, TEXT, false, &mut backed_up);
    assert!(
        outcome.is_ok(),
        "the opened file is written through its link: {outcome:?}"
    );
    assert_eq!(
        fs::read_to_string(&a).unwrap(),
        TEXT,
        "the file the link leads to"
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
        TEXT,
        "and it reads the file it points at"
    );
    assert_eq!(
        fs::read_to_string(backup_of(&a)).unwrap(),
        ORIGINAL,
        "the backup is named for the opened file, not the link"
    );
}

/// 9l §5 test 13: an opened path that is itself a link is not written, because
/// writing it would make the link into a file. Nothing is written anywhere, the
/// file behind the link included, and no backup is made (§4.7 step 2).
#[cfg(unix)]
#[test]
fn an_opened_link_is_not_written() {
    use std::os::unix::fs::symlink;

    let dir: PathBuf = workspace("opened_is_link");
    let real: PathBuf = dir.join("real.gb");
    fs::write(&real, ORIGINAL).unwrap();
    let link: PathBuf = dir.join("link.gb");
    symlink(&real, &link).unwrap();

    let mut backed_up: bool = false;
    let outcome: Result<Option<PathBuf>, String> =
        save_step(&link, &link, TEXT, false, &mut backed_up);
    let message: String = outcome.expect_err("an opened link is refused");
    assert_eq!(
        message,
        format!(
            "{} is a link, not a file: :w NAME writes elsewhere",
            link.display()
        ),
        "the words name the link and say where to write instead"
    );
    assert_eq!(
        fs::read_to_string(&real).unwrap(),
        ORIGINAL,
        "the file behind the link is untouched"
    );
    assert!(
        fs::symlink_metadata(&link)
            .expect("the link is still there")
            .file_type()
            .is_symlink(),
        "the link is still a link"
    );
    assert!(
        backups(&dir).is_empty(),
        "and no backup was made: {:?}",
        backups(&dir)
    );
    assert!(!backed_up, "nor does the session think one is kept");
}

/// 9l §5 test 14: a backup that cannot be written stops the write, and the opened
/// file keeps its bytes rather than being written over without a copy of them
/// (§4.7 step 2).
#[test]
fn a_backup_that_cannot_be_written_writes_nothing() {
    let dir: PathBuf = workspace("backup_dir");
    let a: PathBuf = dir.join("a.gb");
    fs::write(&a, ORIGINAL).unwrap();
    let backup: PathBuf = backup_of(&a);
    fs::create_dir(&backup).unwrap();

    let mut backed_up: bool = false;
    let outcome: Result<Option<PathBuf>, String> = save_step(&a, &a, TEXT, false, &mut backed_up);
    let message: String = outcome.expect_err("a backup that cannot be made refuses the write");
    assert!(
        message.contains(&backup.display().to_string()),
        "the words name the backup: {message}"
    );
    assert!(
        message.ends_with("; nothing written"),
        "and say nothing was written: {message}"
    );
    assert_eq!(
        fs::read_to_string(&a).unwrap(),
        ORIGINAL,
        "the opened file still holds the original"
    );
    assert!(
        backup.is_dir(),
        "the directory at the backup's name is untouched"
    );
    assert!(!backed_up, "and the session still owes the backup");
}

/// 9l §5 test 15: an existing file that is not the opened one is written only
/// when the command typed `!`, and no backup belongs anywhere near it (§4.7 steps
/// 2 and 3).
#[test]
fn an_existing_other_file_needs_replace() {
    let dir: PathBuf = workspace("existing_other");
    let opened: PathBuf = opened(&dir);
    let other: PathBuf = dir.join("other.gb");
    fs::write(&other, "the file that was already here\n").unwrap();

    let mut backed_up: bool = false;
    let outcome: Result<Option<PathBuf>, String> =
        save_step(&opened, &other, TEXT, false, &mut backed_up);
    let message: String = outcome.expect_err("an existing name needs `!`");
    assert!(
        message.contains(":w! replaces it"),
        "the words say what to type instead: {message}"
    );
    assert_eq!(
        fs::read_to_string(&other).unwrap(),
        "the file that was already here\n",
        "the file that was there is unchanged"
    );
    assert_eq!(
        fs::read_to_string(&opened).unwrap(),
        ORIGINAL,
        "and so is the opened file"
    );
    assert!(
        backups(&dir).is_empty(),
        "a refused write makes no backup: {:?}",
        backups(&dir)
    );
    assert!(!backed_up, "and the session still owes one");

    let outcome: Result<Option<PathBuf>, String> =
        save_step(&opened, &other, TEXT, true, &mut backed_up);
    assert!(
        outcome.is_ok(),
        "`replace` writes the file that was there: {outcome:?}"
    );
    assert_eq!(
        outcome.unwrap(),
        None,
        "no backup was made for another file"
    );
    assert_eq!(
        fs::read_to_string(&other).unwrap(),
        TEXT,
        "the text the write handed"
    );
    assert_eq!(
        fs::read_to_string(&opened).unwrap(),
        ORIGINAL,
        "the opened file keeps its own bytes"
    );
    assert!(
        backups(&dir).is_empty(),
        "and no `~` file is made: {:?}",
        backups(&dir)
    );
}

/// 9l §5 test 16, first half: a name nothing holds is written, as 9h had it, with
/// `replace` false whatever the command typed (§4.7 step 4).
#[test]
fn a_new_name_is_written() {
    let dir: PathBuf = workspace("new_name");
    let opened: PathBuf = opened(&dir);
    let target: PathBuf = dir.join("pbr322.edited.gb");

    let mut backed_up: bool = false;
    let outcome: Result<Option<PathBuf>, String> =
        save_step(&opened, &target, TEXT, false, &mut backed_up);
    assert!(outcome.is_ok(), "a free name is written: {outcome:?}");
    assert_eq!(outcome.unwrap(), None, "a new name makes no backup");
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        TEXT,
        "the text the write handed"
    );
    assert_eq!(
        fs::read_to_string(&opened).unwrap(),
        ORIGINAL,
        "the file the view opened is left alone"
    );
    assert!(
        backups(&dir).is_empty(),
        "and nothing else was made: {:?}",
        backups(&dir)
    );
}

/// 9l §5 test 16, second half: an empty name refuses, before any backup and
/// before any write at all (§4.7 step 1).
#[test]
fn an_empty_name_refuses() {
    let dir: PathBuf = workspace("empty_name");
    let opened: PathBuf = opened(&dir);

    let mut backed_up: bool = false;
    let outcome: Result<Option<PathBuf>, String> =
        save_step(&opened, Path::new(""), TEXT, false, &mut backed_up);
    assert_eq!(
        outcome.err(),
        Some(String::from("save: an empty name writes nothing")),
        "the empty name is refused by the terminal"
    );
    assert_eq!(
        fs::read_to_string(&opened).unwrap(),
        ORIGINAL,
        "and nothing else is written"
    );
    assert!(
        !backed_up,
        "the empty name refuses before a backup is asked for"
    );
}
