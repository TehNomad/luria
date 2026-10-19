//! Writing the edited constructs: one FASTA file per member, into the
//! directory the caller named (milestone 5g2 §4).
//!
//! Nothing here re-derives a sequence or a design (D013): the text written is
//! `luria_core::fasta::render_fasta` of the sequence core produced. What this
//! module decides is only where the files go and whether a write may happen at
//! all:
//!
//! - the directory is created only when there is something to write (G8), so
//!   a run that edited nothing leaves no directory behind;
//! - every path is checked before any file is written, so one existing name
//!   refuses the whole set: nothing is written, the free names are not
//!   written either, and the directory is left as it was (G8);
//! - a path that holds anything at all is never written through or replaced:
//!   the check looks at the path itself, so a symlink - a dangling one
//!   included - is a name already taken, and a write creates a new file only,
//!   so a file that appears after the check is still not replaced (found
//!   reviewing 5g2, C-1);
//! - a refusal names the file on its own, never the directory it sits in
//!   (D012), so the result document stays free of absolute paths;
//! - a result name that holds a path separator (`/` or `\`), or one of `:`
//!   `*` `?` `<` `>` `|`, is refused before anything is created (§4.1 and the
//!   owner answer of 2026-10-18), so a request cannot choose where the
//!   constructs are written and cannot name a file that is not portable:
//!   without a separator `<name>.fa` is one path component, and because it
//!   ends in `.fa` it is never `.` or `..`;
//! - two result names that differ only in case are refused at the later one
//!   before anything is created (owner answer, 2026-10-18): on a
//!   case-insensitive file system their files would replace each other while
//!   the document reports both members.

use std::fs;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use luria_core::fasta::render_fasta;
use luria_core::identity::Identifier;
use luria_core::sequence::Sequence;

/// Why no edited construct was written.
#[derive(Clone, Debug, PartialEq)]
pub enum WriteFailure {
    /// A path that already holds something is never written through or
    /// replaced: a file, a directory or a symlink, a dangling one included,
    /// all read as a name already taken. Nothing is written, not even the
    /// files whose names are free (G8). The `String` is the file's own name,
    /// never its directory (D012).
    Exists(String),
    /// The directory could not be created, or a file could not be written.
    /// The `String` is the operating system's message.
    Io(String),
    /// A result name cannot be a file name: it holds a path separator (`/` or
    /// `\`), so `<name>.fa` would not be a file inside the directory, or one
    /// of `:` `*` `?` `<` `>` `|`, which a portable file name may not hold.
    /// Nothing is created. The `String` is the result name. (Planner
    /// amendment, 2026-10-17, §4.1; the characters come from the owner answer
    /// of 2026-10-18, in review.)
    NotAFileName(String),
    /// Two result names differ only in case, so their files would replace each
    /// other on a case-insensitive file system while the document reports both
    /// members. Nothing is created. The `String` is the later name, the one
    /// that clashes with an earlier one. (Owner answer, 2026-10-18.)
    CaseClash(String),
}

/// Write each member's edited construct as FASTA into `directory`, one file
/// per artifact at `directory/<name>.fa`, and return the paths written in the
/// order given.
///
/// Nothing is created when `artifacts` is empty: a run that edited nothing
/// leaves no directory behind. Otherwise the directory is created first
/// (G8), then **every** path is checked, and only then is anything written,
/// so a refusal leaves the directory as it was.
pub fn write_edited(
    directory: &Path,
    artifacts: &[(Identifier, Sequence)],
) -> Result<Vec<PathBuf>, WriteFailure> {
    if artifacts.is_empty() {
        // Nothing was edited, so nothing is written and no directory is
        // created (G8).
        return Ok([].to_vec());
    }

    // §4.1: a result name becomes a file name, so a name holding a path
    // separator would write outside `directory`. Refused first, before anything
    // is created, so a request cannot choose where the constructs go (G8).
    for artifact in artifacts.iter() {
        let (name, _) = artifact;
        if name.value().contains("/") || name.value().contains("\\") {
            return Err(WriteFailure::NotAFileName(name.value().to_string()));
        }
    }

    // The owner answer of 2026-10-18, found reviewing 5g2: `:` `*` `?` `<`
    // `>` `|` do not name a portable file, and two names that differ only in
    // case name one file on a case-insensitive file system. Both are refused
    // here, still before `directory` is created, so nothing is created (G8),
    // and the clash is reported at the later name, walking the artifacts in
    // the order given.
    let mut folded: Vec<String> = [].to_vec();
    for artifact in artifacts.iter() {
        let (name, _) = artifact;
        let value: String = name.value().to_string();
        if has_unportable_character(value.clone()) {
            return Err(WriteFailure::NotAFileName(value.clone()));
        }
        let lower: String = ascii_lowered(value.clone());
        if folded.contains(&lower) {
            return Err(WriteFailure::CaseClash(value.clone()));
        }
        folded.push(lower.clone());
    }

    // G8: a directory that is missing is created, parents included; one that
    // already exists is not a failure.
    fs::create_dir_all(directory).map_err(|problem| WriteFailure::Io(problem.to_string()))?;

    // Each file's own name, and the path it would be written at.
    let mut targets: Vec<(PathBuf, String)> = [].to_vec();
    for artifact in artifacts.iter() {
        let (name, _) = artifact;
        let file_name: String = format!("{}.fa", name.value());
        targets.push((directory.join(file_name.as_str()), file_name));
    }

    // Every path is checked before any file is written, so an existing name
    // refuses the whole set and leaves the directory exactly as it was. The
    // name reported is the file's own, never its directory (D012).
    for (path, file_name) in targets.iter() {
        match fs::symlink_metadata(path) {
            // A file, a directory or a link of any kind, a dangling one
            // included: the name is taken. `try_exists` looked at what a link
            // points at instead, so a dangling one read as free and the write
            // below went through it to wherever it pointed (C-1).
            Ok(_) => return Err(WriteFailure::Exists(file_name.clone())),
            // Only a name nothing holds is free; any other failure is the
            // operating system's own.
            Err(problem) => match problem.kind() {
                ErrorKind::NotFound => (),
                _ => return Err(WriteFailure::Io(problem.to_string())),
            },
        }
    }

    // Each file is opened as a new one: `create_new` refuses a path something
    // already holds, a symlink included, atomically with creating it, so a
    // file or a link that appeared after the check above is still never
    // replaced and nothing is ever written through a link (C-1).
    let mut written: Vec<PathBuf> = [].to_vec();
    for (artifact, (path, file_name)) in artifacts.iter().zip(targets.iter()) {
        let (name, sequence) = artifact;
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Err(problem) => match problem.kind() {
                ErrorKind::AlreadyExists => return Err(WriteFailure::Exists(file_name.clone())),
                _ => return Err(WriteFailure::Io(problem.to_string())),
            },
            Ok(mut handle) => match handle.write_all(render_fasta(name, sequence).as_bytes()) {
                Err(problem) => return Err(WriteFailure::Io(problem.to_string())),
                Ok(_) => written.push(path.clone()),
            },
        }
    }
    Ok(written)
}

/// The characters the owner answer of 2026-10-18 refused in a result name:
/// a portable file name may not hold them. They are not separators, so they
/// cannot move the file out of the directory and are refused for a different
/// reason than `/` and `\` (§4.1): the same name cannot be created on every
/// file system.
const UNPORTABLE: &str = ":*?<>|";

/// Whether a result name holds one of those characters.
fn has_unportable_character(value: String) -> bool {
    value
        .chars()
        .any(|character| UNPORTABLE.contains(character))
}

/// `value` as a case-insensitive file system sees it: only ASCII letters
/// change, and only into their lower case, which is the one fold such a file
/// system applies everywhere. Other characters stay as they are, since no file
/// system folds them the same way everywhere.
fn ascii_lowered(value: String) -> String {
    value.to_ascii_lowercase()
}
