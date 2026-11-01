# Milestone 7k6: the documentation of `luria new`, and D047

Status: **Implemented, 2026-10-22.** Planner: Claude. Parent:
[docs/tasks/contracts/milestone-7k-new.md](/tasks/contracts/milestone-7k-new.md) §4 (7k6). As 7h: the two long texts
are the planner's, appended verbatim.

This contract authorizes the worker to edit [docs/specs/cli.md](/specs/cli.md), `README.md`,
[docs/product/brief.md](/product/brief.md) and [docs/decisions/](/decisions/), exactly as below and
nowhere else. Each replacement's old text occurs exactly once; if it does
not, stop and report it.

## 1. Changes

### 1.1 [docs/specs/cli.md](/specs/cli.md)

1. Replace `is §11 (D046).` (the end of the opening paragraph's sentence
   about `luria review`) with `is §11 (D046). \`luria new\`, a form for a
   new design, is §12 (D047).`
2. In §1's code block, after the two `luria review` lines, add:

   ```
   luria new [--design PROFILE.json] [--export DIR]
       [--backend openai-compatible --endpoint URL --model NAME [--allow-remote]]
   ```

3. In §1's table, after the `review ...` row, add:
   `| \`new ...\` | A form for a template and mutations, which writes and runs the request once you approve (§12). |`
4. Append the whole of [docs/tasks/contracts/milestone-7k6-cli-section.txt](/tasks/contracts/milestone-7k6-cli-section.txt) at the
   end of the file.

### 1.2 `README.md`

1. In the usage block, after the two `luria review` lines, add the two
   `luria new` lines of 1.1 step 2.
2. Replace `\`docs/specs/cli.md\` §11 (D046). The web interface is next` with
   `\`docs/specs/cli.md\` §11 (D046). \`luria new\` is a form for a template and
   mutations, typed in a shorthand or in words, that writes and runs the
   request once you approve; see §12 (D047). The web interface is next`.

### 1.3 [docs/product/brief.md](/product/brief.md)

Replace `table, docs/specs/cli.md section 11 (D046).` with `table, docs/specs/cli.md
section 11 (D046); luria new, a form for a new design, docs/specs/cli.md
section 12 (D047).`

### 1.4 [docs/decisions/](/decisions/)

Append the whole of [docs/tasks/contracts/milestone-7k6-d047.txt](/tasks/contracts/milestone-7k6-d047.txt) at the end.

## 2. Test: `crates/cli/tests/documented_new.rs` (new)

As `documented_review.rs` does for §11:

1. `the_documented_command_lines_are_the_usage_text`: §12's first fenced
   block has two lines, each found in `USAGE` trimmed.
2. `the_documented_shorthand_reads_as_documented`: for each row of
   §12.2's table, every backticked example in the first column parses
   with `shorthand::parse_line` to `Ok` of the kind the second column
   names (`Residue` for "Residue", `Substitution` for "Base … from",
   `Replacement` for "replaced", `Deletion` for "deleted", `Insertion` for
   "inserted").

A row whose meaning starts with `The same` takes the previous row's kind;
any other row naming no kind panics. `D404N:AAC` must also read as a
`Residue` with codon `Some("AAC")` (answer to w7k6, 2026-10-22: the
`D404N:AAC` row names no kind).

Proof: change one example in §12.2 to `D404*` once; test 2 fails.

## 3. Do not

Change only the four documents and the new test. No code, other tests,
fixtures, task contracts, Cargo files or `vendor/`. Do not stage or
commit.

## 4. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```
