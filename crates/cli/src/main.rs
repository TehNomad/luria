//! The `luria` executable: parse the command line, do the one thing this
//! build can do, and report it with an exit status (plan §3.7).

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{stderr, stdin, stdout, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use luria_cli::args::{
    checked_endpoint, checked_key, parse, utf8_arguments, Command, NewBackend, UsageError, USAGE,
};
use luria_cli::exit::ExitStatus;
use luria_cli::interpret::document::render_interpretation;
use luria_cli::interpret::document::InterpretationDocument;
use luria_cli::interpret::run::{
    interpret, interpret_with_profile, request_location_problem, InterpretInput, InterpretOutcome,
    TextSource,
};
use luria_cli::new;
use luria_cli::new::form::NewForm;
use luria_cli::request::{parse_profile, ProfileDocument};
use luria_cli::result::render;
use luria_cli::review;
use luria_cli::review::app::App;
use luria_cli::review::export::{export_plan, write_export, ExportPlan};
use luria_cli::review::proposal::ProposalApp;
use luria_cli::run::{run_with_constructs, run_with_output, Outcome};
use luria_cli::version::render_version;
use luria_nl::openai::OpenAiCompatible;

/// Write `text` on stdout; false when the write or the flush failed (plan
/// §3.7: a document that cannot be written is an internal error, exit 1).
fn write_out(text: &str) -> bool {
    let mut stream = stdout().lock();
    if !(stream.write_all(text.as_bytes()).is_ok()) {
        return false;
    }
    stream.flush().is_ok()
}

/// Write `text` on stderr; a failed write changes nothing, because the run
/// still reports the status of its own failure (plan §3.7).
fn write_err(text: &str) -> bool {
    let mut stream = stderr().lock();
    if !(stream.write_all(text.as_bytes()).is_ok()) {
        return false;
    }
    stream.flush().is_ok()
}

/// Print `luria help`'s usage text on stdout (D041: the text is `args`').
fn emit_help() -> ExitStatus {
    if write_out(USAGE) {
        return ExitStatus::Success;
    }
    ExitStatus::Internal
}

/// Print `luria version`'s document on stdout.
fn emit_version() -> ExitStatus {
    if write_out(render_version().as_str()) {
        return ExitStatus::Success;
    }
    ExitStatus::Internal
}

/// `luria q5 design`: run the request through core and print the document it
/// produced (plan §3.2, §3.7). The run's status is the exit status; a document
/// that cannot be written is an internal error, because the status is the
/// report. When the run itself succeeded but writing the edited constructs did
/// not, the document is still printed and the run's `note` is the report's own
/// line on stderr (plan §3.7); a failed stderr write changes nothing.
fn emit_design(request: PathBuf, write_edited: Option<PathBuf>) -> ExitStatus {
    let outcome: Outcome = run_with_output(&request, write_edited.as_deref());
    if !write_out(render(&outcome.document).as_str()) {
        return ExitStatus::Internal;
    }

    if let Some(text) = outcome.note {
        write_err(format!("luria: {}\n", text).as_str());
    }

    outcome.status
}

/// `luria q5 interpret`: the endpoint is gated before anything is read or
/// sent, so a refused endpoint is a usage error with no file read and nothing
/// sent (plan §3, M6-18). A key set for an endpoint that is not this machine
/// is refused here as a usage error too, before anything is read or sent:
/// this build has no HTTPS, and a bearer key sent over plain HTTP to another
/// machine crosses the network unencrypted (D045, M6-18). The bearer key is
/// read here, from
/// `LURIA_OPENAI_API_KEY` and nowhere else; it is handed to the backend and
/// never written to a document, a message or stderr. One JSON document goes to
/// stdout, `interpret`'s note to stderr when it has one, and `interpret`'s
/// status is the exit status (plan §5.7); a document that cannot be written is
/// an internal error, because the status is the report.
fn emit_interpret(
    template: PathBuf,
    text: TextSource,
    design: Option<PathBuf>,
    endpoint: String,
    model: String,
    allow_remote: bool,
    write_request: Option<PathBuf>,
) -> ExitStatus {
    match checked_endpoint(&endpoint, allow_remote) {
        Err(problem) => report_usage(problem),
        Ok(endpoint) => match checked_key(&endpoint, env::var("LURIA_OPENAI_API_KEY").ok()) {
            Err(problem) => report_usage(problem),
            Ok(key) => {
                let mut backend: OpenAiCompatible = OpenAiCompatible::new(endpoint, &model, key);
                let input: InterpretInput = InterpretInput {
                    template,
                    text,
                    profile: design,
                    write_request,
                };
                let outcome: InterpretOutcome = interpret(&input, &mut backend);

                if !write_out(render_interpretation(&outcome.document).as_str()) {
                    return ExitStatus::Internal;
                }

                if let Some(text) = outcome.note {
                    write_err(format!("luria: {}\n", text).as_str());
                }

                outcome.status
            }
        },
    }
}

/// `luria review`: the screen needs a terminal on both streams, so that is
/// decided before the request is read (plan §3 rule 7), and a refusal is a
/// usage error with nothing read and nothing on stdout (§3.7, exit 2). The
/// request then runs exactly as `q5 design` runs it, and the document it
/// produced is the screen's content: the run's `note` is ignored, because
/// `review` writes nothing (plan §9). The constructs core made for the run's
/// members are kept beside it, so the member screen shows the protein core read
/// on each member's own edited construct, not a translation of the screen's own
/// (7i2, plan §5.3a, M7-21). The `--export` directory, when the command
/// line gave one, is the plan the export screen lists, built from the document
/// on screen, so what the export writes is what the user reviewed; the write
/// itself is done by the terminal, on `w` or `o`, and on `y` at the prompt
/// naming a name that already exists (M7-58, M7-49). The exit status is 0
/// whatever the result's state, and 1 when the terminal itself failed.
fn emit_review(request: PathBuf, export: Option<PathBuf>) -> ExitStatus {
    match review::terminal_problem("review", stdin().is_terminal(), stdout().is_terminal()) {
        Some(problem) => report_usage(problem),
        None => {
            let (outcome, constructs): (Outcome, Vec<(String, luria_core::sequence::Sequence)>) =
                run_with_constructs(&request);
            // The template's bases are read again, and accepted only when their
            // digest is the document's, so the context screen draws the sequence
            // the result was computed on and no other (7d, D005). The same
            // prepared request gives the template's own annotations, which the
            // frame is read on for its coordinates and strand alone (7i2).
            let (letters, frames): (Option<Vec<char>>, Vec<review::MemberFrame>) =
                match luria_cli::prepare::prepare(&request) {
                    Ok(prepared) => (
                        review::matching_template(&outcome.document, &prepared.template),
                        review::member_frames(
                            &outcome.document,
                            &prepared.template,
                            &prepared.annotations,
                            &constructs,
                        ),
                    ),
                    Err(_problem) => (None, Vec::new()),
                };
            // The plan the export screen lists, from the document on screen and
            // the `--export` directory given, if any (7g1).
            let plan: Option<ExportPlan> = export
                .as_deref()
                .map(|directory| export_plan(&outcome.document, &request, directory));
            match review::terminal::run(
                App::new(outcome.document)
                    .with_template(letters)
                    .with_frames(frames)
                    .with_export(plan),
                &mut |app: &App, replace: bool| match app.export_plan() {
                    Some(plan) => write_export(plan, app.document(), replace),
                    // `w` is offered only where a plan exists, so this cannot
                    // be reached; the line says what happened.
                    None => Err("no export directory was given".to_string()),
                },
            ) {
                Ok(()) => ExitStatus::Success,
                Err(problem) => {
                    write_err(format!("luria: the terminal failed: {}\n", problem).as_str());
                    ExitStatus::Internal
                }
            }
        }
    }
}

/// `luria review --template`: interpret a request in words, show the proposal,
/// and write and run it only on the user's approval (7f2, plan §3 rule 4). The
/// order of §2 is the order here: the endpoint and the key are gated as
/// `q5 interpret` gates them, so a refused endpoint is a usage error with
/// nothing read and nothing sent; then 7a's terminal rule, because the screen
/// must open before anything is shown; then the request location, because the
/// place the approval will write is stated on the command line and must be
/// usable before the model is called (plan §3 rule 4), which is exit 4, the
/// same code `q5 interpret` gives for an unusable location. `interpret` runs
/// with `write_request: None`, so the run writes nothing and the model is
/// spoken to once; approving runs the design from the file the approval wrote,
/// never from the proposal in memory, and calls the model no second time. The
/// run's own `note` and status are ignored: this review writes nothing, and its
/// exit status is the screen's (7a's rules). The interpret form's seven fields
/// plus `--export`' directory are the command line's own eight, so no struct is
/// added for one of them (7g1).
#[allow(clippy::too_many_arguments)]
fn emit_review_interpret(
    template: PathBuf,
    text: TextSource,
    design: Option<PathBuf>,
    endpoint: String,
    model: String,
    allow_remote: bool,
    request: PathBuf,
    export: Option<PathBuf>,
) -> ExitStatus {
    match checked_endpoint(&endpoint, allow_remote) {
        Err(problem) => report_usage(problem),
        Ok(endpoint) => match checked_key(&endpoint, env::var("LURIA_OPENAI_API_KEY").ok()) {
            Err(problem) => report_usage(problem),
            Ok(key) => {
                // 7a's rule first: without a terminal on both streams the
                // screen cannot open, so nothing is read and nothing sent.
                match review::terminal_problem(
                    "review",
                    stdin().is_terminal(),
                    stdout().is_terminal(),
                ) {
                    Some(problem) => report_usage(problem),
                    None => {
                        if let Some((_code, message)) =
                            request_location_problem(&template, &request)
                        {
                            write_err(format!("luria: {}\n", message).as_str());
                            return ExitStatus::MalformedInput;
                        }

                        // The template the approval reads, kept while `template`
                        // goes to `interpret`: 7d's context screen is drawn from
                        // the result, so the approval reads it again.
                        let approve_template: PathBuf = template.clone();
                        let mut backend: OpenAiCompatible =
                            OpenAiCompatible::new(endpoint, &model, key);
                        let input: InterpretInput = InterpretInput {
                            template,
                            text,
                            profile: design,
                            write_request: None,
                        };
                        let outcome: InterpretOutcome = interpret(&input, &mut backend);

                        match review::terminal::run_proposal(
                            ProposalApp::new(outcome.document, request.clone()),
                            &mut |app: &ProposalApp| {
                                review::approve(app.document(), &approve_template, app.request())
                            },
                        ) {
                            // Quit before approving: nothing was written and
                            // nothing ran, so 0 is the honest status (7f2 §1).
                            Ok(None) => ExitStatus::Success,
                            Ok(Some(approved)) => {
                                // The request the approval wrote is the file the
                                // export names, and the plan is built from the
                                // document the screen shows (7g1).
                                let plan: Option<ExportPlan> = export.as_deref().map(|directory| {
                                    export_plan(&approved.document, &request, directory)
                                });
                                match review::terminal::run(
                                    App::new(approved.document)
                                        .with_template(approved.template)
                                        .with_frames(approved.frames)
                                        .with_export(plan),
                                    &mut |app: &App, replace: bool| match app.export_plan() {
                                        Some(plan) => write_export(plan, app.document(), replace),
                                        None => Err("no export directory was given".to_string()),
                                    },
                                ) {
                                    Ok(()) => ExitStatus::Success,
                                    Err(problem) => {
                                        write_err(
                                            format!("luria: the terminal failed: {}\n", problem)
                                                .as_str(),
                                        );
                                        ExitStatus::Internal
                                    }
                                }
                            }
                            Err(problem) => {
                                write_err(
                                    format!("luria: the terminal failed: {}\n", problem).as_str(),
                                );
                                ExitStatus::Internal
                            }
                        }
                    }
                }
            }
        },
    }
}

/// The `In words` box's model, as the loop calls it: the template path as typed
/// in `File`, the words as typed, and the design the form shows (M7-41), which
/// the model answers with its document (contract §2.4), so `emit_new` builds it
/// once and `run_new` calls it once per `[ Review ]`.
type WordsModel = dyn FnMut(&str, &str, ProfileDocument) -> InterpretationDocument;

/// `luria new`: the form, the confirmation, and the design an approval runs
/// (milestone 7k5, contract 7k5 §2.4), and the `In words` box's model, which
/// sends the words through `interpret` and writes and runs the proposal on the
/// user's approval, as `luria review --template` does (7k7, M7-40). With a
/// backend, the endpoint and the key are gated exactly as `emit_review_interpret`
/// gates them, before the terminal check and before anything is read, so a
/// refused endpoint is a usage error with nothing read and nothing sent (plan §3,
/// M6-18, D045); the backend is built once here, and the loop calls it once per
/// `[ Review ]`. Without a backend the `In words` box stays disabled whatever the
/// command line says (M7-36) and the closure handed to the loop is never called.
/// 7a's rule is checked next, because without a terminal on both streams the
/// screen cannot open, and a refusal is 7a's usage error with nothing read (plan
/// §3 rule 7, §3.7, exit 2). `--design` is read with `parse_profile`, and a
/// refusal is exit 4 with its message, before the screen opens, as
/// `emit_review_interpret` gives exit 4 for the request location. Nothing is
/// written until `a` then `y` (plan §3 rule 4). The request the approval wrote is
/// the file the export names, and the plan is built from the document the screen
/// shows, so the result screens are 7a's, opened exactly as
/// `emit_review_interpret` opens them (7g1). The exit status is 0 whatever the
/// result's state, and 1 when the terminal itself failed.
fn emit_new(
    design: Option<PathBuf>,
    export: Option<PathBuf>,
    backend: Option<NewBackend>,
) -> ExitStatus {
    // The form is told whether the words box is enabled, so that is read before
    // `backend` goes into the closure the loop calls.
    let words_enabled: bool = backend.is_some();
    let mut interpret_new: Box<WordsModel> = match backend {
        // The box is disabled, so the loop never calls this.
        None => Box::new(
            |_template: &str, _words: &str, _profile: ProfileDocument| -> InterpretationDocument {
                panic!("no backend")
            },
        ),
        Some(given) => match checked_endpoint(&given.endpoint, given.allow_remote) {
            Err(problem) => return report_usage(problem),
            Ok(endpoint) => match checked_key(&endpoint, env::var("LURIA_OPENAI_API_KEY").ok()) {
                Err(problem) => return report_usage(problem),
                Ok(key) => {
                    let mut openai: OpenAiCompatible =
                        OpenAiCompatible::new(endpoint, &given.model, key);
                    Box::new(
                        move |template: &str, words: &str, profile: ProfileDocument| {
                            // The form's `--design` profile opened the form with this
                            // overlap and codon (7k3), so the file is not read again:
                            // the input has no profile, and the given one is the
                            // design the words are read with (M7-41).
                            let input: InterpretInput = InterpretInput {
                                template: PathBuf::from(template),
                                text: TextSource::Inline(words.to_string()),
                                profile: None,
                                write_request: None,
                            };
                            // The run writes nothing: the approval writes the request
                            // the form's `Save as` names (plan §3 rule 4).
                            interpret_with_profile(&input, profile, &mut openai).document
                        },
                    )
                }
            },
        },
    };

    match review::terminal_problem("new", stdin().is_terminal(), stdout().is_terminal()) {
        Some(problem) => report_usage(problem),
        None => {
            let profile: Option<ProfileDocument> = match design.as_deref() {
                None => None,
                Some(path) => match fs::read_to_string(path) {
                    Err(problem) => {
                        write_err(
                            format!("luria: cannot read the design profile: {}\n", problem)
                                .as_str(),
                        );
                        return ExitStatus::MalformedInput;
                    }
                    Ok(text) => match parse_profile(&text) {
                        Err(error) => {
                            write_err(format!("luria: {}\n", error.message).as_str());
                            return ExitStatus::MalformedInput;
                        }
                        Ok(found) => Some(found),
                    },
                },
            };
            // The directory the template path and the written files are relative
            // to (contract §2.4).
            let cwd: PathBuf = match env::current_dir() {
                Ok(there) => there,
                Err(problem) => {
                    write_err(format!("luria: {}\n", problem).as_str());
                    return ExitStatus::Internal;
                }
            };
            let mut written: Option<PathBuf> = None;
            match new::terminal::run_new(
                NewForm::new(profile.as_ref(), words_enabled),
                &cwd,
                &mut written,
                &mut interpret_new,
            ) {
                // Quit before `y`: nothing was written and nothing ran, so 0 is
                // the honest status (7f2 §1).
                Ok(None) => ExitStatus::Success,
                Ok(Some(approved)) => {
                    let request: PathBuf = written.expect("an approval wrote the request it ran");
                    let plan: Option<ExportPlan> = export
                        .as_deref()
                        .map(|directory| export_plan(&approved.document, &request, directory));
                    match review::terminal::run(
                        App::new(approved.document)
                            .with_template(approved.template)
                            .with_frames(approved.frames)
                            .with_export(plan),
                        &mut |app: &App, replace: bool| match app.export_plan() {
                            Some(plan) => write_export(plan, app.document(), replace),
                            None => Err("no export directory was given".to_string()),
                        },
                    ) {
                        Ok(()) => ExitStatus::Success,
                        Err(problem) => {
                            write_err(
                                format!("luria: the terminal failed: {}\n", problem).as_str(),
                            );
                            ExitStatus::Internal
                        }
                    }
                }
                Err(problem) => {
                    write_err(format!("luria: the terminal failed: {}\n", problem).as_str());
                    ExitStatus::Internal
                }
            }
        }
    }
}

/// Report a usage error on stderr, with the usage text repeated (plan §3.7,
/// exit code 2: text on stderr, nothing on stdout).
fn report_usage(problem: UsageError) -> ExitStatus {
    let message: String = format!("luria: {}\n\n{}", problem.message, USAGE);
    write_err(message.as_str());
    ExitStatus::Usage
}

/// Run one `luria` command line.
fn main() -> ExitCode {
    let words: Vec<OsString> = env::args_os().skip(1).collect();
    let status: ExitStatus = match utf8_arguments(words) {
        Err(problem) => report_usage(problem),
        Ok(arguments) => match parse(&arguments) {
            Ok(Command::Help) => emit_help(),
            Ok(Command::Version) => emit_version(),
            Ok(Command::Q5Design {
                request,
                write_edited,
            }) => emit_design(request, write_edited),
            Ok(Command::Q5Interpret {
                template,
                text,
                design,
                endpoint,
                model,
                allow_remote,
                write_request,
            }) => emit_interpret(
                template,
                text,
                design,
                endpoint,
                model,
                allow_remote,
                write_request,
            ),
            Ok(Command::Review { request, export }) => emit_review(request, export),
            Ok(Command::ReviewInterpret {
                template,
                text,
                design,
                endpoint,
                model,
                allow_remote,
                request,
                export,
            }) => emit_review_interpret(
                template,
                text,
                design,
                endpoint,
                model,
                allow_remote,
                request,
                export,
            ),
            Ok(Command::New {
                design,
                export,
                backend,
            }) => emit_new(design, export, backend),
            Err(problem) => report_usage(problem),
        },
    };

    ExitCode::from(status.code())
}
