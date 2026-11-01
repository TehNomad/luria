---
type: Task Contract
title: "Contract okf-c — docs: OKF frontmatter, index files and the log"
description: "Every concept document in docs/ gets an OKF v0.2 frontmatter block (§4.1, §5.4), every directory gets an index.md (§8), and the bundle root gets a log.md (§9)."
status: deprecated
tags: [okf]
---
# Contract okf-c — docs: OKF frontmatter, index files and the log

Planner: Claude. Status: **Implemented 2026-10-31**. Tier: presentation (no biology changes).
Step 3 of 3 of the OKF reorganization; okf-a and okf-b are done.

## 1. What this contract builds

Every concept document in `docs/` gets an OKF v0.2 frontmatter block (§4.1,
§5.4), every directory gets an `index.md` (§8), and the bundle root gets a
`log.md` (§9). After this contract, `docs/` is a conformant OKF bundle
(§11): every `.md` file other than `index.md` and `log.md` starts with a
parseable YAML block that has a non-empty `type`.

## 2. Files the worker may touch

1. `target/tmp/okf-frontmatter.py`, `target/tmp/okf-index.py` — new.
2. Every `*.md` file under `docs/` — change: insert the frontmatter block
   before line 1. No other change.
3. `docs/references/nbc-help-v284.md` — new (§5).
4. `docs/**/index.md` (one per directory in §6) and `docs/log.md` — new.

## 3. The frontmatter block

Exact form; keys in this order; omit `tags` when the table gives none.
Every string value is in double quotes, with `"` and `\` escaped.

```
---
type: Decision
title: "D001: Pi is development tooling"
description: "One sentence."
status: stable
tags: [milestone-7]
---
```

The original first line follows the closing `---` directly.

| Files | `type` | `status` | `tags` |
| --- | --- | --- | --- |
| `product/brief.md` | `Product Brief` | `stable` | — |
| `specs/*.md` | `Specification` | `stable` | — |
| `decisions/D*.md` | `Decision` | `stable` | — |
| `decisions/policy.md` | `Policy` | `stable` | — |
| `agents/worker.md` | `Agent Instructions` | `stable` | — |
| `research/*.md` | `Research Note` | `stable` | — |
| `references/nbc-help-v284.md` | `Reference` | `stable` | — |
| `tasks/TEMPLATE.md` | `Template` | `stable` | — |
| `tasks/contracts/milestone-*.md` | `Task Contract` | `deprecated` | `[milestone-<n>]` |
| `tasks/contracts/docs-okf-*.md` | `Task Contract` | `draft` | `[okf]` |
| `tasks/reviews/review-post-*.md` | `Review` | `deprecated` | `[milestone-<n>]` |
| `tasks/reviews/review-*.md` (others) | `Task Contract` | `deprecated` | `[milestone-<n>]` if the name has `m<n>`, else none |
| `tasks/proposals/*.md` | `Proposal` | `draft` | — |

`<n>` is the milestone number from the file name (`milestone-7k6-...` → `7`,
`review-m5-a1-...` → `5`, `review-post-m4` → `4`).

`title`: the text of the first heading line (`#` or `##`), without the
`#` marks. `product/brief.md` has no heading: use its first line.
`decisions/policy.md`: `"Decision record policy"`.

`description`: one sentence, at most 200 characters, taken from the file:
the first sentence of the first prose paragraph after the heading, skipping
metadata lines (lines that start with `Status:`, `Planner:`, `Date:`,
`Plan`, `Parent:`, `Implements:`, `Code:`, `Working name:`, `**Status`).
For decisions, use the first sentence after the `Decision:` line. Generate
them with the script, then check each by eye. Where the script's sentence is
empty, cut off, longer than 200 characters, or is markup, write a sentence
yourself from the heading and first paragraph. Add no fact, D-number or
path that the file does not state.

## 4. Do not change bodies

The body after the frontmatter must be byte-identical to the file before this
contract. Verify with check 2.

## 5. `docs/references/nbc-help-v284.md`

```
---
type: Reference
title: "NEBaseChanger in-app help, v2.8.4"
description: "Text of the in-app help of NEBaseChanger v2.8.4, kept as a reference for the Q5 workflow."
status: stable
resource: "/references/nbc_help_v284.txt"
---
# NEBaseChanger in-app help, v2.8.4

The help text is in [nbc_help_v284.txt](/references/nbc_help_v284.txt).
```

## 6. Index files and log

`target/tmp/okf-index.py` writes one `index.md` in each of: `docs/`,
`docs/product/`, `docs/specs/`, `docs/decisions/`, `docs/agents/`,
`docs/research/`, `docs/references/`, `docs/tasks/`,
`docs/tasks/contracts/`, `docs/tasks/reviews/`, `docs/tasks/proposals/`.

Format (OKF §8), no frontmatter, one entry per concept and subdirectory,
with relative targets:

```
# <Directory heading>

* [<title>](<file>.md) - <description>
* [<Subdirectory heading>](<subdir>/) - <subdirectory line from the table>
```

Order: decisions by number, then `policy.md`; all others by file name;
subdirectories after files.

The root `docs/index.md` is the only one with frontmatter, exactly:

```
---
okf_version: "0.2"
---
# luria documentation
```

then entries for `log.md` and each subdirectory. Headings and subdirectory
lines:

| Directory | Heading | Subdirectory line |
| --- | --- | --- |
| `product/` | Product | The product brief: purpose, scope and milestones. |
| `specs/` | Specifications | Specifications of the command line and the Q5 workflow. |
| `decisions/` | Decisions | Project decisions D001 to D062, one file each, and the policy for changing them. |
| `agents/` | Agent instructions | Instructions for the worker agent. |
| `research/` | Research notes | Investigations of external tools and model behavior. |
| `references/` | References | External material kept for reference. |
| `tasks/` | Tasks | Task contracts, reviews, proposals and the contract template. |
| `tasks/contracts/` | Task contracts | Task contracts, one per piece of implemented work. |
| `tasks/reviews/` | Reviews | Code reviews and the fix contracts that came from them. |
| `tasks/proposals/` | Proposals | Proposals and deferred work for the owner to decide. |

`docs/log.md`, exactly:

```
# Directory Update Log

## 2026-10-31
* **Update**: Reorganized `docs/` as an OKF v0.2 bundle: specifications to `specs/`, the product brief to `product/`, task files to `tasks/contracts/`, `tasks/reviews/` and `tasks/proposals/`.
* **Creation**: Split the decision record into one file per decision in `decisions/`.
* **Update**: Added frontmatter to every concept and an `index.md` to every directory.
```

## 7. Checks

`scripts/check`, plus:

```bash
# 1. Conformance: no output.
for f in $(git ls-files -co --exclude-standard 'docs/*.md'); do
  case "$(basename "$f")" in index.md|log.md) continue;; esac
  head -1 "$f" | grep -qx -- '---' || echo "no frontmatter: $f"
  awk 'NR>1 && /^---$/ {exit} NR>1' "$f" | grep -qE '^type: [^ ]' || echo "no type: $f"
done
# 2. Bodies unchanged: no output.
for f in $(git diff --name-only -- docs); do
  diff <(git show HEAD:"$f") <(awk 'NR==1 && /^---$/ {fm=1; next} fm && /^---$/ {fm=0; next} !fm' "$f") >/dev/null \
    || echo "body changed: $f"
done
# 3. Descriptions: count, and none over 200 characters.
grep -rh '^description: ' docs | awk 'length($0) > 215' | wc -l
```

## 8. Proving the change bites

| # | Change | Must fail |
| --- | --- | --- |
| 1 | Remove the frontmatter from `docs/specs/cli.md` | check 1 prints `no frontmatter: docs/specs/cli.md` |

Restore and report.

## 9. Do not

- Edit any file outside `docs/`, or any body text in `docs/`.
- Add keys not in §3 or §5 (no `generated`, `verified`, `sources`).
- If a `crates/` test fails because `docs/specs/cli.md` now starts with
  frontmatter, stop and report the test and line; do not edit the test.

## 10. Report

The standard handoff, plus: the list of files whose description you wrote by
hand (with the sentence), the output of the three checks, and the guard
experiment.
