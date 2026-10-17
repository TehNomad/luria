"""Run every fixture check in one command (D039).

Every script that produced or verified a fixture expectation has a `--check`
mode: it regenerates its tracked file in memory, compares it byte for byte,
prints `OK <name>` or `DIFFERS <name>` with the first differing line, exits 1
on a difference, and writes nothing. This script runs all of them and prints
`PASS` or `FAIL` and the name of each; it exits 1 if any check fails, and
prints the failing script's own output so the difference is visible.

Run from the repository root with the pinned environment
(`tests/fixtures/requirements.txt`):

    python -m venv .venv && .venv/bin/activate
    pip install -r tests/fixtures/requirements.txt
    python -I tests/fixtures/verify.py

Each check runs as `sys.executable -I <script> --check`, so the interpreter is
the one that carries the pinned dependencies.

These Python checks are not part of the required cargo checks (D039, D026): the
Rust tests do not depend on Python, and the cargo checks do not run these. Run
them when a fixture, or one of its sources, changes.
"""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)  # tests/fixtures/ -> the repository root

# Every generator and verifier under tests/fixtures/*/reference/ that writes a
# tracked file, as it is named on the command line. `unit-expectations.py` is
# absent because it prints values and writes no tracked file, so it has no
# --check mode (D039).
CHECKS = [
    "tm/reference/verify_imm.py",
    "tm/reference/verify_matched.py",
    "tm/reference/crosscheck.py",
    "q5-primer/reference/native-design.py",
    "q5-primer/reference/native-deletion.py",
    "q5-primer/reference/native-insertion.py",
    "q5-primer/reference/native-replacement.py",
    "q5-primer/reference/neb-q5-sdm-reference.py",
    "codon-usage/reference/parse_kazusa.py",
    "fasta/reference/digests.py",
    "genbank/reference/pbr322.py",
    "genbank/reference/verify_synthetic.py",
]


def run(script):
    """Run one check from the repository root; return status and output."""
    path = os.path.join(HERE, script)
    assert os.path.isfile(path), "no such check: " + path
    result = subprocess.run(
        [sys.executable, "-I", path, "--check"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return result.returncode, result.stdout + result.stderr


failures = []
for script in CHECKS:
    status, output = run(script)
    if status == 0:
        print("PASS " + script)
    else:
        print("FAIL " + script)
        for line in output.splitlines():
            print("  " + line)
        failures.append(script)

if failures:
    print("{} of {} checks failed: {}".format(len(failures), len(CHECKS), ", ".join(failures)))
    sys.exit(1)
print("all {} checks passed".format(len(CHECKS)))
