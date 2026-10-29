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
runs the Q5 workflow and writes one structured JSON result. Milestone 6
adds `luria q5 interpret`: a language model reads a request in words, and
luria checks every word of its answer against yours before proposing the
typed request, or asks what it needs to know. You review the proposal and
run it with `luria q5 design`.

```
luria q5 design REQUEST.json [--write-edited DIR]
luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
luria review REQUEST.json [--export DIR]
luria review --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    --request FILE.json [--design PROFILE.json] [--allow-remote] [--export DIR]
luria new [--design PROFILE.json] [--export DIR]
    [--backend openai-compatible --endpoint URL --model NAME [--allow-remote]]
luria FILE
luria find FILE PATTERN [--aa] [--cds NAME] [--circular]
luria version
luria help
```

See `docs/cli.md` for a worked example, the exit codes and error codes, and
a summary of the request and result. Primer design covers linear templates
only (D029), so a plasmid gives a verified edit with a design failure rather
than primers.

Natural-language interpretation (Milestone 6, D044) works with any
OpenAI-compatible server, such as a local llama.cpp or Ollama, over plain
HTTP; HTTPS and the Anthropic API are deferred. It sends your text and a
summary of the record's CDSs, never the sequence. See `docs/cli.md` §10.
Milestone 7 adds `luria review`: a terminal view of a result or a
proposal, which runs a proposal only once you approve it, and exports
the result, the edited constructs and an IDT order table. See
`docs/cli.md` §11 (D046). `luria new` is a form for a template and mutations,
typed in a shorthand or in words, that writes and runs the request once you
approve; see §12 (D047). Milestone 8 adds `luria FILE`, a read-only
view of a GenBank or FASTA file's sequence, translation and features,
with go-to and search, and `luria find`, the same search as a table;
see §13 (D048 to D058). Milestone 9 is an editor in that view (D059).
