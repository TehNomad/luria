# Milestone 7k2: building a request from the form's values

Status: **Ready for the worker, 2026-10-21.** Planner: Claude. Parent:
`docs/tasks/milestone-7k-new.md` §2 and §3 (owner answers M7-32 to M7-39).
The specification is `tests/fixtures/new/reference/requests.py` (read its
docstring and `build`/`member`); its outputs under `tests/fixtures/new/`
(`request-*.json`, `paste-*.fa`, `refusals.tsv`) are the expected results,
written independently of `crates/` and each run through `luria q5 design`
by the planner.

## 1. What and why

`luria new`'s form collects a template, mutation lines, a codon choice and
an overlap. This contract turns those values into a request document, or a
precise refusal, as a pure function with no terminal. 7k3 builds the form
around it; 7k5 writes the files.

## 2. The module `crates/cli/src/new.rs` (`lib.rs` gains `pub mod new;`)

```rust
/// Where the template comes from.
pub enum TemplateInput {
    /// A FASTA file already read: its file name (no directory) and text.
    Fasta { file_name: String, text: String, name: String,
            topology: TopologySpec, cds: CdsChoice },
    /// A pasted sequence: raw bases or FASTA text, the name to give it, and
    /// the file name it will be saved as.
    Paste { text: String, name: String, save_as: String,
            topology: TopologySpec, cds: CdsChoice },
    /// A GenBank file already read, and the CDS id the user picked.
    GenBank { file_name: String, text: String, cds: String },
}

pub enum CdsChoice {
    Whole(StrandSpec),
    /// 1-based inclusive bases, as the user typed them.
    Bases { first: i64, last: i64, strand: StrandSpec },
}

pub enum CodonChoice { DefaultPolicy, TypedPerMutation }

pub struct Form {
    pub template: TemplateInput,
    /// The mutation lines as typed; blank lines are skipped.
    pub lines: Vec<String>,
    pub codon: CodonChoice,
    /// `None` until the user or a `--design` profile gives it.
    pub overlap: Option<u64>,
}

pub struct Built {
    pub request: RequestDocument,
    /// For a paste: the file name and the FASTA text to write.
    pub paste: Option<(String, String)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildRefusal {
    NoMutations, LineRefused, FromMismatch, OutsideTemplate,
    InsertionAtEnd, MissingCodon, MissingOverlap, DuplicateName,
    /// Core refused the template's text (`read_fasta`, `read_genbank`,
    /// `normalize` or the name's `Identifier`). Added 2026-10-21 on w7k2's
    /// question: `build` never panics on what a user typed.
    TemplateUnreadable,
}
impl BuildRefusal { pub fn code(self) -> &'static str; pub fn message(self) -> &'static str; }

/// The refusal, and the 1-based number of the mutation line it is about,
/// when it is about one.
pub fn build(form: &Form) -> Result<Built, (BuildRefusal, Option<usize>)>;
```

`TopologySpec`, `StrandSpec` and `RequestDocument` are `request.rs`'s. The
refusal codes are `refusals.tsv`'s.

Rules, as `requests.py` applies them, in its order. Use luria's own code for
everything it already does:

- The template's bases and digest: `luria_core::fasta::read_fasta` for
  FASTA text (a paste of raw bases is read as one record named by `name`),
  `luria_core::genbank` for GenBank (read how `prepare.rs` does both; call,
  do not copy), and `Sequence::content_hash()` for `sha256`.
- Each line: `crate::shorthand::parse_line`, with `LineRefused` and its line
  for any refusal.
- The paste's FASTA text: `luria_core::fasta::render_fasta`.
- `TopologySpec` to core's `SequenceTopology`: `prepare.rs`'s
  `topology_of` is private; write the same two-arm `match` in `new.rs`
  (approved: no domain logic).
- The request is built as a `RequestDocument` value. A test then renders it
  with `result::request_echo` and compares it as JSON with the fixture; and
  `request::parse_request` of that rendering must give the same document
  back.

## 3. Tests: `crates/cli/tests/new_build.rs` (new)

The forms are the `BUILT` and `REFUSED` lists of `requests.py`, written out
in Rust with the same values (they are inputs, not expectations). Templates
are read from `tests/fixtures/` at test time.

1. One test per `BUILT` case, `builds_<case>`: the rendered request equals
   `request-<case>.json` as a `serde_json::Value`; a paste case's
   `Built.paste` equals `("syn.fa" or "syn-minus.fa", paste-<case>.fa's
   text)`.
2. `every_refusal_matches`: each `REFUSED` form gives the code and line of
   its `refusals.tsv` row.
3. `a_built_request_round_trips`: for every `BUILT` case,
   `parse_request(render(request))` equals the request.

### 3.1 Rules that must be proven

Disable each once, run, restore; report the failing tests.

1. Reading `from` for a deletion from the template (write the typed text,
   or nothing): `builds_sid1_set` fails.
2. The 1-based CDS bases (`first` instead of `first - 1`):
   `builds_syn_paste` fails.
3. The check of a typed `from` against the template: `wrong-from` in
   `every_refusal_matches` fails.

## 4. Do not

- Change only `crates/cli/src/lib.rs` (one line), the new
  `crates/cli/src/new.rs` and `crates/cli/tests/new_build.rs`.
- No file is written by `build`. No change to luria-core, `request.rs`,
  `prepare.rs`, `shorthand.rs`, fixtures, docs, Cargo files or `vendor/`.
  If a function you need is private elsewhere, stop and report rather than
  copying it.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proofs, and anything that did not match.
