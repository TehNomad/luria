# Milestone 7h: the documentation, and D046

Status: **Implemented 2026-10-21.** Proof failed as required. (The owner confirmed the
plan's §8a choices). Planner: Claude. Parent:
`docs/tasks/contracts/milestone-7-tui.md` §6 (7h).

## 1. What and why

Milestone 7's behaviour is built and tested. This contract writes it down
in `docs/specs/cli.md`, the README, the product brief and `docs/decisions/`,
and adds a test that keeps the page honest where it can drift: the
command lines and the order table's scale rule. The two long pieces are
the planner's, appended verbatim.

This contract authorizes the worker to edit `docs/specs/cli.md`, `README.md`,
`docs/product/brief.md` and `docs/decisions/`, exactly as below and
nowhere else.

## 2. Changes

Each replacement's old text occurs exactly once; if it does not, stop and
report it.

### 2.1 `docs/specs/cli.md`

1. Replace

   ```
   `luria q5 interpret`, which turns a request in words into a proposed
   request, is §10 (D044).
   ```

   with

   ```
   `luria q5 interpret`, which turns a request in words into a proposed
   request, is §10 (D044). `luria review`, which shows a result or a
   proposal in the terminal, is §11 (D046).
   ```

2. In §1's code block, replace the line `luria version` with these four
   lines, then `luria version`:

   ```
   luria review REQUEST.json [--export DIR]
   luria review --template FILE.gb (--text TEXT | --text-file FILE)
       --backend openai-compatible --endpoint URL --model NAME
       --request FILE.json [--design PROFILE.json] [--allow-remote] [--export DIR]
   ```

3. In §1's table, after the `q5 interpret ...` row, add:

   ```
   | `review ...` | Shows a result, or a proposal it runs once you approve, in the terminal, and exports it (§11). |
   ```

4. In §5's table, replace the row for exit 0's text `| 0 | `verified`, and
   every member designed; for `luria q5 interpret`, `proposed` |` with
   `| 0 | `verified`, and every member designed; for `luria q5
   interpret`, `proposed`; for `luria review`, you quit |`.

5. Append the whole of `docs/tasks/contracts/milestone-7h-cli-section.txt` at the
   end of the file.

### 2.2 `README.md`

Replace `The review-oriented TUI is next (Milestone 7, D034).` with:

```
Milestone 7 adds `luria review`: a terminal view of a result or a
proposal, which runs a proposal only once you approve it, and exports
the result, the edited constructs and an IDT order table. See
`docs/specs/cli.md` §11 (D046). The web interface is next (Milestone 8, D034).
```

and in the usage block near the top, after the `luria q5 interpret` lines,
add the four `luria review` lines of §2.1 step 2.

### 2.3 `docs/product/brief.md`

After `Build the TUI around the same application capabilities and typed
operations.` add a blank line and:

```
Implemented: luria review, its result, proposal, variant (sequence and
protein), details, provenance and export screens, and an IDT order
table, docs/specs/cli.md section 11 (D046).
```

### 2.4 `docs/decisions/`

Append the whole of `docs/tasks/contracts/milestone-7h-d046.txt` at the end of the
file.

## 3. Test: `crates/cli/tests/documented_review.rs` (new)

As `tests/documented_interpret.rs` does for §10:
1. `the_documented_command_lines_are_the_usage_text`: the fenced block
   at the start of §11 has four lines, each found in `USAGE` once
   trimmed.
2. `the_documented_scales_are_the_rule`: the table after
   `<!-- luria:order-scales -->` has four rows; for each row `a-b nt |
   code`, `order::scale(a)` and `order::scale(b)` are `Some(code)`;
   `scale(14)` and `scale(201)` are `None`.

Proof: change one documented scale code once; test 2 fails.

## 4. Do not

- Change only the four documents above, as above, and the new test.
- Do not change code, other tests, fixtures, task contracts, any
  `Cargo.toml` or `vendor/`. Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```
