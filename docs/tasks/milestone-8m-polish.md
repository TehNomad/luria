# Milestone 8m: the owner's hand test of the sequence view, as changes

Status: **Implemented, 2026-10-27** (8m1 to 8m3; awaits the owner's
second look). Planner: Claude.

The owner tried `luria FILE` in a terminal after 8l2 and reported that all
the features work. They asked for the changes below. The planner's questions
and the owner's answers are recorded as M8-21 to M8-27. Where an answer
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

Seen at the hand test, with no change asked: the cap sentence is cut short
on the status line at 80 columns; the list window scrolls as soon as the
highlight moves up; the picker has no "no CDS" row; go-to reads an
uppercase name such as `T7` as a residue; a block taller than the body is
not drawn; pBR322 takes about a second to open.

## 2. Contracts, in order

1. `8m1`, where the cursor lands: M8-21 and M8-22.
2. `8m2`, a hit's strand: M8-24.
3. `8m3`, keys: M8-23, M8-25 and M8-26.
