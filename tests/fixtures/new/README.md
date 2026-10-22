# `luria new` fixtures

Planner-written expectations for `luria new` (plan
`docs/tasks/milestone-7k-new.md`), each produced by a script under
`reference/` that shares no code with `crates/`.

## The mutation shorthand

`reference/shorthand.py` writes `shorthand.tsv`: for each of 53 typed
lines, the edit luria must read (kind, zero-based half-open start and
end, bases, codon) or the code of its refusal. The script is its own
parser of the plan's §3 table (M7-35); its docstring states every rule.
The planner checked by hand that `1210-1212 GAT>AAC` is `[1209, 1212)`,
the same interval `q5-primer` row `native-01` uses for SID-1's codon
404, and that `ins GAT after 1209` is point 1209, row `ins-14`'s.

```bash
python3 -I tests/fixtures/new/reference/shorthand.py --check
```
