# Milestone 8m: the owner's hand test of the sequence view, as changes

Status: **Implemented 2026-10-28**, awaiting the owner's hand look at 8m13
to 8m18: 8m1 to 8m18 implemented (8m4 and 8m5 after the owner's second
look: M8-28 and M8-29). After the colours the owner asked for M8-30 to
M8-34, contracts 8m6 to 8m12. At the hand look at 8m5 to 8m12 the owner
asked for M8-35 to M8-40, contracts 8m13 to 8m18. Planner: Claude.

The owner tried `luria FILE` in a terminal after 8l2 and reported that all
the features work. They asked for the changes below. The planner's questions
and the owner's answers are recorded as M8-21 to M8-28. Where an answer
reverses an earlier rule, the row names it. Light tier (D051): keys,
glyphs and wording can change again without a decision entry.

## 1. Decisions

| Item | Topic | Answer (owner, 2026-10-27) |
| --- | --- | --- |
| M8-21 | Where a pick lands | **A CDS picked from the `c` list puts the cursor on its start codon**: the first base, in reading direction, of residue 1's codon (`CdsMap::residue_bases(1)[0]`). For pBR322's `bla`, `complement(3293..4153)`, that is base 4153. **`f` + `enter` goes to a feature's 5′ end**: the first base of its first interval on the plus strand, and the last base of its last interval on the minus strand. A 5′ end marked open (`<` on plus, `>` on minus) is refused as "start not stated" is today. Reverses 8k3 §4 rule 5 ("the cursor does not move") and changes 8k3 §4 rule 4 for minus-strand features. |
| M8-22 | Go-to residue, several CDSs | **`M1` uses the selected CDS. With none selected, `g M1` opens the CDS picker**, and `enter` on a usable CDS selects it and goes to the residue as 8k2 does. If that residue is refused (out of range, wrong letter), the pick still selects the CDS, lands on its start codon (M8-21), and the status line says why. `esc` drops the residue. A record with no CDS says so and opens nothing. Changes 8k2 §4 rule 4's "no CDS is selected". |
| M8-23 | Feature lanes key | **`F` hides and shows the feature lanes** (the lane lines and the `+N more` line). They are shown at open. Reverses D055's "no display toggles in milestone 8" for this one toggle, which D055 left to the hand test. |
| M8-24 | Strand of a hit | **A search marks a hit on the strand it was found on**: a `+` hit on the top strand, a `-` hit on the bottom strand, and a `both` hit (a palindrome such as `GAATTC`) on both. A protein hit is marked on its CDS's or frame's strand. Changes 8l1 §4 rule 8, which marked the top strand only. |
| M8-25 | Topology key | **`T` is offered only where the topology is assumed** (FASTA, D049 M8-3). Where the file states it, `T` already changes nothing and says "the file states its own topology". Now it is also left off the key line and the help. |
| M8-26 | Key line | (planner's choice, for the owner to judge) **`F` and `g` do not fit an 80-column key line beside the keys already there, so `?` opens a list of every key** the open screen has, as M7-53 did for the review screens. The key line keeps the most-used keys and `? keys`. |
| M8-27 | Milestone 9 scope | **Milestone 9 is a general sequence editor in the view**: vim-style `v` (select), `i` (insert) and `r` (replace), with cut, yank and paste of any length, features following the edits, and the edited construct saved. Reverses `proposal-product-direction.md` §5's "not a general sequence editor" and pulls milestone 10's export forward. Its plan and decisions belong to milestone 9 and are not part of 8m. |
| M8-28 | Line width | (owner, 2026-10-27, second look) **A line holds as many bases as fit beside the base numbers, rounded down to a multiple of 3, in every mode**: 66 at 80 columns for pBR322, 108 at 120. The same width with and without a translation, so toggling `t` does not reflow the lines. Replaces plan §5.1's "largest multiple of 10, and of 30 when a translation lane is on". D050's phase and D055's shortened line are unchanged. Contract 8m4. |
| M8-29 | Residue colours | (owner, 2026-10-27, after 8m4) **Residues are coloured by Zappo's seven physicochemical groups**, as a background across all three bases of the codon, in the residue lines (the selected CDS's and every frame's). The letter stays where it is. IUPAC defines no colours; Zappo was chosen by name so the colours are documented (Jalview's Zappo). **`C` turns the colours off and on**; they are on at open unless `NO_COLOR` is set and not empty. The stop and any symbol outside the twenty keep today's look. Planner's choices: the xterm 256-colour palette, which is Zappo's own values or within 25 of them on every channel, so no true-colour detection is needed; black letters (WCAG contrast at least 4.5 on all seven). A second exception to D055's "no display toggles". Contract 8m5. |
| M8-30 | Feature levels | (owner, 2026-10-27) **`F` cycles all features → CDSs only → none → all**, and opens on all. Replaces M8-23's two states. Planner's choices: CDSs only means features with the key `CDS`; the lanes are chosen from that level's features, so a CDS is never left in `+N more` behind a gene; the overview strip still shows every feature. Contract 8m6. |
| M8-31 | Lane colours | (owner, 2026-10-27, from the planner's list) **Each lane is coloured by its feature key**, or by the colour the file gives the feature where it has one (ApE's `/ApEinfo_fwdcolor` and `/ApEinfo_revcolor`; the contract names the exact qualifiers). The strip's bars use the same colours. **The selected CDS's lane is marked** (bold). Contract 8m7. |
| M8-32 | Cursor codon | (owner, 2026-10-27, from the planner's list) **The cursor's codon is highlighted on the residue line, and the status line says what is under the cursor**: residue and number, codon bases, and the features covering the base. Contract 8m8. |
| M8-33 | Ruler and dimming | (owner, 2026-10-27, from the planner's list) **A ruler line with a tick every 10 bases and a number every 30**, next to the top strand. **The bottom strand, the gutter numbers and the frame labels are dimmed.** The ruler changes the blocks' lines, so it is a full-tier contract with the layout tables (8m10). The dimming is drawing only (8m9). |
| M8-34 | The rest of the list | (owner, 2026-10-27, from the planner's list) **A GC-content sparkline** as a third strip line; **ambiguous bases** (anything but `ACGT`) in their own colour; **an arrowhead at a lane's 3′ end** in place of the arrow after the name; **a title badge** with the length and topology; **a dim rule in the blank row between blocks** (no row count changes). `NO_COLOR` turns off every colour, not only M8-29's. Contracts 8m9 (badge, rule, ambiguous bases, `NO_COLOR`), 8m11 (arrowheads) and 8m12 (sparkline). |
| M8-35 | Where go-to and `w`/`b` land | (owner, 2026-10-28, hand look at 8m5–8m12) **`g M1` puts the cursor on the first base of the residue's codon**, in reading direction: `CdsMap::residue_bases(n)[0]`, the highest base of the codon on the minus strand. **`w` and `b` land on codons' first bases too.** The residue letter stays over the middle base (D055). Changes D057's go-to landing and 8k1's `w`/`b`. Contract 8m13. |
| M8-36 | Minus-strand CDS | (owner, 2026-10-28) **With a minus-strand CDS selected, the bottom strand is drawn bright and the top strand dim, and the cursor's cell and its codon are marked on the bottom strand.** This holds while the lanes show that CDS (`t`); with no CDS, a plus CDS or the frames, nothing changes. Changes 8m9's dimming and 8m8's codon highlight. Contract 8m14. |
| M8-37 | Leaving a search | (owner, 2026-10-28; planner's key) **`esc` clears the search hits** when no list or prompt is open. At a prompt `esc` still closes it. Contract 8m15. |
| M8-38 | Half pages | (owner, 2026-10-28; planner's keys) **`ctrl-d` and `ctrl-u` scroll half a page down and up**, as in Vim. `J` and `K` stay free for milestone 9's editor. Contract 8m15. |
| M8-39 | Features list | (owner, 2026-10-28) **Each row of the `f` list is coloured as its feature's lane is** (M8-31), **and the filter matches the feature's key as well as its name.** Contract 8m16. |
| M8-40 | ORF lanes | (owner, 2026-10-28) **Open reading frames are drawn as lanes**: ATG to the first in-frame stop, at least 75 codons with the stop counted or not as the contract states, the longest ORF per stop only, on both strands and across the origin on a circular record. **`O` shows and hides them, off at open.** They go below the feature lanes in their own colour, labelled by length (`ORF 286 aa`). `t`'s frame translations are unchanged. Contracts 8m17 (the finder in core, full tier) and 8m18 (the lanes). |

Seen at the hand test, with no change asked: the cap sentence is cut short
on the status line at 80 columns; the list window scrolls as soon as the
highlight moves up; the picker has no "no CDS" row; go-to reads an
uppercase name such as `T7` as a residue; a block taller than the body is
not drawn; pBR322 takes about a second to open.

## 2. Contracts, in order

1. `8m1`, where the cursor lands: M8-21 and M8-22.
2. `8m2`, a hit's strand: M8-24.
3. `8m3`, keys: M8-23, M8-25 and M8-26.
4. `8m4`, the line width: M8-28.
5. `8m5`, residue colours: M8-29.
6. `8m6`, feature levels: M8-30.
7. `8m7`, lane colours and the selected CDS's lane: M8-31.
8. `8m8`, the cursor's codon and the status line: M8-32.
9. `8m9`, drawing only: dimming (M8-33), the title badge, the rule between
   blocks, ambiguous bases, and `NO_COLOR` for every colour (M8-34).
10. `8m10`, the ruler, with the layout tables (M8-33).
11. `8m11`, arrowheads, with the layout tables (M8-34).
12. `8m12`, the GC sparkline in the strip (M8-34).
13. `8m13`, go-to and `w`/`b` on a codon's first base (M8-35).
14. `8m14`, a minus-strand CDS's strand drawn bright (M8-36).
15. `8m15`, `esc` clears hits; `ctrl-d`/`ctrl-u` half pages (M8-37, M8-38).
16. `8m16`, the features list: colours and the key in the filter (M8-39).
17. `8m17`, the ORF finder in core, with a reference fixture (M8-40).
18. `8m18`, the ORF lanes and `O` (M8-40).
