# Milestone 7e: the text of the proposal screen

Status: **Implemented 2026-10-21** by the local worker; accepted after one
question (rule 2's proof needed a synthetic document, test
`questions_come_before_a_request`). Planner:
Claude. Parent: `docs/tasks/milestone-7-tui.md` §3 rules 1, 2, 3 and 6,
§5.2 screen 1, §6 (7e). Drawing it, approving it and running the design
are 7f.

## 1. What and why

`luria review --template … --text …` (M7-1) first shows what `luria q5
interpret` proposed, so the user can judge it before anything runs. This
contract builds that screen's lines from the `InterpretationDocument`, in
`text.rs`, pure and tested, as 7c1 and 7c2 did for results.

## 2. Requirements this contract applies

- **Plan §3 rule 1:** every value is a document field. Nothing is parsed
  out of a message, re-derived or summarised.
- **Plan §3 rule 2 (what you see is what you export):** the proposed
  request is shown as **exactly the text `--write-request` writes**: the
  request echo through `serde_json::to_string_pretty`, line by line. A
  test compares the two.
- **Plan §3 rule 3:** questions and an error come first, before anything
  else, and are never summarised away.
- **Plan §3 rule 6 (privacy):** the screen shows what was sent to the
  model: the user message in full, and the system prompt's size and
  version.
- ASCII labels; document text is shown as it is.

## 3. Changes

`crates/cli/src/review/text.rs` gains:

```rust
pub fn proposal(document: &InterpretationDocument) -> Vec<String>;
```

(`crate::interpret::document::InterpretationDocument`.) Groups, in this
order, separated by one blank line, each present only when it has
content unless said otherwise:

1. **Questions**, when `questions` is not empty: `questions:`, then per
   question `  <code> (<subject>): <message>`.
2. **Error**, when `error` is `Some`: `error: <code>`, `  rule: <rule>`
   when `Some`, `  <message>`; then, when
   `interpretation.ungrounded` is `Some(u)`, `  not in your text:
   <u.field> "<u.value>"`.
3. **Proposed request**, when `request` is `Some(r)`: `proposed request,
   written exactly so when you approve it:`, then each line of
   `serde_json::to_string_pretty(&r)` prefixed with two spaces. If
   serializing fails, the single line `  the request cannot be shown`
   (it cannot fail in practice; do not panic).
4. **Origins**, when not empty: `where each value came from:`, then per
   origin `  <value>: <source>`.
5. **Assumptions**, when not empty: `assumptions:`, then `  <code>:
   <message>`.
6. **Evidence**, when `interpretation.extraction` is `Some(x)`: `the
   model's reading, checked against your text:`, then
   - `  operation: <q5_residue_substitutions or other>` (a `match` on
     `luria_nl::extraction::Operation`);
   - `  cds: "<cds>"` or `  cds: none`;
   - per mutation `  <token>  quote "<quote>"`, followed by `  codon
     <codon>` when `Some`;
   - `  composition: <separate_variants or combined_construct>` (a
     `match` on `Composition`) or `  composition: none`;
   - `  numbering: "<numbering>"` or `  numbering: none`;
   - per `unsupported` entry, `  unsupported: "<entry>"`.
7. **Template**, always: `template: <file>, LOCUS <locus_name>, sha256
   <sha256>` when `template` is `Some`, else `template: not read`.
8. **Model**, always: `model: <backend.kind> <backend.model> at
   <endpoint or "no endpoint">, prompt <prompt_version>`.
9. **Sent**, always: when `interpretation.messages` is `Some(m)`: `sent
   to the model:`, `  system prompt: <n> lines, not shown`, `n` being
   `m.system.lines().count()`, then `  user message:` and each line of
   `m.user` prefixed with four spaces; when `None`: `nothing was sent to
   the model`.
10. **Reply**, when `interpretation.reply` is `Some(text)`: `the model's
    reply, as received:`, then each line of `text` prefixed with two
    spaces.

## 4. Tests

### 4.1 `crates/cli/tests/review_proposal.rs` (new)

Run corpus cases of `tests/fixtures/nl/cases.tsv` through
`luria_cli::interpret::run::interpret` with
`luria_nl::backend::ScriptedBackend`, exactly as
`tests/interpret_end_to_end.rs` does (copy its `workspace`, `text_of`,
`input` and `interpret_case` helpers). The expected state and codes are
the corpus's own columns, which are independently derived
(`tests/fixtures/nl/README.md`).

1. `the_proposed_request_is_what_is_written`: case `two-separate`, with
   `write_request` set to a new file in the workspace. Assert the state is
   `proposed`. The lines after `proposed request, written exactly so when
   you approve it:` up to the next blank line, each with its two leading
   spaces removed and joined with `\n` plus a final `\n`, equal the
   written file's whole text.
2. `questions_come_first`: case `composition-unstated`: the first line is
   `questions:`, and the second starts with `  composition_unstated (`.
   There is no `proposed request` line.
3. `an_ungrounded_reply_names_the_string`: case `ungrounded-mutation`:
   the first line is `error: ungrounded_extraction`, and a line starts
   with `  not in your text: ` followed by the document's own
   `ungrounded.field`.
4. `a_reply_that_is_not_an_extraction_shows_the_reply`: case
   `reply-extra-field`: the first line is `error: model_output_invalid`,
   there is no `the model's reading` line, and the reply group's lines
   equal the document's own `reply` text, line by line, each prefixed with
   two spaces.
5. `what_was_sent_is_shown`: case `two-separate`: a line `  user
   message:` is followed by the document's `messages.user`, line by line,
   each prefixed with four spaces; the system line gives the document's
   own line count.
6. `the_evidence_quotes_each_mutation`: case `two-separate`: lines `
   A42G  quote "A42G"` and `  W87A  quote "W87A"` (the corpus reply
   `tests/fixtures/nl/reply/two-separate.json` holds these), and `
   composition: separate_variants`.

### 4.2 Unit tests in `text.rs`

7. `a_stated_codon_is_shown_beside_its_quote`: a hand-built document
   whose extraction has a mutation with `codon: Some("GGC")`: the line
   ends with `  codon GGC`.
8. `nothing_sent_says_so`: `messages` `None`: the line `nothing was sent
   to the model`, and no `user message` line.

### 4.3 Rules that must be proven

1. The request shown with `to_string` (compact) instead of pretty:
   test 1 fails.
2. Questions placed after the request: `questions_come_before_a_request`
   fails (added 2026-10-21 at the worker's question: no corpus case has
   both a request and questions, so test 2 cannot catch it; that test
   builds a synthetic document from `two-separate` plus a question and an
   error).
3. The ungrounded line left out: test 3 fails.

## 5. Do not

- Change only `crates/cli/src/review/text.rs` and the new
  `crates/cli/tests/review_proposal.rs`.
- Do not change `interpret/`, `luria-nl`, any fixture, other tests,
  `docs/`, any `Cargo.toml` or `vendor/`.
- Do not draw, write files, or call a model: the scripted backend only.
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
