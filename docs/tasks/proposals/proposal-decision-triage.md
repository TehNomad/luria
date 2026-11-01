# Proposal: triage of the decisions and working rules

Status: **Draft for the owner, 2026-10-23. Not approved and not a task
contract.** Nothing here changes `docs/decisions/` or `AGENTS.md` until
the owner approves it item by item. Companion to
`proposal-product-direction.md`.

## 1. The test

Every rule falls into one of three groups:

- **Keep: scientific invariants.** Breaking one could give a biologist
  wrong DNA, a wrong residue, or a false claim about a result. These stay
  strict and keep fail-closed behaviour.
- **Soften: refusals that could be defaults.** The rule refuses where a
  visible, overridable default would almost always be right, and where a
  wrong default would be visible on the sequence view before anything is
  ordered. Replace the refusal with: use the default, show it, record it
  as an assumption, and let the user change it.
- **Relax: process rules.** These govern how luria is built, not what it
  does. Keep them strict for the scientific core and lighten them for
  presentation work.

The rewording of D005 in section 3 is the general rule. The other items
apply it.

## 2. Keep: scientific invariants

| Decision | Why it stays |
| --- | --- |
| D004, D016 | The model never writes sequence, coordinates, primers or results. |
| D006, D009, D036, D040 | One coordinate convention internally, explicit strand and origin handling. Invisible to users. |
| D007, D021 | Sequence identity and provenance. Kept, but moved out of the user's way (details screen, files). |
| D008 | Normalization never repairs or invents sequence. |
| D010 (core) | Standard code, residue 1 is the first codon, the stop is not a residue. |
| D011 (core) | The named original residue must match the translation; separate and combined variants are different operations. |
| D012, D013, D037 | Deterministic, independently verified edits; premature stops warned. |
| D014, D028–D033, D038, D043 | Q5 primer geometry, Tm and buffer limits. Sourced and tested; any change needs sources, as now. |
| D018, D039 | Fixtures for **scientific** behaviour are independently verified. |
| D020 | `verified` is never `experimentally validated`. |
| D023 | Nothing private leaves the machine by default. |
| D042's warning | luria must not mistranslate a non-ATG start (see section 3 for the rest of D042). |

## 3. Soften: refusals that could be defaults

### D005, reworded

> luria asks or refuses when two or more readings would put the edit on a
> different base or residue, or when the input is broken. Where one reading
> is the convention a biologist would assume, luria uses it, shows it on
> screen, records it as an assumption, and lets the user change it. It
> never takes a default silently.

Ambiguity that still blocks: two CDSs could contain the residue, the
original residue doesn't match, the typed bases don't match the
template, or the sequence has unreadable symbols.

### Items

| # | Today | Proposed | Risk if wrong |
| --- | --- | --- | --- |
| T1 | **D035, D041 (G10):** a circular template is refused for primer design, so a plasmid file can't be designed. | Design on the sequence around the edit. For an edit whose primers don't cross the origin, the geometry is exactly the linear geometry; no new source is needed. Refuse only when a primer would cross the origin, and say so. This matches how NEBaseChanger is used with plasmids today: the user pastes a linear stretch. **Owner, 2026-10-23: refuse across the origin; no rotation.** The same refusal covers a linear template whose end is too close to the edit. The message names the end, says how many bases short it is, and asks the user to add the adjoining plasmid sequence if they have it. | None for edits away from the origin. Origin-crossing edits stay refused. |
| T2 | **D010, D041:** a FASTA file or a paste needs the user to type the CDS bounds and strand. | Find the ORFs (both strands, ATG to stop, above a minimum length) and offer them, longest first, in the view. The user picks one; nothing is designed until one is picked. | Low. The pick is visible, and the translation is shown on screen. |
| T3 | **D042:** a GenBank CDS with a GTG, TTG or CTG start is refused. | Accept it and number codon 1 as Met (what the cell does), with a visible assumption. | Low. A user who names the original residue as V or L at position 1 gets a mismatch. |
| T4 | **D028, D044:** design policy, numbering origin and codon policy must be stated in every request or a profile, or the run asks. | Defaults for the interactive path: `luriaNative`, numbering from the CDS start, the existing default codon policy, overlap 15, buffer `q5_reaction`. All shown on screen and in the request file. The request **schema** stays explicit, so scripts and saved files are unchanged. | None to the science. The values are written into every request and result. |
| T5 | **D041, D044, D047:** words need a GenBank template; a CDS the words don't name is always asked about. | Moot once the view exists. Freeze natural language at what it does now (`proposal-product-direction.md` §3). | None. |
| T6 | **D041 (G8), D047 (M7-49):** output files are never replaced, or only after a prompt. | Keep the prompt. Drop the never-replace rule for `--write-edited` and `--write-request` in favour of the same prompt. | Low. |

### Not softened, deliberately

- **Combined mutants** (D011) stay a separate, refused operation until a
  geometry for two edits exists. That's a scope question, not a default.
- **Codon choice** stays shown. The default policy is fine; hiding which
  codon was written is not.

## 4. Relax: process rules

These are in `AGENTS.md` and D024–D027. They were right for building the
scientific core. They're too heavy for the sequence view, where the
useful feedback is the owner's hand test.

| # | Today | Proposed |
| --- | --- | --- |
| P1 | Every change, including a key binding, needs a numbered contract, decision entries (M7-1 … M7-65) and fixtures. | Two tiers. **Science tier** (anything in `crates/core`, request and result schemas, anything that computes a base, residue, primer or Tm): unchanged process. **Presentation tier** (layout, keys, colours, wording, navigation): one short contract per feature, no decision entry, no fixture scripts. Tested with ratatui's test backend where cheap, and by the owner's hand test. |
| P2 | Words, layout and keys on screen are recorded as owner decisions (D046, D047), so changing them needs approval. | Record only behaviour that affects the science (what is written, what is designed). Screen wording and keys can change freely in the presentation tier. |
| P3 | Small contracts, one function each. | Keep for the science tier. In the presentation tier, one contract per user-visible feature. |
| P4 | Decisions are written as long rule lists. | A short decision per real choice; details live in the contract. |
| P5 | `docs/specs/cli.md` documents every screen in detail. | One user guide written for biologists: tasks first ("make a point mutant in a plasmid"), reference second. |

Unchanged: the required checks, the git rules, worker and planner roles,
and independent verification of scientific fixtures.

## 5. Order of work if approved

1. Amend D005 (section 3) and record P1–P2 in `AGENTS.md`.
2. T1 (plasmids), T4 (interactive defaults), with the sequence view.
3. T2 (ORF detection), T3 (alternative starts).
4. T6 when convenient.

## 6. Questions for the owner

1. Approve, amend or reject each of T1–T6 and P1–P5.
2. T1: answered (refuse, no rotation).
3. T2: a minimum ORF length for the list (for example 75 codons), or list
   all and sort?
