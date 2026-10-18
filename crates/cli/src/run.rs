//! run: one prepared request through core and back as the one document
//! `luria q5 design` prints (milestone 5g1 §1).
//!
//! 5e2 built everything core needs from a request file, and 5f1–5f4 built
//! everything the document needs from core's answers; this module only joins
//! them. It calls `luria_core::q5::execute` or `q5::execute_set` and the
//! matching design entry point, then hands what they returned to 5f's view
//! functions. Nothing here re-verifies an edit, re-derives a design, rounds a
//! float or computes a value a core accessor or a view function gives (D013):
//! every value in the document is one core gave.
//!
//! # The rules this module applies (§2)
//!
//! - one `Outcome` for every path: the document and the status are both always
//!   present, because a failure is a document too (plan §3.7);
//! - a request that could not be prepared fails with its own category: the
//!   CLI's refusals are `MalformedInput`, a core refusal keeps its class;
//! - a prepared request whose core call failed is `failed_after`, at no path in
//!   the request: the request was understood, so the error is not located in it;
//! - a run that needs clarification designs nothing and reports the gate's
//!   clarifications and candidates as the whole answer (D005, D020);
//! - G7 (D041): an edit that is applied and verified but cannot be designed is
//!   one member's `designFailure`, state `verified` and exit 7 - the same shape
//!   as a one-member set - and not a failed run, even though `NoPrimerDesignFits`
//!   is an `Unsupported` error that on its own would give exit 6;
//! - the design's absence is what makes a run partial, never its own re-check:
//!   §3.2 is core's invariant, that `Ok` means a clarification or a verified
//!   edit, and `a_verified_run_is_verified_by_core` pins it rather than trusting
//!   it; a verified run whose design is missing is `Internal`, because a
//!   document must never claim a design it does not carry;
//! - `--write-edited` writes the constructs core produced, once the document is
//!   built (plan §3.2, G8): every edited construct is one FASTA file, an existing
//!   file name is `output_exists` with nothing written and exit 4, and a result
//!   name that holds a path separator is `output_name_unusable` the same way -
//!   nothing is created, so a request cannot write outside the directory it named
//!   (§4.1, D012); an I/O failure leaves the document as it is - the run still
//!   succeeded - with exit 1 and the operating system's message as `note`, which
//!   `main.rs` puts on stderr (plan §3.7).

use crate::exit::ExitStatus;
use crate::prepare::{prepare, Operation, PrepareFailure, Prepared, RequestFailure};
use crate::request::RequestErrorCode;
use crate::result::{
    category_name, clarification_document, designed_member, domain_error_view, failed_after,
    failed_document, member_view, operation_warning_view, undesigned_member, verified_document,
    DetailValue, Details, ErrorView, MemberView, OperationWarning, ResultDocument,
};
use crate::write::{write_edited, WriteFailure};

use luria_core::cds::Cds;
use luria_core::error::{DomainError, ErrorCategory};
use luria_core::identity::Identifier;
use luria_core::mutation::MutationResult;
use luria_core::q5::{
    execute, execute_set, Q5EditWarning, Q5SdmRequest, Q5VariantSetExecution, Q5VariantSetRequest,
    Q5WorkflowExecution,
};
use luria_core::q5_design::{design_run, design_set, Q5DesignedSet, Q5MemberDesign};
use luria_core::sequence::Sequence;
use std::path::Path;

/// What one `luria q5 design` run produced: the document to print and the
/// status to exit with. Both are always present: a failure is a document too
/// (plan §3.7).
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub document: ResultDocument,
    pub status: ExitStatus,
    /// A line for stderr, when something outside the document failed. The
    /// document is still the run's answer.
    pub note: Option<String>,
}

/// One edited construct to write: the member's own name, which is the
/// caller's (D007), and the sequence core produced for it (plan §3.2).
type Artifacts = Vec<(Identifier, Sequence)>;

/// The category a preparation failure reports: the CLI's own refusals are a
/// malformed request, and a core refusal keeps its own class (§3.1).
fn failure_category(failure: &RequestFailure) -> ErrorCategory {
    match failure {
        RequestFailure::Request(_) => ErrorCategory::MalformedInput,
        RequestFailure::Domain { error, .. } => error.category(),
    }
}

/// A request that could not be prepared: the whole failed document, which
/// carries what was known when it failed, and the status its category gives
/// (§3.1, step 1).
fn unprepared(problem: &PrepareFailure) -> (Outcome, Artifacts) {
    let outcome: Outcome = Outcome {
        document: failed_document(problem),
        status: ExitStatus::for_failure(failure_category(&problem.failure)),
        note: None,
    };
    (outcome, [].to_vec())
}

/// A prepared request whose core call failed: the failed document, which keeps
/// the template's provenance and echoes the request it read, and the status the
/// error's category gives. The request was understood, so the error sits at no
/// path in it (§3.1, step 2), and no construct was edited, so there is nothing
/// to write.
fn refusal(prepared: &Prepared, error: &DomainError) -> (Outcome, Artifacts) {
    let outcome: Outcome = Outcome {
        document: failed_after(prepared, domain_error_view(error, None)),
        status: ExitStatus::for_failure(error.category()),
        note: None,
    };
    (outcome, [].to_vec())
}

/// One edit: run it, design it, and report the one member it produced (§3.1).
/// Its construct is reported too, for `--write-edited`: the member's own name
/// and the sequence core produced, whenever the edit was applied (plan §3.2,
/// G7).
fn single_edit(
    prepared: &Prepared,
    request: Q5SdmRequest,
    name: Identifier,
) -> (Outcome, Artifacts) {
    let execution: Q5WorkflowExecution =
        match execute(request, prepared.annotations.clone(), name.clone()) {
            Ok(done) => done,
            Err(error) => return refusal(prepared, &error),
        };

    if execution.needs_clarification() {
        // D005, D020: the scope did not resolve, so nothing was edited, and
        // nothing is designed: the clarification is the answer.
        return (
            Outcome {
                document: clarification_document(prepared, &execution.scope()),
                status: ExitStatus::NeedsClarification,
                note: None,
            },
            [].to_vec(),
        );
    }

    // G7 (D041), plan §3.2: the edit was applied, so its construct is one to
    // write even when no primer pair fits it and the run is partial.
    let artifacts: Artifacts = match execution.edited() {
        Some(edited) => vec![(name.clone(), edited.clone())],
        None => [].to_vec(),
    };

    // §3.2: core returns `Ok` only for a clarification or for a run whose edit
    // it verified, so here the mutation is present and the edit is verified;
    // `a_verified_run_is_verified_by_core` pins that rather than trusting it.
    let target: Option<Cds> = execution.scope().target();
    let warnings: Vec<Q5EditWarning> = execution.warnings();
    let mutation: MutationResult = execution.mutation().unwrap();
    let member: MemberView = member_view(&name, &mutation, &warnings);

    let outcome: Outcome = match design_run(execution.clone(), prepared.design.clone()) {
        Ok(designed) => match designed.design() {
            Some(design) => Outcome {
                document: verified_document(
                    prepared,
                    target.as_ref(),
                    vec![designed_member(member, &design)],
                    vec![],
                ),
                status: ExitStatus::Success,
                note: None,
            },
            // §3.2: a verified run always has a design, so this is the
            // broken invariant. The member is reported as built and the run
            // is internal, because a document must never claim a design it
            // does not carry.
            None => Outcome {
                document: verified_document(prepared, target.as_ref(), vec![member], vec![]),
                status: ExitStatus::Internal,
                note: None,
            },
        },
        // G7 (D041): the edit was applied and verified, so this is one member
        // that could not be designed - state `verified`, the member's own
        // `designFailure`, exit 7 - and not a failed run.
        Err(error) => Outcome {
            document: verified_document(
                prepared,
                target.as_ref(),
                vec![undesigned_member(member, &error)],
                vec![],
            ),
            status: ExitStatus::PartialDesign,
            note: None,
        },
    };

    (outcome, artifacts)
}

/// A variant set, or the expansion of one degenerate edit into one: run every
/// member, design each one, and report one member per variant in request
/// order. `warnings` are the operation's own, which an expansion raises for the
/// members it skipped (§3.1). Its constructs are reported in the same order,
/// one per member, for `--write-edited` (plan §3.2).
fn variant_set(
    prepared: &Prepared,
    set: Q5VariantSetRequest,
    warnings: Vec<OperationWarning>,
) -> (Outcome, Artifacts) {
    let execution: Q5VariantSetExecution = match execute_set(set, prepared.annotations.clone()) {
        Ok(done) => done,
        Err(error) => return refusal(prepared, &error),
    };

    if execution.needs_clarification() {
        // D005, D020: no member of the set is edited and none is designed.
        return (
            Outcome {
                document: clarification_document(prepared, &execution.scope()),
                status: ExitStatus::NeedsClarification,
                note: None,
            },
            [].to_vec(),
        );
    }

    let designed: Q5DesignedSet = match design_set(execution.clone(), prepared.design.clone()) {
        Ok(done) => done,
        Err(error) => return refusal(prepared, &error),
    };

    // Plan §3.2: one edited construct per member, in request order, whatever
    // its design's outcome is.
    let artifacts: Artifacts = execution
        .results()
        .iter()
        .map(|result| (result.identifier(), result.edited()))
        .collect();

    let outcomes: Vec<Q5MemberDesign> = designed.members();

    // `design_set` returns one outcome per member, in request order, so the
    // results and the outcomes stay paired by name: a member that could not be
    // designed keeps its own error and the rest are still designed (`docs/q5-sdm.md`
    // §12). An outcome that is neither a design nor a failure, or none at all,
    // is the same broken invariant as a single edit whose design is missing
    // (§3.2): the run is internal and the member is reported as built.
    let mut views: Vec<MemberView> = Vec::new();
    let mut internal: bool = false;
    let mut undesigned: bool = false;
    for result in execution.results().iter() {
        let member: MemberView =
            member_view(&result.identifier(), &result.mutation(), &result.warnings());
        let outcome: Option<&Q5MemberDesign> = outcomes
            .iter()
            .find(|one| one.result_name() == result.identifier());
        match outcome {
            Some(one) => match one.design() {
                Some(design) => views.push(designed_member(member, &design)),
                None => {
                    undesigned = true;
                    match one.failure() {
                        Some(problem) => views.push(undesigned_member(member, &problem)),
                        None => {
                            internal = true;
                            views.push(member);
                        }
                    }
                }
            },
            None => {
                internal = true;
                views.push(member);
            }
        }
    }

    let status: ExitStatus = if internal {
        ExitStatus::Internal
    } else if undesigned {
        ExitStatus::PartialDesign
    } else {
        ExitStatus::Success
    };

    let target: Option<Cds> = execution.target();
    let outcome: Outcome = Outcome {
        document: verified_document(prepared, target.as_ref(), views, warnings),
        status,
        note: None,
    };

    (outcome, artifacts)
}

/// The run's answer once `--write-edited` had its chance (plan §3.2, G8,
/// D012, §3.7): the same outcome when nothing was to be written or when
/// everything was, an `output_exists` failure - with nothing written at all -
/// when one file's name is already taken, an `output_name_unusable` failure the
/// same way when a result name cannot be a file name (§5.3, §4.1), and the same
/// document with `Internal` and a `note` when the write itself failed, because
/// the run itself still got its answer.
fn write_outcome(
    prepared: &Prepared,
    outcome: Outcome,
    artifacts: &Artifacts,
    directory: Option<&Path>,
) -> Outcome {
    match directory {
        None => outcome,
        Some(path) => match write_edited(path, artifacts) {
            Ok(_) => outcome,
            // G8: an existing file is never overwritten, and no file at all was
            // written, so the run's own answer is replaced by the refusal. It
            // names the file alone, never its directory (D012).
            Err(WriteFailure::Exists(name)) => Outcome {
                document: failed_after(
                    prepared,
                    ErrorView {
                        code: RequestErrorCode::OutputExists.code(),
                        rule: None,
                        category: category_name(ErrorCategory::MalformedInput),
                        message: format!("will not overwrite the existing file '{}'", name),
                        path: None,
                        details: Details(vec![("file", DetailValue::Text(name.clone()))]),
                    },
                ),
                status: ExitStatus::MalformedInput,
                note: None,
            },
            // §4.1, §5.3: the result name cannot be a file name - it holds a
            // path separator - so `directory` was never created and nothing was
            // written: the same refusal as an existing file, with its own code,
            // and naming the result name alone, never a directory (D012).
            Err(WriteFailure::NotAFileName(name)) => Outcome {
                document: failed_after(
                    prepared,
                    ErrorView {
                        code: RequestErrorCode::OutputNameUnusable.code(),
                        rule: None,
                        category: category_name(ErrorCategory::MalformedInput),
                        message:
                            format!(
                                "the result name '{}' cannot be a file name: it contains a path separator",
                                name,
                            ),
                        path: None,
                        details: Details(vec![("name", DetailValue::Text(name.clone()))]),
                    },
                ),
                status: ExitStatus::MalformedInput,
                note: None,
            },
            // §3.7: an I/O failure is exit 1 and the document still goes to
            // stdout, because the run succeeded and its result is still true.
            Err(WriteFailure::Io(problem)) => Outcome {
                document: outcome.document.clone(),
                status: ExitStatus::Internal,
                note: Some(problem.clone()),
            },
        },
    }
}

/// Run one request file through core and report it, writing each edited
/// construct into `write_edited` when one was given (plan §3.2, G8).
///
/// The only failure that is not a document is one this function cannot reach:
/// every path returns an `Outcome`, and an I/O failure keeps the document and
/// adds a `note` (plan §3.7).
pub fn run_with_output(request_path: &Path, write_edited: Option<&Path>) -> Outcome {
    match prepare(request_path) {
        Err(problem) => unprepared(&problem).0,
        Ok(prepared) => {
            let (outcome, artifacts): (Outcome, Artifacts) = match prepared.operation.clone() {
                Operation::Single { request, name } => single_edit(&prepared, *request, name),
                Operation::Set(set) => variant_set(&prepared, set, vec![]),
                Operation::Degenerate(expansion) => variant_set(
                    &prepared,
                    expansion.set(),
                    expansion
                        .warnings()
                        .iter()
                        .map(operation_warning_view)
                        .collect(),
                ),
            };
            write_outcome(&prepared, outcome, &artifacts, write_edited)
        }
    }
}

/// Run one request file through core and report it, writing nothing.
pub fn run(request_path: &Path) -> Outcome {
    run_with_output(request_path, None)
}
