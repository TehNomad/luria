# Milestone 6j: the documentation, and D044

Status: **Ready for the worker, 2026-10-19** (6h2 is committed; the §10.3
example was run against the local model by the planner, see 6i §8).
Planner: Claude. Parent: `docs/tasks/milestone-6-nl.md` §6 (6j).

## 1. What and why

Milestone 6's behaviour is built and tested; this contract writes it down
where users and future contributors look: `docs/cli.md`, the README, the
product brief, the Q5 workflow note, and decision D044 in
`docs/decisions.md`. The two long pieces are written by the planner in
files of their own and are appended verbatim; every other change is an
exact replacement given below. A small test keeps the page honest about
the parts that can drift: the command line, the question codes and the
profile example.

This contract authorizes the worker to edit `docs/cli.md`, `README.md`,
`docs/decisions.md`, `docs/product-brief.md` and `docs/q5-sdm.md`, exactly
as below and nowhere else.

## 2. Changes

Each replacement's old text occurs exactly once; if it does not, stop and
report it.

### 2.1 `docs/cli.md`

1. Replace

   ```
   This page describes `luria-cli` 0.1.0, which reads requests in schema
   `luria.q5-request/1` and writes results in schema `luria.q5-result/1`.
   The decisions behind it are D041 and D034 in `docs/decisions.md`.
   ```

   with

   ```
   This page describes `luria-cli` 0.1.0, which reads requests in schema
   `luria.q5-request/1` and writes results in schema `luria.q5-result/1`.
   The decisions behind it are D041 and D034 in `docs/decisions.md`.
   `luria q5 interpret`, which turns a request in words into a proposed
   request, is §10 (D044).
   ```

2. In §1's code block, replace the line

   ```
   luria q5 design REQUEST.json [--write-edited DIR]
   ```

   (the one inside the fenced block under `## 1. Commands`) with

   ```
   luria q5 design REQUEST.json [--write-edited DIR]
   luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
       --backend openai-compatible --endpoint URL --model NAME
       [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
   ```

3. In §1's table, after the row that starts `` | `--write-edited DIR` | ``,
   add the row:

   ```
   | `q5 interpret ...` | Asks a language model what a request in words means and prints the proposed request, or luria's questions, as one document. It runs nothing (§10). |
   ```

4. In §5's table, after the row that starts `| 7 |`, add:

   ```
   | 8 | `failed`, from `luria q5 interpret`: the model's reply was refused (§10.7) |
   ```

5. In §6's table, replace `Mapping those errors to request fields is
   planned for Milestone 6.` with `Milestone 6 left this as it is (M6-12).`

6. Append the whole content of `docs/tasks/milestone-6j-cli-section.txt`
   to the end of the file, byte for byte. It starts with an empty line, so
   the page's last line is followed by one blank line and `## 10. ...`.

### 2.2 `README.md`

Replace

````
Milestone 5 adds the `luria` executable: it reads a typed JSON request,
runs the Q5 workflow and writes one structured JSON result.

```
luria q5 design REQUEST.json [--write-edited DIR]
luria version
luria help
```
````

(the paragraph and the fenced block after it) with

````
Milestone 5 adds the `luria` executable: it reads a typed JSON request,
runs the Q5 workflow and writes one structured JSON result. Milestone 6
adds `luria q5 interpret`: a language model reads a request in words, and
luria checks every word of its answer against yours before proposing the
typed request, or asks what it needs to know. You review the proposal and
run it with `luria q5 design`.

```
luria q5 design REQUEST.json [--write-edited DIR]
luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
luria version
luria help
```
````

and replace

```
Natural-language interpretation is the next milestone (Milestone 6). See
`docs/decisions.md` D034.
```

with

```
Natural-language interpretation (Milestone 6, D044) works with any
OpenAI-compatible server, such as a local llama.cpp or Ollama, over plain
HTTP; HTTPS and the Anthropic API are deferred. It sends your text and a
summary of the record's CDSs, never the sequence. See `docs/cli.md` §10.
The review-oriented TUI is next (Milestone 7, D034).
```

### 2.3 `docs/decisions.md`

Append the whole content of `docs/tasks/milestone-6j-d044.txt` to the end
of the file, byte for byte. It starts with an empty line.

### 2.4 `docs/product-brief.md`

Replace

```
Add an LLM adapter that converts user requests into typed operations and
clarification requests.
```

with

```
Add an LLM adapter that converts user requests into typed operations and
clarification requests.

Implemented: luria q5 interpret, the luria.q5-interpretation/1 and
luria.q5-design-profile/1 schemas, a local OpenAI-compatible backend, and
docs/cli.md section 10 (D044).
```

### 2.5 `docs/q5-sdm.md`

Replace `` `proposed` is Milestone 6; `` with `` `proposed` is the state of a
`luria q5 interpret` document (Milestone 6, D044); ``.

## 3. Test (planner-written; add verbatim)

New file `crates/cli/tests/documented_interpret.rs`:

```rust
//! `docs/cli.md` §10 against the code (contract 6j): the command line it
//! shows is the one `luria help` prints, its question table lists exactly
//! the codes `build` can ask, and its profile example is a profile luria
//! reads, the corpus's own.

use luria_cli::args::USAGE;
use luria_cli::interpret::build::QuestionCode;
use luria_cli::request::parse_profile;
use std::fs;

const PAGE: &str = "../../docs/cli.md";

fn section_10() -> String {
    let page: String = fs::read_to_string(PAGE).expect("docs/cli.md is readable");
    page.split("## 10. Natural language: `luria q5 interpret`")
        .nth(1)
        .expect("docs/cli.md has §10")
        .to_string()
}

/// The first fenced block after `marker` in `text`, without its fences.
fn fenced_after(text: &str, marker: &str) -> String {
    let after: &str = text.split(marker).nth(1).unwrap_or_else(|| panic!("no {:?}", marker));
    let block: &str = after.split("```").nth(1).expect("a fenced block");
    block.split_once('\n').expect("a fence line").1.to_string()
}

#[test]
fn the_documented_command_line_is_the_usage_text() {
    let section: String = section_10();
    let shown: String = fenced_after(&section, "D044 and `docs/tasks/milestone-6-nl.md`.");
    let lines: Vec<&str> = shown.lines().map(|line| line.trim()).collect();
    assert_eq!(lines.len(), 3);
    for line in lines {
        assert!(USAGE.contains(line), "luria help does not show {:?}", line);
    }
}

#[test]
fn the_question_table_lists_every_code_build_can_ask() {
    let section: String = section_10();
    let table: &str = section
        .split("<!-- luria:question-codes -->")
        .nth(1)
        .expect("the question-codes marker");
    let mut documented: Vec<String> = table
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|line| {
            line.split('`')
                .nth(1)
                .expect("a code in backticks")
                .to_string()
        })
        .collect();
    documented.sort();
    let mut codes: Vec<String> = [
        QuestionCode::OperationUnsupported,
        QuestionCode::MutationUnstated,
        QuestionCode::MutationNotationUnsupported,
        QuestionCode::ResidueNumberUnstated,
        QuestionCode::OriginalResidueUnstated,
        QuestionCode::TargetResidueUnstated,
        QuestionCode::SubstitutionDirectionUnclear,
        QuestionCode::CodonUnstated,
        QuestionCode::CdsUnnamed,
        QuestionCode::CdsNotFound,
        QuestionCode::CdsUnusable,
        QuestionCode::CompositionUnstated,
        QuestionCode::NumberingUnstated,
        QuestionCode::NumberingUnsupported,
        QuestionCode::DesignUnstated,
    ]
    .iter()
    .map(|code| code.code().to_string())
    .collect();
    codes.sort();
    assert_eq!(documented, codes);
}

#[test]
fn the_documented_profile_is_the_corpus_profile() {
    let section: String = section_10();
    let shown: String = fenced_after(&section, "and pass it with `--design`:");
    let corpus: String =
        fs::read_to_string("../../tests/fixtures/nl/profile/full.json").expect("the corpus profile");
    assert_eq!(parse_profile(&shown).ok(), parse_profile(&corpus).ok());
    assert!(parse_profile(&shown).is_ok());
}

#[test]
fn d044_is_recorded() {
    let decisions: String = fs::read_to_string("../../docs/decisions.md").expect("decisions");
    assert_eq!(
        decisions.matches("## D044: Natural-language interpretation (Milestone 6)").count(),
        1
    );
}
```

`cargo fmt` may reflow it; do not change content. If a test fails, report
what it found; do not change the page or the test to make it pass.

### 3.1 Proving the test is load-bearing (AGENTS.md)

Temporarily delete the `design_unstated` row from the page's question
table, confirm `the_question_table_lists_every_code_build_can_ask` fails,
and restore it. Temporarily change `"minAnnealingOverlap": 15` in the
page's profile example to `16`, confirm
`the_documented_profile_is_the_corpus_profile` fails, and restore it. Show
each restore with `git diff` of the page against your finished version.

## 4. Do not

- Change any wording of the two appended files, or anything in the five
  documents beyond §2.
- Change any code, any existing test, `docs/tasks/`, `AGENTS.md` or
  `tests/fixtures/`.

## 5. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace` (the M5 documented-example test must still pass)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`
- `tail -c 200 docs/cli.md | cmp - <(tail -c 200 docs/tasks/milestone-6j-cli-section.txt)`
  and the same for `docs/decisions.md` against `milestone-6j-d044.txt`:
  both must print nothing.

## 6. Handoff

Report the changed files, the exact commands and their results, the §3.1
results, and anything that did not match this contract. Quote command
output for every claim; say plainly what you did not check.
