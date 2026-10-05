"""TDD tests for the NEBaseChanger parser and the mutation engine.

These tests define the contract for ``luria.nebasechanger``. They are written
against the domain model, not against any particular parser implementation.

Assumed contract (to be implemented):

    parse_summary_file(path: Path) -> list[SDMSummaryRecord]

        One record per data row of the tab-delimited NEBaseChanger summary.
        Coordinates are canonical: zero-based and half-open (decision D004),
        so the fixture's 1-based inclusive codon [1210, 1212] is stored as
        start = 1209, end = 1212.

        SDMSummaryRecord fields used here:
            id                  str
            description         str
            target.start        int   (zero-based, half-open)
            target.end          int   (zero-based, half-open)
            replacement_bases   str
            ta                  int
            forward_primer      Primer
            reverse_primer      Primer

        Primer fields used here:
            id, sequence, length, tm

    apply_mutation(template: str, mutation: SDMSummaryRecord) -> str

        Returns the edited sequence. The substitution must not change
        sequence length for a same-length codon replacement.

Fixture facts (tests/fixtures/README.md):
    D404N, codon 1210-1212 (1-based inclusive) replaced by AAC.
    The summary's "between bases 1209-1213" is the primer-spanning window,
    not the replaced interval; the replaced interval is 1210-1212.

Sequence case is treated as non-semantic: the raw NEBaseChanger FASTA is
lowercase while the input cDNA FASTA is mixed case, so comparisons are
case-normalised.
"""

from __future__ import annotations

import json
from pathlib import Path

from luria.nebasechanger import apply_mutation, parse_summary_file

FIXTURE_DIR = Path(__file__).parent / "fixtures" / "example-construct"

INPUT_FASTA = FIXTURE_DIR / "input" / "sid-1_cDNA.fa"
SUMMARY_FILE = FIXTURE_DIR / "raw" / "sdmsummary.txt"
# tests/fixtures/README.md documents this file as raw/final_sequence.fasta;
# the file actually present in the fixture is raw/sdmfinalseq.fa.
FINAL_FASTA = FIXTURE_DIR / "raw" / "sdmfinalseq.fa"
EXPECTED_JSON = FIXTURE_DIR / "expected" / "expected_mutation.json"

# Ground truth from the fixture, stated in the fixture's own convention:
# 1-based, inclusive.
TARGET_START_1BASED = 1210
TARGET_END_1BASED_INCLUSIVE = 1212
REPLACEMENT_BASES = "AAC"
ORIGINAL_BASES = "GAT"


def _read_fasta_sequence(path: Path) -> str:
    """Minimal FASTA reader, kept independent of the code under test."""
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or not lines[0].startswith(">"):
        raise AssertionError(f"{path} is not a FASTA file")
    return "".join(line.strip() for line in lines[1:] if line.strip())


def _single_record():
    records = parse_summary_file(SUMMARY_FILE)
    assert len(records) == 1, f"expected exactly one summary row, got {len(records)}"
    return records[0]


def test_parse_nebasechanger_summary():
    """Parser output must match the canonical expected target data."""
    record = _single_record()
    expected = json.loads(EXPECTED_JSON.read_text(encoding="utf-8"))

    assert record.id == expected["id"]

    # Boundary conversion: 1-based inclusive [1210, 1212] -> zero-based
    # half-open [1209, 1212). The parser must not carry the raw 1-based
    # numbers into the canonical model.
    assert record.target.start == TARGET_START_1BASED - 1
    assert record.target.end == TARGET_END_1BASED_INCLUSIVE
    assert [record.target.start, record.target.end] == [
        expected["wild_type_range"][0] - 1,
        expected["wild_type_range"][1],
    ]

    assert record.replacement_bases.upper() == expected["replacement_bases"].upper()
    assert record.ta == expected["ta_recommended"]

    for side, primer in (
        ("forward", record.forward_primer),
        ("reverse", record.reverse_primer),
    ):
        exp = expected[f"{side}_primer"]
        assert primer.id == exp["id"]
        assert primer.sequence.upper() == exp["sequence"].upper()
        # Reported length must agree with the sequence the parser produced.
        assert primer.length == exp["length"]
        assert primer.length == len(primer.sequence)
        assert primer.tm == exp["tm"]

    # The summary's "between bases 1209-1213" is the primer-spanning window,
    # not the replaced codon. The parsed target must be the replaced codon
    # (1210-1212), which the coordinate assertions above already enforce.


def test_apply_mutation_matches_nebasechanger_output():
    """In silico substitution must reproduce the NEBaseChanger result."""
    template = _read_fasta_sequence(INPUT_FASTA)
    record = _single_record()

    # Anchor the test to the fixture's stated coordinates before applying.
    start, end = record.target.start, record.target.end
    assert (start, end) == (TARGET_START_1BASED - 1, TARGET_END_1BASED_INCLUSIVE)
    assert template[start:end].upper() == ORIGINAL_BASES
    assert record.replacement_bases.upper() == REPLACEMENT_BASES

    mutated = apply_mutation(template, record)

    assert len(mutated) == len(template), "same-length substitution changed length"
    assert mutated[:start].upper() == template[:start].upper()
    assert mutated[start:end].upper() == REPLACEMENT_BASES
    assert mutated[end:].upper() == template[end:].upper()

    expected_final = _read_fasta_sequence(FINAL_FASTA)
    assert mutated.upper() == expected_final.upper()
