# luria

A molecular-biology toolbox with a natural-language interface,
built in Rust with Python workers for selected scientific computations.

luria translates user requests into explicit, validated operations.
Scientific results come from deterministic code, not LLM-generated
sequence calculations.

## Status

Early development. `crates/core` implements Milestones 1–4:
sequences, features, CDSs and coordinates; mutation semantics with
independent verification; and the Q5 site-directed mutagenesis workflow,
including primer design for substitutions, deletions, insertions,
replacements and degenerate expansion.

There is no executable yet. The next milestone is a minimal CLI with
machine-readable output (Milestone 5); natural-language interpretation
follows (Milestone 6). See `docs/decisions.md` D034.
