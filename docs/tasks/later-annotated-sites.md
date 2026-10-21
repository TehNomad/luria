# Later: naming a residue by an annotated site

Status: **Deferred by the owner, 2026-10-19. Not scheduled and not a task
contract: do not implement.** Placing it in a milestone is a scope decision
for the owner (D034). Raised during Milestone 6
(`docs/tasks/milestone-6-nl.md`).

## The request it would serve

"Mutate the histidine in the catalytic triad to an alanine." The user names
the residue by its role, not its number.

## What luria does today

It asks. On the local model (prompt v2 of
`docs/research/m6-local-model-extraction-2026-10-19.md`), that sentence gives
the token `H?A`: histidine to alanine, number unstated. The evidence check
turns it into a `residue_number_unstated` question. That is the right
behaviour until a deterministic source for the number exists.

## Why the model must not supply the number

D004: the model never provides a coordinate. Here the danger is concrete. A
model will "know" that chymotrypsin's triad is His57, Asp102 and Ser195, but
those are **chymotrypsin numbering**, a convention that often differs from
the sequential numbering of the user's construct. Signal peptides, tags and
truncations shift it further. An answer from memory would sound right and
land on the wrong residue.

## The sketch

The same pattern as Milestone 6, grounding against the **record** as well as
the user's text:

1. luria reads site annotations from the GenBank record (`Site` and `Region`
   features, `/site_type="active"`, `/note="catalytic triad"`), and lists
   them in the summary the model is shown, beside the CDSs.
2. The model **chooses** a listed site, with a verbatim quote of the user's
   words as evidence. It can choose; it never writes a number.
3. Rust resolves the site to a residue number from the record's own
   coordinates, and the expected-residue check (D011) confirms the user's
   "histidine" against the translation. If the site holds a serine, luria
   refuses.
4. The proposal shows the chain: the user's words, the record's feature, the
   residue number, and the check.

No matching site, or two histidines in one annotated triad: luria asks
(D005).

## What it would need

- **An owner decision on site sources.** The record's own annotations first:
  offline, private, already in the user's file. Later options are external
  databases (UniProt active sites, the Mechanism and Catalytic Site Atlas)
  and motif or structure analysis by Python workers (D017). Each external
  source raises privacy (D023) and provenance (D021) questions.
- An extraction entry for "refers to a listed site", new question codes
  (site not found, site ambiguous), and site features in the catalogue.
- Fixtures: a hand-written record with an annotated triad, and corpus cases
  that mix numbering conventions on purpose.

Nothing in Milestone 6 needs to change to allow this: the evidence quotes,
the split between model I/O and science, and the question mechanism all
carry over.
