# Task contracts

* [Contract okf-a — docs: move files into the OKF layout and split the decisions](docs-okf-a-move.md) - docs/ becomes the bundle root.
* [Contract okf-b — docs: turn path references into OKF links](docs-okf-b-links.md) - Inside the bundle (docs/), a reference to another bundle file becomes a standard markdown link with a bundle-absolute target (OKF v0.2 §6.1.
* [Contract okf-c — docs: OKF frontmatter, index files and the log](docs-okf-c-frontmatter.md) - Every concept document in docs/ gets an OKF v0.2 frontmatter block (§4.1, §5.4), every directory gets an index.md (§8), and the bundle root gets a log.md (§9).
* [Milestone 1 task record: core sequence foundation](milestone-1-core-foundation.md) - This file records the concrete semantic choices made while implementing Milestone 1.
* [Milestone 2 task record: deterministic mutation domain](milestone-2-mutation.md) - This file records the concrete semantic choices made while implementing Milestone 2, in the same style as milestone-1-core-foundation.md.
* [Milestone 3 task record: Q5 SDM workflow contract](milestone-3-q5-workflow.md) - This file records the concrete choices made while implementing the Q5 slice, in the style of the Milestone 1 and Milestone 2 records.
* [Milestone 3b — Specify the Q5 primer-design contract](milestone-3b-q5-primer-contract.md) - the contract for a future deterministic Q5 primer-design layer.
* [Milestone 3c task contract: wiring the Q5 workflow to the design layer](milestone-3c-q5-design-wiring.md) - Two layers exist and do not meet: crates/core/src/q5.rs runs the Q5 workflow and produces no primers, and crates/core/src/primer_design.rs designs a primer pair.
* [Milestone 3d task contract: melting temperature, and Tm-driven primer length](milestone-3d-tm.md) - luria emits primer pairs whose geometry satisfies D029 and whose melting temperature nobody has computed.
* [Milestone 3e task record: primer-quality warnings](milestone-3e-primer-quality.md) - A design that breaks a guideline is still emitted, with the guideline it broke recorded against the primer.
* [Milestone 4 task contract: insertions and deletions](milestone-4-indels.md) - luria designs primers for substitutions of 1–3 bases.
* [Milestone 4a task contract: represent, apply and verify a deletion](milestone-4a-deletion.md) - A DNA-level deletion of a contiguous region can be requested, applied to a sequence, and independently verified.
* [Milestone 4b-1 task contract: deletion primers and pairs in primer.rs](milestone-4b1-deletion-primers.md) - construct_primer and assemble_pair accept a deletion result and check a deletion pair against D031's invariant.
* [Milestone 4b-2 task contract: deletion primer design (D031)](milestone-4b2-deletion-design.md) - design_primers designs a verified deletion under D031, reusing D030's sizing unchanged.
* [Milestone 4c-1 task contract: deletions in the Q5 workflow, with a frameshift warning](milestone-4c1-workflow-deletion.md) - A Q5 request can ask for a deletion.
* [Milestone 4c-2 task contract: deletion runs reach a primer pair](milestone-4c2-design-deletion-run.md) - Prove, in tests, that a deletion requested through the Q5 workflow reaches a D031 primer pair through design_run and design_set.
* [Milestone 4d-1 task contract: represent, apply and verify an insertion](milestone-4d1-insertion.md) - A DNA-level insertion can be requested, applied and independently verified.
* [Milestone 4d-2 task contract: 5′ tails and insertion pairs in primer.rs](milestone-4d2-insertion-primers.md) - primer.rs can represent a primer with a non-annealing 5′ tail, and can check an insertion pair.
* [Milestone 4d-3a task contract: insertion primer design (D032)](milestone-4d3a-insertion-design.md) - design_primers designs a verified insertion under D032, reusing D030's sizing through the existing size_pair, exactly as design_deletion does.
* [Milestone 4d-3b task contract: insertion fixture tests and the 100-nt warning](milestone-4d3b-insertion-fixtures.md) - Check every row of tests/fixtures/q5-primer/native-insertion.tsv against luria.
* [Milestone 4d-4 task contract: insertions in the Q5 workflow](milestone-4d4-workflow-insertion.md) - A Q5 request can ask for an insertion.
* [Milestone 4e-1 task contract: expanding IUPAC ambiguity codes](milestone-4e1-degenerate-expansion.md) - sequence.rs can say which concrete bases an IUPAC code stands for, and can expand a text containing ambiguity codes into every concrete sequence it denotes, in D033's order.
* [Milestone 4e-2 task contract: represent, apply and verify a replacement](milestone-4e2-replacement.md) - A DNA-level replacement of a region by a text of any length can be requested, applied and independently verified.
* [Milestone 4e-3a task contract: replacement primers and pairs in primer.rs](milestone-4e3a-replacement-primers.md) - primer.rs accepts a replacement result: untailed primers on the inline path, tailed primers and the pair invariant on the tail path.
* [Milestone 4e-3b task contract: design a replacement (D033)](milestone-4e3b-replacement-design.md) - design_primers designs a verified replacement instead of refusing it, and matches all 15 rows of tests/fixtures/q5-primer/native-replacement.tsv.
* [Milestone 4e-4a task contract: replacements in the Q5 workflow](milestone-4e4a-workflow-replacement.md) - A Q5 request can ask for a replacement.
* [Milestone 4e-4b task contract: degenerate replacement requests](milestone-4e4b-degenerate-requests.md) - A caller can ask for a replacement containing IUPAC ambiguity codes.
* [Milestone 5 plan: the luria CLI and machine-readable output](milestone-5-cli.md) - Every scientific capability so far is a Rust library call.
* [Milestone 5a1: a stable code for every DomainError](milestone-5a1-error-codes.md) - The CLI will print every failure as JSON with a code field.
* [Milestone 5a2: a rule on every InvalidMutationRequest](milestone-5a2-mutation-request-rules.md) - InvalidMutationRequest is raised at 30 sites for 20 different reasons, and the only way to tell them apart is the reason text.
* [Milestone 5a3: a rule on every InvalidPrimerCandidate](milestone-5a3-primer-candidate-rules.md) - InvalidPrimerCandidate is raised at 19 sites in primer.rs for 13 different reasons, told apart only by reason text; this contract adds a typed rule.
* [Milestone 5b: the FASTA reader](milestone-5b-fasta.md) - D007: a sequence artifact records its identifier, normalized text, topology, length, source metadata when available, and a SHA-256 content hash.
* [Milestone 5c1: the GenBank location parser](milestone-5c1-genbank-location.md) - A GenBank feature location (complement(join(2001..2400,1..150))) has to become luria intervals and a strand before any CDS can be built from it.
* [Milestone 5c2: the GenBank record reader](milestone-5c2-genbank-record.md) - This contract reads the structure of one GenBank flat-file record.
* [Milestone 5c3: importing named CDSs from a GenBank record](milestone-5c3-genbank-cds.md) - A request names the CDSs it wants from a GenBank record, by id.
* [Milestone 5d: the luria-cli crate skeleton](milestone-5d-cli-skeleton.md) - This contract adds the second crate and the luria executable, with everything that doesn't need a request: argument parsing, luria version, luria help, usage errors, and the exit-code type.
* [Milestone 5e1: the strict request document](milestone-5e1-request-document.md) - The request file is the typed surface that luria q5 design reads, and that the Milestone 6 adapter will write (D004, D016).
* [Milestone 5e2: preparing a request for the workflow](milestone-5e2-request-preparation.md) - 5e1 turns request text into a RequestDocument without touching files or core.
* [Milestone 5f1: the result document's skeleton, provenance and failures](milestone-5f1-result-skeleton.md) - luria q5 design prints one JSON document for success, clarification and failure alike (plan §3.5).
* [Milestone 5f2: targets, mutation results, warnings and clarifications](milestone-5f2-result-members.md) - 5f1 declared CdsView, OperationWarning and MemberView as empty placeholders.
* [Milestone 5f3: primer designs and design failures in the result](milestone-5f3-result-design.md) - 5f2 left DesignView empty and every member's design and designFailure null.
* [Milestone 5f4: the details of each error](milestone-5f4-error-details.md) - 5f1 gave ErrorView everything but details; this contract adds details, the payload fields of a failed result's error.
* [Milestone 5g1: luria q5 design, wired](milestone-5g1-run-wiring.md) - Everything the command needs now exists: prepare turns a request file into core types (5e2), and result.rs turns core's answers into the result document (5f1–5f4).
* [Milestone 5g2: --write-edited](milestone-5g2-write-edited.md) - q5 design prints its result but writes no sequence text: plan §3.5 keeps the edited construct out of the JSON so a 64-member expansion of a plasmid does not produce a multi-megabyte document.
* [Milestone 5g3: end-to-end tests](milestone-5g3-end-to-end.md) - Plan §6 is how this milestone establishes that the CLI is right: run the binary, parse its JSON, and compare each scientific field with an independently verified fixture.
* [Milestone 5h: the CLI's documentation](milestone-5h-docs.md) - Scope, as the owner set it on 2026-10-18: docs/specs/cli.md holds the worked example, the luria version document and short reference tables.
* [Milestone 6 plan: natural-language interpretation](milestone-6-nl.md) - Milestone 5 made the Q5 workflow runnable from a typed request file, luria.q5-request/1.
* [Milestone 6a: the luria-nl crate and the extraction parser](milestone-6a-nl-crate.md) - Milestone 6 turns a user's prose into a typed request.
* [Milestone 6b: mutation notation and the residue-name table](milestone-6b-notation.md) - The model reports each mutation as a token such as A42G (plan §5.2).
* [Milestone 6c: the CDS catalogue](milestone-6c-catalogue.md) - To name a CDS, the model has to know which CDSs the template offers.
* [Milestone 6d: the design profile](milestone-6d-profile.md) - Prose rarely states a primer-design policy, a buffer or a codon, and D028 forbids luria defaulting them.
* [Milestone 6e1: what a quote shows](milestone-6e1-evidence.md) - The model reports a mutation as a canonical token plus a verbatim quote of the user's words (M6-15).
* [Milestone 6e2: from an extraction to a request or questions](milestone-6e2-build.md) - This is the trust boundary of plan §3, as one pure function.
* [Milestone 6f: the prompt and the backend interface](milestone-6f-prompt-backend.md) - Before a model can be called, luria needs two things in luria-nl: the prompt: the exact messages sent, built deterministically from the user's text and a CDS summary and the backend interface.
* [Milestone 6g1: the OpenAI-compatible backend, over plain HTTP](milestone-6g1-openai.md) - This is the first code that sends anything off the process: the prompt, to a model served behind an OpenAI-compatible /chat/completions endpoint (llama.cpp, Ollama, vLLM and others).
* [Milestone 6h1: interpret, its document and exit status 8](milestone-6h1-interpret-run.md) - Everything luria q5 interpret does except reading its command line and choosing a backend.
* [Milestone 6h2: the luria q5 interpret command](milestone-6h2-interpret-command.md) - 6h1 made interpret a library call with the backend as a parameter; 6g1 made the real backend.
* [Milestone 6i: the corpus end to end](milestone-6i-end-to-end.md) - This contract adds tests only.
* [Milestone 6i0: the linear GenBank fixture](milestone-6i0-linear-genbank.md) - Every GenBank fixture so far is circular, and luria designs primers on linear templates only (D035, D041 G10).
* [Milestone 6j: the documentation, and D044](milestone-6j-docs.md) - Milestone 6's behaviour is built and tested; this contract writes it down where users and future contributors look.
* [Milestone 7 plan: the review-oriented terminal interface](milestone-7-tui.md) - Milestones 5 and 6 produce two JSON documents: the proposal of luria q5 interpret (luria.q5-interpretation/1) and the result of luria q5 design (luria.q5-result/1).
* [Milestone 7a: luria review REQUEST.json, the skeleton](milestone-7a-review-skeleton.md) - The first runnable piece of the TUI: luria review REQUEST.json runs the request exactly as luria q5 design does, then opens a full-screen terminal view of the result.
* [Milestone 7b: the review state machine](milestone-7b-app.md) - 7a's App holds a result document and knows one rule: q and Esc quit.
* [Milestone 7c1: the text of the overview, problem and provenance screens](milestone-7c1-overview-text.md) - Each result screen is a list of plain text lines built from the ResultDocument.
* [Milestone 7c2: the text of the member screen](milestone-7c2-member-text.md) - The member screen is where a primer pair is reviewed: the edit, its warnings, and each primer with its parts, Tm and warnings, or the reason no pair was designed.
* [Milestone 7c3: drawing the result screens, with scrolling](milestone-7c3-draw-screens.md) - 7c1 and 7c2 build each screen's lines; 7b says which screen is open.
* [Milestone 7d: the sequence-context screen](milestone-7d-context.md) - The sequence-context screen lays a designed primer pair out against the template, so a person can see where each primer sits, which bases anneal, and where the edit is.
* [Milestone 7d2: the reverse tail's context, tested](milestone-7d2-reverse-tail.md) - This contract adds the test that compares 7d's output with the planner's context-ins-04.txt, and proves 7d's reverse-tail rule.
* [Milestone 7e: the text of the proposal screen](milestone-7e-proposal-text.md) - luria review --template … --text … (M7-1) first shows what luria q5 interpret proposed, so the user can judge it before anything runs.
* [Milestone 7f1: the request file's rules, callable on their own](milestone-7f1-request-file.md) - In the TUI flow (M7-1), interpret runs without writing the request: only on approval is the request written and designed.
* [Milestone 7f2: review a proposal, approve it, and run it](milestone-7f2-approve-and-run.md) - The second entry point of M7-1: it interprets the text, shows the proposal, and on the user's explicit approval writes the request to --request and designs from that file.
* [Milestone 7g1: export from the review screen](milestone-7g1-export.md) - M7-4: the reviewed result can be exported as the result JSON, the edited constructs as FASTA, and the IDT order table.
* [Milestone 7g2: the IDT order table](milestone-7g2-order-table.md) - The owner orders primers from IDT by pasting a table into its Bulk Input.
* [Milestone 7h: the documentation, and D046](milestone-7h-docs.md) - Milestone 7's behaviour is built and tested.
* [Milestone 7i1: the mutant reading frame, in luria-core](milestone-7i1-mutant-frame.md) - The redesigned context screen shows the mutant protein between the two strands, each residue letter over the middle base of its codon.
* [Milestone 7i2: each member's mutant protein, for the review](milestone-7i2-member-frames.md) - The redesigned member screen shows the mutant protein.
* [Milestone 7i3: the text of the redesigned member screen's sequence block](milestone-7i3-member-sequence.md) - Opening a member will show its sequence: the forward primer above the top strand, the mutant protein between the strands, the bottom strand, the reverse primer below.
* [Milestone 7i4: the redesigned member screen, its keys, and the trimmed overview](milestone-7i4-member-screen.md) - The owner found the review screens too full.
* [Milestone 7j1: the screens in plain words](milestone-7j1-plain-words.md) - The owner found the screens still cluttered and in programmer's notation.
* [Milestone 7j2: borders, the title badge, colour and the help pop-up](milestone-7j2-styling.md) - The owner asked for the screens to look like popular ratatui apps.
* [Milestone 7k: luria new, entering a design in the terminal (plan draft)](milestone-7k-new.md) - What the user does: luria new [--design PROFILE.json] is one form, then the screens that exist already.
* [Milestone 7k1: the mutation shorthand parser](milestone-7k1-shorthand.md) - luria new takes mutations as one line each, in a shorthand luria reads itself, with no model.
* [Milestone 7k10: Ctrl-C quits; an overlap too large to read is a blocker](milestone-7k10-ctrl-c-overlap.md) - M7-43: Ctrl-C quits, writing nothing, on every screen of luria new and luria review.
* [Milestone 7k1b: a range is a replacement](milestone-7k1b-substitution-is-one-base.md) - 7k1 read a same-length range such as 1210-1212 GAT>AAC as a Substitution; a request's substitution is one base only, as q5 design refuses it.
* [Milestone 7k2: building a request from the form's values](milestone-7k2-build.md) - luria new's form collects a template, mutation lines, a codon choice and an overlap.
* [Milestone 7k3: the luria new form's state](milestone-7k3-form-state.md) - The form holds what the user has typed and chosen, moves focus, and on [ Review ] turns itself into a new::Form and calls new::build.
* [Milestone 7k4: the luria new form's text and drawing](milestone-7k4-form-view.md) - The form exists as state (7k3).
* [Milestone 7k5: the luria new command](milestone-7k5-new-command.md) - The form opens (7k3/7k4).
* [Milestone 7k6: the documentation of luria new, and D047](milestone-7k6-docs.md) - Replace is §11 (D046). (the end of the opening paragraph's sentence about luria review) with is §11 (D046).
* [Milestone 7k7: luria new's "In words" box, through the model](milestone-7k7-words.md) - M7-40 (owner, 2026-10-22): words and shorthand are either one, not both.
* [Milestone 7k8: luria new's unknown-option message names every option](milestone-7k8-new-message.md) - parse_new's refusal of an unknown word in crates/cli/src/args.rs: the message now names every option it takes, and nothing else in parse_new changes.
* [Milestone 7k9: words use the form's values; pasted line ends; a waiting line](milestone-7k9-words-values.md) - M7-41: with words, the form's values are the design; the form now gives interpret a profile built from what it shows.
* [Milestone 7m: the owner's first hand test, as changes](milestone-7m-polish.md) - The owner's answers of 2026-10-22 on ten points of the luria new form, from the prefilled overlap to the file picker and the colours.
* [Milestone 7m1: the review screens after the owner's hand test](milestone-7m1-review-screens.md) - M7-52: the Provenance screen goes.
* [Milestone 7m1c: a coloured run's first character takes its colour](milestone-7m1c-first-character.md) - crates/cli/src/review/view.rs's runs starts with style = Style::default() and changes style only when index > 0.
* [Milestone 7m2: export names constructs by template, and asks before replacing](milestone-7m2-export.md) - M7-57: constructs are <template id>_<member>.fa, e.g.
* [Milestone 7m3: the form's fields after the owner's hand test](milestone-7m3-form-fields.md) - M7-45: Overlap is prefilled with 15, shown and editable; a --design profile's value replaces it.
* [Milestone 7m4: one pop-up to save and run, asking before it replaces](milestone-7m4-save-popup.md) - M7-48: a pop-up on the form asks for the request's file name and shows the pasted FASTA's name.
* [Milestone 7m5: a filtered file list under File](milestone-7m5-file-picker.md) - M7-50: a filtered list under File: .fa .fasta .fna .gb .gbk .genbank files and folders in the directory being typed, filtered as you type, ↑/↓ to pick, enter to choose.
* [Milestone 7m6: the docs for the owner's hand-test changes](milestone-7m6-docs.md) - request and result files stay zero-based (§3).
* [Milestone 7m7: words in luria new use the form's CDS](milestone-7m7-words-cds.md) - In the owner's backend hand test, luria new with synthetic-linear.gb and the words change alanine 42 to glycine gave a proposal with the one question cds_unnamed.
* [Milestone 7m8a: run a proposed request without writing it](milestone-7m8a-preview-run.md) - M7-60: when the model's reading can be approved, luria new shows the variant screens before anything is saved.
* [Milestone 7m8b: the words preview on the variant screens](milestone-7m8b-preview-screens.md) - M7-60: when the model's reading can be approved, luria new runs the design in memory and shows the variant screens as a preview, before anything is saved.
* [Milestone 7m9a: [ Save and run ] writes and runs at once](milestone-7m9a-save-at-once.md) - M7-62: a words reading with no questions is written and run at once, under the form's file name.
* [Milestone 7m9b: remove the in-memory run](milestone-7m9b-remove-preview-run.md) - M7-65: 7m8a's in-memory run goes, because nothing uses it any more.
* [Milestone 8 plan: the sequence view (luria FILE) and luria find](milestone-8-view.md) - A read-only view of a construct: both strands, the translation of a chosen CDS or of all six frames, base and residue numbering, features, an overview of the whole sequence, go-to, and search.
* [Contract 8b1 — core: the display-only location parser (§4.1, M8-1)](milestone-8b1-display-location.md) - crates/core/src/display_location.rs: parse_display_location, which reads a GenBank feature location for drawing only.
* [Contract 8b2 — core: ViewRecord and open_record (§4.1)](milestone-8b2-view-record.md) - crates/core/src/view_record.rs: open_record(name, text), the one core entry point the view and luria find use to read a file.
* [Contract 8b3 — core: CDS import without per-CDS copies](milestone-8b3-cds-import-speed.md) - import_cds (genbank_cds.rs:551) costs time that grows faster than the square of a record's CDS count.
* [Milestone 8c: the CDS map (core)](milestone-8c-cds-map.md) - The view must answer two questions without doing coordinate arithmetic of its own: which three reference bases encode a numbered residue, and which residue a base the reader points at belongs to.
* [Contract 8d — core: frame translation (§4.3)](milestone-8d-frames.md) - crates/core/src/frames.rs: the six reading frames of a record — three per strand — as the residues a frame produces, in the frame's own numbering, each with the position of its codon's middle base.
* [Contract 8e — core DNA search (plan §4.4)](milestone-8e-dna-search.md) - One new module, crates/core/src/search.rs, plus one line in crates/core/src/lib.rs: pub mod search; after pub mod q5_expand; and before pub mod sequence; (module list is alphabetical.
* [Contract 8f — core: protein search (§4.5)](milestone-8f-protein-search.md) - crates/core/src/protein_search.rs: find_protein, which finds an amino-acid pattern in one CDS's translation or in the six frames of a record.
* [Contract 8g — core: the overview scale (§4.6)](milestone-8g-overview-scale.md) - crates/core/src/overview.rs has two functions.
* [Contract 8h — cli: opening a bare file (§5.6)](milestone-8h-open-file.md) - luria FILE opens the sequence view (8k1) on any existing path that is not a subcommand word.
* [Contract 8i1 — core: matching a name to a CDS (M8-15)](milestone-8i1-name-matcher.md) - crates/core/src/name_match.rs: one matcher that turns a value the user typed into one item, several candidates, or none, by D049's rules.
* [Contract 8i2 — cli: luria find (§5.5)](milestone-8i2-find.md) - luria find FILE PATTERN [--aa] [--cds VALUE] [--circular]: read a sequence file the way the view will, search it for a DNA or protein pattern, and print the hits as tab-separated rows.
* [Contract 8j — cli: the block layout (§5.1)](milestone-8j-block-layout.md) - crates/cli/src/sequence_view/layout.rs: pure functions that split a record into blocks for a width and a translation mode, and turn one block into ratatui lines.
* [Contract 8k1 — cli: the view's state, cursor and status line (§5.1, §5.2)](milestone-8k1-view-state.md) - The view as a screen: a pure state that moves a cursor over 8j's blocks, scrolls them, cycles the translation lanes and picks the CDS on open.
* [Contract 8k2 — cli: go-to (§5.3)](milestone-8k2-go-to.md) - g opens a one-line prompt on the status line.
* [Contract 8k3 — cli: the feature list and the CDS picker (§5.2, M8-5)](milestone-8k3-feature-list.md) - Two lists over the view: f lists every feature, c lists every CDS.
* [Contract 8l1 — cli: the view's search prompt (§5.4)](milestone-8l1-search.md) - / opens a search prompt with a DNA/AA mode badge.
* [Contract 8l2 — cli: the overview inset (§4.6, §5.1)](milestone-8l2-overview.md) - A two-line strip under the title: line 1 is the whole record scaled to the width with the longest features as bars; line 2 marks the part on screen, the cursor and the search hits.
* [Milestone 8m: the owner's hand test of the sequence view, as changes](milestone-8m-polish.md) - The owner's answers of 2026-10-27 on seven points of the view: where a pick lands, go-to with several CDSs, the feature lanes key, the strand of a hit and the topology key.
* [Contract 8m1 — cli: where the cursor lands after a pick (M8-21, M8-22)](milestone-8m1-jumps.md) - Three changes to where the cursor goes: a CDS picked from the c list moves the cursor to its start codon, and f + enter on a minus-strand feature goes to its 5′ end.
* [Contract 8m10 — cli: a ruler line over the top strand (M8-33)](milestone-8m10-ruler.md) - Every block gains one line, the ruler, directly above its top strand.
* [Contract 8m11 — cli: a lane's arrowhead at the feature's 3′ end (M8-34)](milestone-8m11-arrowheads.md) - Today every lane run reads name ▸ ═══ (or ◂).
* [Contract 8m12 — cli: a GC-content sparkline as the strip's third line (M8-34)](milestone-8m12-gc-sparkline.md) - The overview strip gains a third line under its two (8l2).
* [Contract 8m13 — cli: go-to and w/b land on a codon's first base (M8-35)](milestone-8m13-codon-first-base.md) - A residue typed at go-to (g M1), and every w and b, put the cursor on the first base of the codon in reading direction, where today they put it on the middle base.
* [Contract 8m14 — cli: a minus-strand CDS's strand drawn bright (M8-36)](milestone-8m14-minus-strand-bright.md) - While the lanes show a minus-strand CDS (LaneMode::Cds(i) where CDS i is on the minus strand), the strand that CDS reads is the one drawn as primary.
* [Contract 8m15 — cli: esc clears the hits; ctrl-d/ctrl-u half pages (M8-37, M8-38)](milestone-8m15-esc-half-pages.md) - esc, with no list, prompt or key list open, clears the search hits: the marks go, n/N have nothing to step, and the cursor stays.
* [Contract 8m16 — cli: the feature list coloured, and filtered by key too (M8-39)](milestone-8m16-feature-list-keys.md) - Each row of the f list is drawn in its feature's lane colour (8m7's feature_colour: the file's ApE colour, else the key's), as the row's foreground, with the colours on.
* [Contract 8m17 — core: the ORF finder, with a reference table (M8-40)](milestone-8m17-orf-finder.md) - crates/core/src/orf.rs: one function that finds the open reading frames of a sequence on both strands, as M8-40 defines them, and a reference script and table that pin its output.
* [Contract 8m18 — cli: ORF lanes, shown and hidden with O (M8-40)](milestone-8m18-orf-lanes.md) - O shows and hides lanes for the open reading frames 8m17's find_orfs finds (at least ORF_MIN_RESIDUES residues).
* [Contract 8m2 — cli: a hit is marked on its strand (M8-24)](milestone-8m2-strands.md) - Today a search marks every hit's bases yellow on the top strand only.
* [Contract 8m3 — cli: F, T and the key list (M8-23, M8-25, M8-26)](milestone-8m3-keys.md) - F hides and shows the feature lanes.
* [Contract 8m4 — cli: lines fill the width in steps of 3 (M8-28)](milestone-8m4-width.md) - A line holds as many bases as fit beside the base numbers, rounded down to a multiple of 3, in every mode (today: a multiple of 10, or 30 with a translation).
* [Contract 8m5 — cli: residues coloured by Zappo group, C toggles (M8-29)](milestone-8m5-colours.md) - In every residue line (the selected CDS's line and each frame's line), each residue's codon gets a background colour by Zappo group across all three of its bases.
* [Contract 8m6 — cli: F cycles all features, CDSs only, none (M8-30)](milestone-8m6-levels.md) - F now cycles through three feature levels, opening on the first: all features (as today), CDSs only, and none.
* [Contract 8m7 — lanes coloured by key or by the file, the selected CDS bold (M8-31)](milestone-8m7-lane-colours.md) - Each feature lane is drawn in a colour: its name, its arrow and its ═.
* [Contract 8m8 — cli: the cursor's codon highlighted on the residue line (M8-32)](milestone-8m8-cursor-codon.md) - When the cursor stands in a codon of the selected CDS and that CDS's residue line is drawn, the codon's three cells on that line are reversed and bold.
* [Contract 8m9 — cli: dimming, title badge, rule between blocks, ambiguous bases, colours off means none (M8-33, M8-34)](milestone-8m9-styling.md) - Drawing only: no line, row, column or text of a block changes, except the blank row between two blocks and the title.
* [Contract 8n — docs: luria FILE and luria find in the docs, D058, D059](milestone-8n-docs.md) - Milestone 8's user documentation: docs/specs/cli.md gains §13 and the two commands in §1, and docs/decisions/ records D058 and D059.
* [Contract 8r1 — cli: three bugs from the milestone 8 code review](milestone-8r1-review-fixes.md) - Three separate fixes, each with a test that fails without it.
* [Contract 8r2 — core: frame windows and DNA search without whole-record work](milestone-8r2-core-speed.md) - Two functions get faster, and neither may change a single output: frames::frame_residues reads only the codons whose middle base is in its window.
* [Contract 8r3 — cli: the view builds each block once, and reuses the CDS it holds](milestone-8r3-view-speed.md) - Three changes to crates/cli/src/sequence_view/.
* [Withdrawn: worktree safety and luria commit (this file was labelled 8a2)](milestone-8x-worktree-safety-withdrawn.md) - Withdrawn before a single line of it was committed: worktree safety and luria commit.
* [Milestone 9 plan: a sequence editor in the view](milestone-9-editor.md) - M8-27 (owner, 2026-10-27): a general sequence editor in the view.
* [Contract 9b — core: the edit buffer](milestone-9b-edit-buffer.md) - A new core module, edit_buffer, that holds a record's bases and its features while the user edits them.
* [Contract 9b2 — core: the editor's reading of a location, and the buffer from a record](milestone-9b2-edit-location.md) - The editor's own reading of a GenBank location text into 9b's EditLocation.
* [Contract 9c1 — core: the GenBank reader keeps what a save writes back](milestone-9c1-reader-keeps-header.md) - The GenBank reader starts keeping what the writer (9c3) needs to write a record back.
* [Contract 9c2 — core: writing a location back as GenBank text](milestone-9c2-location-writer.md) - A function that writes an EditLocation (9b) as GenBank location text.
* [Contract 9c3 — core: the GenBank writer](milestone-9c3-genbank-writer.md) - write_genbank: an edit buffer written back as one GenBank record, next to the record it was opened from.
* [Contract 9d — cli: edit modes, the selection, and the blocked record](milestone-9d-modes-selection.md) - The view learns two things in this contract: whether its record can be edited, and Visual mode.
* [Contract 9e — core and cli: the view after an edit](milestone-9e-recompute.md) - After an edit, the view shows the edited record: its features, CDSs, frames, ORFs, search hits and the overview strip.
* [Contract 9f — cli: insert, overtype, replace and the terminal's paste](milestone-9f-typing.md) - Typing bases: i and a insert, R overtypes, and r replaces one base, or every base of a selection.
* [Contract 9g — core and cli: cut, yank, paste, undo and redo](milestone-9g-cut-paste-undo.md) - x and d cut and y yanks, into one register kept for the session.
* [Contract 9h — cli: saving the edited record, and quitting with edits unsaved](milestone-9h-save-and-quit.md) - s opens a "save as" prompt with a name filled in.
* [Contract 9h2 — cli: the save prompt is drawn](milestone-9h2-save-prompt-shown.md) - Contract 9h's s opens the save prompt (ViewState::save_prompt), but nothing draws it.
* [Contract 9j — docs: the editor in [docs/specs/cli.md](/specs/cli.md), the decisions, the brief](milestone-9j-docs.md) - It records milestone 9 in three documents.
* [Contract 9k — cli: the messages that promise milestone 9](milestone-9k-reword-promises.md) - Two refusals tell the user that something "comes in milestone 9".
* [Contract 9l — cli: the : command line, ZZ and ZQ](milestone-9l-command-line.md) - Saving and quitting work as in Vim (plan M9-19, approved by the owner, 2026-10-31): : opens a command line on the status line, ZZ and ZQ work,.
* [Contract 9m — cli: the ? key list in sections, scrolled](milestone-9m-key-list.md) - The ? list names every key the view has, edit keys and : commands included.
* [Contract 9n — core: a save replaces luria's own earlier stamp](milestone-9n-one-stamp.md) - Each GenBank save adds the line Edited with luria <version>: N edits. (M9-11).
* [Contract 9o — cli: backspace in R brings the base back](milestone-9o-overtype-backspace.md) - In R (overtype), backspace takes back the last base typed in this R, as Vim does.
* [Contract 9p1 — cli: measure where an edit's time goes](milestone-9p1-measure-recompute.md) - One edit on a 10 000 000-base record takes about 1.65 s in a release build (9e §10).
* [Contract 9p2 — core: the two slow steps of an edit, made fast](milestone-9p2-faster-recompute.md) - 9p1 §8 measured one edit on a 10 000 000-base record: 2.0 s, or 5.4 s with ORFs shown.
* [Contract 9q — core, cli: the review's five correctness fixes](milestone-9q-review-fixes.md) - A code review of milestone 9 (2026-10-31) found five bugs.
* [Contract 9r — core, cli: the review's memory and speed fixes](milestone-9r-review-memory.md) - These are findings 6 to 9 of the milestone 9 review (2026-10-31).
