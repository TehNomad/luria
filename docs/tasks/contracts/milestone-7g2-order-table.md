# Milestone 7g2: the IDT order table

Status: **Implemented 2026-10-21** by the local worker; accepted on the
first review. The table equals the fixture byte for byte.
Planner: Claude. Parent: `docs/tasks/contracts/milestone-7-tui.md` §5.4 (owner
answers M7-4, M7-6 to M7-13), §6 (7g2). The expected table is in
`tests/fixtures/review/order-sid1-del11-ins14.tsv` (see that directory's
`README.md`). Writing it to a file from the screen is 7g1's export
mechanism; this contract only builds the bytes.

## 1. What and why

The owner orders primers from IDT by pasting a table into its Bulk Input.
This contract builds that table from a result document, in a pure module,
so its rules are tested without a terminal or a file system.

## 2. Requirements this contract applies (plan §5.4, binding)

- Header row `Name`, `Sequence`, `Scale`, `Purification`, tab-separated.
- One row per primer of every member with a `design`, forward then
  reverse, members in `results` order, fields tab-separated, `\n` after
  every row, the header included.
- Name: `<template id>_<member name>_F` for the forward primer and `_R`
  for the reverse, the template id being `provenance.template.id`.
- Sequence: `PrimerView.sequence` exactly as the document holds it, tail
  included.
- Scale, by the primer's full length `PrimerView.length`, inclusive:
  15–60 `25nm`; 61–90 `100nm`; 91–100 `250nm`; 101–200 `4nmU`.
- Purification: always `STD`.
- **Refusals, never a guessed value** (D005): the table is refused when
  - the document's `state` is not `verified`;
  - no member has a `design`;
  - the document has no `provenance.template`;
  - any primer's length is under 15 or over 200: the refusal names the
    primer's row name and length;
  - any primer's sequence holds a letter other than `A`, `C`, `G` or `T`
    (upper case): the refusal names the row name.
- A member with a `design_failure` (or neither) gets no row, and is
  listed by the caller as left out (§3, `left_out`).
- luria writes a file and never submits an order (product brief,
  non-goals).

## 3. Changes

New file `crates/cli/src/review/order.rs`, pure; `review.rs` gains
`pub mod order;`.

```rust
/// The IDT scale code for a primer of `length` bases (plan §5.4, M7-10,
/// M7-13), or `None` outside 15 to 200.
pub fn scale(length: u64) -> Option<&'static str>;

/// Why no table can be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderRefusal {
    NotVerified { state: String },
    NothingDesigned,
    NoTemplate,
    LengthOutsideScales { name: String, length: u64 },
    NotPlainDna { name: String },
}

impl OrderRefusal {
    /// One line for the screen.
    pub fn message(&self) -> String;
}

/// The table, as the file's bytes, and the names of the members left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderTable {
    pub text: String,
    pub left_out: Vec<String>,
}

pub fn order_table(document: &ResultDocument) -> Result<OrderTable, OrderRefusal>;
```

`message()`, exactly:
- `NotVerified`: `no order table: the result is <state>, not verified`
- `NothingDesigned`: `no order table: no member has a primer pair`
- `NoTemplate`: `no order table: the result names no template`
- `LengthOutsideScales`: `no order table: <name> is <length> nt; IDT
  scales here cover 15 to 200 nt`
- `NotPlainDna`: `no order table: <name> holds a base other than A, C, G
  or T`

The checks run in the order of §2's list; the first that fails is
returned. The length and base checks run over every primer in row order,
before any text is returned.

`left_out` holds the name of every member without a `design`, in order.

## 4. Tests

### 4.1 `crates/cli/tests/review_order.rs` (new)

Build a two-member set the way `tests/review_context.rs` builds its
SID-1 single edits (a SID-1 workspace under `env!("CARGO_TARGET_TMPDIR")`, the
SID-1 template of `tests/end_to_end.rs`, bound 15), with `"variants":
{"composition": "separate_variants", "members": [...]}` holding, in this
order, `{"name": "del-11", "deletion": {"interval": [1209, 1212], "from":
"GAT"}}` and `{"name": "ins-14", "insertion": {"point": 1209, "before":
"A", "after": "G", "insert": "GAT"}}`. Run it with
`luria_cli::run::run`.

1. `the_table_is_the_fixture`: `order_table(&document)` is `Ok`, its
   `text` equals the whole of
   `tests/fixtures/review/order-sid1-del11-ins14.tsv`, and `left_out` is
   empty.
2. `a_design_failure_is_left_out_and_nothing_else_is_ordered`: the pBR322
   `T99C` document of `tests/review_text.rs` (copy its builder): `Err(
   NothingDesigned)`.
3. `a_failed_result_is_not_ordered`: a request path that does not exist:
   `Err(NotVerified { state: "failed" })`.

### 4.2 Unit tests in `order.rs`

Build documents by hand, as `text.rs`'s unit tests do.

4. `the_scale_follows_the_length`: `scale` at 14, 15, 60, 61, 90, 91,
   100, 101, 200 and 201 gives `None`, `25nm`, `25nm`, `100nm`, `100nm`,
   `250nm`, `250nm`, `4nmU`, `4nmU`, `None`.
5. `a_primer_outside_the_scales_refuses_the_table`: a forward primer of
   length 201: `LengthOutsideScales { name: "<id>_<member>_F", length:
   201 }`; and one of 14 on the reverse: `.._R`, 14.
6. `a_long_tail_primer_is_an_ultramer`: one member whose forward primer
   is 120 nt: its row's scale is `4nmU`, purification `STD`.
7. `an_iupac_base_refuses_the_table`: a primer holding `N`:
   `NotPlainDna`; a lower-case `a`: `NotPlainDna`.
8. `a_member_without_a_design_is_left_out`: two members, the first with a
   `design_failure`, the second designed: `Ok`, two rows (plus the
   header), `left_out == ["<first name>"]`.
9. `a_clarification_is_not_ordered`: `state` `needs_clarification`:
   `NotVerified`.
10. `no_template_is_refused`: `provenance.template` `None`, otherwise
    designed: `NoTemplate`.
11. `every_message_is_one_line`: each variant's `message()` holds no `\n`
    and equals §3's text.

### 4.3 Rules that must be proven

1. The `61` boundary moved to `62` (`25nm` up to 61): test 4 fails.
2. The upper limit removed (`4nmU` for everything above 100): tests 4 and
   5 fail.
3. Rows reverse before forward: test 1 fails.
4. The base check removed: test 7 fails.

## 5. Do not

- Change only `crates/cli/src/review.rs` (the one `pub mod` line),
  `crates/cli/src/review/order.rs` and the new
  `crates/cli/tests/review_order.rs`.
- Do not draw anything, write any file, or add an export key: that is
  7g1/7g3.
- Do not change any fixture, other tests, `docs/`, any `Cargo.toml` or
  `vendor/`.
- Do not add dependencies. Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §4.3, and any point where this contract
does not match the code; stop and report rather than guess.
