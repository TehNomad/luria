"""Check the hand-derived linear GenBank expectations with Biopython (D039).

../synthetic-linear.gb is a 393-base linear record written by hand for the
natural-language milestone (docs/tasks/milestone-6i0-linear-genbank.md, plan
M6-13). It holds one CDS, `synA`, that luria can design primers on, unlike
every other GenBank fixture, which is circular (D035, D041 G10). Its three
expectation tables were derived by hand, as ../README.md records:

* ../synthetic-linear-record.tsv: the LOCUS name, the first ACCESSION, the
  VERSION, the length, the topology, the SHA-256 of the upper-cased sequence
  (luria's canonical text, D007), and the number of features; the columns of
  ../pbr322-record.tsv.
* ../synthetic-linear-cds.tsv: the CDS's feature index, its id (D041 G2) and
  the qualifier it came from, its interval (zero-based half-open), strand,
  /codon_start, /transl_table, first codon, residue count under the standard
  code without the terminal stop, and whether that translation equals the
  record's /translation; the columns of ../pbr322-cds.tsv.
* ../synthetic-linear-residues.tsv: the residues the natural-language corpus
  names, with each one's letter, its codon and the codon's template interval
  (zero-based half-open).

This script writes nothing. It re-derives every column with Biopython's
parser and hashlib, which share no code with `crates/core`, and asserts the
tables agree.

Run from the repository root with the pinned environment and `python -I`:

    python -I tests/fixtures/genbank/reference/verify_synthetic_linear.py
    python -I tests/fixtures/genbank/reference/verify_synthetic_linear.py --check

Both forms print `OK <name>` for each table and exit 0, or print
`DIFFERS <name>` with the first disagreement and exit 1. --check is accepted
so verify.py can run this like every other check.
"""
import hashlib
import os
import sys

from Bio import SeqIO

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)

record = SeqIO.read(os.path.join(FIXTURE, "synthetic-linear.gb"), "genbank")
SEQ = str(record.seq).upper()


def rows(name):
    with open(os.path.join(FIXTURE, name), encoding="ascii") as f:
        lines = f.read().splitlines()
    header = lines[0].split("\t")
    return [dict(zip(header, line.split("\t"))) for line in lines[1:]]


def identity(feature):
    """D041 G2: the first of /locus_tag, /gene, /label, and its qualifier."""
    for name in ("locus_tag", "gene", "label"):
        if name in feature.qualifiers:
            return feature.qualifiers[name][0], name
    return "-", "-"


def qualifier(feature, name):
    return feature.qualifiers.get(name, ["-"])[0]


def the_cds():
    found = [(i, f) for i, f in enumerate(record.features) if f.type == "CDS"]
    assert len(found) == 1, "expected one CDS, found %d" % len(found)
    return found[0]


def compare(name, expected, derived):
    """Print OK or DIFFERS for one table; return whether it agreed."""
    if len(expected) != len(derived):
        print("DIFFERS %s: %d rows, derived %d" % (name, len(expected), len(derived)))
        return False
    for number, (row, want) in enumerate(zip(expected, derived), start=2):
        for column, value in want.items():
            if row.get(column) != value:
                print(
                    "DIFFERS %s: line %d, %s is %r, derived %r"
                    % (name, number, column, row.get(column), value)
                )
                return False
    print("OK %s" % name)
    return True


def check_record():
    derived = [
        {
            "locus_name": record.name,
            "accession": record.annotations["accessions"][0],
            "version": record.id,
            "length": str(len(SEQ)),
            "topology": record.annotations["topology"],
            "sha256": hashlib.sha256(SEQ.encode("ascii")).hexdigest(),
            "features": str(len(record.features)),
        }
    ]
    return compare("synthetic-linear-record.tsv", rows("synthetic-linear-record.tsv"), derived)


def check_cds():
    index, feature = the_cds()
    coding = feature.extract(record.seq).upper()
    protein = str(coding.translate(table=1))
    assert protein.endswith("*") and protein.count("*") == 1, "one stop, at the end"
    ident, source = identity(feature)
    derived = [
        {
            "index": str(index),
            "id": ident,
            "id_source": source,
            "start": str(int(feature.location.start)),
            "end": str(int(feature.location.end)),
            "strand": "+" if feature.location.strand == 1 else "-",
            "codon_start": qualifier(feature, "codon_start"),
            "transl_table": qualifier(feature, "transl_table"),
            "start_codon": str(coding[:3]),
            "residues": str(len(protein) - 1),
            "translation_matches": (
                "yes" if protein[:-1] == qualifier(feature, "translation") else "no"
            ),
        }
    ]
    return compare("synthetic-linear-cds.tsv", rows("synthetic-linear-cds.tsv"), derived)


def check_residues():
    _, feature = the_cds()
    assert feature.location.strand == 1, "the codon arithmetic below is plus-strand"
    coding = str(feature.extract(record.seq).upper())
    protein = str(feature.extract(record.seq).translate(table=1))
    cds_start = int(feature.location.start)
    expected = rows("synthetic-linear-residues.tsv")
    derived = []
    for row in expected:
        number = int(row["number"])
        offset = 3 * (number - 1)
        start = cds_start + offset
        assert SEQ[start : start + 3] == coding[offset : offset + 3]
        derived.append(
            {
                "number": str(number),
                "residue": protein[number - 1],
                "codon": coding[offset : offset + 3],
                "start": str(start),
                "end": str(start + 3),
            }
        )
    return compare("synthetic-linear-residues.tsv", expected, derived)


def main():
    args = sys.argv[1:]
    if args not in ([], ["--check"]):
        print("usage: verify_synthetic_linear.py [--check]")
        return 2
    results = [check_record(), check_cds(), check_residues()]
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
