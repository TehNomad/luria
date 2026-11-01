---
type: Research Note
title: "Milestone 6: extraction on a weak local model"
description: "Prompt research for contract 6f, run on the owner's request (\"I want to try to get the NLP working on less powerful models\")."
status: stable
---
# Milestone 6: extraction on a weak local model

**What this is.** Prompt research for contract 6f, run on the owner's request
("I want to try to get the NLP working on less powerful models"). It measures
how well the local model that serves as luria's development worker fills in
the extraction of plan §5.2 from a user's prose, and what the trust boundary
(plan §3, §5.3a) does when it gets something wrong.

**Date.** 2026-10-19.

**What this is not.** Not a correctness claim and not a fixture (plan §8:
a live model's score is development information). No product code was
written. The harness is a throwaway under `target/tmp/m6/eval/` (untracked),
and its end-state scorer is a rough Python approximation of the plan's
grounding and building, which do not exist yet (contract 6e). Every reply was
also read by the real 6a parser (`luria_nl::extraction::parse_extraction`),
from a scratch crate outside the workspace.

## Setup

- **Model.** `qwen3.8-flash-next-coder-iq1_m`, an IQ1_M quantization (about
  1.75 bits per weight), served by Strata 0.1.45, an OpenAI-compatible
  llama.cpp server, at `http://127.0.0.1:8080/v1`, one slot, 131k context.
  This is the model behind the `worker` subagent.
- **Call.** `POST /v1/chat/completions`, exactly what the planned
  `openai-compatible` backend will send (plan §5.6): a system prompt, one
  user message, `temperature` 0, `seed` 1.
- **What the model saw** (M6-4): the user's text and a CDS summary of the
  record — never the sequence, never a coordinate.
- **Arms.** Prompt v0, v1 and v2 (below); with and without
  `response_format: json_schema` (llama.cpp turns the schema into a decoding
  grammar); and v0 with the model's thinking on
  (`chat_template_kwargs.enable_thinking`).
- **Texts.** The 24 corpus texts (`tests/fixtures/nl/`), and 14 harder
  paraphrases written for this note (below).
- **Verdicts.** *correct*: the end state and, for a proposal, the exact
  operation a correct reading gives. *correct-ish*: a question, but with a
  different code from the corpus's. *safe*: a question or refusal where a
  proposal was right. *UNSAFE*: a proposal that differs from the right one.
  For the four corpus cases whose scripted reply is deliberately faulty
  (`ungrounded-*`, `reply-*`), and for `number-not-bounded`, the yardstick is
  what a correct model makes of the text (a proposal), not the corpus's
  scripted outcome. The scorer reproduced all 24 corpus outcomes from the
  corpus's own scripted replies before any live run.

## Results

| Arm | Corpus (24) | Stress (14) | Corpus time |
| --- | --- | --- | --- |
| v0, schema | 23 correct, 1 correct-ish | 14 correct | 103 s |
| v0, free | 22 correct, 1 correct-ish, 1 safe | — | 92 s |
| v0, schema, thinking on | 23 correct, 1 correct-ish | — | 456 s |
| v1, schema | 22 correct, 2 correct-ish | 14 correct | 107 s |
| v1, free | 22 correct, 2 correct-ish | 14 correct | 93 s |
| **v2, schema** | **23 correct, 1 correct-ish** | **14 correct** | 100 s |
| **v2, free** | **23 correct, 1 correct-ish** | **14 correct** | 94 s |

**No arm produced an UNSAFE outcome**, and every reply of every arm parsed
under the real 6a parser, including the unconstrained ones: no markdown
fence, no extra field, no wrong type.

## Findings

1. **A weak model is enough for this extraction.** Prompt v2 gets every
   outcome right on 38 texts at about 4 s a call: prose with no token,
   three-letter and hyphenated codes, an arrow, "two separate mutants: one
   with…", a chatty paragraph, a lowercase codon, and all six cases that must
   end in a question.
2. **Thinking does not help.** Same verdicts, 4.4 times the time. The
   backend should not turn it on for extraction.
3. **The schema grammar is not needed by this model, but costs nothing.**
   Free and constrained arms scored the same with v2. Constrained decoding
   guarantees the shape for a model that is worse at JSON; recommended where
   the server supports it.
4. **The evidence check caught a real hallucination.** With prompt v1, for
   "change alanine to glycine" the model wrote `?12G`: it marked the stated
   alanine as unknown and invented the number 12, **copied from the prompt's
   own example `?12T`**. The quote did not show 12, so the scorer — like 6e
   will — asked instead of proposing A12G. v2 gives the `?` example without a
   number (`S?T`) and says never to take values from the examples; the model
   then wrote `A?G`.
5. **The `?` placeholder works.** Asked to write `?` for an unstated part,
   the model did so correctly (`A?G`, and `?42G` for "position 42 to
   glycine"). The plan now has the notation parser recognise it (§5.3a), so
   the question names the missing part.
6. **A deletion takes the `unsupported` route, not a token.** For "delete
   K12" every arm reported `operation: other` with `unsupported: ["delete
   K12"]`, as the prompt asks, so the outcome is `operation_unsupported`. The
   corpus's scripted reply takes the other route (token `K12del`,
   `mutation_notation_unsupported`). Both are questions; 6e must word both
   well. This is the only *correct-ish* case left with v2.
7. **Temperature 0 is not reproducible on this server.** Two v2 runs gave 2
   of 24 replies that differ byte for byte — only in which span was quoted
   (`"A42G"` against the whole sentence; a trailing full stop) — with the same
   end state. This supports the plan's design: the interpretation document
   records the raw reply, and its determinism is over a given reply, never
   over the model call (§5.7).
8. **Primer design read as an extra task.** v0, unconstrained, filed "design
   Q5 SDM primers" under `unsupported` — safe, but wrong. v1 onward say that
   designing primers is part of the operation.

## Recommendations for 6f and 6g1

- Start the prompt from v2, verbatim below, with the catalogue line format
  below. Give `PROMPT_VERSION` to exactly this text.
- Keep every example in the prompt free of numbers and residues a model
  could copy into a plausible answer, and keep a test that the prompt's own
  example tokens never appear in a corpus text. (One stress text, `other-gene`,
  uses `S12T`, the prompt's example, on purpose: its outcome is a question
  either way.)
- 6g1: send `response_format: json_schema` with the schema below. Thinking is
  a server- and model-specific switch (`chat_template_kwargs`); whether the
  backend sends it, or leaves it to server configuration, is an open item for
  6g1 — it should not be on for extraction.
- A live, opt-in `#[ignore]` test (plan §8) should run the corpus and this
  stress set and report the verdict table above.

## Caveats

38 texts, one synthetic record with one usable CDS (and `synthetic.gb` for
one case), one model, single runs except the repeat in finding 7. The
stress texts and their ideal outcomes were written by the planner for this
note and are not independently reviewed fixtures. The scorer approximates
6e; in particular its numbering-phrase rule ("cds" or "coding", and "start",
"beginning" or "first") is a stand-in for 6e's fixed list.

## Prompt v2

System message:

```
You read one molecular-biology request and report what it asks for, as JSON. You do no biology yourself. Copy the user's words; never invent a value.

Fields:
- operation: "q5_residue_substitutions" if the user asks to change amino-acid residues of a protein-coding sequence into other amino acids; otherwise "other". Asking to design primers (Q5, SDM, site-directed mutagenesis) for those substitutions is part of this operation, never unsupported.
- cds: the name of the coding sequence or gene exactly as the user wrote it, or null if the user named none. Never choose one from the CDS list yourself.
- mutations: one object per amino-acid substitution the user asks for:
  - token: original one-letter amino acid, residue number, new one-letter amino acid, e.g. "S12T". Use exactly the number the user wrote. If the user does not state the original residue, the number or the new residue, write "?" in that place only, e.g. "S?T" when no number is given. Never take a number or residue from these examples.
  - quote: the exact words of the user's text that state this substitution, copied character for character.
  - codon: a DNA codon the user names for this substitution, copied exactly, or null.
- composition: "separate_variants" if the user wants the substitutions as separate, independent variants; "combined_construct" if they want them together in one construct; null if they did not say.
- numbering: the user's exact words about how residues are numbered, or null if they said nothing about it.
- unsupported: exact quotes of anything else the user asks for that these fields cannot hold, such as deletions, insertions, stop codons, DNA changes, tags or questions. An empty list if there is none.

Every string except token must be copied verbatim from the user's text. Do not use the CDS list to fill in anything the user did not say. Reply with the JSON object only.
```

User message (catalogue, then the text):

```
CDS features in the template:
- synA (gene synA; product synthetic protein A; 100 residues; + strand; usable)

Request:
<the user text>
```

`response_format.json_schema.schema`:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": [
    "operation",
    "cds",
    "mutations",
    "composition",
    "numbering",
    "unsupported"
  ],
  "properties": {
    "operation": {
      "type": "string",
      "enum": [
        "q5_residue_substitutions",
        "other"
      ]
    },
    "cds": {
      "type": [
        "string",
        "null"
      ]
    },
    "mutations": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": [
          "token",
          "quote",
          "codon"
        ],
        "properties": {
          "token": {
            "type": "string"
          },
          "quote": {
            "type": "string"
          },
          "codon": {
            "type": [
              "string",
              "null"
            ]
          }
        }
      }
    },
    "composition": {
      "enum": [
        "separate_variants",
        "combined_construct",
        null
      ]
    },
    "numbering": {
      "type": [
        "string",
        "null"
      ]
    },
    "unsupported": {
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  }
}
```

## The stress paraphrases

All against `synthetic-linear.gb`.

| Id | Text | Profile | Ideal |
| --- | --- | --- | --- |
| `arrow` | Can you make Trp87→Ala in synA? Residues numbered from the CDS start. | `full.json` | propose W87A |
| `position-of` | Please mutate the alanine at position 42 of synA to a glycine, counting from the start of the CDS. | `full.json` | propose A42G |
| `hyphen-three` | In synA replace Ala-42 with Gly (numbering from the CDS start). | `full.json` | propose A42G |
| `two-mutants` | I need two separate mutants of synA: one with A42G and another with W87A. Number from the beginning of the CDS. | `full.json` | propose A42G, W87A (separate_variants) |
| `together` | Introduce A42G and W87A together into synA, numbering from the CDS start. | `full.json` | propose A42G, W87A (combined_construct) |
| `chatty` | Hi! I'm finishing my thesis and my PI wants an alanine scan, but for now just one: in synA, could you swap alanine 42 for glycine? We number from the CDS start. Thanks so much! | `full.json` | propose A42G |
| `lower-codon` | In synA, change A42 to glycine using codon ggc, numbering from the CDS start. | `no-codon.json` | propose A42G codon GGC |
| `no-original` | In synA, change position 42 to glycine, numbering from the CDS start. | `full.json` | ask |
| `stop-codon` | Put a stop codon at residue 42 of synA, numbering from the CDS start. | `full.json` | ask |
| `other-gene` | Make the S12T mutation in lacZ, numbering from the CDS start. | `full.json` | ask |
| `list-no-comp` | synA: A42G, W87A. Numbering from the CDS start. | `full.json` | ask |
| `lower-cds` | in syna change a42g, numbering from the cds start | `full.json` | ask |
| `insertion` | Insert a glycine after residue 42 of synA, numbering from the CDS start. | `full.json` | ask |
| `each-own` | For synA (numbered from the CDS start), make A42G and W87A, each on its own. | `full.json` | propose A42G, W87A (separate_variants) |

## Prompt v3

Post-M6 review, 2026-10-20 (D045). One line of v2 changes, the quote's: it must state one
substitution only, and include a codon the user names, because luria now asks when a quote
holds more than one number or misses its codon. The schema and the user message are unchanged.

System message:

```
You read one molecular-biology request and report what it asks for, as JSON. You do no biology yourself. Copy the user's words; never invent a value.

Fields:
- operation: "q5_residue_substitutions" if the user asks to change amino-acid residues of a protein-coding sequence into other amino acids; otherwise "other". Asking to design primers (Q5, SDM, site-directed mutagenesis) for those substitutions is part of this operation, never unsupported.
- cds: the name of the coding sequence or gene exactly as the user wrote it, or null if the user named none. Never choose one from the CDS list yourself.
- mutations: one object per amino-acid substitution the user asks for:
  - token: original one-letter amino acid, residue number, new one-letter amino acid, e.g. "S12T". Use exactly the number the user wrote. If the user does not state the original residue, the number or the new residue, write "?" in that place only, e.g. "S?T" when no number is given. Never take a number or residue from these examples.
  - quote: the exact words of the user's text that state this substitution and no other, copied character for character. If the user names a codon for it, the quote includes the codon.
  - codon: a DNA codon the user names for this substitution, copied exactly, or null.
- composition: "separate_variants" if the user wants the substitutions as separate, independent variants; "combined_construct" if they want them together in one construct; null if they did not say.
- numbering: the user's exact words about how residues are numbered, or null if they said nothing about it.
- unsupported: exact quotes of anything else the user asks for that these fields cannot hold, such as deletions, insertions, stop codons, DNA changes, tags or questions. An empty list if there is none.

Every string except token must be copied verbatim from the user's text. Do not use the CDS list to fill in anything the user did not say. Reply with the JSON object only.
```

User message (catalogue, then the text):

```
CDS features in the template:
- synA (gene synA; product synthetic protein A; 100 residues; + strand; usable)

Request:
<the user text>
```

`response_format.json_schema.schema`:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": [
    "operation",
    "cds",
    "mutations",
    "composition",
    "numbering",
    "unsupported"
  ],
  "properties": {
    "operation": {
      "type": "string",
      "enum": [
        "q5_residue_substitutions",
        "other"
      ]
    },
    "cds": {
      "type": [
        "string",
        "null"
      ]
    },
    "mutations": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": [
          "token",
          "quote",
          "codon"
        ],
        "properties": {
          "token": {
            "type": "string"
          },
          "quote": {
            "type": "string"
          },
          "codon": {
            "type": [
              "string",
              "null"
            ]
          }
        }
      }
    },
    "composition": {
      "enum": [
        "separate_variants",
        "combined_construct",
        null
      ]
    },
    "numbering": {
      "type": [
        "string",
        "null"
      ]
    },
    "unsupported": {
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  }
}
```

Measured: see "Prompt v3 results" below.

### Prompt v3 results

Planner, 2026-10-20, after D045's rules and the v3 prompt were in the code:
the same model, server and endpoint as v2 (`qwen3.8-flash-next-coder-iq1_m`
at `http://127.0.0.1:8080/v1`), with the schema, through the shipped
`OpenAiCompatible` and `interpret`, single runs.

| Set | Result |
| --- | --- |
| Corpus (24), `live_corpus_report` | 18 end as the scripted replies do; the other 6 are the expected ones (`notation-deletion` asks `operation_unsupported`; `number-not-bounded` proposes A420G, the text's own number; `ungrounded-*` and `reply-*` propose A42G, correctly). By this note's yardstick: 23 correct, 1 correct-ish. |
| Stress (14), planner scratch harness | 14 correct: every proposal the right one, every question where one is due. |

No outcome was UNSAFE, and no D045 rule turned a correct proposal into a
question. The changed line had its effect: in `codon-stated` and
`lower-codon` the model put the codon inside the quote ("change W87 to
alanine using the codon GCG."), so both are proposed. The corpus run took
402 s.
