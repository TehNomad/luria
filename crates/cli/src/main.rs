//! The `luria` executable: parse the command line, do the one thing this
//! build can do, and report it with an exit status (plan §3.7).

use std::env;
use std::io::{stderr, stdout, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use luria_cli::args::{parse, Command, UsageError, USAGE};
use luria_cli::exit::ExitStatus;
use luria_cli::result::render;
use luria_cli::run::{run, Outcome};
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
/// that cannot be written is an internal error, and the status is the report,
/// so nothing more goes to stderr on that path.
fn emit_design(request: PathBuf) -> ExitStatus {
    let outcome: Outcome = run(&request);
    if write_out(render(&outcome.document).as_str()) {
        return outcome.status;
    }
    ExitStatus::Internal
}

/// `--write-edited` arrives in 5g2: until then the flag is refused before
/// anything runs, and the status is the report.
const WRITE_EDITED_UNAVAILABLE: &str =
    "luria: '--write-edited' is not available in this build yet\n";

fn write_edited_unavailable() -> ExitStatus {
    write_err(WRITE_EDITED_UNAVAILABLE);
    ExitStatus::Internal
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
    let arguments: Vec<String> = env::args().skip(1).collect();
    let status: ExitStatus = match parse(&arguments) {
        Ok(Command::Help) => emit_help(),
        Ok(Command::Version) => emit_version(),
        Ok(Command::Q5Design {
            request,
            write_edited,
        }) => match write_edited {
            Some(_) => write_edited_unavailable(),
            None => emit_design(request),
        },
        Err(problem) => report_usage(problem),
    };

    ExitCode::from(status.code())
}
