# Design agent integration

Normative requirements for automated design tools that participate in this repository. Wording uses RFC 2119: **MUST**, **MUST NOT**, **SHOULD**.

## Scope

Design tools **MUST** emit [design specification](schemas/design_spec.schema.json) JSON (`spec_version` `0.1.0`) as the only sequence-bearing artifact toward synthesis. Raw FASTA, vendor carts, and wet-laboratory protocols **MUST NOT** bypass `bloc screen`. Release policy **MUST** come from `bloc screen`, not from the design tool.

## Interface

The host integration surface is the `inhibit_submit_design` tool (see `tools/agent/inhibit_submit_design.json`), which runs `bloc screen` on a specification file and returns a [verdict](schemas/verdict.schema.json) and card.

```
inhibit_submit_design  →  design specification JSON  →  bloc screen  →  Verdict + card
```

Hosts **MUST NOT** expose parallel paths (for example, writing `construct.fa` and calling a synthesizer API) that skip screening.

## Design specification fields

Required root fields: `construct_id`, `designer` (`agent`, `human`), `chassis`, `intended_function`, `intended_bsl`, `not_for_synthesis`, `fragments[]` (`id`, `role`, `alphabet`, `sequence`). Additional properties **MUST NOT** appear at the schema root.

Enumerations match `schemas/design_spec.schema.json`. Sequences **MUST** match the declared alphabet. Notes, citations, and justification fields **MUST NOT** be added; they do not alter HOLD.

## Censor (`bloc speccheck`)

`speccheck` **MUST** refuse the plan (exit `2`) when any of the following hold:

- `intended_bsl` is `BSL-3`
- `assembly.order_split` is `true`
- unknown JSON keys
- alphabet or sequence mismatch
- empty `fragments`
- stitched DNA length above 50 kb (v0 cap)
- synthesis, oligo printing, or wet work would occur before a `RELEASE` verdict exists

## Suppressor (`bloc screen`)

Pipeline order:

1. Stitch fragments in list order; normalise (`U→T`, uppercase, no whitespace).
2. Window (50 nt recorded; 30 nt oligo floor for oligo roles).
3. Structural novelty gate (repeat-array plus RT-plausible CDS, or declared `reverse_transcriptase`, `programmable_nuclease_system`, or `unknown` function).
4. Local IBBIS `commec` when configured.
5. Policy reduction (total order, no votes):

```
commec missing or crashed     → Undefined / ESCALATE
commec uncleared hit          → Flag / HOLD
ART-shaped pattern or
  RT / programmable / unknown → Undefined / ESCALATE
else, and commec cleared      → No Flag / RELEASE
```

`RELEASE` **MUST NOT** be emitted unless `commec` completed and cleared. Integrations **MUST** branch on exit codes (`0` RELEASE, `10` HOLD, `20` ESCALATE, `2` error), not on card prose.

## Post-verdict behavior

| Policy | Requirement |
|---|---|
| `RELEASE` | Textual build guidance for BSL-1 intent is permitted; wet-laboratory execution **MUST NOT** be performed by the tool. |
| `HOLD` | Further synthesis steps **MUST** stop; the human reviewer **MUST** receive the specification and commec output. |
| `ESCALATE` | A concise review packet **SHOULD** be supplied: intended function, chassis, triggering gate, and feasible BSL-1 or BSL-2 options. Pathogen names **MUST NOT** be invented. |

Batch discovery **MUST** screen every candidate. All `RELEASE` verdicts **MUST** be retained. At most twenty `ESCALATE` outcomes **MAY** be queued for human review; overflow is logged, excluded from the human queue, and **MUST NOT** be treated as `RELEASE`.

## Receipt identity

- `spec_sha256` — SHA-256 of canonical JSON (sorted keys, no insignificant whitespace).
- `fasta_sha256` — SHA-256 of normalised stitched FASTA.
- `identity` — SHA-256 of `spec_sha256 || 0x1E || fasta_sha256`.

Identical specification and molecule **MUST** yield the same `identity`. Timestamp and card text are metadata outside `identity`.

## Metadata routers

After an `identity` miss, an optional router **MAY** choose `rescreen`, `escalate`, or `drop` using metadata only. Routers **MUST NOT** receive raw sequence and **MUST NOT** assign `policy: RELEASE`.

## Out of scope for this repository

- Customer or agent FASTA uploaded to third parties by default
- Numeric risk scores as the primary decision object
- Sequences-of-concern databases, evasion fixtures, or commec bypass tests in git
- BSL-3 intent or laboratory execution by automated tools
