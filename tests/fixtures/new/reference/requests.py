#!/usr/bin/env python3
"""Expected requests built by `luria new` from a filled form (plan 7k, M7-32 to M7-37).

Writes, under ../:

* request-<case>.json  the `luria.q5-request/1` document `new::build`
                       (contract 7k2) must produce for each form below,
                       compared as JSON values (key order free);
* paste-<case>.fa      for a pasted template, the FASTA file luria writes;
* refusals.tsv         forms that must be refused, with the refusal code.

Independent of crates/: templates are read here, each mutation line is read
by ./shorthand.py, and every rule below is the plan's, not luria's code.

Rules:

* FASTA file or paste: `template` is `{id, fasta, topology, sha256, cds}`.
  `id` is the form's name; `fasta` is the file's name only (the request is
  written beside it, M7-37); `sha256` is the SHA-256 of the bases, upper
  case, no whitespace. `cds` is one CDS whose id is the template's id, with
  `intervals` `[[first - 1, last]]` from the form's 1-based inclusive bases,
  or `[[0, length]]` for "whole sequence", the form's strand (`+` or `-`),
  `completeness` `complete`, `geneticCode` `standard`, `numbering`
  `from-cds-start`.
* GenBank file: `template` is `{id, genbank, sha256, cds: [<chosen id>]}`;
  `id` is the LOCUS name.
* `scope` is the CDS's id; a residue edit names it as its `cds`.
* One mutation line gives `edit`; two or more give `variants` with
  `composition` `separate_variants` (M7-34), members in line order.
* Each line is read by shorthand.py. A substitution's or replacement's
  typed `from` must equal the template's bases there; a deletion's `from`,
  and an insertion's `before` and `after`, are read from the template.
* A residue's codon is the line's own (`D404N:AAC`) when it has one;
  otherwise the form's codon choice must be "default policy", giving
  `"policy": "default"`.
* `design` is `{policy: luriaNative, style: q5_polymerase,
  minAnnealingOverlap, placement: inline, buffer: q5_reaction}`, the
  overlap from the form; placement is always `inline`, shown fixed on the
  form (M7-34, M7-39: `q5 design` refuses `confine_to_tail`, D029).
* Member names: a residue `D404N`; one base `G1210A`; a range `GAT1210-1212AAC`
  when both base texts are at most 10 bases, else `rep1210-1212`; a
  deletion `del1210-1212`, or `del46` for one base; an insertion
  `ins1209GAT` when the insert is at most 10 bases, else `ins1209+210`.
  Positions in names are the 1-based ones the user typed.

Refusal codes (refusals.tsv): `no_mutations`, `line_refused` (shorthand.py
refused a line), `from_mismatch`, `outside_template` (a position past the
template's end, or a CDS outside it), `insertion_at_end` (core refuses an
insertion at either end, D032), `missing_codon`,
`missing_overlap`, `duplicate_name`. Columns: case_id, code, line (the
1-based mutation line that is refused, or `-`).

Run from the repository root:

    python3 -I tests/fixtures/new/reference/requests.py           rewrite
    python3 -I tests/fixtures/new/reference/requests.py --check   compare
"""
import hashlib
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.dirname(HERE)
FIXTURES = os.path.dirname(OUT)
sys.path.insert(0, HERE)
import shorthand  # noqa: E402  this directory's own script

SID1 = "example-construct/input/sid-1_cDNA.fa"
SYNLIN = "genbank/synthetic-linear.gb"
SYN = ("ATGGCTTGGAATCAGCAGCAGCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCAGCCGAG"
       "CGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA")


class Refused(Exception):
    def __init__(self, code, line="-"):
        self.code = code
        self.line = line


def fasta_bases(relative):
    with open(os.path.join(FIXTURES, relative), encoding="ascii") as f:
        lines = f.read().splitlines()
    assert lines[0].startswith(">")
    return "".join(line.strip() for line in lines[1:]).upper()


def genbank(relative):
    with open(os.path.join(FIXTURES, relative), encoding="ascii") as f:
        text = f.read()
    locus = text.splitlines()[0].split()[1]
    origin = text.split("\nORIGIN", 1)[1].split("\n//", 1)[0]
    return locus, re.sub(r"[^A-Za-z]", "", origin).upper()


def digest(bases):
    return hashlib.sha256(bases.encode("ascii")).hexdigest()


def render_fasta(name, bases):
    rows = [bases[i:i + 60] for i in range(0, len(bases), 60)]
    return ">%s\n%s\n" % (name, "\n".join(rows))


def short(text):
    return len(text) <= 10


def member(number, line, bases, cds_id, codon_choice):
    reading = shorthand.read(line)
    kind, start, end, frm, to, codon = reading
    if kind == "refused":
        raise Refused("line_refused", number)
    n = len(bases)
    if kind == "residue":
        if codon != "policy":
            choice = {"codon": codon}
        elif codon_choice == "default":
            choice = {"policy": "default"}
        else:
            raise Refused("missing_codon", number)
        edit = {"cds": cds_id, "number": int(start), "from": frm, "to": to}
        edit.update(choice)
        return "%s%s%s" % (frm, start, to), {"residue": edit}
    s, e = int(start), int(end)
    if e > n or s > n:
        raise Refused("outside_template", number)
    if kind in ("substitution", "replacement"):
        if bases[s:e] != frm:
            raise Refused("from_mismatch", number)
        if e - s == 1 and len(to) == 1:
            name = "%s%d%s" % (frm, e, to)
        elif short(frm) and short(to):
            name = "%s%d-%d%s" % (frm, s + 1, e, to)
        else:
            name = "rep%d-%d" % (s + 1, e)
        return name, {kind: {"interval": [s, e], "from": frm, "to": to}}
    if kind == "deletion":
        name = "del%d" % e if e - s == 1 else "del%d-%d" % (s + 1, e)
        return name, {"deletion": {"interval": [s, e], "from": bases[s:e]}}
    assert kind == "insertion"
    if s == 0 or s == n:
        raise Refused("insertion_at_end", number)
    name = "ins%d%s" % (s, to) if short(to) else "ins%d+%d" % (s, len(to))
    return name, {"insertion": {"point": s, "before": bases[s - 1],
                                "after": bases[s], "insert": to}}


def build(form):
    """The request for `form`, or Refused."""
    lines = [line for line in form["lines"] if line.strip() != ""]
    if not lines:
        raise Refused("no_mutations")
    if form.get("overlap") is None:
        raise Refused("missing_overlap")

    source = form["template"]
    if source["kind"] == "genbank":
        tid, bases = genbank(source["file"])
        cds_id = source["cds"]
        template = {"id": tid, "genbank": os.path.basename(source["file"]),
                    "sha256": digest(bases), "cds": [cds_id]}
    else:
        bases = (fasta_bases(source["file"]) if source["kind"] == "fasta"
                 else "".join(source["paste"].split()).upper())
        tid = source["name"]
        cds_id = tid
        if source["cds"] == "whole":
            interval = [0, len(bases)]
        else:
            first, last = source["cds"]
            if first < 1 or last > len(bases) or first > last:
                raise Refused("outside_template")
            interval = [first - 1, last]
        fasta = (os.path.basename(source["file"]) if source["kind"] == "fasta"
                 else source["save_as"])
        template = {"id": tid, "fasta": fasta, "topology": source["topology"],
                    "sha256": digest(bases),
                    "cds": [{"id": tid, "intervals": [interval],
                             "strand": source["strand"],
                             "completeness": "complete",
                             "geneticCode": "standard",
                             "numbering": "from-cds-start"}]}

    members = []
    for number, line in enumerate(form["lines"], start=1):
        if line.strip() == "":
            continue
        name, edit = member(number, line, bases, cds_id, form["codon"])
        if any(m["name"] == name for m in members):
            raise Refused("duplicate_name", number)
        entry = {"name": name}
        entry.update(edit)
        members.append(entry)

    request = {"schema": "luria.q5-request/1",
               "coordinates": "zero-based-half-open",
               "template": template, "scope": cds_id}
    if len(members) == 1:
        request["edit"] = members[0]
    else:
        request["variants"] = {"composition": "separate_variants",
                               "members": members}
    request["design"] = {"policy": "luriaNative", "style": "q5_polymerase",
                         "minAnnealingOverlap": form["overlap"],
                         "placement": "inline",
                         "buffer": "q5_reaction"}
    paste = None
    if source["kind"] == "paste":
        paste = (source["save_as"], render_fasta(tid, bases))
    return request, paste


def sid1(**over):
    form = {"template": {"kind": "fasta", "file": SID1, "name": "sid-1",
                         "topology": "linear", "cds": "whole", "strand": "+"},
            "lines": ["D404N"], "codon": "default", "overlap": 15}
    form.update(over)
    return form


def syn_paste(**over):
    form = {"template": {"kind": "paste", "paste": SYN[:50] + "\n  " + SYN[50:].lower(),
                         "name": "syn", "save_as": "syn.fa",
                         "topology": "linear", "cds": (1, 99), "strand": "+"},
            "lines": ["del 46", "51-53 ATC>TT"], "codon": "typed",
            "overlap": 15}
    form.update(over)
    return form


BUILT = [
    ("sid1-codon", sid1(lines=["D404N:AAC"], codon="typed")),
    ("sid1-set", sid1(lines=["D404N", "del 1210-1212", "", "ins GAT after 1209",
                             "1210 G>A", "1210-1212 GAT>AAC"])),
    ("synlin-genbank", {"template": {"kind": "genbank", "file": SYNLIN, "cds": "synA"},
                        "lines": ["A42G", "W87A"], "codon": "default",
                        "overlap": 20}),
    ("syn-paste", syn_paste()),
    ("syn-minus", syn_paste(template={"kind": "paste", "paste": SYN, "name": "syn",
                                      "save_as": "syn-minus.fa", "topology": "linear",
                                      "cds": "whole", "strand": "-"},
                            lines=["ins GATTACAGATTA after 50"])),
]

REFUSED = [
    ("no-lines", sid1(lines=["", "  "])),
    ("bad-line", sid1(lines=["D404N", "D404*"])),
    ("wrong-from", sid1(lines=["1210 A>G"])),
    ("past-end", sid1(lines=["del 3150-3154"])),
    ("insert-at-end", sid1(lines=["ins GAT after 3153"])),
    ("insert-at-start", sid1(lines=["ins GAT after 0"])),
    ("no-codon", sid1(codon="typed")),
    ("no-overlap", sid1(overlap=None)),
    ("same-twice", sid1(lines=["D404N", "d404n"])),
    ("cds-outside", syn_paste(template=dict(syn_paste()["template"], cds=(1, 101)))),
]


def outputs():
    files = {}
    for case, form in BUILT:
        request, paste = build(form)
        files["request-%s.json" % case] = json.dumps(request, indent=2) + "\n"
        if paste:
            files["paste-%s.fa" % case] = paste[1]
    rows = ["case_id\tcode\tline"]
    for case, form in REFUSED:
        try:
            build(form)
        except Refused as refusal:
            rows.append("%s\t%s\t%s" % (case, refusal.code, refusal.line))
        else:
            raise SystemExit("%s was not refused" % case)
    files["refusals.tsv"] = "".join(row + "\n" for row in rows)
    return files


def main():
    ok = True
    for name, text in outputs().items():
        path = os.path.join(OUT, name)
        if "--check" not in sys.argv[1:]:
            with open(path, "w", encoding="utf-8") as f:
                f.write(text)
            print("wrote", name)
            continue
        try:
            with open(path, encoding="utf-8") as f:
                tracked = f.read()
        except FileNotFoundError:
            tracked = None
        print(("OK " if tracked == text else "DIFFERS ") + name)
        ok = ok and tracked == text
    if not ok:
        sys.exit(1)


if __name__ == "__main__":
    main()
