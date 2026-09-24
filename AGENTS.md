# AGENTS.md

Binding contract for any automated agent that emits nucleic-acid designs in this repository. Emit a design specification JSON only. Do not place synthesis orders, print oligos, or author wet-laboratory protocols from raw sequence side channels. bloc censors the plan and suppresses the molecule; the agent does not self-certify release.

## Role

The agent proposes constructs. Only bloc `screen` may return RELEASE.

## bloc

One Rust binary, two subcommands:

- **Censor** (`bloc speccheck`) refuses a malformed or out-of-policy plan.
- **Suppressor** (`bloc screen`) decides the molecule: `RELEASE`, `HOLD`, or `ESCALATE`.

After a screen, the card is the user-facing object—not model logits, not a risk essay.

## Sequence submission

```
inhibit_submit_design  →  design specification JSON  →  bloc screen  →  Verdict + card
```

Raw FASTA outside this path is a protocol violation. If the host allows `construct.fa` and direct vendor API calls, treat that as a host defect; do not use it.

## Design specification

`spec_version` is `"0.1.0"`. `additionalProperties` is false at the schema root.

Required fields:

- `construct_id`
- `designer.agent`, `designer.human`
- `chassis` ∈ `none | e_coli_k12 | e_coli_b | yeast | cell_free | other_bsl1 | other`
- `intended_function` ∈ `reporter | housekeeping | metabolic | binding | nuclease | polymerase | reverse_transcriptase | programmable_nuclease_system | unknown`
- `intended_bsl` ∈ `BSL-1 | BSL-2 | BSL-3 | unknown`
- `not_for_synthesis` (boolean)
- `fragments[]` each with `id`, `role`, `alphabet`, `sequence`

Roles: `cds | promoter | rbs | terminator | repeat_array | spacer | oligo | other`. Alphabets: `dna | rna | aa`. Sequence must match alphabet.

No notes, citations, or justification fields. Justification does not clear HOLD.

## Censor hard stops

Refuse and stop when:

- `intended_bsl` is `BSL-3`
- `assembly.order_split` is `true`
- extra JSON keys
- alphabet or sequence mismatch
- stitched DNA longer than 50 kb (v0 cap)
- an order, oligo print, or wet protocol would precede a `RELEASE` card

`speccheck` failure exits `2`.

## Suppressor pipeline

1. Stitch fragments in listed order. Normalise (`U→T`, uppercase, no whitespace).
2. Window the molecule (50 nt recorded; 30 nt oligo floor noted).
3. Structural novelty gate (evenly spaced repeat array plus RT-plausible CDS, or declared programmable / reverse-transcriptase / unknown function).
4. Run IBBIS `commec` locally when present.
5. Reduce, with no votes:

```
commec missing or crashed     → Undefined / ESCALATE
commec uncleared hit          → Flag / HOLD
ART-shaped pattern or
  RT / programmable / unknown → Undefined / ESCALATE
else, and commec cleared      → No Flag / RELEASE
```

`RELEASE` requires commec to have run and cleared. The agent cannot argue the frame into RELEASE.

Exit codes: `0` RELEASE, `10` HOLD, `20` ESCALATE, `2` schema or engine error. Branch on the code, not on card English.

## After the card

| Policy | Next act |
|---|---|
| `RELEASE` | May describe a BSL-1 build to a human. Do not perform laboratory work. |
| `HOLD` | Stop. Direct the human to the design specification and commec JSON. Do not soften stated risk. |
| `ESCALATE` | Provide a one-page brief: intended function, chassis, which gate fired, what a human could run at BSL-1 or BSL-2. Do not invent a pathogen name. |

High-volume discovery must stream every specification through `bloc screen`. Retain all `RELEASE` cards. Retain at most twenty escalations for human review; additional escalations are logged and dropped from the human queue and are never treated as RELEASE.

## Hashes

- `spec_sha256` — SHA-256 of canonical JSON (sorted keys, no insignificant whitespace).
- `fasta_sha256` — SHA-256 of normalised stitched FASTA.
- `identity` — SHA-256 of `spec_sha256 || 0x1E || fasta_sha256`.

Same plan and molecule → same identity on any machine. Timestamp is metadata beside the key. Duplicate submit → reuse row. One fragment change → new identity.

## Metadata routers

A typed decision model may route a hash miss on metadata only: `rescreen | escalate | drop`. It must not see raw sequence and must not write `policy: RELEASE` to the ledger.

## Prohibited

- Invent a sequences-of-concern database.
- Treat the agent as authority for RELEASE.
- Upload customer or agent FASTA to third parties by default.
- Emit a numeric risk score as the clinical object.
- Clear HOLD via justification text.
- Train or publish function models on restricted corpora from this repository.
- Add evasion fixtures, commec bypass cases, or sequences of concern to git.
- Perform laboratory work. BSL-1 and BSL-2 only; BSL-3 is out of scope.

## Normative preamble

The following aligns with `speccheck` and must govern design-agent behavior:

```
Emit design specification JSON only.
Do not order, print oligos, or write wet protocols for HOLD or ESCALATE.
Do not argue the suppressor into RELEASE.
For ESCALATE, supply a one-page human brief:
intended function, chassis, why the gate fired, feasible BSL-1 or BSL-2 options.
Laboratory work is human. BSL-3 is out of scope.
```
