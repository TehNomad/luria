---
type: Task Contract
title: "Contract okf-d — docs: keep a copy of the OKF specification"
description: "A byte-identical copy of the OKF v0.2 specification goes into docs/references/ with OKF frontmatter."
status: deprecated
tags: [okf]
---
# Contract okf-d — docs: keep a copy of the OKF specification

Planner: Claude. Status: **Implemented 2026-11-01**. Tier: presentation (no biology changes).
Follows okf-a to okf-c (`docs-okf-*.md`).

## 1. What this contract builds

The OKF v0.2 specification, which `docs/` follows, is kept in the bundle as
a reference. The planner has downloaded it to `target/tmp/okf-spec/SPEC.md`
(upstream commit `0b87c52c6ef999286c745e19998fdfcd03d5dbee`, Apache-2.0).
The worker has no network; use that file.

## 2. Files the worker may touch

1. `docs/references/okf-spec-v0.2.md` — new: the frontmatter in §3, then
   the bytes of `target/tmp/okf-spec/SPEC.md` unchanged.
2. `docs/references/index.md` — change: add the entry in §4.
3. `docs/log.md` — change: add the section in §4.

Everything else is out of scope.

## 3. The frontmatter, exactly

```
---
type: Reference
title: "Open Knowledge Format (OKF) specification, v0.2"
description: "The OKF v0.2 specification that this bundle follows, copied unchanged from the upstream repository."
status: stable
resource: "https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/0b87c52c6ef999286c745e19998fdfcd03d5dbee/SPEC.md"
license: "Apache-2.0"
sources:
  - id: okf-spec
    resource: "https://raw.githubusercontent.com/GoogleCloudPlatform/open-knowledge-format/0b87c52c6ef999286c745e19998fdfcd03d5dbee/SPEC.md"
    title: "OKF SPEC.md"
    last_modified: 2026-08-21T19:31:43Z
---
```

The spec's first line (`# Open Knowledge Format (OKF)`) follows the
closing `---` directly. Build the file with `cat`, not an editor, so the
body stays byte-identical.

## 4. Index and log

Add this line to `docs/references/index.md`, entries ordered by file name
(it goes after the `nbc-help-v284.md` entry):

```
* [Open Knowledge Format (OKF) specification, v0.2](okf-spec-v0.2.md) - The OKF v0.2 specification that this bundle follows, copied unchanged from the upstream repository.
```

Add this section to `docs/log.md` above `## 2026-10-31` (newest first),
with one blank line before the next heading:

```
## 2026-11-01
* **Creation**: Added a copy of the [OKF v0.2 specification](/references/okf-spec-v0.2.md) to `references/`.
```

## 5. Checks

`scripts/check`, plus:

```bash
# 1. Body byte-identical: no output.
tail -n +14 docs/references/okf-spec-v0.2.md | cmp - target/tmp/okf-spec/SPEC.md
# 2. Frontmatter is 13 lines: prints `---` twice.
sed -n '1p;13p' docs/references/okf-spec-v0.2.md
```

## 6. Do not

- Change any byte of the spec text, including its links and code blocks.
- Edit any file other than those in §2.
- Fetch anything from the network.

## 7. Report

The standard handoff, plus the output of the two checks in §5.
