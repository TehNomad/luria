# FASTA reader fixture

`expected.tsv` records, for each single-record FASTA input, the header (the
first line without `>`, blanks at both ends removed), the sequence length, and
the SHA-256 of the canonical text: the letters, upper-cased, with line
breaks and blanks removed (D007). `crates/core/tests/fasta_fixtures.rs`
reads each file with `luria_core::fasta::read_fasta` and compares all three.

The inputs are the SID-1 files in `../example-construct/`: the wild-type cDNA
(`input/sid-1_cDNA.fa`, upper case with one lower-case run) and the
NEBaseChanger output (`raw/sdmfinalseq.fa`, all lower case, no final newline,
two spaces inside its header). Both lengths, 3153, are the ones that
fixture's README already records.

## Verification

`reference/digests.py` computes all three columns with the Python standard
library only, sharing no code with `crates/core`. The digests were also
checked by hand on 2026-10-17 with

    grep -v '^>' FILE | tr -d '\n\r \t' | tr a-z A-Z | sha256sum

which printed the same two values. Rerun the script's check from the
repository root:

    python -I tests/fixtures/fasta/reference/digests.py --check
