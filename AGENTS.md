# Molecular biology toolbox: agent instructions

## Naming
- The project name is `luria`, styled in lowercase.
- Use “molecular-biology toolbox” as a descriptive phrase, not a second name.
- Use `luria` for the eventual user-facing executable.
- Prefer `luria-<component>` for Rust package names when components are added.
- Do not rename the project or introduce branding without approval.

## Product boundary
Build a standalone Rust application. Pi is the development harness, not a
runtime dependency. Rust owns domain models, canonical sequence semantics,
validation, and orchestration. Future Python workers perform selected
scientific computations behind a versioned request/response interface.
The LLM may parse requests into typed operations; it must not invent
sequences, primer designs, coordinates, or experimental results.

## Working method
- Implement one agreed task contract at a time.
- Do not run git add, commit, amend, reset, rebase, merge, or push.
- Do not expand scope or change biological assumptions without asking.
- Prefer small, testable changes to speculative framework construction.
- Treat docs/decisions.md and fixtures/README.md as requirements.
- If coordinates, strand, CDS, numbering, or expected biology are
  ambiguous, stop and report the ambiguity rather than choosing silently.

## First milestone
Implement only sequence, feature, and coordinate models and their tests
in crates/core. Do not build an LLM adapter, Python worker, database,
primer generator, TUI, or web interface yet.

## Handoff
Report changed files, exact checks run and their results, assumptions,
unverified behavior, and any deviations from the task contract.
Do not describe a check as passing unless you ran it successfully.
