# Decisions

* [D001: Pi is development tooling](D001.md) - Pi assists with planning, implementation, and review.
* [D002: Rust owns canonical domain semantics](D002.md) - Rust owns canonical biological domain semantics.
* [D003: Separate domain, workflow, and presentation layers](D003.md) - Luria is organized conceptually into three layers.
* [D004: Natural language becomes a proposed typed request](D004.md) - The LLM converts user language into a structured proposed operation.
* [D005: Fail closed on ambiguity](D005.md) - Luria must prefer clarification or structured failure over silent inference.
* [D006: Coordinate conventions](D006.md) - Internal nucleotide coordinates are zero-based and half-open.
* [D007: Sequence artifacts have explicit identity](D007.md) - A sequence is an explicit artifact, not merely an in-memory string.
* [D008: Sequence normalization is explicit](D008.md) - Sequence normalization must occur at a documented boundary.
* [D009: Topology, intervals, and strand are explicit](D009.md) - Topology is explicit, feature intervals use reference-sequence coordinates and strand determines biological orientation.
* [D010: Initial CDS and translation scope is deliberately narrow](D010.md) - For the initial milestone, support only explicitly complete CDSs under an explicitly supported genetic code.
* [D011: Mutation intent is explicit](D011.md) - An amino-acid substitution identifies a target CDS, a residue position, an expected original residue and a replacement residue.
* [D012: Scientific operations are deterministic](D012.md) - Every supported scientific operation must have a deterministic execution path independent of natural-language interpretation.
* [D013: Generation and verification are separate responsibilities](D013.md) - Candidate generation and result verification are separate responsibilities.
* [D014: Q5 is an explicit supported workflow](D014.md) - Q5 site-directed mutagenesis is implemented as a distinct workflow.
* [D015: Deterministic vertical slice before natural language](D015.md) - Before integrating an LLM, Luria must support at least one complete workflow through deterministic code.
* [D016: Operations are explicit typed capabilities](D016.md) - Runtime operations are explicit application capabilities.
* [D017: Python worker boundary](D017.md) - The initial Python worker boundary is a subprocess with one versioned JSON request and one versioned JSON response.
* [D018: Fixtures are part of the scientific specification](D018.md) - Shared scientific fixtures live in tests/fixtures/.
* [D019: Test pathological cases explicitly](D019.md) - The initial test corpus must include, at minimum, ordinary, reverse-strand, circular, origin-spanning, invalid-coordinate, incomplete-CDS, internal-stop and ambiguous-input cases.
* [D020: Results have explicit lifecycle state](D020.md) - An operation result must distinguish at least proposed, needs_clarification, validated, calculated, verified and failed.
* [D021: Result provenance is first-class](D021.md) - Scientific results identify input artifact identifiers and hashes, operation type, normalized parameters, algorithm and dependency versions, warnings, assumptions and validation status.
* [D022: Reproducibility precedes persistence infrastructure](D022.md) - Luria must define what must be reproducible before choosing a database or persistence architecture.
* [D023: Privacy defaults are conservative](D023.md) - Scientific operations should run locally where practical.
* [D024: Development and approval workflow](D024.md) - Use one bounded task per task branch.
* [D025: Minimal architecture first](D025.md) - Start with a Rust workspace and a core library.
* [D026: Validation commands](D026.md) - Once the Rust workspace exists, run the formatting, test, clippy and git checks.
* [D027: First milestone acceptance criteria](D027.md) - The first milestone is complete only when: canonical sequence semantics are implemented, supported topology and strand behavior are explicit and supported CDS extraction is validated.
* [D028: A design names a policy, and a compatibility policy names a build](D028.md) - Every primer design carries a designPolicy.
* [D029: The Luria-native candidate geometry](D029.md) - Under luriaNative, an edit whose replacement region is the contiguous template span [s, e) gets a primer pair that abuts at a template boundary.
* [D030: Tm-driven primer length for luriaNative](D030.md) - A luriaNative design sizes each primer by melting temperature.
* [D031: Deletion primer geometry for luriaNative](D031.md) - For a deletion of [s, e) on a linear template, the geometry, the pair invariant and the starting lengths are fixed, and everything else is D030.
* [D032: Insertion representation and primer geometry for luriaNative](D032.md) - An InsertionPoint is one 0-based coordinate p, meaning "between base p − 1 and base p".
* [D033: Unequal replacements and degenerate expansion for luriaNative](D033.md) - A replacement names a target region [s, e) of width w >= 1 and a replacement text of length m >= 1.
* [D034: Milestone order after Milestone 4](D034.md) - Milestone 4 is insertions, deletions, unequal replacements and degenerate expansion (D031–D033), as implemented under docs/tasks/contracts/milestone-4-*.md.
* [D035: Primer design is for linear templates only](D035.md) - luriaNative primer design refuses a circular template.
* [D036: A circular feature is split only at the origin](D036.md) - A feature or CDS on a circular sequence is one interval, or exactly two intervals that cross the origin: [a, n) followed by [0, b).
* [D037: An in-frame DNA edit that creates a premature stop is warned](D037.md) - verify stays a DNA-level check for DNA-level edits: it confirms that the edit happened as recorded, and does not re-translate.
* [D038: ConfineToTail is accepted where it changes nothing](D038.md) - A luriaNative request with placement ConfineToTail is designed exactly as the same request with Inline.
* [D039: Fixture verification is reproducible from tracked files](D039.md) - Every script that produced or verified a fixture expectation is tracked next to the fixture it serves, under that fixture's reference/ directory.
* [D040: A codon's recorded coordinates follow the feature interval order](D040.md) - The intervals a residue result records for its codon are listed the way D036 writes a feature.
* [D041: The Milestone 5 CLI, its inputs and its output](D041.md) - The new luria-cli crate may depend on serde and serde_json, and on nothing else.
* [D042: Every imported GenBank CDS starts with ATG](D042.md) - A CDS a request names from a GenBank record is imported only when its first codon is ATG, whatever its /transl_table (absent, 1 or 11).
* [D043: Upper limits on the primer-design buffer](D043.md) - A buffer is refused (invalid_buffer_model) when its monovalent salt is above 1000 mM, its primer concentration above 10000 nM (10 µM), or its DMSO above 20 %.
* [D044: Natural-language interpretation (Milestone 6)](D044.md) - luria q5 interpret turns a request in words into a proposed request, or into questions, and writes it as one interpretation document.
* [D045: Tighter grounding of the model's extraction](D045.md) - A mutation's quote supports its token only when it holds exactly one run of digits, equal to the residue number, with the original residue named before it and the target after it.
* [D046: The review terminal interface (Milestone 7)](D046.md) - luria review REQUEST.json runs a request as luria q5 design does and shows its result in the terminal.
* [D047: Entering a design in the terminal (Milestone 7, luria new)](D047.md) - A form builds a luria.q5-request/1 request from a template, mutation lines and the overlap.
* [D048: What the Milestone 8 view selects, shows, and refuses to read](D048.md) - A record with exactly one usable CDS opens with it selected.
* [D049: Reading a record for the view: partials, format, topology, CDS names](D049.md) - A feature location with < or > — <1..12, 1..>900 — is read for display by a separate display-only parser, which takes the stated base as the bound and records which end is uncertain.
* [D050: What the Milestone 8 search and frame fixtures encode](D050.md) - Overlapping hits are all reported: AA on AAA is three hits, and on a circular sequence an equal-length pattern matches at every rotation.
* [D051: The two-tier process, for Milestone 8 only](D051.md) - The science tier keeps the full process; the milestone 8 contracts are written as the milestones so far were.
* [D052: Frames, ambiguity codes, palindromic sites, and what find exits with](D052.md) - Letting frames wrap, matching ambiguity codes by intersection, reporting a palindromic site twice, exiting 1 on no matches and deciding greyness in the ui each need approval.
* [D053: How the view draws partial features and names refusals](D053.md) - A < is drawn when its base is the first interval's start, and a > when its base is the last interval's end, both in D036 order.
* [D054: Protein-search matching and order, and the overview's columns](D054.md) - In a protein pattern X matches any residue, an unreadable codon ?, and *.
* [D055: How the view's blocks meet codons, frames and numbers](D055.md) - With a CDS selected, one line is shortened rather than the grid shifted.
* [D056: The byte ceiling, the CDS matcher's home, and the cli order](D056.md) - D048 asked for a byte check before a file is read and set no number.
* [D057: A bare number in go-to, and the feature list before go-to](D057.md) - In go-to, digits alone go to that 1-based base, the numbering the status line shows, whatever CDS is selected.
* [D058: What the hand test changed beyond keys and wording (Milestone 8)](D058.md) - The view draws open reading frames (M8-40), replacing D052's rule that ORF finding stays in milestone 9.
* [D059: The milestone order from Milestone 8 on](D059.md) - Milestone 8 is the sequence view: luria FILE, a read-only view of a GenBank or FASTA file, and luria find, as implemented under docs/tasks/contracts/milestone-8-*.md and recorded in D048 to D058.
* [D060: Editing in the view (Milestone 9)](D060.md) - Placing Q5 mutations from the view and designing their primers is a later milestone.
* [D061: How features follow an edit (Milestone 9)](D061.md) - A feature's coordinates are biology: a rule that moved them wrongly without a word would give wrong constructs.
* [D062: Saving an edited record (Milestone 9)](D062.md) - GenBank in, GenBank out; FASTA in, FASTA out (M9-7).
* [Decision record policy](policy.md) - Changes to these decisions require explicit human approval.
