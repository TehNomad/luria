"""Write ../expected.tsv: the header, length and SHA-256 of each FASTA input (D039).

luria's sequence digest is SHA-256 over the canonical text: the sequence
letters with ASCII lower case folded to upper case and every line break and
blank removed (`crates/core/src/sequence.rs`, D007). This script computes the
same quantity with the Python standard library only, sharing no code with
`crates/core`, so `crates/core/tests/fasta_fixtures.rs` checks the Rust FASTA
reader against an independent reading of the same files.

Each input is a single-record FASTA file. The header is the first line with
its leading `>` removed and surrounding blanks stripped; every later line is
sequence text. The script asserts that shape rather than handling anything
else: it is a check on two known files, not a FASTA parser.

Run from the repository root with `python -I`:

    python -I tests/fixtures/fasta/reference/digests.py
    python -I tests/fixtures/fasta/reference/digests.py --check

Without --check, writes ../expected.tsv. With --check, regenerates the same
text in memory, compares it byte for byte with that tracked file, prints `OK
expected.tsv` or `DIFFERS expected.tsv` followed by the first differing line,
exits 1 on a difference, and writes nothing (D039).
"""
import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)
FIXTURES = os.path.dirname(FIXTURE)
OUT = os.path.join(FIXTURE, "expected.tsv")

# Paths relative to tests/fixtures/, as the Rust test names them.
INPUTS = [
    "example-construct/input/sid-1_cDNA.fa",
    "example-construct/raw/sdmfinalseq.fa",
]


def describe(relative):
    with open(os.path.join(FIXTURES, relative), "rb") as f:
        text = f.read().decode("ascii")
    lines = text.splitlines()
    assert lines and lines[0].startswith(">"), relative + ": no header line"
    assert not any(line.startswith(">") for line in lines[1:]), relative + ": second record"
    header = lines[0][1:].strip(" \t")
    body = "".join("".join(line.split()) for line in lines[1:])
    assert body.isalpha() and body.isascii(), relative + ": non-letter in sequence text"
    canonical = body.upper()
    digest = hashlib.sha256(canonical.encode("ascii")).hexdigest()
    return [relative, header, str(len(canonical)), digest]


def render():
    rows = [["file", "header", "length", "sha256"]]
    rows += [describe(relative) for relative in INPUTS]
    return "".join("\t".join(row) + "\n" for row in rows)


def main():
    text = render()
    if "--check" in sys.argv[1:]:
        with open(OUT, "r", encoding="ascii", newline="") as f:
            tracked = f.read()
        if tracked == text:
            print("OK expected.tsv")
            return 0
        for number, (want, have) in enumerate(
            zip(text.splitlines(), tracked.splitlines()), start=1
        ):
            if want != have:
                print("DIFFERS expected.tsv line %d: %r != %r" % (number, have, want))
                return 1
        print("DIFFERS expected.tsv: line counts differ")
        return 1
    with open(OUT, "w", encoding="ascii", newline="") as f:
        f.write(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
