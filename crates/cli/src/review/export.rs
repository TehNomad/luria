//! The export of a reviewed result (milestone 7g1, plan §3 rule 2, §5.4
//! "Writing", M7-4, M7-57, M7-49): the result JSON, the IDT order table and the
//! edited constructs, as the files `luria q5 design` gives, in the directory the
//! command line named with `--export DIR`.
//!
//! `export_plan` decides every name and what the table holds, before anything is
//! written, so the screen can list exactly what a write will do. `taken` names the
//! plan's names that are already on disk, so the screen can ask before replacing
//! (M7-49). `write_export` writes them, or none of them (G8, D041): a name taken
//! refuses the whole export unless the user answered `y` to that prompt, the
//! request is run once more so what is written is the document on screen
//! (D012), and the constructs, the result and the table are each written by
//! `write::write_text`, so the FASTA bytes are `--write-edited`' own, the result
//! bytes `result::render` of the document on screen, which is what
//! `luria q5 design REQUEST.json` prints (plan §3 rule 2: an export is byte for
//! byte what the CLI gives). Nothing here is a terminal file: `main` gives the
//! plan to the app, and the terminal calls `write_export` only on a write key.

use crate::result::{render, ResultDocument};
use crate::review::order::{order_table, OrderRefusal, OrderTable};
use crate::run::{run_with_constructs, Outcome};
use crate::write::{file_name_problem, write_text, WriteFailure};

use luria_core::fasta::render_fasta;
use luria_core::identity::Identifier;
use luria_core::sequence::Sequence;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// What an export will write, decided before anything is written.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportPlan {
    pub directory: PathBuf,
    pub request: PathBuf,
    pub result_file: String,
    pub order_file: String,
    pub order: Result<OrderTable, OrderRefusal>,
    pub constructs: Vec<String>,
}

/// Pure. `result_file` is `<stem>.result.json` and `order_file`
/// `<stem>.order.tsv`, `<stem>` being `request`'s file stem (`Path::file_stem`,
/// lossily as UTF-8). `constructs` is `<template id>_<member>.fa` for each member
/// of `document.results`, in order, the template id being
/// `provenance.template.id` (M7-57), as the order table names its primers, so two
/// runs on two templates do not write the same name. With no template in the
/// document, the name is `<member>.fa` as before. The table is `order_table` of
/// the same document, so the screen shows what a write will write, refusal
/// included, and the members left out of it.
pub fn export_plan(document: &ResultDocument, request: &Path, directory: &Path) -> ExportPlan {
    let stem: String = match request.file_stem() {
        Some(stem) => stem.to_string_lossy().into_owned(),
        None => String::new(),
    };
    // M7-57: the template names the construct, and the name is the one the plan
    // writes, the screen lists and `taken` looks for, so it is decided once here.
    let prefix: String = match &document.provenance.template {
        Some(template) => template.id.clone() + "_",
        None => String::new(),
    };
    ExportPlan {
        directory: directory.to_path_buf(),
        request: request.to_path_buf(),
        result_file: format!("{}.result.json", stem),
        order_file: format!("{}.order.tsv", stem),
        order: order_table(document),
        constructs: document
            .results
            .iter()
            .map(|member| format!("{}{}.fa", prefix, member.name))
            .collect(),
    }
}

/// The plan's names that exist in `plan.directory`, in the plan's order: the
/// constructs, then the result, then the order table when there is one (M7-49).
/// Each is looked at by `symlink_metadata`, so a symlink - a dangling one
/// included - and a directory are names already taken, as `write_edited` reads
/// them (C-1). A name nothing holds is not in the list, and a failure to look at
/// a name is not a name that exists: the write of that name then gives the
/// operating system's own line.
pub fn taken(plan: &ExportPlan) -> Vec<String> {
    // The plan's names in the order the export writes them.
    let mut considered: Vec<&str> = plan.constructs.iter().map(|name| name.as_str()).collect();
    considered.push(plan.result_file.as_str());
    if plan.order.is_ok() {
        considered.push(plan.order_file.as_str());
    }

    // A name that holds something is taken, looked at by `symlink_metadata`, so
    // a link is a taken name, as `write_edited` reads it (C-1). A failure to
    // look is not a name that exists: the write of that name then gives the
    // operating system's own line.
    considered
        .into_iter()
        .filter(|name| fs::symlink_metadata(plan.directory.join(name)).is_ok())
        .map(|name| name.to_string())
        .collect()
}

/// Write the plan's files, or none of them, and return the names written, in the
/// order written. `shown` is the document on screen. `replace` is the answer of
/// the prompt over the export screen (M7-49): `false` is the rule of D041, a name
/// already taken refuses the whole export, and `true` writes over a regular file,
/// never through a link, because `write_text` looks at the path itself (C-1).
///
/// The order is §2.2's: first the construct names are checked, as
/// `write_edited` checks them, then a name taken refuses when `replace` is
/// `false`, then one run of the request must give the document on screen, its
/// constructs being the sequences to write, then the directory is created, and
/// each file is written by `write_text` in the plan's order: the constructs, the
/// result, then the table when there is one. A refusal after some files were
/// written names them, because the caller's report must say what is on disk.
pub fn write_export(
    plan: &ExportPlan,
    shown: &ResultDocument,
    replace: bool,
) -> Result<Vec<String>, String> {
    // 1. A construct name that cannot be a portable file name, or two that
    // differ only in case, refuse the export before anything is written, as
    // `write_edited` refuses them the same way (G8, §4.1).
    if let Some(failure) = file_name_problem(plan.constructs.as_slice()) {
        return Err(name_problem(&failure));
    }

    // 2. A name taken refuses the whole export, the free names included, and
    // names the file on its own, never its directory (D012, D041). With
    // `replace` the prompt was answered `y`, so the names are replaced.
    if !replace {
        if let Some(name) = taken(plan).first() {
            return Err(format!(
                "{} already exists; luria never replaces a file; nothing was written",
                name
            ));
        }
    }

    // 3. One run: the request must still give the document on screen, so what is
    // written is what the user reviewed (D012), and the constructs it made are
    // the sequences to write.
    let (outcome, constructs): (Outcome, Vec<(String, Sequence)>) =
        run_with_constructs(&plan.request);
    if outcome.document != *shown {
        return Err("the request gives a different result now; nothing was written".to_string());
    }

    // 4. The directory, parents included; a directory that exists is not a
    // failure, as `write_edited` treats it.
    fs::create_dir_all(&plan.directory).map_err(|problem| {
        format!(
            "cannot create the export directory: {}; nothing was written",
            problem
        )
    })?;

    // 5. Each construct, as `render_fasta` of the member's own name and the
    // sequence the run made, which is the file `--write-edited` writes for that
    // member, byte for byte.
    let mut written: Vec<String> = Vec::new();
    for (file_name, (name, sequence)) in plan.constructs.iter().zip(constructs.iter()) {
        // The header stays the member's name, as `--write-edited` writes it, and
        // the name written is the plan's, which names the template (M7-57).
        let identifier: Identifier = match Identifier::create(name.clone()) {
            Ok(given) => given,
            // A member name is a valid identifier in a run's own document, so
            // this is not reachable; the line is the name check's own, naming
            // the construct alone (D012).
            Err(_problem) => {
                return Err(name_problem(&WriteFailure::NotAFileName(file_name.clone())))
            }
        };
        let text: String = render_fasta(&identifier, sequence);
        write_text(&plan.directory.join(file_name.as_str()), &text, replace)
            .map_err(|problem| unfinished(&written, file_name, &problem))?;
        written.push(file_name.clone());
    }

    // 6. and 7. The result, then the table when there is one.
    write_text(
        &plan.directory.join(plan.result_file.as_str()),
        &render(shown),
        replace,
    )
    .map_err(|problem| unfinished(&written, &plan.result_file, &problem))?;
    written.push(plan.result_file.clone());
    if let Ok(table) = &plan.order {
        write_text(
            &plan.directory.join(plan.order_file.as_str()),
            &table.text,
            replace,
        )
        .map_err(|problem| unfinished(&written, &plan.order_file, &problem))?;
        written.push(plan.order_file.clone());
    }
    Ok(written)
}

/// The line a construct name check gives: the name alone, never a directory
/// (D012), and `nothing was written`, as every export line ends. `file_name_problem`
/// gives the first two; `Exists` and `Io` are not produced by these checks.
fn name_problem(failure: &WriteFailure) -> String {
    match failure {
        WriteFailure::NotAFileName(name) => format!(
            "the construct name '{}' cannot be a file name: it contains a path separator or one of : * ? < > |; nothing was written",
            name
        ),
        WriteFailure::CaseClash(name) => format!(
            "the construct name '{}' differs from an earlier one only in case, so their files would collide; nothing was written",
            name
        ),
        WriteFailure::Exists(name) => {
            format!("{} already exists; nothing was written", name)
        }
        WriteFailure::Io(problem) => format!("{}; nothing was written", problem),
    }
}

/// The line a later file's failure gives: `written` is what is on disk.
fn unfinished(written: &[String], name: &str, problem: &std::io::Error) -> String {
    let done: String = if written.is_empty() {
        "nothing".to_string()
    } else {
        written.join(", ")
    };
    format!("{} written; could not write {}: {}", done, name, problem)
}
