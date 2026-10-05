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

        Returns the edited sequence. A same-length codon replacement must not
        change sequence length.

NEBaseChanger is an external reference, not the source of truth for luria
(decisions D008, D011). The raw files under tests/fixtures/ are captured tool
output; the strongest assertion compares the complete edited sequence against
the independently captured NEBaseChanger final FASTA.

Sequence case is non-semantic: the input cDNA FASTA is predominantly upper
and the NEBaseChanger output FASTA is entirely lower case, so comparisons are
case-normalised.
"""

from __future__ import annotations

import json
from pathlib import Path

from luria.nebasechanger import apply_mutation, parse_summary_file

FIXTURE_DIR = Path(__file__).parent / "fixtures" / "example-construct"

INPUT_FASTA = FIXTURE_DIR / "input" / "sid-1_cDNA.fa"
SUMMARY_FILE = FIXTURE_DIR / "raw" / "sdmsummary.txt"
FINAL_FASTA = FIXTURE_DIR / "raw" / "sdmfinalseq.fa"
EXPECTED_JSON = FIXTURE_DIR / "expected" / "expected_mutation.json"

# Independent invariants, stated in the fixture's own convention: 1-based,
# inclusive. These come from the raw NEBaseChanger files and tests/fixtures/
# README.md, not from expected_mutation.json.
TARGET_START_1BASED = 1210  # first base of the replaced codon
TARGET_END_1BASED_INCLUSIVE = 1212  # last base of the replaced codon
ORIGINAL_BASES = "GAT"
REPLACEMENT_BASES = "AAC"

# NEBaseChanger's summary description names the wild-type bases that flank
# the edit (A at 1209, T at 1213). These are exclusive anchors: the replaced
# bases lie strictly between them. Kept here so the test can prove the
# parser did not mistake the anchors for the edit bounds.
DESCRIPTION_ANCHORS_1BASED = (1209, 1213)


def _half_open(start_1based: int, end_1based_inclusive: int) -> tuple[int, int]:
    """External 1-based inclusive -> canonical zero-based half-open (D004)."""
    return start_1based - 1, end_1based_inclusive


# The one canonical interval this fixture is about: [1209, 1212).
TARGET = _half_open(TARGET_START_1BASED, TARGET_END_1BASED_INCLUSIVE)


def _read_fasta_sequence(path: Path) -> str:
    """Minimal single-record FASTA reader, independent of luria's code."""
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or not lines[0].startswith(">"):
        raise AssertionError(f"{path} is not a FASTA file")
    if sum(1 for line in lines if line.startswith(">")) != 1:
        raise AssertionError(f"{path} holds more than one FASTA record")
    return "".join(line.strip() for line in lines[1:] if line.strip())


def _raw_summary_row() -> list[str]:
    """The single data row of the raw summary, read without the parser."""
    lines = SUMMARY_FILE.read_text(encoding="utf-8").splitlines()
    assert lines[0].startswith("ID\tDescription\t"), "unexpected summary header"
    rows = [line.split("\t") for line in lines[1:] if line.strip()]
    assert len(rows) == 1, f"expected exactly one summary row, got {len(rows)}"
    return rows[0]


def _single_record():
    records = parse_summary_file(SUMMARY_FILE)
    assert len(records) == 1, f"expected exactly one summary row, got {len(records)}"
    return records[0]


def assert_same_sequence(actual: str, expected: str, why: str) -> None:
    """Sequence equality; case is non-semantic across the two FASTA files."""
    assert actual.upper() == expected.upper(), why


def test_parse_nebasechanger_summary():
    """Parser output must match the reviewed reference and the raw row."""
    record = _single_record()
    expected = json.loads(EXPECTED_JSON.read_text(encoding="utf-8"))

    assert record.id == expected["id"]

    # The raw description gives exclusive anchors around the edit, not its
    # bounds: "between bases 1209-1213" means the replaced bases are 1210-1212.
    description = _raw_summary_row()[1]
    low, high = DESCRIPTION_ANCHORS_1BASED
    anchor_text = f"between bases {low}-{high}"
    assert anchor_text in description, "unexpected summary description"

    # Boundary conversion (D004): 1-based inclusive [1210, 1212] -> [1209, 1212).
    parsed_target = (record.target.start, record.target.end)
    assert parsed_target == TARGET

    # Treating the anchors as edit bounds would give [1208, 1213), one base
    # too wide on each side. The parser must not do that.
    assert parsed_target != _half_open(*DESCRIPTION_ANCHORS_1BASED)

    # The reviewed reference agrees with the independently stated invariants.
    assert _half_open(*expected["wild_type_range"]) == TARGET

    assert record.replacement_bases.upper() == REPLACEMENT_BASES
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


def test_apply_mutation_matches_nebasechanger_output():
    """In silico substitution must reproduce the NEBaseChanger result."""
    template = _read_fasta_sequence(INPUT_FASTA)
    record = _single_record()

    # Anchor to the fixture's own coordinates before applying anything.
    start, end = record.target.start, record.target.end
    assert (start, end) == TARGET
    assert_same_sequence(template[start:end], ORIGINAL_BASES, "target codon is not GAT")
    assert record.replacement_bases.upper() == REPLACEMENT_BASES

    mutated = apply_mutation(template, record)

    assert len(mutated) == len(template), "same-length substitution changed length"
    assert_same_sequence(mutated[:start], template[:start], "prefix changed")
    assert_same_sequence(mutated[start:end], REPLACEMENT_BASES, "codon not replaced")
    assert_same_sequence(mutated[end:], template[end:], "suffix changed")

    # Strongest check: the complete edited sequence against the independently
    # captured NEBaseChanger output, not against anything apply_mutation says.
    assert_same_sequence(
        mutated,
        _read_fasta_sequence(FINAL_FASTA),
        "edited sequence differs from NEBaseChanger final FASTA",
    )
