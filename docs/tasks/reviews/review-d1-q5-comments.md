---
type: Task Contract
title: "Review cleanup D1: comments in q5.rs"
description: "crates/core/src/q5.rs has 657 comment lines in about 1680 lines of non-test code."
status: deprecated
---
# Review cleanup D1: comments in `q5.rs`

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. Comment lines 657 → 578; module doc 95 → 53.
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) §D.

**Comments only.** No code, string, test or behavior change.

## 1. The problem

`crates/core/src/q5.rs` has 657 comment lines in about 1680 lines of non-test
code. Much of that text is milestone history ("the second entry point this
task adds", "Milestone 4e-2's …"). Several statements are now false, because
the decisions they describe as open have since been made or implemented.

## 2. Rules for every comment you touch (shared with D2 and D3)

1. **Describe the code as it is now.** Remove history: which milestone or
   task added something, what an earlier version did, "now", "no longer".
   Keep a date only when it dates an owner decision that a rule rests on
   (e.g. "owner, 2026-10-15").
2. **Keep every source citation for a rule that is still in the code.** That
   means each D-number (`D0xx`), each spec section ([docs/specs/q5-sdm.md](/specs/q5-sdm.md) §n,
   [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §n, H1–H8, I-1–I-13), and each owner decision.
   If you cut a paragraph, carry its citation into the shorter sentence that
   states the rule.
3. **Do not restate another module's rules** at length. Name the function
   that enforces them (e.g. "`mutation::verify` checks …") instead.
4. **Module docs (`//!`)** say:
   - what the module owns;
   - its public entry points;
   - the rules it decides itself, with their sources;
   - what it deliberately leaves to other modules.

   For `q5.rs`, aim for about 40 lines, down from 95.
5. **Item docs (`///`)** start with a one-line summary. After that, keep only
   what a caller needs: inputs, the errors returned, invariants, and sources.
6. **Inline `//` comments** inside function bodies: fix the false ones, and
   shorten ones longer than about 6 lines that narrate history. Leave
   accurate, short ones alone.
7. **Trailing comments** (code, then `//` on the same line): don't touch them.
   The guard in §5 treats those lines as code.
8. **Test modules** (`#[cfg(test)] mod tests` and below): don't touch. Their
   comments record how expected values were derived.
9. **Keep any comment that starts with `Defensive check:`** as it is.
10. **Make no new claim about behavior** that you haven't confirmed by reading
    the code. Where the code and a comment disagree, the comment is wrong.
    If you think the code is wrong, stop and report.
11. **Length is not the goal; accuracy is.** A shorter comment that drops a
    rule's source is a regression.

## 3. Facts to use (verified by the planner)

[docs/specs/q5-sdm.md](/specs/q5-sdm.md) §10. The owner settled all six items on 2026-10-14:

| # | Settled behavior |
| --- | --- |
| 1 | An unscoped DNA edit outside the resolved CDS proceeds. |
| 2 | Candidate CDSs are not narrowed by whether they can hold the residue. |
| 3 | A named CDS that matches no annotation is a clarification. |
| 4 | A single annotation is used when the caller names none. |
| 5 | A residue request whose expected original equals its replacement is legal when the codon changes. |
| 6 | Two variant-set members sharing a result identifier are refused at construction (`InvalidMutationRequest`), compared exactly. |

Other facts:
- Primer design and Tm rules exist: D029–D033, `primer_design.rs`, `tm.rs`.
  `q5.rs` itself designs no primer. `q5_design.rs` composes this module with
  `primer_design.rs`.
- `Q5MutationIntent` has six variants: `Nucleotide`, `Deletion`,
  `Insertion`, `Replacement`, `Residue` and `ResidueWithoutCds`.
- `run_resolved` dispatches to `apply_nucleotide`, `apply_deletion`,
  `apply_insertion`, `apply_replacement` and `apply_residue`, then
  `verify`.
- Frameshift warnings: deletion D031, insertion D032, replacement D033's
  addenda, all owner decisions of 2026-10-15. A CDS cannot have an intron
  here: `validate_structure` refuses gaps between intervals.

## 4. Known false or stale statements to fix in `q5.rs`

Line numbers are approximate.

- **Module doc, lines 5–10 and 50–55:** says primer-design and
  thermodynamic rules are "not written down". They are (§3).
- **Module doc, line 57:** "`execute_set` is the second entry point this
  task adds". This is history.
- **Module doc, lines 78–95:** says six §10 entries are "recorded rather than
  settled", and that "a variant set checks no member's result identifier
  against another's". All six are settled, and item 6 is now refused (§3).
- **Lines 47, 840, 938, 1596:** list only `apply_nucleotide`, `apply_residue`
  and `verify`. Name all five operations, or say "the `mutation::apply_*`
  operation for the intent".
- **`execute`'s doc, around line 843:** "those rules are unwritten (D014)".
- **`Q5MutationIntent::named_cds`'s doc, around line 169:** "the other two
  intents". There are five other variants.
- **The `edit_warnings` doc, around lines 612–614:**
  - "each stated once here and nowhere else": the point-inside test is also
    in `mutation::require_point_within_feature`;
  - "an intron position": not representable (§3).
- **`Q5SdmRequest` and `execute` docs:** long passages on the containment
  question (§4). That question is settled (item 1), so state the rule and
  cite §10 item 1 rather than argue it.

## 5. Guard: non-comment lines must be unchanged

Run this from the repo root after editing. It must print nothing and exit 0:

```bash
python3 -I - crates/core/src/q5.rs <<'EOF'
import subprocess, sys
def code(text):
    return [l.rstrip() for l in text.splitlines()
            if l.strip() and not l.strip().startswith('//')]
bad = False
for f in sys.argv[1:]:
    old = subprocess.run(['git', 'show', 'HEAD:' + f], capture_output=True,
                         text=True, check=True).stdout
    if code(old) != code(open(f).read()):
        print('NON-COMMENT CHANGE:', f); bad = True
sys.exit(1 if bad else 0)
EOF
```

## 6. Do not

- Edit any file other than `crates/core/src/q5.rs`.
- Change any line that isn't a whole-line comment.
- Touch the test module.

## 7. Checks

Prefix cargo with `TMPDIR=$PWD/target/tmp` (run `mkdir -p target/tmp`
first). The sandbox denies the linker read-write access to `/tmp`.

```bash
<the §5 guard>
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 8. Handoff

Report:
- comment line counts before and after (non-test part);
- each §4 item and how you fixed it;
- any citation you dropped, and why;
- the exact check results.
