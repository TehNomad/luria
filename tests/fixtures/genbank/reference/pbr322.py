"""Write the pBR322 cross-check tables from Biopython's GenBank parser (D039).

Source: NCBI nuccore J01749.1, "Cloning vector pBR322, complete sequence",
fetched once on 2026-10-17 with

    https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nuccore&id=J01749.1&rettype=gb&retmode=text

and saved unchanged as ../raw/J01749.1.gb (SHA-256
7660b0fe95a91d492b102192dc72924508a69fda762988aa0923b3d5cc2073f6, checked
below). This script reads only that file; it contacts nothing.

Biopython's parser shares no code with `crates/core`, so these tables are an
independent reading of the record for `crates/core/tests/genbank_fixtures.rs`
to compare luria's reader and CDS import against:

* ../pbr322-record.tsv: the LOCUS name, the first ACCESSION, the VERSION,
  the length, the topology, the SHA-256 of the upper-cased sequence (luria's
  canonical text, D007), and the number of features.
* ../pbr322-features.tsv: every feature in file order, with its key and, for
  a CDS, the id luria would use (D041 G2: the first of /locus_tag, /gene,
  /label) and the qualifier it came from, or `-`.
* ../pbr322-cds.tsv: every CDS, with its interval converted to luria's
  zero-based half-open form (Biopython's FeatureLocation already is), its
  strand, /codon_start, /transl_table, its first codon, its residue count
  under the standard code without the terminal stop, and whether that
  translation equals the record's /translation.

The standard-code translation is computed here with Biopython (table 1,
terminal stop removed), not with luria.

Run from the repository root with the pinned environment
(../../requirements.txt) and `python -I`:

    python -I tests/fixtures/genbank/reference/pbr322.py
    python -I tests/fixtures/genbank/reference/pbr322.py --check

Without --check, writes the three tables. With --check, regenerates them in
memory, compares each byte for byte with its tracked file, prints `OK <name>`
or `DIFFERS <name>` followed by the first differing line, exits 1 on any
difference, and writes nothing (D039).
"""
import hashlib
import os
import sys

from Bio import SeqIO

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)
RAW = os.path.join(FIXTURE, "raw", "J01749.1.gb")
RAW_SHA256 = "7660b0fe95a91d492b102192dc72924508a69fda762988aa0923b3d5cc2073f6"

with open(RAW, "rb") as f:
    raw = f.read()
assert hashlib.sha256(raw).hexdigest() == RAW_SHA256, "raw/J01749.1.gb has changed"

record = SeqIO.read(RAW, "genbank")
sequence = str(record.seq).upper()


def one(feature, name):
    """The single value of a qualifier, or None; refuse a repeated one."""
    values = feature.qualifiers.get(name, [])
    assert len(values) <= 1, f"{feature.type} repeats /{name}"
    return values[0] if values else None


def identity(feature):
    """D041 G2: the first of /locus_tag, /gene, /label, and where it came from."""
    for name in ("locus_tag", "gene", "label"):
        value = one(feature, name)
        if value is not None:
            return value, name
    return "-", "-"


def render_record():
    rows = [["locus_name", "accession", "version", "length", "topology", "sha256", "features"]]
    rows.append([
        record.name,
        record.annotations["accessions"][0],
        record.id,
        str(len(sequence)),
        record.annotations["topology"],
        hashlib.sha256(sequence.encode("ascii")).hexdigest(),
        str(len(record.features)),
    ])
    return rows


def render_features():
    rows = [["index", "key", "id", "id_source"]]
    for index, feature in enumerate(record.features):
        cds_id, source = identity(feature) if feature.type == "CDS" else ("-", "-")
        rows.append([str(index), feature.type, cds_id, source])
    return rows


def render_cds():
    rows = [[
        "index", "id", "id_source", "start", "end", "strand", "codon_start",
        "transl_table", "start_codon", "residues", "translation_matches",
    ]]
    for index, feature in enumerate(record.features):
        if feature.type != "CDS":
            continue
        location = feature.location
        # pBR322's CDSs are single ranges; a join would need D036 ordering.
        assert len(location.parts) == 1, f"CDS {index} is a join"
        coding = str(feature.extract(record.seq)).upper()
        protein = str(record.seq.__class__(coding).translate(table=1))
        if protein.endswith("*"):
            protein = protein[:-1]
        stated = one(feature, "translation")
        matches = "-" if stated is None else ("yes" if stated == protein else "no")
        cds_id, source = identity(feature)
        rows.append([
            str(index),
            cds_id,
            source,
            str(int(location.start)),
            str(int(location.end)),
            "+" if location.strand == 1 else "-",
            one(feature, "codon_start") or "-",
            one(feature, "transl_table") or "-",
            coding[:3],
            str(len(protein)),
            matches,
        ])
    return rows


def text_of(rows):
    return "".join("\t".join(row) + "\n" for row in rows)


TABLES = {
    "pbr322-record.tsv": render_record,
    "pbr322-features.tsv": render_features,
    "pbr322-cds.tsv": render_cds,
}


def main():
    check = "--check" in sys.argv[1:]
    failed = False
    for name, render in TABLES.items():
        text = text_of(render())
        path = os.path.join(FIXTURE, name)
        if not check:
            with open(path, "w", encoding="ascii", newline="") as f:
                f.write(text)
            continue
        with open(path, "r", encoding="ascii", newline="") as f:
            tracked = f.read()
        if tracked == text:
            print("OK " + name)
            continue
        failed = True
        for number, (want, have) in enumerate(zip(text.splitlines(), tracked.splitlines()), 1):
            if want != have:
                print("DIFFERS %s line %d: %r != %r" % (name, number, have, want))
                break
        else:
            print("DIFFERS %s: line counts differ" % name)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
