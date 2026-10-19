# Milestone 6 plan: natural-language interpretation

Status: **Plan approved by the owner, 2026-10-19; every question in §4 and
§9 is answered.** In progress; see the per-contract files `milestone-6*.md`. Planner: Claude. Decision: D034 (Milestone 6 is "an LLM adapter
that turns user requests into typed operations and clarification
requests"). Branch: `feat/milestone-6`.

## 1. What this milestone is

Milestone 5 made the Q5 workflow runnable from a typed request file,
`luria.q5-request/1`. This milestone adds a second way to get that file:
the user describes the edit in ordinary language, and luria proposes the
request.

```
luria q5 interpret --template plasmid.gb --design profile.json \
    --backend openai-compatible --endpoint http://127.0.0.1:11434/v1 \
    --model qwen3 --text "Design Q5 primers for A42G and W87A in bla, as separate variants."
```

It prints one JSON document, `luria.q5-interpretation/1`, in one of three
states:
- `proposed`: it holds a complete `luria.q5-request/1` for the user to
  inspect, then run with the existing `luria q5 design`, unchanged;
- `needs_clarification`: it holds structured questions with stable codes,
  and no request;
- `failed`: it holds an error, for example a model reply that could not be
  read or that named something the user never wrote.

The milestone succeeds when (product brief, "Definition of success")
natural language produces **the same typed request**, and therefore the
same byte-identical result, as the hand-written request for the same edit,
with the scientific path unchanged.

## 2. What exists today, established 2026-10-19

These are facts about the code, not design choices:
1. `luria_cli::request::RequestDocument` and its `*Spec` types have public
   fields, so a request can be built in code. `result::request_echo`
   serializes one, and parsing the echo gives the same request back
   (`docs/cli.md` §8).
2. `luria_core::genbank::read_genbank` and
   `luria_core::genbank_cds::import_cds(record, &[])` list every feature,
   with its id (the first of `/locus_tag`, `/gene` and `/label`) and, for a
   CDS, the refusal importing it would give. Every CDS the record offers,
   and whether it can be used, can be listed without new core code.
3. A request without `scope` already lets core resolve the target and ask
   when more than one CDS could be meant (`docs/cli.md` §7).
4. `luria-cli` depends on `serde` and `serde_json` only (D041). `luria-core`
   has no dependencies.
5. Exit statuses 0 to 7 are taken (`crates/cli/src/exit.rs`).
6. The post-M5 review recorded seven M6 inputs and deferred B-8, path
   confinement and size caps (`docs/tasks/review-post-m5.md` §B). §7 says
   where each is handled.

## 3. The trust boundary

This section turns D004, D005 and D016 into rules the code must enforce.

**The model extracts, and Rust decides.** The model returns a small JSON
document (§5.2) that says what the user asked for, in the user's own
words. Rust:
1. parses it strictly: an unknown field or a wrong type is a failure, not
   a repair;
2. **grounds** it (M6-15). Every string the model returns that is not one
   of the schema's fixed values must be found in the user's text,
   verbatim: the CDS name, each mutation's evidence `quote`, the numbering
   words and each `unsupported` quote as an exact substring, and a codon
   case-insensitively. A string that is not there is
   `ungrounded_extraction` and the state is `failed`: the model cannot
   introduce a value the user did not write. The one exception is a
   mutation's `token`, which is a canonical form rather than the user's
   words, and is checked against its own quote instead (§5.3a);
3. parses the mutation notation itself (§5.3). The model copies `A42G`
   verbatim; Rust reads the residue letter, the number and the target;
4. checks each named CDS against the record's catalogue (§5.1);
5. takes everything the prose does not state from the design profile
   (§5.4), and asks when the profile does not state it either;
6. builds the `RequestDocument`, renders it with `request_echo`, and parses
   the rendered text back with `parse_request` before it reports
   `proposed`.

The model never writes a request field, a coordinate, a sequence, a codon
it was not given or a design value. Removing the model leaves a request
the user can write by hand, run through the same `luria q5 design`.

**What leaves the machine** (owner answer M6-4). The user's text, and a
summary of the record's CDSs: id, `/gene`, `/product`, length in codons,
strand and whether luria can use it. **Never the sequence, and never a
coordinate.** The interpretation document records the exact messages sent,
so the user can audit them (D023).

**Cloud use is explicit** (D023). An endpoint whose host is not a loopback
address, and every Anthropic call, needs `--allow-remote`. Without it the
command refuses before anything is sent.

## 4. Owner decisions (answered 2026-10-19)

| # | Question | Answer |
| --- | --- | --- |
| M6-1 | Which LLM backend | **Both**: a local OpenAI-compatible endpoint and the Anthropic API, behind one backend trait. |
| M6-2 | How luria reaches the model | **A new crate, `luria-nl`, with `ureq`** (blocking, rustls). |
| M6-3 | What the command produces | **Two steps**: `luria q5 interpret` proposes a request; the user inspects it and runs `luria q5 design`. |
| M6-4 | What the model sees | **The user's text and a CDS summary.** Never the sequence. |
| M6-5 | What the model emits | **A narrow extraction; Rust builds the request** (§3). |
| M6-6 | Values the prose doesn't state | **A design profile file** (§5.4). Without one, each missing value is a question. |
| M6-7 | Templates | **GenBank only.** A FASTA template is refused (`interpret_fasta_unsupported`). |
| M6-8 | Notation scope (§9 Q1) | **Residue substitutions only**: `A42G` and `Ala42Gly`, optionally `p.`-prefixed. |
| M6-9 | Unnamed CDS (§9 Q2) | **Always ask** `cds_unnamed`, whatever the record holds. |
| M6-10 | Template id (§9 Q3) | **The LOCUS name**, recorded as an assumption. |
| M6-11 | Numbering (§9 Q4) | **The profile may state** `"numbering": "from-cds-start"`. Without that or a statement in the text, the question is `numbering_unstated`. |
| M6-12 | `q5 design` codes (§9 Q5) | **Unchanged in M6.** `luria.q5-result/1` keeps its schema. |
| M6-13 | Linear fixture (§9 Q6) | **Yes**: a short, hand-written, independently verified linear GenBank record with one CDS. |
| M6-14 | Exit status (§9 Q7) | **New exit 8** for a refused model reply. |
| M6-16 | A combined construct (asked 2026-10-19) | **Proposed faithfully.** `interpret` emits the request and core refuses it (`unsupported_composition`, exit 6). D003: the adapter must not duplicate a scientific scope limit core owns. |
| M6-15 | Grounding (asked 2026-10-19, after §5.2 was first written) | **Evidence quotes**: the model returns a canonical `token` and a verbatim `quote` from the user's text, and Rust checks the quote supports the token (§5.3a). Strict verbatim tokens would refuse ordinary prose. |

These become D044 when the plan is approved.

## 5. Proposed shape

### 5.1 Crates

| Crate | Package | What it holds |
| --- | --- | --- |
| `crates/core` | `luria-core` | Unchanged. Stays dependency-free. |
| `crates/nl` | `luria-nl` (new) | Model I/O only: the backend trait, the two HTTP backends, a scripted backend for tests, the prompt, and the strict parser of the model's reply into an `Extraction`. Depends on `serde`, `serde_json` and `ureq`. No dependency on `luria-cli` and no scientific logic. |
| `crates/cli` | `luria-cli` | Gains `interpret.rs`: the catalogue, grounding, notation parsing, the profile, request building and the interpretation document. Depends on `luria-nl` (amends D041). |

**Why this direction.** If `luria-nl` built the request, it would need
`luria-cli`'s request types, and the `luria` binary could not depend on
`luria-nl` without a cycle. So `luria-nl` stops at "what the model said",
and everything deterministic about the request stays beside the request
parser it has to satisfy. `ureq` reaches the binary through `luria-nl`,
which it has to: the binary makes the call.

### 5.2 The extraction (`luria-nl`)

What the model must return, enforced by a JSON schema in the call (OpenAI
`response_format: json_schema`; Anthropic, a forced tool call) and parsed
strictly in any case:

```json
{
  "operation": "q5_residue_substitutions",
  "cds": "bla",
  "mutations": [
    { "token": "A42G", "quote": "alanine 42 to glycine", "codon": null },
    { "token": "W87A", "quote": "W87A", "codon": "GCG" }
  ],
  "composition": "separate_variants",
  "numbering": null,
  "unsupported": []
}
```

- `operation`: `q5_residue_substitutions`, or `other` when the user asked
  for something else;
- `cds`: the CDS name exactly as the user wrote it, or null;
- `mutations[].token`: the mutation in canonical one-letter form, which the
  model may write even when the user did not (`A42G`);
- `mutations[].quote`: the span of the user's own text that says so, copied
  verbatim. It is the model's evidence for the token (§5.3a);
- `mutations[].codon`: the codon the user named, copied verbatim, or null;
- `composition`: `separate_variants`, `combined_construct`, or null when
  the user did not say;
- `numbering`: the user's own words about numbering, or null;
- `unsupported`: short verbatim quotes of anything the user asked for that
  the fields above cannot hold.

The reply is capped in size (B-8). A reply that is not this document is
`model_output_invalid`.

### 5.3a Evidence quotes (M6-15)

Strict verbatim tokens would refuse ordinary prose: "change alanine 42 to
glycine" contains no `A42G`. So the model reports a canonical `token` and,
beside it, the `quote` from the user's text that supports it. Rust checks
the evidence:

1. the `quote` occurs in the user's text as an exact substring. If it does
   not, the model fabricated it: `ungrounded_extraction`, `failed`;
2. the `token` parses as a residue substitution (§5.3), giving `from`,
   `number` and `to`;
3. each of those three is **visible in the quote**:
   - the `number` as a digit run equal to it, bounded by non-digits, so
     `42` is not found inside `420`;
   - `from` and `to` each as the one-letter code (a standalone capital
     letter), the three-letter code, or the full English residue name, the
     last two case-insensitively, each bounded by non-letters.

A quote that occurs in the text but does not show one of the three is not
treated as fabrication, because the user may simply not have said it.
Luria asks instead, naming what is missing: `residue_number_unstated`,
`original_residue_unstated` or `target_residue_unstated`. It never asserts
an expected original residue the user did not state (D011), and never takes
a coordinate from the model (D004).

**`?` for an unstated part** (planner clarification, 2026-10-19, from the
local-model research in `docs/research/m6-local-model-extraction-2026-10-19.md`).
The prompt tells the model to write `?` for a part the user did not state:
`A?G` for "change alanine to glycine". The notation parser recognises `?`
in any of the three places and reports which parts are unstated, so the
question is the specific `*_unstated` code above rather than
`mutation_notation_unsupported`. A `?` never becomes a value: it only
chooses which question is asked. The IQ1_M local model used it unprompted
by example, on the first try.

The residue-name table (`A`/`Ala`/`alanine`, and so on, 20 residues) lives
in `luria-cli` beside the notation parser, and is tested against
`luria_core::genetic_code`'s own residue letters so the two cannot drift.

### 5.3 Mutation notation (Rust, `luria-cli`)

M6 accepts residue substitutions only, in two notations (§9 Q1):
- one letter: `A42G`, optionally prefixed `p.`;
- three letters: `Ala42Gly`, optionally prefixed `p.`.

Each token becomes a `residue` edit: `from`, `number` and `to` from the
token, `cds` the resolved CDS, and the member `name` the token as written.
A stop target (`*`, `Ter`), a deletion, an insertion, a DNA-level edit
(`c.123A>G`) or any other form is `mutation_notation_unsupported`: a
question naming the token, not a guess.

### 5.4 The design profile

`luria.q5-design-profile/1`, a small JSON file the user keeps:

```json
{
  "schema": "luria.q5-design-profile/1",
  "design": {
    "policy": "luriaNative",
    "style": "q5_polymerase",
    "minAnnealingOverlap": 15,
    "placement": "inline",
    "buffer": "q5_reaction"
  },
  "codon": "default",
  "numbering": "from-cds-start"
}
```

- `design` has exactly the shape of the request's `design` and is parsed
  by the same code, made public in `request.rs`.
- `codon` is the codon policy for a mutation whose text names no codon.
  The only value is `"default"`. When it is absent, such a mutation is
  `codon_unstated`.
- `numbering` is the numbering origin when the text states none. The only
  value is `"from-cds-start"` (M6-11). When both are silent, the question is
  `numbering_unstated`.
- `design`, `codon` and `numbering` are each optional. Every absent value
  that the request needs is a question, never a default.
- The interpretation document records, for each request value, whether it
  came from the user's text or from the profile.

There is no built-in profile. D028 stands: luria does not default design
values; the user's own file does.

### 5.5 Building the request

- **Template**: `genbank` is the `--template` path, written relative to the
  directory the request will be saved in (§5.7). `id` is the LOCUS name,
  recorded as an assumption (M6-10). `sha256` is always filled, so the
  request pins the exact sequence it was proposed against.
- **CDS**: one named CDS that is in the catalogue and usable is imported and
  becomes `scope`. A name that matches nothing is `cds_not_found`. A name
  matching a CDS that luria cannot use is `cds_unusable`, with that CDS's
  refusal code. When the text names no CDS, the question is `cds_unnamed`,
whatever the record holds (M6-9).
- **Member names**: a member's `name` is the canonical token (`A42G`), not
  the user's words, because a request name is an identifier with no
  whitespace (review input 4).
- **Operation**: one mutation is `edit`, and two or more are `variants`
  (review input 7). With two or more and no stated composition, the
  question is `composition_unstated` (product brief: separate and combined
  are never assumed).
- **Numbering**: the text's own statement wins, and the profile is used when
  the text says nothing. Neither: `numbering_unstated`. A statement that
  does not mean "from the start of the CDS" is `numbering_unsupported`.
  Whether a statement means that is decided by Rust, from a fixed list of
  phrasings in contract 6e, not by the model; anything off the list is
  `numbering_unsupported`.
- `operation: other`, or a non-empty `unsupported`, is
  `operation_unsupported`, quoting the user.

`interpret` does not check residues against the translation. That is
`luria q5 design`'s job, and it already asks or fails with stable codes.
`interpret` does read the template and import the CDS, so a proposal never
names a file or CDS that `design` would refuse.

### 5.6 Backends (`luria-nl`)

```rust
pub trait Backend {
    fn complete(&self, prompt: &Prompt) -> Result<Reply, BackendError>;
    fn describe(&self) -> BackendInfo; // kind, endpoint host, model; never a key
}
```

- **`openai-compatible`**: `POST {endpoint}/chat/completions`. It needs
  `--endpoint` and `--model`, with no defaults. It sends an optional bearer
  key from `LURIA_OPENAI_API_KEY`.
- **`anthropic`**: `POST https://api.anthropic.com/v1/messages`, with
  `x-api-key` from `ANTHROPIC_API_KEY` and a pinned `anthropic-version`.
  It needs `--model` and `--allow-remote`.
- **`scripted`** (tests only, not selectable from the command line): it
  returns canned replies and records the prompts it was given.
- Temperature 0 everywhere. A seed is sent when the backend accepts one.
- HTTP backends are tested against a local `std::net::TcpListener` stub in
  the test itself. No test uses the network.

The prompt is a versioned constant (`PROMPT_VERSION`), built
deterministically from the text and the catalogue.

### 5.7 The command and its document

```
luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
    [--design PROFILE.json] --backend KIND [--endpoint URL] --model NAME
    [--allow-remote] [--write-request FILE.json]
```

- Prints one `luria.q5-interpretation/1` document on stdout, in every
  state, as `q5 design` does.
- `--write-request FILE` also writes the bare request when the state is
  `proposed`. The file must be in the template's directory, because request
  paths are relative to the request (review input 3). An existing file is
  never overwritten, and `write.rs`'s no-follow, create-new rule is reused.
- The document holds `schema`, `state`, `request` (or null), `questions`
  (each with a stable `code`, a `subject` and a `message`; review input 1),
  `origins` (text or profile, per value), `assumptions`, `error`, and
  `interpretation`: backend kind, endpoint host, model, `PROMPT_VERSION`,
  the exact messages sent, the raw reply, and the parsed extraction. That
  record is kept apart from scientific provenance (D021).
- Determinism: for the same reply, the document is byte-identical. The
  model's own variability is recorded in the raw reply, not hidden.
- Size caps (B-8): text 8 KiB, profile 64 KiB, template 32 MiB, reply
  64 KiB. Values to be confirmed in contract 6h.
- Exit codes: 0 proposed, 3 needs clarification, 4 / 5 / 6 by category as
  today, 1 for a network or I/O failure, and the new **8** for a model reply
  that was refused (`model_output_invalid`, `ungrounded_extraction`).

## 6. Contracts

Each is small, with tests first, as in M5:

| # | Contract | Content |
| --- | --- | --- |
| 6a | `milestone-6a-nl-crate.md` | `crates/nl` skeleton, workspace member, `Extraction` and its strict parser. No network, no `ureq` yet. |
| 6b | `milestone-6b-notation.md` | `interpret::notation`: the two notations of §5.3, with every refusal. |
| 6c | `milestone-6c-catalogue.md` | The CDS catalogue from a GenBank record (§3, what is sent). |
| 6d | `milestone-6d-profile.md` | The profile schema; `request.rs`'s design parser made public, unchanged in behavior. |
| 6e | `milestone-6e-build.md` | Grounding and request building: `Extraction` + catalogue + profile → `RequestDocument` or questions. A pure function, where most of the tests sit. |
| 6f | `milestone-6f-prompt-backend.md` | `Prompt`, `PROMPT_VERSION`, the `Backend` trait and the scripted backend. |
| 6g1 | `milestone-6g1-openai.md` | The OpenAI-compatible backend, with `ureq`, against a local stub. |
| 6g2 | `milestone-6g2-anthropic.md` | The Anthropic backend, against a local stub. |
| 6h | `milestone-6h-command.md` | Arguments, the remote gate, the document, `--write-request`, exit 8. |
| 6i0 | `milestone-6i0-linear-genbank.md` | The linear GenBank fixture and its verification (planner), and `genbank_fixtures.rs` tests reading it (worker). No production code. |
| 6i | `milestone-6i-end-to-end.md` | The NL corpus (§8) through the scripted backend: interpret, then design, compared byte for byte with the hand-written request's result. |
| 6j | `milestone-6j-docs.md` | `docs/cli.md` section, README, D044. |

## 7. The review inputs, and where each goes

| Input (`review-post-m5.md`) | Handled |
| --- | --- |
| 1. Clarifications are prose; candidates are bare ids | Interpretation questions get stable codes (6e). `q5 design`'s own clarifications are unchanged in M6 (M6-12). |
| 2. Domain errors point at the edit object | Not needed by `interpret`, which builds the request itself. Unchanged (M6-12). |
| 3. No stdin, no absolute paths | `--write-request` must sit in the template's directory (6h). |
| 4. Case-sensitive residues, names without whitespace | Notation parser normalizes residue letters; member names are the token (6b, 6e). |
| 5. BOM, `-0`, exponent integers | Not reachable: Rust writes the request. |
| 6. Code-mapping oddities | Catalogue and building map them to `cds_unusable` with the code (6c, 6e). |
| 7. `edit` against a one-member `variants` | One mutation is `edit` (6e). |
| B-8. Paths and size caps | The model writes no path; caps in 6h. |

## 8. Fixtures and verification

- **NL corpus** (`tests/fixtures/nl/`): **written, 2026-10-19.** 24 cases,
  indexed by `cases.tsv`: 5 proposed, 15 needs-clarification, 4 failed.
  Each is a prose text, the model reply it is paired with, a profile, the
  expected state and codes, and for a `proposed` case the expected request,
  **written by hand from the prose and the record**, not by running
  `interpret`. `tests/fixtures/nl/README.md` records how every expectation
  was derived and what is compared how. It includes
  negative cases: a hallucinated token, a CDS the record lacks, unstated
  composition, a deletion, three-letter notation, and a reply with extra
  fields.
- **Templates**: the existing GenBank fixtures are all circular, so every
  design from them ends at exit 7 (D041 G10). The end-to-end claim (same
  request, so same result) holds regardless. A new, short, linear GenBank
  record with one designable CDS (M6-13), written by hand and verified
  independently, shows a fully designed result through natural language.
  It gets its own contract, 6i0, before 6i.
- **Live models**: an opt-in `#[ignore]` test, run by hand against a
  configured backend, reports how many corpus cases a real model gets
  right. It is never part of the required checks, and its score is
  development information, not a correctness claim.

## 9. Questions put to the owner (answered 2026-10-19 as M6-8 to M6-14)

1. **Notation scope.** Residue substitutions only, in `A42G` and `Ala42Gly`
   (§5.3)? Or also stops, deletions such as `K12del`, or DNA notation?
2. **Unnamed CDS.** When the text names no CDS: (a) always ask
   `cds_unnamed`; (b) use the record's only CDS when it has exactly one,
   recorded as an assumption, and ask otherwise; or (c) import every usable
   CDS with no `scope` and let `q5 design` ask, as core does today.
   Planner recommends (a): D005 lists "multiple possible target CDSs", and a
   record whose other CDSs are unusable still offers more than one.
3. **Template id.** Use the LOCUS name as `template.id`, recorded as an
   assumption, or require `--template-id`?
4. **Numbering.** D005 lists "unspecified numbering origin" as a reason to
   ask. When the text says nothing, should the profile be allowed to state
   `"numbering": "from-cds-start"`, or must every request say it?
5. **`q5 design` clarification codes.** Leave `q5 design`'s prose
   clarifications as they are in M6 (recommended), or give them stable
   codes here too?
6. **Linear GenBank fixture.** Add one, written by hand and independently
   verified, so the corpus shows a designed result? D041 G9 rejected a
   generated one, so it would have to be short.
7. **Exit 8.** A new status for a refused model reply, or reuse 4?
