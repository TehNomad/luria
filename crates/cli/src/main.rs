//! The `luria` executable: parse the command line, do the one thing this
//! build can do, and report it with an exit status (plan §3.7).

use std::env;
use std::ffi::OsString;
use std::io::{stderr, stdout, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use luria_cli::args::{parse, utf8_arguments, Command, UsageError, USAGE};
use luria_cli::exit::ExitStatus;
use luria_cli::result::render;
use luria_cli::run::{run_with_output, Outcome};
use luria_cli::version::render_version;

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
            Err(problem) => report_usage(problem),
        },
    };

    ExitCode::from(status.code())
}
