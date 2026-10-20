# Review M6 B-4: a CDS name is grounded as a whole name (D045)

Status: **Ready for the worker** (after `review-m6-b3-numbering-veto.md` is
committed). Planner: Claude. Parent: `docs/tasks/review-post-m6.md` B-4.
Fail-closed; no owner decision needed.

Files (no others):
- `crates/cli/src/interpret/build.rs`
- `docs/decisions.md` (one bullet of D045, §2.3)

## 1. Defect

The CDS name is grounded with `stated_in` (`build.rs`), a bare substring
test. With catalogue entries `bla` and `blaB`, the text "In blaB, change
A42G, numbering from the CDS start." and a reply `"cds": "bla"` are
proposed in `bla`: primers for the wrong gene.

## 2. Required change

### 2.1 `build.rs`: a new private function

```rust
/// Whether the text names the CDS `name` as a whole name (D045): `name` is
/// not blank, and at some occurrence of it in `text` both hold:
/// - the char just before it and the char just after it are each absent,
///   or not an ASCII letter, ASCII digit, `_` or `-`;
/// - no catalogue id longer than `name` occurs in `text` at a span that
///   contains this occurrence, so `bla` in "bla.1" is not `bla` when the
///   record has a CDS `bla.1`.
fn names_cds(text: &str, name: &str, catalogue: &[CatalogueEntry]) -> bool
```

Work in chars (`Vec<char>`), never bytes, as `evidence.rs` does. Check
every occurrence (overlapping included); one that passes both rules is
enough. "A span that contains it": a longer id at chars `[a, b)` contains
the occurrence `[s, e)` when `a <= s` and `e <= b`.

### 2.2 `build.rs`: grounding uses it

`first_ungrounded` gains the catalogue as a third parameter:
`fn first_ungrounded(text: &str, extraction: &Extraction, catalogue: &[CatalogueEntry]) -> Option<Ungrounded>`.
For the `cds` field it uses `names_cds(text, cds.as_str(), catalogue)`
instead of `stated_in`. A name that fails is `Ungrounded { field: "cds",
value }`, as now. Nothing else in grounding changes. `build` passes its
`catalogue` through. Update `first_ungrounded`'s doc comment to say the
CDS name must be a whole name (D045).

### 2.3 `docs/decisions.md`

In D045, replace the bullet that starts `* **A CDS name is a whole name**
(B-4).` (two lines) with exactly:

```markdown
* **A CDS name is a whole name** (B-4). It must occur in the text with no
  letter, digit, `_` or `-` on either side, and not inside a longer CDS id
  the record has.
```

Change nothing else in `docs/decisions.md`.

## 3. Tests (write first; confirm each new one fails before the change)

In `build.rs`'s test module, existing tests unchanged. Use the existing
helpers, `profile(true, true)`, `facts()`, the mutation
`{"token":"A42G","quote":"A42G","codon":null}`, composition null,
unsupported `[]`. A helper building a `Usable { residues: 286, strand:
Strand::Plus }` entry with id and gene equal to a given name is fine.

1. `a_cds_name_inside_a_longer_word_is_ungrounded`: catalogue `bla`,
   `blaB`; text `"In blaB, change A42G, numbering from the CDS start."`;
   reply `"cds":"bla"`, numbering `"numbering from the CDS start"` →
   `Built::Ungrounded(Ungrounded { field: "cds", value: "bla" })`.
2. `a_cds_name_inside_a_longer_catalogue_id_is_ungrounded`: catalogue
   `bla`, `bla.1`; text `"In bla.1, change A42G, numbering from the CDS start."`;
   reply `"cds":"bla"` → `Ungrounded { field: "cds", value: "bla" }`.
3. `a_whole_cds_name_is_grounded`: catalogue `bla`, `blaB`; for each text
   - `"In bla, change A42G, numbering from the CDS start."`
   - `"Change A42G in bla. Numbering from the CDS start."`
   - `"In (bla) change A42G, numbering from the CDS start."`
   - `"In blaB and then bla, change A42G, numbering from the CDS start."`

   with reply `"cds":"bla"` and numbering `"numbering from the CDS start"`
   (for the second text, `"Numbering from the CDS start"`) →
   `Built::Proposed` with `request.scope == Some("bla")`.
4. `a_name_joined_by_a_hyphen_or_underscore_is_not_whole`: catalogue
   `bla`; texts `"In bla-2, change A42G, numbering from the CDS start."`
   and `"In my_bla, change A42G, numbering from the CDS start."`, reply
   `"cds":"bla"` → `Ungrounded { field: "cds", value: "bla" }` for each.

Check every text in test 3 against D045's numbering and quote rules
before running; if one is not proposed for a reason other than the CDS,
report it rather than changing the text.

Disabling proofs (once each, run, restore, report):
- `stated_in` for `cds` again → tests 1, 2 and 4 fail;
- the longer-id rule removed → test 2 fails;
- `_` and `-` dropped from the boundary rule → test 4 fails.

The corpus must keep every outcome (`ungrounded-cds` stays ungrounded,
`cds-not-found` stays `cds_not_found`). If one changes, stop and report.

## 4. Do not

- Add dependencies, or change any file not listed above.
- Change how any other string is grounded, or any question.
- Change any fixture or existing test.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, each disabling proof and its result, the check results, and any
deviation.
