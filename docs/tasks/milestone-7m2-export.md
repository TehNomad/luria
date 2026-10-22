# Milestone 7m2: export names constructs by template, and asks before replacing

Status: **Draft, waits for 7m1** (both change `review/app.rs`). Planner:
Claude. Parent: `docs/tasks/milestone-7m-polish.md`, M7-57 and the export
half of M7-49.

## 1. The owner's answers this contract applies

- M7-57: constructs are `<template id>_<member>.fa`, e.g.
  `sid-1_cDNA_D404N.fa`, as the order table already names primers
  (`sid-1_del-11_F`). The template id is the result's
  `provenance.template.id`; with no template, the name is `<member>.fa`
  as now.
- M7-49 (export): when a file the export would write exists, a prompt
  names it and `y` replaces it; any other key writes nothing.
  `luria q5 design --write-edited` still never replaces (its code and
  tests are unchanged).

## 2. Changes

### 2.1 `crates/cli/src/write.rs`: one shared writer

Add `pub fn write_text(path: &Path, text: &str, replace: bool) ->
std::io::Result<()>`:

- with `replace` false: `create_new`, exactly as today's writers;
- with `replace` true: if `path` exists and is not a regular file (a
  directory or a symlink of any kind, by `symlink_metadata`), refuse
  with `ErrorKind::AlreadyExists`, so nothing is ever written through a
  link. Otherwise write `text` to a new file `.<name>.luria-tmp` in the
  same directory with `create_new`, then `fs::rename` it over `path`, so
  a reader never sees half a file. A failure removes the temporary file.

`write_edited` is unchanged. Its name checks (separators, `:*?<>|`, case
clashes) move into `pub(crate) fn file_name_problem(names: &[String]) ->
Option<WriteFailure>`, which `write_edited` calls. Its behaviour and
tests are unchanged.

### 2.2 `crates/cli/src/review/export.rs`

- `export_plan`: `constructs` are `<template id>_<member>.fa` (§1).
- `pub fn taken(plan: &ExportPlan) -> Vec<String>`: the plan's names that
  exist in `plan.directory` (by `symlink_metadata`), in the plan's order:
  constructs, then the result, then the order table when there is one.
- `write_export(plan, shown, replace: bool)`, in this order:
  1. `file_name_problem` over the construct names: refuse, nothing
     written.
  2. Unless `replace`, a taken name refuses as now: `<name> already
     exists; …; nothing was written`.
  3. One `run_with_constructs(&plan.request)`: its document must equal
     `shown`, else the existing refusal. Its constructs are the
     sequences to write. This replaces both of today's runs, the check
     and `run_with_output`, with one.
  4. Create the directory, then write each construct as
     `render_fasta(<member name>, sequence)` (the header stays the
     member's name, as `--write-edited` writes it), then the result, then
     the table, each with `write_text(…, replace)`. A failure names what
     is already written, as `unfinished` does now.

### 2.3 `crates/cli/src/review/app.rs` and `review/view.rs`

- `Screen::Replace` is a prompt over the Export screen. On `Export`, `w`
  or `o` gives `Action::Export` when `taken` is empty, as now. When it is
  not empty, it opens `Replace` instead. For that, `App` gains
  `taken: Vec<String>`, set by the shell with `App::set_taken(names)`
  before each key on `Export`; it stays pure.
- `Replace` lines: `these files exist in <directory>:`, one line per
  name (two spaces, the name), an empty line, then `y replace them  any
  other key: write nothing`. `y` gives `Action::ExportReplace`. Any other
  key goes back to `Export` with the message `nothing was written`.
  `ctrl-c` quits.
- `review/terminal.rs`: before each key on `Export`, call
  `app.set_taken(taken(plan))`. On `ExportReplace`, call the export
  closure with `replace` true. The closure's type gains the `bool`.
  `main.rs`'s three closures pass it to `write_export`.

## 3. Tests

Approved changes to existing assertions: in
`crates/cli/tests/review_export.rs` and `review/app.rs`, the construct
names become `sid-1_del-11.fa` and `sid-1_ins-14.fa` (template id
`sid-1`), and `write_export` calls gain `false`. Change nothing else.

New tests, in `crates/cli/tests/review_export.rs`:

1. `constructs_are_named_by_template`: the plan's constructs are
   `sid-1_del-11.fa`, `sid-1_ins-14.fa`.
2. `a_written_construct_is_the_write_edited_bytes`: the exported
   `sid-1_del-11.fa` equals the `del-11.fa` that `luria q5 design
   --write-edited` writes for the same request, byte for byte.
3. `replace_writes_over_a_taken_name`: put `old\n` in `sid-1_del-11.fa`.
   With `replace` false, nothing is written and the file still holds
   `old\n`. With `replace` true, the export succeeds and the file equals
   test 2's bytes.
4. `replace_never_writes_through_a_link` (unix only): `sid-1_del-11.fa` is
   a symlink to a file outside the directory. With `replace` true, the
   export refuses, and the link's target is unchanged.
5. `taken_lists_existing_names_in_order`.

In `review/app.rs`: `w` with names taken opens `Replace`. `y` there gives
`ExportReplace`, and `n` goes back with `nothing was written`.

In `crates/cli/src/write.rs`'s tests or `tests/write_edited.rs`: every
existing test passes unchanged.

### 3.1 Rules that must be proven

1. The link rule (drop the symlink check in `write_text`): test 4 fails.
2. Replace false never replaces (pass `true` through regardless): test 3
   fails.

## 4. Do not

- Change only `write.rs` (§2.1), `review/export.rs`, `review/app.rs`,
  `review/view.rs`, `review/terminal.rs`, `main.rs`'s three export
  closures, and the tests in §3.
- No change to `--write-edited`'s behaviour, `run.rs`, luria-core,
  fixtures, docs, Cargo files or `vendor/`.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proofs, and every existing assertion
changed.
