# `luria new` fixtures

Planner-written expectations for `luria new` (plan
`docs/tasks/contracts/milestone-7k-new.md`), each produced by a script under
`reference/` that shares no code with `crates/`.

## The mutation shorthand

`reference/shorthand.py` writes `shorthand.tsv`: for each of 53 typed
lines, the edit luria must read (kind, zero-based half-open start and
end, bases, codon) or the code of its refusal. The script is its own
parser of the plan's §3 table (M7-35); its docstring states every rule.
The planner checked by hand that `1210-1212 GAT>AAC` is `[1209, 1212)`,
the same interval `q5-primer` row `native-01` uses for SID-1's codon
404, and that `ins GAT after 1209` is point 1209, row `ins-14`'s. A range
is always a `replacement`: a request's `substitution` is one base only
(found when `q5 design` refused `GAT>AAC` as a substitution; 7k1b).

## The requests a form builds

`reference/requests.py` writes `request-<case>.json` for five filled
forms (a FASTA template with a typed codon; a five-variant set on SID-1
covering every shorthand kind; a GenBank template; a pasted template with
a CDS stated by bases; a minus-strand paste with a 12-base insertion),
`paste-<case>.fa` for the two pastes, and `refusals.tsv` for ten forms
that must be refused. It reads the templates and digests them itself and
reads each line with `shorthand.py`. The planner ran all five requests
through `luria q5 design`: every one is `verified` and every variant has
a primer pair; the GenBank case is equal, as JSON, to the request written
by hand in milestone 7's review trials.

```bash
python3 -I tests/fixtures/new/reference/shorthand.py --check
python3 -I tests/fixtures/new/reference/requests.py --check
```
