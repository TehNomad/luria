---
type: Review
title: "Post-Milestone-6 code review: findings"
description: "After Phase 2 (f6e67c8): cargo fmt --check clean, 905 tests pass and 1 is ignored (the opt-in live test), cargo clippy -D warnings clean, python -I tests/fixtures/verify.py 13 of 13 pass."
status: deprecated
tags: [milestone-6]
---
# Post-Milestone-6 code review: findings

Status: Phase 1 (read-only review) and Phase 2 (fixes) complete,
2026-10-21. All eleven contracts are done; items recorded with no change
are listed under each section. Branch: `feat/milestone-6`; the review
started at `86ea936`. Planner: Claude.

After Phase 2 (`f6e67c8`): `cargo fmt --check` clean, 905 tests pass and 1
is ignored (the opt-in live test), `cargo clippy -D warnings` clean,
`python -I tests/fixtures/verify.py` 13 of 13 pass. `cargo test` passes
under the nono sandbox with `TMPDIR=$PWD/target/tmp`. Reviewer B's
eleven attack replies, re-run against the final tree, each end in a
question or `ungrounded_extraction`; none is proposed. Prompt v3 was
measured live on the local model: corpus 18/24 as scripted (the six
expected differences), stress 14/14, nothing unsafe.

Baseline at review start (`86ea936`): `cargo fmt --check` clean, `cargo
clippy -D warnings` clean, `git diff --check` clean, `python -I
tests/fixtures/verify.py` 13 of 13 pass. `cargo test --workspace`: 856 pass,
1 ignored (the opt-in live-model test), 1 fails under the nono sandbox only
(C-8).

Method: three read-only reviewers, each on its own `git archive` copy of
`86ea936` under `target/tmp/review-m6-*`:
- (A) the `luria-nl` crate: reply parsing, prompt, endpoint, HTTP backend;
- (B) the trust boundary in `luria-cli`: notation, evidence, catalogue,
  profile, build;
- (C) the command, its document, `--write-request`, the end-to-end tests,
  the fixtures and the docs.

**Confirmed** items were reproduced by a scratch test or run, or verified
by reading. The planner re-ran B's scratch tests against unmodified `HEAD`
sources: every High item reaches `proposed`, and B-1's request designs to
`state: verified` (exit 0).

**Merge recommendation: do not merge before B-1 to B-4 are fixed.** Each
lets a model reply turn into a proposal for an edit the user did not ask
for, which is the claim Milestone 6 exists to make (plan §3: "the model
cannot introduce a value the user did not write").

---

## B. The trust boundary (`crates/cli/src/interpret/`)

- **B-1, High, confirmed.** A quote may support a token assembled from two
  statements. `evidence` (`evidence.rs:209`) asks only that the number, the
  original and the target each appear somewhere in the quote, original
  before target. Text "make A37G and A42S as separate variants", reply
  `{token: "A37S", quote: "A37G and A42S"}` → `proposed` `A37S`; synA's
  residue 37 is Ala, so `q5 design` verifies it.
- **B-2, High, confirmed.** The residue number may come from any digit run
  (`number_visible`, `evidence.rs:122`): a CDS name (`ydj42`, "change the
  alanine to a glycine" → `A42G`) or "see figure 42". The user stated no
  residue number; `residue_number_unstated` should have been asked.
- **B-3, High, confirmed.** `NUMBERING_VETO` is applied to the model's
  quote, not to the user's statement (`evidence.rs:279`, `build.rs:601`).
  Text "Do not number residues from the start of the CDS; use UniProt
  numbering instead", reply `numbering: "from the start of the CDS"` →
  `proposed`, numbered against the user's instruction.
- **B-4, High, confirmed.** A CDS name is grounded as a bare substring
  (`stated_in`, `build.rs:189`). Catalogue `bla` and `blaB`, text "In blaB,
  change A42G", reply `cds: "bla"` → `proposed` in `bla`.
- **B-5, Medium, confirmed.** `composition` is a fixed value and is never
  grounded. With no composition in the text, a reply claiming
  `separate_variants` is proposed and designs. Plan §5.5 and the product
  brief say separate and combined are never assumed; today that rests on
  the model volunteering `null`. (`combined_construct` is caught by core,
  M6-16.)
- **B-6, Medium, confirmed.** Nothing checks the reply accounts for the
  text. Text asks for "A42G and W87A as separate variants", reply lists
  only `A42G` → `proposed` `edit` `A42G`.
- **B-7, Low–Medium, confirmed.** A codon is grounded anywhere in the
  text, case-insensitively, as a substring (`stated_codon_in`,
  `build.rs:196`): `GGC` inside "ggcx", or two Gly codons swapped between
  two mutations, both pass. Core checks only that the codon encodes the
  target.
- **B-8, Low, confirmed.** Two identical mutations are proposed as a
  two-member `variants`; `q5 design` then refuses it (exit 4,
  `duplicate_variant_name`). The user is handed a dead proposal at exit 0.
- **B-9, Low, test gap, proved by disabling.** 7 of 18 `NUMBERING_VETO`
  words (`never`, `no`, `instead`, `rather`, `precursor`, `construct`,
  `vector`) and 2 of 9 `ORIGIN_PHRASES` can be deleted with every test
  green.
- **B-10, Low, test gap.** `interpret_build.rs:95-104` sorts and dedups the
  codes before comparing, so question order and repeats are unseen; no
  corpus case covers `substitution_direction_unclear`.

Recorded, no change: no reachable panic (char-indexed throughout, 15
hostile inputs); full-width and non-ASCII digits never count; 8 KiB text
with a 64 KiB reply builds in 0.4 s (debug); notation matches 6b (37
refusals pinned; residue table pinned both ways against core); catalogue
never holds a sequence or coordinate and its usability is core's,
ambiguous names included; codon-vs-residue checking is core's; the profile
shares the request's paths and messages and defaults nothing (D028); the
echo/parse-back really runs (`run.rs:607-620`).

## A. The `luria-nl` crate

- **A-1, Medium, confirmed.** `parse_extraction` (`extraction.rs:120`)
  accepts a JSON array in place of an object, at the top and for each
  mutation: serde's derived `visit_seq` skips field names and
  `deny_unknown_fields`. Breaks plan §3 step 1 ("a wrong type is a failure,
  not a repair"). Grounding still checks every string.
- **A-2, Medium, test gap, proved by disabling.** `.proxy(None)`
  (`openai.rs:71`) is what stops `HTTP_PROXY`/`ALL_PROXY` from sending the
  prompt and key to a host the loopback gate never checked. Removing it
  leaves every test green.
- **A-3, Low, confirmed.** `ExtractionError.message` echoes model strings
  with newlines, breaking its own one-line rule (`extraction.rs:33`).
- **A-4, Low, confirmed.** A 200 body that is not UTF-8 is
  `backend_unreachable` (exit 1, "network"); it should be
  `backend_response`.
- **A-5, Low, confirmed.** `Endpoint::parse` accepts path characters
  `http::Uri` rejects (`{ } | ^ \ " < >`), so the error surfaces at send time
  as `backend_unreachable` rather than as a usage error. Fails safe.
- **A-6, Low, by reading.** `localhost` passes the gate by name, then is
  resolved by the system resolver; the module doc (`endpoint.rs:4-7`) says
  no name is resolved. Fix the doc or connect to `127.0.0.1`/`[::1]`.
- **A-7, Low, test gap.** Adversarial hosts that behave correctly today
  have no regression test: `127.0.0.1.evil.com`, `localhost.`,
  `[::ffff:127.0.0.1]`, `127.0.0.1\@evil.com`, `evil.com@127.0.0.1`,
  `2130706433`, `0177.0.0.1`, `0x7f.0.0.1`, `[fe80::1%25eth0]`.
- **A-8, Low, test gap.** The response-cap test would pass if the body were
  read whole and checked afterwards; an endless chunked body would show
  the difference.

Recorded, no change: 300,000 random endpoints, 0 host mismatches between
`Endpoint::host()` and `http::Uri`; userinfo, percent-encoding, zones and
Unicode hosts refused; `https://` refused in `parse`; redirects not
followed; the key is never in `describe`, the URL, an error or the
document; the 600 s timeout covers the whole call; the response cap stops
an endless body while reading; the prompt is deterministic and has no
field for a sequence or coordinate; no `ring` or `rustls` in the tree.
Prompt injection through `/product` (U+2028, forged list lines) is a low
risk because usability is re-checked in Rust.

## C. The command, fixtures and docs

- **C-1, Low, confirmed.** `--write-request new.json/` passes step 4
  (`run.rs:503-531`), the model is called, then the write fails (exit 1).
  Breaks "nothing is sent until every local input is valid" (6h1, [docs/specs/cli.md](/specs/cli.md)
  §10.7).
- **C-2, Low, suspected.** Step 4 treats any `symlink_metadata` error as
  "free" (`run.rs:528`); `write.rs:132-140` treats only `NotFound` so.
- **C-3, Low, confirmed.** `--template ..` is reported as "the template
  file's name is not UTF-8" (`run.rs:401-410`).
- **C-4, Medium, docs.** [docs/specs/cli.md](/specs/cli.md) §5's table says exit 0 is
  "`verified`" and exit 1 is output failures; §10.7 says interpret's codes
  are "those of §5, plus 8". Nothing says interpret's 0 is `proposed`, 3 is
  `needs_clarification`, 1 includes backend failures, or that a failed
  `--write-request` still prints a `proposed` document and exits 1.
- **C-5, Low, docs.** §10.7's `model_output_invalid` row says "over 64
  KiB"; a response body over 1 MiB is `backend_response`, exit 1.
- **C-6, Low, test gap, proved by disabling.** The text-file and profile
  caps can be raised to the template cap with every test green; non-UTF-8
  text, profile and template, and the write-failure path, are untested.
- **C-8, Medium, environment.** `run.rs:662`
  `a_write_never_replaces_a_file` uses `std::env::temp_dir()`, which is
  write-only under nono. Fix: build the directory from
  `env!("CARGO_MANIFEST_DIR")` + `../../target/tmp`, as AGENTS.md directs
  (reviewer C verified this passes). C also reports that relinking a test
  binary under nono needs `TMPDIR=$PWD/target/tmp`.
- **C-9, Low, fixture note.** `cases.tsv` row `two-combined` says "a
  combined construct is never assumed"; the case is M6-16 (stated,
  proposed, refused by core).

Recorded, no change: caps are enforced while reading (`take(cap + 1)`); the
same-directory check canonicalizes both parents, so `..` and symlinked
directories resolve as `open` does; the gate runs before `interpret`; the
§10.3 example and its `q5 design` both exit 0; the linear record verified
afresh without Biopython (393 bp, CDS `[42,345)`, translation, residues 1,
42, 87, 100); every corpus claim holds; the end-to-end test compares whole
`q5 design` documents against the tracked hand-written requests; D044,
README and the product brief match. **C-7**: `backend.endpoint` records
the full URL where plan §5.6 says "host"; 6g1 specified `endpoint.url()`,
so the code follows its contract. **Origins** list design, numbering and
codons only; 6e2 fixed that list. Both: plan wording, no change.

---

## Owner decisions, 2026-10-20

| # | Question | Answer |
| --- | --- | --- |
| B-1, B-2 | How strict a mutation's quote is | **One number, in order.** The quote holds exactly one maximal ASCII digit run; some mention of the original ends at or before its start and some mention of the target starts at or after its end. Otherwise ask; a quote with more than one digit run gets a new question code. All 24 corpus quotes already satisfy it. |
| B-3 | Where the numbering veto applies | **By word kind.** Negation and contrast words in the sentence that holds the numbering quote (split on `.` `;` `!` `?` and newline); other-origin words anywhere in the text. *Corrected the same day* (the planner's list had lumped in common nouns): numbering-scheme words (`mature`, `precursor`, `signal`, `uniprot`, `pdb`, `structure`) anywhere; `plasmid`, `construct`, `vector`, `tag` only in the numbering words' sentence, so "in a single construct" (`two-combined`) is unaffected. |
| B-5 | How composition is decided | **Rust phrase list; the model must agree.** Rust finds a fixed composition phrase in the text, as for numbering. None → `composition_unstated`; disagreement with the model's value → ask. No schema or prompt change. |
| B-7 | How a codon is grounded | **In its own mutation's quote, as a whole word** (bounded by non-letters). The prompt says so (version bump); the `codon-stated` reply's quote becomes `W87 to alanine using the codon GCG`, which the owner approves. |
| B-6 | A dropped mutation | **Scan for notation tokens.** Ask (new code) when the text holds a bounded `A42G`, `Ala42Gly` or `p.`-prefixed token that no mutation's quote covers. A dropped prose mutation stays the user's review; the docs say so. |
| B-8 | Duplicate mutations | **Ask in interpret**, one question naming the repeated token. A request-validity rule, not a scientific limit (D003). |
| A (question) | An API key over plain HTTP | **Refuse a key with a non-loopback endpoint** before anything is sent, until TLS exists (M6-18). Keyless remote use with `--allow-remote` still works. |
| C-8 | `TMPDIR` in AGENTS.md | The owner edits AGENTS.md; the planner drafts the line. |

B-4 (CDS name bounded by characters that cannot be part of a name) is
fail-closed and needs no decision. These answers, with B-4, become D045
in the first fix contract.

## Phase 2: fix contracts

Tests first in each; every new rule names the test that fails with the
rule disabled. Run in this order, one commit each.

| Contract | Items |
| --- | --- |
| `review-m6-b1-quote-locality.md` | B-1, B-2, D045 |
| `review-m6-b3-numbering-veto.md` | B-3, B-9 |
| `review-m6-b4-cds-name.md` | B-4 |
| `review-m6-b5-composition.md` | B-5 |
| `review-m6-b7-codon.md` | B-7, prompt v3, the `codon-stated` reply; the planner measures v3 live before the commit |
| `review-m6-b6-coverage.md` | B-6, B-8 |
| `review-m6-a1-reply-shape.md` | A-1, A-3 |
| `review-m6-a2-backend.md` | A-2, A-4, A-5, A-6, A-7, A-8 |
| `review-m6-a3-remote-key.md` | the key-over-HTTP refusal |
| `review-m6-c1-command.md` | C-1, C-2, C-3, C-6, C-8 |
| `review-m6-c4-docs.md` | C-4, C-5, C-9 |

Recorded, no change: **B-10.** `tests/fixtures/nl/README.md` deliberately
compares `codes` as a set ("the plan fixes no order for questions, so the
fixture must not pin one"); question order is pinned by `build`'s own
`every_unstated_value_is_asked_about_at_once_in_order`, and the direction
question by `a_swapped_token_is_asked_about_not_proposed`.
