#!/usr/bin/env python3
"""Print the unit-test expectations that native-design.py's --synthetic misses.

D039. `native-design.py`'s `--synthetic` mode covers only the synthetic
templates in its SYNTHETIC, so the expectations hard-coded in the Rust unit
tests below had no tracked driver:

- `crates/core/src/primer_design.rs`'s `TEMPLATE` (the 60-base T60 below) and
  the edits its tests make on it;
- `crates/core/src/q5_design.rs`'s 90-base template (the 90-base T90 below) and
  its edits;
- the figures near `primer_design.rs` line 2097, which come from
  `native-replacement.py`'s `design()` on the two templates those tests build.

This script loads those two generators from its own directory and prints, for
each case, the trace and the design or failure the Rust test asserts. It writes
nothing, so it has no `--check`. Run from the repository root:

    python -I tests/fixtures/q5-primer/reference/unit-expectations.py

The Tm figures are printed to two decimals, which is the tolerance the Rust
tests use (`close`, 0.006 °C). Spans are `start:end`, 0-based half-open, as the
generators report them.
"""
import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))


def load(name):
    """Load one of the generators that sits next to this script."""
    path = os.path.join(HERE, name + ".py")
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


nd = load("native-design")
nr = load("native-replacement")

# primer_design.rs's TEMPLATE, and q5_design.rs's template_text(): ATG 14 times,
# GCT, ATG 15 times. Residue 15 sits at 42, residue 20 at 57, residue 30 at 87.
T60 = "ATGGCTTGGAATCAGCAGCAGTTTAAACCCGGGATCGATCGTACGTACGGCATGCATGCA"
T90 = "ATG" * 14 + "GCT" + "ATG" * 15

# D030 cases: design(template, start, replacement, bound).
cases = {
    "pd nuc30 A b15": (T60, 30, "A", 15),
    "pd nuc35 A b21": (T60, 35, "A", 21),
    "pd nuc30 A b6": (T60, 30, "A", 6),
    "pd codon30 TGG b15": (T60, 30, "TGG", 15),
    "pd minus codon30 CCA b15": (T60, 30, "CCA", 15),
    "pd nuc25 G b15": (T60, 25, "G", 15),
    "pd nuc24 G b15": (T60, 24, "G", 15),
    "pd nuc49 A b15": (T60, 49, "A", 15),
    "pd nuc50 A b15": (T60, 50, "A", 15),
    "pd short35 nuc25 G": (T60[:35], 25, "G", 15),
    "pd ambig10 nuc30": (T60[:10] + "N" + T60[11:], 30, "A", 15),
    "qd res15 TGG": (T90, 42, "TGG", 15),
    "qd nuc45 C": (T90, 45, "C", 15),
    "qd res20 TGG": (T90, 57, "TGG", 15),
    "qd res30 TGG": (T90, 87, "TGG", 15),
}

# D033 cases: design(template, start, end, replacement, bound). The templates
# are the ones the two Rust tests build: primer_design.rs's balanced() from the
# generator's own blocks, and its 110-base template of MIX, GC, ATGGCTTGGA,
# twenty G, GC, MIX.
balanced = nr.SYNTHETIC["balanced"]
unequal_inline = nr.MIX + nr.GC + "ATGGCTTGGA" + "G" * 20 + nr.GC + nr.MIX
replacements = {
    "pd repl50:57 T b15": (balanced, 50, 57, "T", 15),
    "pd repl50:52 C b15": (unequal_inline, 50, 52, "C", 15),
}

for name, (t, s, rep, b) in cases.items():
    out, trace = nd.design(t, s, rep, b)
    print(f"{name:26} {sorted(trace)}\n    {out}")

for name, (t, s, e, rep, b) in replacements.items():
    out, trace = nr.design(t, s, e, rep, b)
    print(f"{name:26} {sorted(trace)}\n    {out}")
