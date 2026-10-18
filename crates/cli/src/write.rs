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
//! - every path is checked for existence before any file is written, so one
//!   existing name refuses the whole set: nothing is written, the free names
//!   are not written either, and the directory is left as it was (G8);
//! - a refusal names the file on its own, never the directory it sits in
//!   (D012), so the result document stays free of absolute paths;
//! - a result name that holds a path separator is refused before anything is
//!   created (§4.1), so a request cannot choose where the constructs are
//!   written: without a separator `<name>.fa` is one path component, and
//!   because it ends in `.fa` it is never `.` or `..`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use luria_core::fasta::render_fasta;
use luria_core::identity::Identifier;
use luria_core::sequence::Sequence;

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
        match path.try_exists() {
            Ok(true) => return Err(WriteFailure::Exists(file_name.clone())),
            Err(problem) => return Err(WriteFailure::Io(problem.to_string())),
            Ok(false) => (),
        }
    }

    // None of these paths existed before, so writing them cannot replace
    // anything a caller had.
    let mut written: Vec<PathBuf> = [].to_vec();
    for (artifact, (path, _)) in artifacts.iter().zip(targets.iter()) {
        let (name, sequence) = artifact;
        match fs::write(path, render_fasta(name, sequence)) {
            Err(problem) => return Err(WriteFailure::Io(problem.to_string())),
            Ok(_) => written.push(path.clone()),
        }
    }
    Ok(written)
}
