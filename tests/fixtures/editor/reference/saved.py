"""Build the two pBR322 records a save writes (D039, contract 9c3 §5).

../pbr322-saved.gb and ../pbr322-substituted.gb are what
`luria_core::genbank_write::write_genbank` (contract 9c3) writes for pBR322
opened from `../../genbank/raw/J01749.1.gb`: once with nothing edited, and
once after one `Substitute` of base 100. The Rust tests in
`crates/core/tests/genbank_write_fixtures.rs` compare the writer's output
with these two files byte for byte, so every rule of contract 9c3 §4 that
fires on pBR322 is pinned here: the LOCUS line's new date, the VERSION line
dropped, the COMMENT line luria adds, every feature kept exactly as the file
wrote it, and the ORIGIN bases written lower-case, 60 to a line.

Neither file comes from running luria. Each is the source file, read as
text, with these plain text operations:

  pbr322-saved.gb — the source with five changes:
    * the LOCUS line's date set to 29-OCT-2026;
    * the VERSION line (source line 5) removed, with its continuation lines;
    * `            Edited with luria 9.9.9: 0 edits.` inserted as the last
      line of the COMMENT block, which is the last line before FEATURES;
    * the ORIGIN line's trailing spaces removed (source line 399);
    * line ends made `\n`, and the one blank line the source ends with
      dropped, so the record ends with `//` and one line end.

  pbr322-substituted.gb — the same again, with:
    * the comment saying `1 edit.`;
    * base 100 (zero-based 99) changed in the ORIGIN block to `a`, the base
      there being `t`, in the same case and the same place within its line;
    * the tet CDS's `/translation` lines removed, from the `/translation=`
      line to the line where its quoted value closes: the substitution
      touched that CDS's bases, and M9-6 (h) drops a stale translation.

The tet CDS (86..1276) covers base 100, so it is the one feature the
substitution moves off the file's own text: every other feature is written
as the file wrote it, and that CDS is written fresh from the buffer with the
same location text and the same qualifiers less the translation.

The check mode regenerates both files in memory and compares them byte for
byte with the tracked files, writing nothing. Both modes then read all three
records with Biopython 1.88 and assert what the files claim: the same
feature keys and locations as the source, the same qualifiers apart from the
one removed `/translation`, and the source's sequence with only base 100
changed in the substituted file. The script shares no code with `crates/`,
never runs luria, and its only output is the two lines it prints.

Run from the repository root with the pinned environment:

    python3 -I tests/fixtures/editor/reference/saved.py
    python3 -I tests/fixtures/editor/reference/saved.py --check

Both forms print `OK pbr322-saved.gb (<n> lines)` and `OK
pbr322-substituted.gb (<n> lines)`, and exit 0, or print `DIFFERS <name>`
with the first differing line and exit 1.
"""
import io
import os
import re
import sys

from Bio import SeqIO

HERE = os.path.dirname(os.path.abspath(__file__))
SOURCE = os.path.join(HERE, "..", "..", "genbank", "raw", "J01749.1.gb")
OUTPUTS = [
    ("pbr322-saved.gb", os.path.join(HERE, "..", "pbr322-saved.gb")),
    ("pbr322-substituted.gb", os.path.join(HERE, "..", "pbr322-substituted.gb")),
]

# What the stamp carries: contract 9c3 §4's SaveStamp, fixed by the tests.
DATE = "29-OCT-2026"
LURIA_VERSION = "9.9.9"
# The one edit of the substituted file, zero-based (D006): base 100.
BASE_INDEX = 99
DATE_IN_LOCUS = re.compile(r"\d{2}-[A-Z]{3}-\d{4}")


def source_lines():
    """The source file's lines, as the writer sees a record: line ends made
    `\n`, and the blank line the file ends with dropped so the last line is
    the `//` that ends the record."""
    with open(SOURCE, encoding="utf-8", newline="") as handle:
        text = handle.read()
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    lines = text.split("\n")
    while lines and lines[-1] == "":
        lines.pop()
    assert lines and lines[-1] == "//", "the source record does not end with //"
    return lines


def line_at(lines, start):
    """The index of the first line starting with `start` at column 0, which
    is how a section line looks: every header, FEATURES and ORIGIN line."""
    found = [index for index, line in enumerate(lines) if line.startswith(start)]
    assert found, "no line starts with %r" % start
    return found[0]


def new_locus(line):
    """The LOCUS line with its date replaced: the only part of the line a
    save changes (M9-11, M9-15)."""
    new, count = DATE_IN_LOCUS.subn(DATE, line, count=1)
    assert count == 1, "the LOCUS line holds no date: %r" % line
    return new


def add_comment(lines, edits):
    """The line the save adds after the last line of the last COMMENT block,
    which for these records is the last header line (M9-11, M9-15)."""
    word = "edit" if edits == 1 else "edits"
    added = "            Edited with luria %s: %d %s." % (LURIA_VERSION, edits, word)
    at = line_at(lines, "FEATURES")
    assert line_at(lines, "COMMENT") < at, "the COMMENT block is not the last header"
    lines.insert(at, added)
    return lines


def saved(lines, edits):
    """The record as an unedited save writes it: five changes, every other
    line of the source untouched."""
    lines = list(lines)
    lines[0] = new_locus(lines[0])
    version = line_at(lines, "VERSION")
    drop = [version]
    while drop[-1] + 1 < len(lines) and lines[drop[-1] + 1].startswith(" "):
        drop.append(drop[-1] + 1)
    for index in reversed(drop):
        del lines[index]
    lines = add_comment(lines, edits)
    origin = line_at(lines, "ORIGIN")
    lines[origin] = lines[origin].rstrip()
    return lines


def new_base(old):
    """What base 100 becomes: `a` unless it is already `a`, and then `c`
    (§5), written in the case the file wrote it in."""
    letter = "c" if old in ("a", "A") else "a"
    return letter.upper() if old.isupper() else letter


def substitute_base(lines, index):
    """Change the base at zero-based `index` in the ORIGIN block by §5's rule,
    in place: the same line, the same group, the same column, the same case.
    It becomes `a` if it is not `a`, and `c` if it is. Returns the base that
    was there."""
    origin = line_at(lines, "ORIGIN")
    seen = 0
    for position in range(origin + 1, len(lines)):
        line = lines[position]
        if line == "//":
            break
        # The standard form: the line's first base, right-aligned in 9, one
        # space, then the 10-base groups one space apart. No ORIGIN line of
        # this record has a trailing space.
        head, separator, body = line[:9], line[9], line[10:]
        assert separator == " ", "ORIGIN line %r has no column 10" % line
        groups = body.split(" ")
        assert "%9d %s" % (int(head), " ".join(groups)) == line, (
            "ORIGIN line is not in the standard form: %r" % line)
        assert int(head) == seen + 1, "ORIGIN line numbering jumps at %r" % line
        for group, bases in enumerate(groups):
            if seen <= index < seen + len(bases):
                column = 10 + 11 * group + (index - seen)
                old = line[column]
                new = new_base(old)
                assert old != new, "base %d changes to itself" % (index + 1)
                lines[position] = line[:column] + new + line[column + 1:]
                assert len(lines[position]) == len(line)
                return old
            seen += len(bases)
    raise AssertionError("no ORIGIN base at %d" % (index + 1))


def drop_translation(lines):
    """Remove the `/translation` block of the tet CDS: from its
    `/translation=` line to the line its quoted value closes.

    The reader's own rule for where a quoted value closes (contract 5c's
    §4.6, `genbank.rs`): the value is open while the quotes it holds number
    odd, and a doubled `""` inside it counts as two, so the line closing it
    is the one that makes the count even.
    """
    start = [
        index for index, line in enumerate(lines) if line.strip().startswith("/translation=")
    ][0]
    key = [
        index for index in range(start, -1, -1)
        if lines[index].startswith("     ") and not lines[index].startswith("      ")
    ][0]
    assert lines[key] == "     CDS             86..1276", (
        "the first /translation is not the tet CDS's: %r" % lines[key])
    end = start
    quotes = 0
    while True:
        quotes += lines[end].count('"')
        if quotes % 2 == 0:
            break
        end += 1
    del lines[start:end + 1]
    return lines


def text_of(lines):
    return "\n".join(lines) + "\n"


def both_texts():
    """The two records, as bytes-text: (saved, substituted)."""
    original = source_lines()
    saved_record = saved(original, 0)
    substituted_record = list(saved(original, 1))
    old = substitute_base(substituted_record, BASE_INDEX)
    substituted_record = drop_translation(substituted_record)
    # What §5 says of pBR322: its base 100 is a `t`, so the new base is `a`.
    assert old == "t" and new_base(old) == "a", "base 100 of the source is " + old
    return text_of(saved_record), text_of(substituted_record)


def read(text):
    return SeqIO.read(io.StringIO(text), "genbank")


def every_feature(record):
    """One record's features as Biopython reads them: key, location, and the
    qualifiers as a dict in file order."""
    return [
        (feature.type, feature.location, dict(feature.qualifiers))
        for feature in record.features
    ]


def tetracycline(feature):
    """True when Biopython reads pBR322's tet CDS, the one the source writes
    at 86..1276: Biopython's own numbering is 0-based, so its start there is
    85 and its end 1276."""
    return feature[0] == "CDS" and int(feature[1].start) == 85 and int(feature[1].end) == 1276


def check_claims(saved_text, substituted_text):
    """What the two files claim, asked a second time with Biopython alone
    (§5): the same feature keys and locations as the source, the same
    qualifiers less the one removed `/translation`, and the source's sequence
    with only base 100 changed in the substituted file."""
    original = read(open(SOURCE, encoding="utf-8").read())
    wanted = every_feature(original)
    # The features the substitution touches, and so the ones written fresh:
    # base 100 (zero-based 99) belongs to them. M9-6 (h) drops a stale
    # /translation from each.
    touched = [
        number for number, feature in enumerate(wanted, 1)
        if feature[0] == "CDS"
        and int(feature[1].start) <= BASE_INDEX < int(feature[1].end)
    ]
    assert touched and all(tetracycline(wanted[n - 1]) for n in touched), (
        "the CDS covering base 100 is not the tet CDS: %s" % touched)
    for name, text, dropped in (
        ("pbr322-saved.gb", saved_text, False),
        ("pbr322-substituted.gb", substituted_text, True),
    ):
        found = every_feature(read(text))
        assert len(found) == len(wanted), (
            "%s: %d features read, the source has %d" % (name, len(found), len(wanted)))
        for number, (have, want) in enumerate(zip(found, wanted), 1):
            keys = list(want[2])
            if dropped and number in touched:
                assert "translation" in keys, "the source's tet CDS has no translation"
                keys.remove("translation")
            assert have[0] == want[0], (
                "%s: feature %d is %s, the source has %s"
                % (name, number, have[0], want[0]))
            assert str(have[1]) == str(want[1]), (
                "%s: feature %d %s at %s, the source has %s"
                % (name, number, have[0], have[1], want[1]))
            assert list(have[2]) == keys, (
                "%s: feature %d %s has %s, expected %s"
                % (name, number, have[0], sorted(have[2]), keys))
            for key in keys:
                assert have[2][key] == want[2][key], (
                    "%s: feature %d %s /%s is %r, the source has %r"
                    % (name, number, have[0], key, have[2][key], want[2][key]))
            if dropped and have[0] == "CDS":
                assert ("translation" in have[2]) == (number not in touched), (
                    "%s: feature %d %s keeps or loses its translation wrongly"
                    % (name, number, have[0]))

    saved_record = read(saved_text)
    assert str(saved_record.seq) == str(original.seq), "the saved record's sequence"
    substituted = read(substituted_text)
    assert len(substituted.seq) == len(original.seq), "the sequences are not of one length"
    changed = [
        number for number in range(len(original.seq))
        if substituted.seq[number] != original.seq[number]
    ]
    assert changed == [BASE_INDEX], "bases changed: %s" % [n + 1 for n in changed]


def check_tracked(name, path, text):
    """Regenerated vs the tracked file, byte for byte (§5)."""
    with open(path, encoding="utf-8", newline="") as handle:
        tracked = handle.read()
    if tracked == text:
        print("OK %s (%d lines)" % (name, text.count("\n")))
        return True
    print("DIFFERS %s" % name)
    tracked_lines = tracked.splitlines()
    want_lines = text.splitlines()
    for number, (have, want) in enumerate(zip(tracked_lines, want_lines), 1):
        if have != want:
            print("  line %d:\n    tracked:   %r\n    generated: %r"
                  % (number, have, want))
            break
    else:
        print("  %d tracked line(s), %d generated"
              % (len(tracked_lines), len(want_lines)))
    return False


def main(argv):
    saved_text, substituted_text = both_texts()
    check_claims(saved_text, substituted_text)
    check = "--check" in argv
    failed = False
    for name, path in OUTPUTS:
        text = saved_text if name == "pbr322-saved.gb" else substituted_text
        if check:
            if not check_tracked(name, path, text):
                failed = True
        else:
            with open(path, "w", encoding="utf-8", newline="") as handle:
                handle.write(text)
            print("wrote %s (%d lines)" % (name, text.count("\n")))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
