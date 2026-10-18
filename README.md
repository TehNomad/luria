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

Milestone 5 adds the `luria` executable: it reads a typed JSON request,
runs the Q5 workflow and writes one structured JSON result.

```
luria q5 design REQUEST.json [--write-edited DIR]
luria version
luria help
```

See `docs/cli.md` for the request and result schemas, a worked example and
the exit codes. Primer design covers linear templates only (D029), so a
plasmid gives a verified edit with a design failure rather than primers.

Natural-language interpretation is the next milestone (Milestone 6). See
`docs/decisions.md` D034.
