"""Check the hand-derived synthetic GenBank expectations with Biopython (D039).

../synthetic.gb is a 60-base circular record written by hand for luria's
GenBank CDS import (docs/tasks/contracts/milestone-5c3-genbank-cds.md). Its two
expectation tables were derived by hand, as ../README.md records:

* ../synthetic-expected.tsv: what importing each named CDS gives, one row
  per request name: the intervals (zero-based half-open, D036 order), the
  strand, the qualifier the id came from, the /transl_table, the residues
  under the standard code, or the error and its detail.
* ../synthetic-unused.tsv: every feature in file order, with the CDS id luria
  would use (D041 G2) and the refusal luria would give it, when nothing is
  imported.

This script writes nothing. It re-derives every column that does not depend
on luria's own rule names with Biopython's parser, which shares no code with
`crates/core`, and asserts the tables agree: feature order and keys, ids and
their qualifiers, intervals and strands, translations, start codons,
qualifier values, location forms, the first residue where a stated
/translation differs, the internal stop codon and the CDS length. The rule
names themselves are luria's contract, checked by the Rust tests.

Run from the repository root with the pinned environment and `python -I`:

    python -I tests/fixtures/genbank/reference/verify_synthetic.py
    python -I tests/fixtures/genbank/reference/verify_synthetic.py --check

Both forms print `OK synthetic-expected.tsv` and `OK synthetic-unused.tsv`
and exit 0, or print `DIFFERS <name>` with the first disagreement and exit 1.
--check is accepted so verify.py can run this like every other check.
"""
import os
import sys
import warnings

from Bio import BiopythonParserWarning, SeqIO
from Bio.SeqFeature import BeforePosition, AfterPosition, CompoundLocation

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)

with warnings.catch_warnings():
    # Biopython 1.88 gives no parse warning for this record. The filter stays
    # so that a version which warns about the locations the record holds on
    # purpose, such as the remote X00001.1:1..12, can't fail the check.
    warnings.simplefilter("ignore", BiopythonParserWarning)
    record = SeqIO.read(os.path.join(FIXTURE, "synthetic.gb"), "genbank")
SEQ = record.seq
N = len(SEQ)
assert N == 60 and record.annotations["topology"] == "circular"


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
    return None, None


def d036(location):
    """Biopython's parts as luria intervals: one, or [a, n) then [0, b)."""
    parts = sorted((int(p.start), int(p.end)) for p in location.parts)
    if len(parts) == 2 and parts[0][0] == 0 and parts[1][1] == N:
        parts = [parts[1], parts[0]]
    return ",".join("%d-%d" % part for part in parts)


def standard(coding):
    protein = str(coding.translate(table=1))
    return protein[:-1] if protein.endswith("*") else protein


def first_difference(stated, translated):
    for index in range(max(len(stated), len(translated))):
        a = stated[index] if index < len(stated) else "-"
        b = translated[index] if index < len(translated) else "-"
        if a != b:
            return "%d:%s:%s" % (index + 1, a, b)
    return None


CDS = [f for f in record.features if f.type == "CDS"]


def only(name):
    found = [f for f in CDS if identity(f)[0] == name]
    return found


def check_expected():
    problems = []
    for row in rows("synthetic-expected.tsv"):
        name, error, rule = row["name"], row["error"], row["rule"]
        found = only(name)
        if error == "unusable_genbank_cds" and rule == "not_found":
            if found:
                problems.append(name + ": a CDS has this id")
            continue
        if rule == "ambiguous_name":
            if len(found) < 2:
                problems.append(name + ": fewer than two CDSs share this id")
            continue
        if len(found) != 1:
            problems.append(name + ": %d CDSs have this id" % len(found))
            continue
        feature = found[0]
        qualifiers = feature.qualifiers
        location = feature.location
        if error == "-" or error == "genbank_translation_mismatch":
            coding = location.extract(SEQ)
            protein = standard(coding)
            if protein != row["residues"]:
                problems.append(name + ": residues %s, Biopython %s" % (row["residues"], protein))
        if error == "-":
            if d036(location) != row["intervals"]:
                problems.append(name + ": intervals %s, Biopython %s" % (row["intervals"], d036(location)))
            strand = "plus" if location.strand == 1 else "minus"
            if strand != row["strand"]:
                problems.append(name + ": strand " + row["strand"])
            if identity(feature)[1] != row["id_qualifier"]:
                problems.append(name + ": id qualifier " + row["id_qualifier"])
            table = qualifiers.get("transl_table", ["-"])[0]
            if table != row["transl_table"]:
                problems.append(name + ": transl_table " + row["transl_table"])
            if str(location.extract(SEQ))[:3] != "ATG":
                problems.append(name + ": imported without an ATG start")
            stated = qualifiers.get("translation", [None])[0]
            if stated is not None and stated != row["residues"]:
                problems.append(name + ": /translation differs")
        elif rule == "alternative_start":
            start = str(location.extract(SEQ))[:3]
            if start == "ATG" or start != row["detail"]:
                problems.append(name + ": start is not " + row["detail"])
        elif rule in ("codon_start", "transl_table"):
            if qualifiers.get(rule) != [row["detail"]]:
                problems.append(name + ": /%s is not %s" % (rule, row["detail"]))
        elif rule == "pseudo":
            if "pseudo" not in qualifiers:
                problems.append(name + ": no /pseudo")
        elif rule == "repeated_qualifier":
            if len(qualifiers.get(row["detail"], [])) < 2:
                problems.append(name + ": /%s is not repeated" % row["detail"])
        elif error == "genbank_translation_mismatch":
            stated = qualifiers["translation"][0]
            if first_difference(stated, row["residues"]) != row["detail"]:
                problems.append(name + ": first difference is not " + row["detail"])
        elif rule == "fuzzy_end":
            ends = [location.start, location.end]
            if not any(isinstance(p, (BeforePosition, AfterPosition)) for p in ends):
                problems.append(name + ": no fuzzy end")
        elif rule == "order":
            if not (isinstance(location, CompoundLocation) and location.operator == "order"):
                problems.append(name + ": not order()")
        elif rule == "unsupported_join":
            parts = sorted((int(p.start), int(p.end)) for p in location.parts)
            crossing = len(parts) == 2 and parts[0][0] == 0 and parts[1][1] == N
            if not (isinstance(location, CompoundLocation) and location.operator == "join") or crossing:
                problems.append(name + ": not a join away from the origin")
        elif rule == "single_position":
            if len(location) != 1:
                problems.append(name + ": not a single base")
        elif rule == "remote_reference":
            if location.ref is None:
                problems.append(name + ": no reference to another record")
        elif error == "cds_length_not_divisible_by_three":
            if str(len(location)) != row["detail"] or len(location) % 3 == 0:
                problems.append(name + ": length is not " + row["detail"])
        elif error == "internal_stop_codon":
            protein = str(location.extract(SEQ).translate(table=1))
            if protein.find("*") + 1 != int(row["detail"]) or protein.find("*") == len(protein) - 1:
                problems.append(name + ": first internal stop is not codon " + row["detail"])
        else:
            problems.append(name + ": no check for %s %s" % (error, rule))
    return problems


def check_unused():
    problems = []
    table = rows("synthetic-unused.tsv")
    if len(table) != len(record.features):
        return ["%d rows, %d features" % (len(table), len(record.features))]
    for row, feature in zip(table, record.features):
        if row["key"] != feature.type:
            problems.append("feature %s: key %s, Biopython %s" % (row["index"], row["key"], feature.type))
        expected_id = identity(feature)[0] if feature.type == "CDS" else None
        if (expected_id or "-") != row["id"]:
            problems.append("feature %s: id %s, Biopython %s" % (row["index"], row["id"], expected_id))
        if row["refusal"] == "unusable_genbank_cds:no_id" and expected_id is not None:
            problems.append("feature %s: has an id" % row["index"])
        if row["refusal"] == "invalid_identifier_characters" and " " not in (expected_id or ""):
            problems.append("feature %s: id has no blank" % row["index"])
        if row["refusal"] == "unusable_genbank_cds:ambiguous_name" and len(only(expected_id)) < 2:
            problems.append("feature %s: id is not shared" % row["index"])
        if row["refusal"] == "unusable_genbank_cds:alternative_start" and str(feature.location.extract(SEQ))[:3] == "ATG":
            problems.append("feature %s: starts with ATG" % row["index"])
    return problems


def main():
    failed = False
    for name, check in (("synthetic-expected.tsv", check_expected), ("synthetic-unused.tsv", check_unused)):
        problems = check()
        if problems:
            failed = True
            print("DIFFERS %s: %s" % (name, problems[0]))
        else:
            print("OK " + name)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
