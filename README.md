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
The review-oriented TUI is next (Milestone 7, D034).
