# Milestone 7m7: words in `luria new` use the form's CDS

Status: **Ready for the worker, 2026-10-22.** Planner: Claude. Parent:
`docs/tasks/milestone-7m-polish.md`, M7-59. Follows 7m6 (committed
`8b46cb2`).

## 1. Why, and the owner's answer

In the owner's backend hand test, `luria new` with `synthetic-linear.gb`
and the words `change alanine 42 to glycine` gave a proposal with the one
question `cds_unnamed`. A proposal with a question cannot be approved, so
there was no way to save. The form had already picked the CDS (`synA`)
on its CDS field.

M7-59 (owner, 2026-10-22): in `luria new`, words that name no CDS use the
CDS picked on the form, and this is recorded as an assumption. Words that
name a *different* CDS than the form's are asked about. M6-9 ("always ask
`cds_unnamed`") still holds for `luria q5 interpret` and `luria review
--template`, which have no CDS field. Their behaviour must not change.

## 2. Changes

### 2.1 `crates/cli/src/interpret/build.rs`

- Add `QuestionCode::CdsNotTheForms`, code `"cds_not_the_forms"`, after
  `CdsUnusable`.
- Add
  `pub fn build_with_cds(text, extraction, catalogue, profile, template, form_cds: Option<&str>) -> Built`.
  Use the same types as `build`. Move `build`'s body into it, and make
  `build` call `build_with_cds(..., None)`. With `None`, the behaviour is
  exactly what it is today.
- Step 3c, with `form_cds: Some(c)`:
  - The extraction names no CDS: there is no question, and the scope is
    `c`. Step 4 adds a second assumption after `template_id_from_locus`:
    code `"cds_from_form"`, message
    `format!("the text names no CDS; it is {}, the CDS picked on the form", c)`.
  - The extraction names `name != c`: ask
    `Question { code: CdsNotTheForms, subject: name, message: format!("the text names the CDS \"{}\", but the form picked {}", name, c) }`.
    The existing not-found and unusable checks for `name` still run after
    it, unchanged.
  - The extraction names `c`: as today.
- Step 4's `scope` guard: when `named` is `None`, use `form_cds` if it is
  `Some`, and only otherwise answer `cds_unnamed`.
- Add `(QuestionCode::CdsNotTheForms, "cds_not_the_forms")` to the unit
  test that lists every code.

Grounding (step 1) is unchanged. A CDS that comes from the form is not
the model's, so it is not grounded against the text.

### 2.2 `crates/cli/src/interpret/run.rs`

Add `pub fn interpret_with_form(input: &InterpretInput, profile: ProfileDocument, cds: &str, backend: &mut dyn Backend) -> InterpretOutcome`.
It is `interpret_with_profile` with `form_cds: Some(cds)` passed through
to `build_with_cds`. Thread an `Option<&str>` through the private `run`.
`interpret` and `interpret_with_profile` pass `None` and behave as today.
`interpret_with_profile` stays: one test still calls it.

### 2.3 The form and the loop

- `crates/cli/src/new/form.rs`: `FormAction::Interpret` gains
  `cds: String`, after `profile`. It is the id of
  `cds_list[cds_index]`. `review()` already refuses a non-GenBank
  template before `Interpret`. If `cds_list` is empty, the existing
  blockers stop the form first. If they do not, stop and report it.
- `crates/cli/src/new/terminal.rs`: the `interpret` closure becomes
  `&mut dyn FnMut(&str, &str, ProfileDocument, &str) -> InterpretationDocument`.
  The fourth argument is the CDS, which the loop passes from
  `Interpret { cds, .. }`.
- `crates/cli/src/main.rs`: `WordsModel` and the closure gain the
  `cds: &str` argument and call `interpret_with_form(&input, profile, cds, &mut openai)`.
  The no-backend closure takes four arguments and still panics.

### 2.4 Docs

- `docs/cli.md` §10's question table: after the `cds_unusable` row, add

  ```
  | `cds_not_the_forms` | In `luria new` only: your words name a CDS other than the one picked on the form. |
  ```

  Change the `cds_unnamed` row's text to
  `The text names no CDS. luria never picks one, even when the record has one; in \`luria new\`, the CDS picked on the form is used instead (§12.3).`
- `docs/cli.md` §12.3: replace

  ```
  counted from the CDS's first codon unless your words say otherwise. The
  ```

  with

  ```
  counted from the CDS's first codon unless your words say otherwise. If
  your words name no CDS, the one picked on the form is used, and the
  proposal says so; if they name another, it asks. The
  ```

- `docs/decisions.md` D047's Words bullet: replace
  `* **Words** (M7-33, M7-41, M7-42, M7-47).` with
  `* **Words** (M7-33, M7-41, M7-42, M7-47, M7-59).`, and replace

  ```
    the default codon policy and residues counted from the CDS start; it
  ```

  with

  ```
    the default codon policy, residues counted from the CDS start, and,
    when the words name no CDS, the form's CDS as an assumption (M7-59;
    M6-9 still holds for `luria q5 interpret`); it
  ```

  In D047's Status line, `answers M7-45 to M7-58` becomes
  `answers M7-45 to M7-59`.

Each old text occurs exactly once. If one does not, stop and report it.

## 3. Tests

1. `crates/cli/tests/new_words.rs`, `words_naming_no_cds_use_the_forms`:
   use corpus case `cds-unnamed`. Its reply says `cds: null`, and its text
   is `Change alanine 42 to glycine, numbering from the CDS start.`. Build
   the form as `words_form_without_profile` does. `Enter` gives
   `Interpret` with `cds == "synA"`. Run `interpret_with_form` with the
   scripted reply and `write_request: None`. The state is `proposed`, and
   there are no questions. The request equals
   `tests/fixtures/nl/request/edit-a42g.json`, and the assumptions
   include code `cds_from_form` with the message above.
2. `build.rs` unit test, `a_form_cds_answers_an_unnamed_one`: the reply
   has `cds: null`. With `build_with_cds(..., Some("synA"))` the result
   is `Proposed` with scope `synA` and the `cds_from_form` assumption.
   With `build` it asks `cds_unnamed`, as today.
3. `build.rs` unit test, `words_naming_another_cds_than_the_forms_ask`:
   the text names `synA`, the reply has `cds: "synA"`, and
   `form_cds = Some("synB")`. It asks `cds_not_the_forms` with subject
   `synA`.
4. `documented_interpret.rs`'s code list gains `QuestionCode::CdsNotTheForms`.
   This is the one approved change to an existing assertion list.
5. The existing tests that build or destructure `FormAction::Interpret`
   in `new_words.rs` gain the `cds` field (`cds: _` in patterns, and
   `cds: "synA".to_string()` where it is built, if that is the record's
   CDS). Nothing else in them changes.

Corpus case `cds-unnamed` through `interpret` (no form) must still ask
`cds_unnamed`. The existing corpus tests prove it unchanged.

### 3.1 Rule that must be proven

Make `build_with_cds` ignore `form_cds` when the CDS is unnamed: tests 1
and 2 fail, and every corpus test still passes. Restore it.

## 4. Do not

- Change only `interpret/build.rs`, `interpret/run.rs`, `new/form.rs`,
  `new/terminal.rs`, `main.rs`, the tests named in §3, `docs/cli.md` and
  `docs/decisions.md` as in §2.4.
- No change to `luria q5 interpret`'s or `luria review`'s behaviour,
  fixtures, prompts, `luria-nl`, Cargo files or `vendor/`.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proof, and any existing assertion changed
beyond §3 items 4 and 5.
