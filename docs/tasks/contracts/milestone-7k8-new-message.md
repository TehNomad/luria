# Milestone 7k8: `luria new`'s unknown-option message names every option

Status: **Implemented, 2026-10-22.** Planner: Claude. Follows
7k7, which added `--backend`, `--endpoint`, `--model` and
`--allow-remote` to `luria new` but left the old message.

## 1. Change

`crates/cli/src/args.rs`, `parse_new`'s refusal of an unknown word: the
message

```
'new' takes only --design and --export, got '<word>'
```

becomes

```
'new' takes only --design, --export, --backend, --endpoint, --model and --allow-remote, got '<word>'
```

Nothing else in `parse_new` changes.

## 2. Test

`crates/cli/tests/cli.rs`, `new_with_an_unknown_word_is_a_usage_error_naming_it`:
its expected message changes to the §1 text (approved). No other test
changes.

Proof: restore the old message once; the test fails; put the new one back.

## 3. Do not

Change only those two places. No other code, test, doc, fixture, Cargo
file or `vendor/`. Do not stage or commit.

## 4. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```
