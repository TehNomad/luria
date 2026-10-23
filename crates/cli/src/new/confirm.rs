//! The writing behind `luria new`'s pop-up (milestone 7m4, contract 7m4 §2.2):
//! the files one approval writes, the names that are already taken, and the
//! write and the run that follow the pop-up's `enter`.
//!
//! 7f2's rule is the rule here (plan §3 rule 4): nothing is written before the
//! pop-up's `enter`, quitting writes nothing, and the design runs from the file
//! the approval wrote, never from the form in memory. The request written is
//! byte for byte the text `interpret::run::request_text` writes (plan §3 rule
//! 2, the rule 7e's proposal screen keeps), each file is written as a new file,
//! and only with the user's `y` replaces one, so `write_text` is the writer, and
//! the run is `review::run_approved` of that file, so `new` and `review
//! --template` design the same request the same way.
//!
//! The screens are the pop-up of 7m4, which `new/form` holds and `new/view`
//! draws, so nothing here is a screen, no terminal, no drawing, and
//! `new/terminal` stays the only file in the module that does I/O (plan §5.1).

use crate::interpret::run::{request_location_problem, request_text};
use crate::new;
use crate::request::TemplateSpec;
use crate::result::request_echo;
use crate::review;
use crate::review::Approved;
use crate::write::write_text;

use std::fs;
use std::path::{Path, PathBuf};

/// The files one approval writes, and the form they came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Planned {
    /// Where each file goes: for a template file, the request beside it
    /// (M7-37, as `--write-request` requires); for a paste, the pasted
    /// FASTA and the request both in the current directory.
    pub request: PathBuf,
    pub paste: Option<(PathBuf, String)>,
    pub built: new::Built,
}

/// The files a valid form's approval writes: `save_as` is the name the form
/// keeps for the pop-up, `template_dir` the directory of a template file, and
/// `None` for a paste, whose FASTA and request both go in `cwd`. The request
/// goes beside the template, as M7-37 and `prepare`'s rule that a request's
/// template path is relative to the request (plan §3.4) require.
pub fn plan(built: new::Built, template_dir: Option<&Path>, cwd: &Path, save_as: &str) -> Planned {
    let dir: &Path = match template_dir {
        Some(found) => found,
        None => cwd,
    };
    Planned {
        request: dir.join(save_as),
        paste: built
            .paste
            .as_ref()
            .map(|(name, text)| (cwd.join(name), text.clone())),
        built,
    }
}

/// The file names `planned` would write that already exist, in the order the
/// pop-up names them: the pasted FASTA first, then the request (M7-49). Each
/// path is looked at itself by `symlink_metadata`, as `write_text` reads it, so
/// a file, a directory, and a link of any kind, a dangling one included, is a
/// name already taken (C-1). A name nothing holds, and a name the operating
/// system cannot look at, is not taken.
pub fn taken(planned: &Planned) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    if let Some((paste, _)) = &planned.paste {
        if holds(paste) {
            names.push(name_of(paste));
        }
    }
    if holds(&planned.request) {
        names.push(name_of(&planned.request));
    }
    names
}

/// Whether a path already holds something, read from the path itself, so a
/// dangling link, which points at nothing, is a name that is taken.
fn holds(path: &Path) -> bool {
    matches!(fs::symlink_metadata(path), Ok(_found))
}

/// The template file the request is written beside: the file name the request
/// states, in the directory the request goes in. For a paste that is the FASTA
/// file the approval writes first, so the location check below sees the pair as
/// `prepare` does (plan §3.4), and for a template file it is that file itself
/// (M7-37).
fn template_of(planned: &Planned) -> PathBuf {
    let name: &str = match &planned.built.request.template {
        TemplateSpec::Fasta { path, .. } | TemplateSpec::GenBank { path, .. } => path,
    };
    let dir: &Path = match planned.request.parent() {
        Some(found) => found,
        None => Path::new(""),
    };
    dir.join(name)
}

/// A path's file name, as the pop-up and the messages name a file.
fn name_of(path: &Path) -> String {
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.to_string_lossy().into_owned(),
    }
}

/// Write the paste (if any), then the request, each with `write_text`, then run
/// the design from the request file exactly as `review::approve` does after its
/// write, giving `review::Approved`.
///
/// `replace` is the user's `y` on the pop-up that named the taken files
/// (M7-49): without it each file is written as a new file only, and a name
/// taken by a file, a directory or a link is refused; with it a regular file is
/// replaced by rename, and nothing is ever written through a link, which is
/// `write_text`'s own rule (C-1).
///
/// The location is checked before anything is written, as `approve` checks it:
/// for a template file, `request_location_problem` is the rule 7f1 states, and
/// a paste and its request share the current directory, so the check passes by
/// construction, and is run anyway. Its `output_exists` refusal is ignored with
/// `replace`, since the user has been asked about that name and agreed. The
/// `Err` is one line for the pop-up; a paste already written stands on disk,
/// nothing is deleted, and the line names it.
pub fn approve_new(planned: &Planned, replace: bool) -> Result<Approved, String> {
    if let Some((code, message)) = request_location_problem(&template_of(planned), &planned.request)
    {
        // M7-49: the user's `y` covers a name that is there, so only that
        // refusal goes; any other, the location not usable, is the refusal.
        if !(replace && code == "output_exists") {
            return Err(message);
        }
    }

    // The paste first: the template the request names, written by the same
    // rule as the request.
    if let Some((paste, text)) = &planned.paste {
        if let Err(problem) = write_text(paste, text, replace) {
            return Err(format!("could not write {}: {}", name_of(paste), problem));
        }
    }

    // The request, byte for byte the text the pop-up named.
    match write_text(
        &planned.request,
        &request_text(&request_echo(&planned.built.request)),
        replace,
    ) {
        Err(problem) => Err(match &planned.paste {
            Some((paste, _)) => format!(
                "{} is written, but luria cannot write {}: {}",
                name_of(paste),
                name_of(&planned.request),
                problem
            ),
            None => format!("could not write {}: {}", name_of(&planned.request), problem),
        }),
        Ok(()) => Ok(review::run_approved(&planned.request)),
    }
}
