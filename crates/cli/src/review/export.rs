//! The export of a reviewed result (milestone 7g1, plan §3 rule 2, §5.4
//! "Writing", M7-4): the result JSON, the IDT order table and the edited
//! constructs, as the files `luria q5 design` gives, in the directory the
//! command line named with `--export DIR`.
//!
//! `export_plan` decides every name and what the table holds, before anything
//! is written, so the screen can list exactly what `y` will do. `write_export`
//! writes them, or none of them (G8, D041): a name already taken refuses the
//! whole export, `--write-edited`' own rules run unchanged for the constructs
//! because the request is run again with `run_with_output`, and the result
//! bytes are `result::render` of the document on screen, which is what
//! `luria q5 design REQUEST.json` prints (plan §3 rule 2: an export is byte
//! for byte what the CLI gives). Nothing here is a terminal file: `main` gives
//! the plan to the app, and the terminal calls `write_export` only on `y`.

use crate::result::{render, ResultDocument};
use crate::review::order::{order_table, OrderRefusal, OrderTable};
use crate::run::{run, run_with_output, Outcome};

use std::fs;
use std::io::ErrorKind;
use std::io::Write;
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
/// lossily as UTF-8); `constructs` is `<name>.fa` for each member of
/// `document.results`, in order. The table is `order_table` of the same
/// document, so the screen shows what `y` will write, refusal included, and the
/// members left out of it.
pub fn export_plan(document: &ResultDocument, request: &Path, directory: &Path) -> ExportPlan {
    let stem: String = match request.file_stem() {
        Some(stem) => stem.to_string_lossy().into_owned(),
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
            .map(|member| format!("{}.fa", member.name))
            .collect(),
    }
}

/// Write the plan's files, or none of them, and return the names written, in
/// the order written. `shown` is the document on screen.
///
/// The order is §3.2's: first the names this export writes itself must be free
/// (G8), then a fresh run must give the document on screen, then the directory
/// is created, then `run_with_output` writes the constructs under `--write-
/// edited`' own rules, and only then the result and the table, each as a new
/// file. A refusal after constructs were written names them, because the
/// caller's report must say what is on disk.
pub fn write_export(plan: &ExportPlan, shown: &ResultDocument) -> Result<Vec<String>, String> {
    // 1. A name this export writes is never taken: the result file, and the
    // order file only when there is a table to write (D041).
    free_name(&plan.directory, &plan.result_file)?;
    if plan.order.is_ok() {
        free_name(&plan.directory, &plan.order_file)?;
    }

    // 2. The request must still give the document on screen, so what is written
    // is what the user reviewed (D012).
    if run(&plan.request).document != *shown {
        return Err("the request gives a different result now; nothing was written".to_string());
    }

    // 3. The directory, parents included; a directory that exists is not a
    // failure, as `write_edited` treats it.
    fs::create_dir_all(&plan.directory).map_err(|problem| {
        format!(
            "cannot create the export directory: {}; nothing was written",
            problem
        )
    })?;

    // 4. The constructs, by `--write-edited`' rules unchanged: its refusal
    // replaces the document, and its `note` is the operating system's own line.
    let outcome: Outcome = run_with_output(&plan.request, Some(&plan.directory));
    if outcome.document != *shown {
        // `--write-edited`' refusal, which names the file alone, never its
        // directory (D012), so the line the screen shows is the document's.
        let message: &str = match &outcome.document.error {
            Some(error) => error.message.as_str(),
            None => "the request gives a different result now",
        };
        return Err(format!("{}; nothing was written", message));
    }
    if let Some(reason) = outcome.note {
        return Err(format!("could not write the edited constructs: {}", reason));
    }

    // 5. and 6. The result, then the table when there is one, each written as a
    // new file, so a name that appeared after step 1 is not replaced either.
    let mut written: Vec<String> = plan.constructs.clone();
    write_new(&plan.directory, &plan.result_file, &render(shown), &written)?;
    written.push(plan.result_file.clone());
    if let Ok(table) = &plan.order {
        write_new(&plan.directory, &plan.order_file, &table.text, &written)?;
        written.push(plan.order_file.clone());
    }
    Ok(written)
}

/// Step 1: `name` in `directory` must hold nothing. A link, a dangling one
/// included, is a name already taken, as `write_edited` reads it (C-1); any
/// other failure to look is the operating system's own.
fn free_name(directory: &Path, name: &str) -> Result<(), String> {
    match fs::symlink_metadata(directory.join(name)) {
        Ok(_) => Err(format!(
            "{} already exists; luria never replaces a file; nothing was written",
            name
        )),
        Err(problem) => match problem.kind() {
            ErrorKind::NotFound => Ok(()),
            _ => Err(format!(
                "cannot check whether {} exists: {}; nothing was written",
                name, problem
            )),
        },
    }
}

/// Write `text` to `directory/<name>` as a new file only: `create_new` refuses
/// a path something already holds, a link included, atomically with creating
/// it, as `write_edited` does (C-1). The `Err` names what the export has
/// written so far, because that is what is on disk.
fn write_new(directory: &Path, name: &str, text: &str, written: &[String]) -> Result<(), String> {
    let mut handle: std::fs::File = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(name))
    {
        Ok(opened) => opened,
        Err(problem) => return Err(unfinished(written, name, &problem)),
    };
    match handle.write_all(text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(problem) => Err(unfinished(written, name, &problem)),
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
