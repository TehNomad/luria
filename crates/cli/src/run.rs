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
//!   document must never claim a design it does not carry.

use crate::exit::ExitStatus;
use crate::prepare::{prepare, Operation, PrepareFailure, Prepared, RequestFailure};
use crate::result::{
    clarification_document, designed_member, domain_error_view, failed_after, failed_document,
    member_view, operation_warning_view, undesigned_member, verified_document, MemberView,
    OperationWarning, ResultDocument,
};

use luria_core::cds::Cds;
use luria_core::error::{DomainError, ErrorCategory};
use luria_core::identity::Identifier;
use luria_core::mutation::MutationResult;
use luria_core::q5::{
    execute, execute_set, Q5EditWarning, Q5SdmRequest, Q5VariantSetExecution, Q5VariantSetRequest,
    Q5WorkflowExecution,
};
use luria_core::q5_design::{design_run, design_set, Q5DesignedSet, Q5MemberDesign};
use std::path::Path;

/// What one `luria q5 design` run produced: the document to print and the
/// status to exit with. Both are always present: a failure is a document too
/// (plan §3.7).
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub document: ResultDocument,
    pub status: ExitStatus,
}

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
fn unprepared(problem: &PrepareFailure) -> Outcome {
    Outcome {
        document: failed_document(problem),
        status: ExitStatus::for_failure(failure_category(&problem.failure)),
    }
}

/// A prepared request whose core call failed: the failed document, which keeps
/// the template's provenance and echoes the request it read, and the status the
/// error's category gives. The request was understood, so the error sits at no
/// path in it (§3.1, step 2).
fn refusal(prepared: &Prepared, error: &DomainError) -> Outcome {
    Outcome {
        document: failed_after(prepared, domain_error_view(error, None)),
        status: ExitStatus::for_failure(error.category()),
    }
}

/// One edit: run it, design it, and report the one member it produced (§3.1).
fn single_edit(prepared: &Prepared, request: Q5SdmRequest, name: Identifier) -> Outcome {
    let execution: Q5WorkflowExecution =
        match execute(request, prepared.annotations.clone(), name.clone()) {
            Ok(done) => done,
            Err(error) => return refusal(prepared, &error),
        };

    if execution.needs_clarification() {
        // D005, D020: the scope did not resolve, so nothing was edited, and
        // nothing is designed: the clarification is the answer.
        return Outcome {
            document: clarification_document(prepared, &execution.scope()),
            status: ExitStatus::NeedsClarification,
        };
    }

    // §3.2: core returns `Ok` only for a clarification or for a run whose edit
    // it verified, so here the mutation is present and the edit is verified;
    // `a_verified_run_is_verified_by_core` pins that rather than trusting it.
    let target: Option<Cds> = execution.scope().target();
    let warnings: Vec<Q5EditWarning> = execution.warnings();
    let mutation: MutationResult = execution.mutation().unwrap();
    let member: MemberView = member_view(&name, &mutation, &warnings);

    match design_run(execution.clone(), prepared.design.clone()) {
        Ok(designed) => match designed.design() {
            Some(design) => Outcome {
                document: verified_document(
                    prepared,
                    target.as_ref(),
                    vec![designed_member(member, &design)],
                    vec![],
                ),
                status: ExitStatus::Success,
            },
            // §3.2: a verified run always has a design, so this is the
            // broken invariant. The member is reported as built and the run
            // is internal, because a document must never claim a design it
            // does not carry.
            None => Outcome {
                document: verified_document(prepared, target.as_ref(), vec![member], vec![]),
                status: ExitStatus::Internal,
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
        },
    }
}

/// A variant set, or the expansion of one degenerate edit into one: run every
/// member, design each one, and report one member per variant in request
/// order. `warnings` are the operation's own, which an expansion raises for the
/// members it skipped (§3.1).
fn variant_set(
    prepared: &Prepared,
    set: Q5VariantSetRequest,
    warnings: Vec<OperationWarning>,
) -> Outcome {
    let execution: Q5VariantSetExecution = match execute_set(set, prepared.annotations.clone()) {
        Ok(done) => done,
        Err(error) => return refusal(prepared, &error),
    };

    if execution.needs_clarification() {
        // D005, D020: no member of the set is edited and none is designed.
        return Outcome {
            document: clarification_document(prepared, &execution.scope()),
            status: ExitStatus::NeedsClarification,
        };
    }

    let designed: Q5DesignedSet = match design_set(execution.clone(), prepared.design.clone()) {
        Ok(done) => done,
        Err(error) => return refusal(prepared, &error),
    };

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
    Outcome {
        document: verified_document(prepared, target.as_ref(), views, warnings),
        status,
    }
}

/// Run one request file through core and report it.
///
/// The only failure that is not a document is one this function cannot reach:
/// every path returns an `Outcome`.
pub fn run(request_path: &Path) -> Outcome {
    match prepare(request_path) {
        Err(problem) => unprepared(&problem),
        Ok(prepared) => match prepared.operation.clone() {
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
        },
    }
}
