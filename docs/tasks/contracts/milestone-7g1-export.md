---
type: Task Contract
title: "Milestone 7g1: export from the review screen"
description: "M7-4: the reviewed result can be exported as the result JSON, the edited constructs as FASTA, and the IDT order table."
status: deprecated
tags: [milestone-7]
---
# Milestone 7g1: export from the review screen

Status: **Implemented 2026-10-21** by the local worker; accepted on the
first review. Proof 3 differs: step 1's existence check stops the second
export before `create_new` is reached, so `create` alone passes test 2;
with step 1 also removed it fails. `create_new` guards the race between
check and write. The terminal wiring in `main.rs` is verified by
compilation only, until the owner runs it. Note since drafting: 7f2
factored the interpret-form parsing into `parse_interpret_form` in
`args.rs`; add `--export` to the review form there, not to `q5
interpret`. Planner:
Claude. Parent: [docs/tasks/contracts/milestone-7-tui.md](/tasks/contracts/milestone-7-tui.md) §3 rules 2 and 4, M7-4,
§5.4 ("Writing"), §6 (7g1). Uses 7g2's `order_table`.

## 1. What and why

M7-4: the reviewed result can be exported as the result JSON, the edited
constructs as FASTA, and the IDT order table. Plan §3 rule 2: an export
is byte for byte what the CLI gives. This contract adds an export screen
that lists exactly which files will be written, and writes them only on
`y`.

**Planner choices, recorded in the plan's §8a for the owner:**
- The export directory is given on the command line, `--export DIR`, on
  both forms of `luria review`. Without it the export screen says how to
  enable it. (A TUI path prompt would be new text input, for no gain.)
- File names: `<request file stem>.result.json` and `<request file
  stem>.order.tsv`, beside `--write-edited`'s `<member name>.fa`.
- The edited constructs are written by running the request file again
  with `run_with_output(request, Some(dir))`, so `--write-edited`'s rules
  apply unchanged; the export first checks that a fresh run gives the
  document on screen, and refuses otherwise (D012: it always does unless
  a file changed).

## 2. Requirements this contract applies

- **Byte-identical:** `<stem>.result.json` holds `result::render` of the
  document on screen, which is what `luria q5 design REQUEST.json`
  prints. `<stem>.order.tsv` holds `order_table(document).text`. The
  FASTA files are `--write-edited`'s.
- **Never replace a file** (G8, D041): if `<stem>.result.json`, the order
  file, or any `<member>.fa` exists, nothing is written.
- **No silent action:** only `y` on the export screen writes.
- The order table is written only when `order_table` is `Ok`; otherwise
  the screen shows the refusal's `message()` and the other files are
  still written.

## 3. Changes

### 3.1 `crates/cli/src/args.rs`

Both `review` forms take an optional `--export DIR` (a path option: empty
is `an empty path is not a file`; twice is `--export is given more than
once`). `Command::Review` and `Command::ReviewInterpret` gain `export:
Option<PathBuf>`. In the request form, `--export` must come before a `--`
that ends the options, as `--write-edited` must in `q5 design`. `USAGE`'s
two review lines gain ` [--export DIR]`.

### 3.2 `crates/cli/src/review/export.rs` (new; `review.rs` gains `pub mod export;`)

```rust
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
/// `<stem>.order.tsv`, `<stem>` being `request`'s file stem
/// (`Path::file_stem`, lossily as UTF-8); `constructs` is `<name>.fa` for
/// each member of `document.results`, in order.
pub fn export_plan(document: &ResultDocument, request: &Path, directory: &Path) -> ExportPlan;

/// Write the plan's files, or none of them, and return the names written,
/// in the order written. `shown` is the document on screen.
pub fn write_export(plan: &ExportPlan, shown: &ResultDocument) -> Result<Vec<String>, String>;
```

`write_export`, in order:
1. For `result_file`, and `order_file` when `order` is `Ok`: if
   `symlink_metadata(directory.join(name))` succeeds, `Err("<name>
   already exists; luria never replaces a file; nothing was written")`;
   any error other than `NotFound` is `Err("cannot check whether <name>
   exists: <reason>; nothing was written")`.
2. `run(&plan.request)`; if its document differs from `shown`, `Err("the
   request gives a different result now; nothing was written")`.
3. `fs::create_dir_all(&plan.directory)`; a failure is `Err("cannot
   create the export directory: <reason>; nothing was written")`.
4. `run_with_output(&plan.request, Some(&plan.directory))`: if its
   document differs from `shown` (that is `--write-edited`'s refusal,
   such as an existing `.fa` file), `Err(<that document's error message>
   + "; nothing was written")`; if its `note` is `Some(n)`, `Err("could not
   write the edited constructs: <n>")`.
5. Write `result_file` with `render(shown)`, then `order_file` with the
   table's text when `Ok`, each with `OpenOptions::new().write(true)
   .create_new(true)`; a failure is `Err("<constructs written so far>
   written; could not write <name>: <reason>")`, naming what was written.
6. `Ok` with the construct names (those that `run_with_output` wrote,
   which are `plan.constructs` for a run with members), then
   `result_file`, then `order_file` when written.

### 3.3 `crates/cli/src/review/app.rs`

- `Screen` gains `Export`. `Action` gains `Export`.
- `App` gains `with_export(self, plan: Option<ExportPlan>) -> App`,
  `export_plan(&self) -> Option<&ExportPlan>`, `export_message(&self) ->
  Option<&str>`, and `export_done(&mut self, outcome: Result<Vec<String>,
  String>)`, which sets the message to `exported <n> files to
  <directory>: <names joined with ", ">` or `export failed: <message>`,
  and stays on `Export`.
- `handle`: `Overview` `Char('x')` → `Export` (always; the screen
  explains when there is no plan). `Export`: `Char('y')` → `Action::Export`
  only when there is a plan, else nothing; `Char('n')`, `Esc`,
  `Backspace` → `Overview`; `q` quits. A change of screen clears nothing
  but the scroll.

### 3.4 `crates/cli/src/review/text.rs`

```rust
pub fn export(plan: Option<&ExportPlan>, message: Option<&str>) -> Vec<String>;
```

- `None` plan: `start luria review with --export DIR to export this
  result`.
- Otherwise: `export to <directory>:`; `  <result_file>  the result, as
  luria q5 design prints it`; either `  <order_file>  IDT order table,
  <rows> primers` (`rows` = lines of the text minus the header) or `
  order table not written: <refusal message()>`; per construct `
  <name>  edited construct`; when `left_out` is not empty, `members left
  out of the order table: <names joined with ", ">`; then a blank line and
  `nothing is replaced: if any of these files exists, nothing is
  written`.
- When `message` is `Some(m)`: a blank line, then `m`.

### 3.5 `crates/cli/src/review/view.rs`

`Screen::Export` draws `text::export(app.export_plan(),
app.export_message())`. Key lines: `Export`: `q quit  y write  n back`
(`y write` only with a plan). `Overview`: `q quit  enter open  j/k move
p provenance  x export`, then `  e problem` as before.

### 3.6 `crates/cli/src/review/terminal.rs` and `main.rs`

`terminal::run` gains a parameter `export: &mut dyn FnMut(&App) ->
Result<Vec<String>, String>`, called on `Action::Export`, its outcome
passed to `app.export_done`. `main.rs` builds the plan when `export` is
`Some(dir)`, with the request path (`REQUEST.json`, or 7f2's `--request`
once approved), and passes `|app| write_export(plan, app.document())`.

## 4. Tests

### 4.1 `crates/cli/tests/review_export.rs` (new)

Use the SID-1 `del-11` + `ins-14` set of `tests/review_order.rs` (copy its
builder), in a workspace under `env!("CARGO_TARGET_TMPDIR")`, exporting
to `<workspace>/out` (which does not exist yet).

1. `the_export_is_what_the_cli_writes`: `write_export` → `Ok` with
   `["del-11.fa", "ins-14.fa", "set.result.json", "set.order.tsv"]` (the
   request file is `set.json`). `set.result.json` equals
   `render(&run(request).document)`; `set.order.tsv` equals
   `tests/fixtures/review/order-sid1-del11-ins14.tsv`; each `.fa` equals
   the file `run_with_output(request, Some(other_dir))` writes for the
   same member in a second, empty directory.
2. `nothing_is_replaced`: export twice; the second is `Err` naming
   `set.result.json`, and every file's content is unchanged.
3. `an_existing_construct_blocks_the_export`: create `out/del-11.fa`
   first: `Err`, and neither `set.result.json` nor `set.order.tsv`
   exists.
4. `a_changed_request_is_not_exported`: change the request file after
   the run (for example its `minAnnealingOverlap` to 16): `Err("the request
   gives a different result now; nothing was written")`, and `out` holds
   nothing.
5. `a_result_with_no_design_exports_without_a_table`: the pBR322 `T99C`
   document (copy its builder): `Ok`; the names hold the result file and
   the construct but no `.order.tsv`, and the plan's `order` is
   `Err(NothingDesigned)`.

### 4.2 Unit tests

6. `app.rs`: `x_opens_export_and_y_asks_only_with_a_plan`;
   `export_done_reports_and_stays`.
7. `text.rs`: `no_plan_says_how_to_export`; `a_refused_table_is_named`.
8. `args.rs`: `--export` on both forms; given twice; empty.

### 4.3 Existing tests to change, and only these

In `tests/review_view.rs`, the overview key line gains `  x export`
before any `  e problem`: change exactly the expected strings in
`the_state_and_validation_are_on_screen` and
`the_key_line_offers_problem_only_with_a_problem`. In `args.rs`'s tests
and `tests/cli.rs`, update only what `USAGE`'s two lines and the new
`export` fields require.

### 4.4 Rules that must be proven

1. Skipping step 1's existence check: test 2 fails.
2. Skipping step 2's comparison: test 4 fails.
3. Writing `result_file` with `File::create`: test 2 fails.
4. `y` writing without a plan: test 6 fails.

## 5. Do not

- Change only `args.rs`, `main.rs`, `review.rs`, `review/export.rs`,
  `review/app.rs`, `review/text.rs`, `review/view.rs`,
  `review/terminal.rs`, and the tests §4 names.
- Do not change `run.rs`, `write.rs`, `order.rs`, fixtures, [docs/](/), any
  `Cargo.toml` or `vendor/`.
- Do not add dependencies. Do not stage or commit. Do not run `luria
  review` by hand.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §4.4, and any point where this contract
does not match the code; stop and report rather than guess.
