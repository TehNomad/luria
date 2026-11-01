# Fixture: codon usage, Kazusa entry 316407

Date: capture 2026-10-16. Parser and test added by
`docs/tasks/reviews/review-a7c-codon-usage.md` (D039).

## Purpose

`crates/core/src/codon_usage.rs` embeds 64 codon counts that the `default`
codon-selection policy reads (D011). Those counts were transcribed by hand from
a web page, which is exactly the kind of value that cannot be checked from
memory (D018): this fixture keeps a copy of the page, parses it with a script
that shares no code with `crates/core`, and puts the two copies in contact with
a Rust test. The embedded table, the fixture and the source page move together.

It also checks luria's own genetic code independently: the page prints the
residue each codon stands for, and `genetic_code::lookup` must agree with it.

## Files

| Path | Role |
| --- | --- |
| `raw/kazusa-316407-standard.html` | The page as served, byte for byte. Not edited. |
| `reference/parse_kazusa.py` | The parser. Stdlib only; writes `../counts.tsv`, or checks it with `--check`. |
| `counts.tsv` | 64 rows: `codon`, `amino_acid`, `count`. Generated, never transcribed. |

## Source

Codon Usage Database (Kazusa DNA Research Institute):

- entry: *Escherichia coli* W3110 `[gbbct]`, species **316407**. W3110 is a K-12
  substrain; the entry named "Escherichia coli K12" (species 83333) is built from
  only 14 CDSs (5122 codons), so this entry is the one `crates/core` uses;
- **4332 CDSs**, **1372057 codons**, genetic code **1 (standard)**;
- URL: <https://www.kazusa.or.jp/codon/cgi-bin/showcodon.cgi?species=316407&aa=1&style=N>.

The page's `<PRE>` block carries one row per codon:
`[triplet] [amino acid] [fraction] [frequency: per thousand] ([number])`, with
`U` standing for `T`. The numbers in parentheses are the raw counts; the
fraction and per-thousand columns are derived from them and are not carried
into `counts.tsv`.

## Provenance

Two readings of the same page, in the other direction from the parser:

- `crates/core/src/codon_usage.rs` was **transcribed by hand on 2026-10-14**
  from that page;
- this capture was **fetched on 2026-10-16** from the same URL, and is a copy of
  what the site returned then, not a re-derivation of the Rust table.

Capture SHA-256:

    350caaf43968dc2fe8638574f0f980b073070daf18d259ea094bcbea9f5e02fc

4583 bytes. The parser recomputes that hash from the raw file and writes it in
`counts.tsv`, so a capture that no longer matches the table is reported rather
than silently accepted. Nothing here was produced by running luria's codon code.

## Rerunning the parse

From the repository root, with the pinned Python environment
(`tests/fixtures/requirements.txt` — this script needs only the standard
library, the pin is for the other fixture generators):

```
python -I tests/fixtures/codon-usage/reference/parse_kazusa.py            # rewrite counts.tsv
python -I tests/fixtures/codon-usage/reference/parse_kazusa.py --check    # compare, write nothing
```

`--check` regenerates the text in memory, compares it byte for byte with the
tracked `counts.tsv`, prints `OK counts.tsv` or `DIFFERS counts.tsv` followed by
the first differing line, exits 1 on a difference, and writes nothing (D039).
It also re-asserts 64 rows and a total of 1372057, so a truncated capture fails
rather than passing a shrunken table.

It is one of the nine checks `tests/fixtures/verify.py` runs; the others are the
Tm and q5-primer generators. Those Python checks are not part of the required
cargo checks (D039, D026): the Rust tests stay independent of Python.

## Which test reads `counts.tsv`

`crates/core/tests/codon_usage_fixture.rs`:

- `the_embedded_table_equals_the_kazusa_capture` — the embedded table equals the
  TSV codon by codon and count by count, in order;
- `the_capture_totals_the_published_codon_count` — the counts sum to 1372057;
- `the_capture_agrees_with_the_standard_genetic_code` — for each row,
  `genetic_code::lookup(codon)` is `Stop` when `amino_acid` is `*`, and
  otherwise a residue whose one-letter symbol is `amino_acid`.
