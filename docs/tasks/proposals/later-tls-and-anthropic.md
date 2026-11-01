---
type: Proposal
title: "Later: HTTPS endpoints and the Anthropic backend"
description: "Owner answer M6-1 asked for two backends: a local OpenAI-compatible endpoint and the Anthropic API."
status: draft
---
# Later: HTTPS endpoints and the Anthropic backend

Status: **Deferred by the owner, 2026-10-19 (M6-18). Not scheduled and not a
task contract: do not implement.** Raised during Milestone 6
([docs/tasks/contracts/milestone-6-nl.md](/tasks/contracts/milestone-6-nl.md)).

## Why it is deferred

Owner answer M6-1 asked for two backends: a local OpenAI-compatible endpoint
and the Anthropic API. The Anthropic API is HTTPS only. `ureq`'s TLS stack,
`rustls`, uses `ring`, whose build script compiles C and reads the system
headers in `/usr/include`. The nono sandbox the project is developed in denies
that path:

    $ nono why --self --path /usr/include/stdc-predef.h --op read
    DENIED
      Reason: path_not_granted

so no crate that depends on `ring` builds in a development session. With its
default features off, `ureq` 3.4.1 builds without `ring` or `rustls` and
speaks plain HTTP, which is enough for a model served on the same machine or
the local network.

## What Milestone 6 does instead

- `luria-nl` depends on `ureq = { version = "3", default-features = false }`.
- The only backend is `openai-compatible`, over `http://`. An `https://`
  endpoint is refused before anything is sent.
- `--allow-remote` still gates any endpoint whose host is not a loopback
  address, and the documentation says that plain HTTP to another machine is
  unencrypted.

## What lifting it needs

1. A development sandbox that can read `/usr/include`, for example a profile
   extending the current one with `"filesystem": {"read": ["/usr/include"]}`,
   reviewed and promoted by the owner outside the sandbox.
2. `ureq`'s `rustls` feature, and an owner decision on trust roots (bundled
   WebPKI roots or the platform verifier).
3. The Anthropic backend as planned in the milestone 6 plan §5.6: `POST
   /v1/messages`, `x-api-key` from `ANTHROPIC_API_KEY`, a pinned
   `anthropic-version`, the extraction as a forced tool call, always behind
   `--allow-remote`, and tested against a local stub.
