//! Contract 8h §5: `luria FILE`, run as a process.
//!
//! The tests have no terminal on stdin or stdout, so none of them can open
//! the view: what they prove is §4's order in front of it — every subcommand
//! word first (rule 1), then the path (rule 2), then the terminal check
//! before a byte of the file is read (rule 4), and that a word naming no
//! path is the unknown-command error, with or without a suggestion (rule 3).
//! Whether `luria FILE` opens the screen when a terminal is there is the
//! owner's hand test (8k1), not one of these.

use luria_cli::args::USAGE;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Run one `luria` command line, as `cli.rs` runs the executable: no terminal
/// on stdin, and stdout and stderr captured.
fn luria(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(arguments)
        .output()
        .expect("the luria executable runs")
}

/// The same, run in `directory` — so a bare word is looked up there.
fn luria_in(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("the luria executable runs")
}

/// The test's own directory, the way `find_cli.rs` makes one: under `target/`,
/// which is gitignored.
fn workspace(test: &str) -> PathBuf {
    let dir: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("open-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the test directory is created");
    dir
}

/// pBR322 copied into that directory, and its absolute path: the message names
/// the file as the command line gave it, so an absolute path makes an absolute
/// expectation.
fn pbr(test: &str) -> PathBuf {
    let path: PathBuf = workspace(test).join("J01749.1.gb");
    fs::copy("../../tests/fixtures/genbank/raw/J01749.1.gb", &path)
        .expect("the pBR322 fixture is copied");
    path
}

/// One file of these exact bytes, written into the test's directory.
fn written(test: &str, name: &str, bytes: &[u8]) -> PathBuf {
    let path: PathBuf = workspace(test).join(name);
    fs::write(&path, bytes).expect("the file is written");
    path
}

/// stdout or stderr as text; every case here prints text.
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("the output is UTF-8")
}

/// 1: a file that exists is not a command, and with no terminal the view
/// cannot open — the usage error D048 M8-13 gives, with the usage text (plan
/// §5.6).
#[test]
fn a_file_needs_a_terminal() {
    let record: PathBuf = pbr("needs-a-terminal");
    let output: Output = luria(&[record.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let printed: String = text(&output.stderr);
    assert!(
        printed.starts_with(&format!(
            "luria: '{}' needs a terminal on stdin and stdout",
            record.display()
        )),
        "{}",
        printed
    );
    assert!(printed.contains(USAGE), "{}", printed);
    // Rule 4's other half: the alternative the view points at is `find`, not
    // `q5 design`. §5 test 1 also requires the usage text in stderr, and
    // `USAGE` names `q5 design` twice, so "no q5 design" is read as the
    // message line alone (reported to the planner).
    assert!(
        printed.contains("'luria find FILE PATTERN' searches a file without one"),
        "{}",
        printed
    );
    let message: String = printed
        .lines()
        .next()
        .expect("the usage error is printed on its own line")
        .to_string();
    assert!(!message.contains("q5 design"), "{}", message);
}

/// 2: the terminal check runs before the file is read, so a file that is not
/// UTF-8 gives the terminal error and not `is not UTF-8 text` (plan §5.6: a
/// bad path and a missing terminal cannot both claim the error).
#[test]
fn the_terminal_comes_before_the_read() {
    let path: PathBuf = written("terminal-before-read", "not-utf8.gb", b">x\n\xff\n");
    let output: Output = luria(&[path.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let printed: String = text(&output.stderr);
    assert!(
        printed.contains(&format!(
            "'{}' needs a terminal on stdin and stdout",
            path.display()
        )),
        "{}",
        printed
    );
    assert!(!printed.contains("is not UTF-8 text"), "{}", printed);
}

/// 3: the view takes the file and nothing else — a pattern beside it is a
/// usage error pointing at `luria find` (rule 2).
#[test]
fn a_file_takes_no_further_arguments() {
    let record: PathBuf = pbr("no-further-arguments");
    let output: Output = luria(&[record.to_str().unwrap(), "GAATTC"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        text(&output.stderr),
        format!(
            "luria: '{}' opens the view and takes no further arguments; \
             'luria find FILE PATTERN' searches a file\n\n{}",
            record.display(),
            USAGE
        )
    );
}

/// 4: a `.json` file is a design, and opening one is milestone 9. The ASCII
/// case of the suffix does not matter (rule 2).
#[test]
fn a_design_file_is_refused() {
    for name in ["d.json", "D.JSON"] {
        let path: PathBuf = written("design-refused", name, b"{}");
        let output: Output = luria(&[path.to_str().unwrap()]);

        assert_eq!(output.status.code(), Some(2), "{}", name);
        assert!(output.stdout.is_empty(), "{}", name);
        assert_eq!(
            text(&output.stderr),
            format!(
                "luria: opening a design comes in milestone 9; use 'luria review {}'\n\n{}",
                path.display(),
                USAGE
            ),
            "{}",
            name
        );
    }
}

/// 5: a near miss names the command nearest it, and anything further away is
/// the plain unknown-command error (rule 3; the distances are the planner's,
/// as §5 says).
#[test]
fn a_near_miss_suggests_the_command() {
    let near: [(&str, &str); 4] = [
        ("fnd", "find"),
        ("revew", "review"),
        ("Version", "version"),
        ("zzfind", "find"),
    ];
    for (word, name) in near {
        let output: Output = luria(&[word]);

        assert_eq!(output.status.code(), Some(2), "{}", word);
        assert!(output.stdout.is_empty(), "{}", word);
        assert_eq!(
            text(&output.stderr),
            format!(
                "luria: unknown command '{}'; did you mean 'luria {}'?\n\n{}",
                word, name, USAGE
            ),
            "{}",
            word
        );
    }

    for word in ["zzzfind", "frobnicate", "x"] {
        let output: Output = luria(&[word]);

        assert_eq!(output.status.code(), Some(2), "{}", word);
        assert!(output.stdout.is_empty(), "{}", word);
        assert_eq!(
            text(&output.stderr),
            format!("luria: unknown command '{}'\n\n{}", word, USAGE),
            "{}",
            word
        );
    }
}

/// 6: a word `parse` knows is that subcommand even where a file of the same
/// name sits in the working directory (rule 1). The unit half of this test is
/// in `args.rs`.
#[test]
fn a_command_word_beats_a_file() {
    let directory: PathBuf = workspace("command-word");
    fs::write(directory.join("version"), "").expect("the file named version is written");

    let output: Output = luria_in(&directory, &["version"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(
        text(&output.stdout).contains("\"schema\": \"luria.version/1\""),
        "{}",
        text(&output.stdout)
    );
}

/// 7: a name that looks like a sequence file but exists nowhere is not a
/// path, and is near no command either (rules 2-3).
#[test]
fn a_missing_path_is_an_unknown_command() {
    let directory: PathBuf = workspace("missing-path");
    let output: Output = luria_in(&directory, &["absent.gb"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        text(&output.stderr),
        format!("luria: unknown command 'absent.gb'\n\n{}", USAGE)
    );
}
