# bloc

**System card.** bloc is a local gate that screens agent-proposed nucleic-acid designs before synthesis or assembly. It accepts a structured design specification, not raw sequence files. The only object meant for human action is a short typed card. Tracking: [MCHU-63](https://linear.app/mchu001/issue/MCHU-63/bloc).

bloc evaluates whether a proposed construct may proceed toward ordering, assembly, or transformation under the stated biosafety level. One program performs plan review and molecule review. Policy is fixed by rule, not by narrative from the design tool.

---

## Clinical abstract

Automated design tools can now propose genetic constructs quickly. The practical control point remains the moment someone orders DNA, assembles a plasmid, or introduces material into cells. bloc sits at that control point on a local machine.

The session host or biosafety officer does not read a long model explanation. They read one card of seven lines or fewer:

```
BLOC     construct_id=…
verdict: HOLD | RELEASE | ESCALATE
rubric:  Flag | No Flag | Undefined
why:     short clause
do:      physical instruction
next:    named human, or none
receipt: content-addressed identity
```

Three outcomes, aligned with sequence-biosecurity practice:

| Rubric | Policy | Synthesis or assembly | Human review |
|---|---|---|---|
| No Flag | RELEASE | permitted at declared BSL-1 intent | not required |
| Flag | HOLD | forbidden | required unless a prior exemption applies |
| Undefined | ESCALATE | forbidden | required |

Undefined is the expected outcome when structure or declared function is novel—for example, a regular repeat array adjacent to a reverse-transcriptase-like coding region. It is not a software crash. It is a request for qualified review.

The card is not a clinical diagnosis, a pathogen identification, a quantitative risk score, or authorization for BSL-3 work. Wet-laboratory steps remain human responsibilities. If the local commec screen does not run, the policy is ESCALATE. Absence of a warning is not clearance.

---

## Intended use

Use bloc to require a typed verdict before a designed sequence moves toward synthesis. Use it to screen a design specification on a laptop during a Wetware session. Use the receipt so an identical plan and molecule receive the same decision later.

Do not use bloc to replace IBBIS, SBRC, SecureDNA, or a commercial provider’s compliance program. Do not use it for customer identity, hosted sequence upload, or training predictive models on restricted corpora.

---

## Engineering specification

### System model

A design agent emits a design specification only. The censor (`speccheck`) validates the plan. The suppressor (`screen`) evaluates the stitched molecule. A reducer assigns rubric and policy. The card and receipt are the user-facing outputs.

```
design agent
    │  design specification (JSON)
    ▼
CENSOR   bloc speccheck     refuse plan (exit 2)
    │
    ▼
molecule (stitched, normalised)
    │
SUPPRESSOR   bloc screen
    │  windows · pattern gate · commec (local)
    ▼
reducer (total order, no votes)
    │
Verdict + card + receipt
    │
RELEASE | HOLD | ESCALATE     exits 0 / 10 / 20
```

The implementation is a single Rust binary with two subcommands. IBBIS commec supplies local homology and regulated-taxonomy screening; it does not set release policy. Automated design tools do not set release policy.

### Inputs and outputs

Input is design specification JSON at `spec_version` `0.1.0`. Unknown keys are rejected. Required fields: construct identifier, designer (agent and human), chassis, intended function, intended BSL, `not_for_synthesis`, and fragments each with role, alphabet, and sequence.

Output is a Verdict JSON object and a card of at most seven lines. The policy word is authoritative; the physical line is either `do not order / assemble / transform` or `cleared for BSL-1 construct build`.

### Censor predicates

The censor fails closed before external databases run. It refuses BSL-3 intent, `assembly.order_split: true`, extra JSON fields, alphabet or sequence mismatch, an empty fragment list, and stitched DNA above the fifty-kilobase v0 cap.

### Suppressor engines

The normaliser converts `U` to `T`, uppercases letters, concatenates fragments in list order, records fifty-nucleotide windows (thirty-nucleotide minimum for oligo roles), and exposes six-frame translation to adapters.

The pattern gate is structural only. It fires on evenly spaced repeats (≥6 units, period 20–50 nt) together with a reverse-transcriptase-plausible coding sequence, or when intended function is `reverse_transcriptase`, `programmable_nuclease_system`, or `unknown`. It does not embed a pathogen list.

The commec adapter invokes a local subprocess. An uncleared biorisk or regulated-taxonomy hit yields Flag and HOLD. Absence or failure yields Undefined and ESCALATE. RELEASE requires a completed commec run with clearance.

### Policy reducer

```
engine unavailable          → Undefined / ESCALATE
commec uncleared hit        → Flag / HOLD
pattern or RT/programmable
  / unknown function        → Undefined / ESCALATE
else, commec cleared        → No Flag / RELEASE
```

Additional engines may raise HOLD or ESCALATE in future versions. None may be the sole basis for RELEASE.

### Receipt

```
spec_sha256  = SHA-256(canonical JSON)
fasta_sha256 = SHA-256(normalised stitched FASTA)
identity     = SHA-256(spec_sha256 || 0x1E || fasta_sha256)
```

Canonical JSON uses sorted keys and no insignificant whitespace. Timestamp and card text sit beside `identity`, not inside it. Identical submissions reuse the same ledger row; one changed codon yields a new row.

### Repeat screening

An exact `identity` match reuses the stored verdict without re-invoking design tools. On a miss, an optional metadata-only router may choose `rescreen`, `escalate`, or `drop`; it must not receive raw sequence and must not assign RELEASE. A generative tool may draft the ESCALATE brief; it must not set policy.

### Commands

```
bloc speccheck  spec.json
bloc screen     spec.json [--commec-db DIR] [--commec-bin PATH]
bloc card       verdict.json
bloc receipt    verdict.json
```

```
cargo test
cargo run -- speccheck fixtures/benign/gfp_spec.json
cargo run -- screen    fixtures/benign/gfp_spec.json
cargo run -- screen    fixtures/synthetic/art_shape_spec.json
```

Exit codes: `0` RELEASE, `10` HOLD, `20` ESCALATE, `2` schema or engine error. Integrations should branch on the numeric code, not on card wording.

### Failure and misuse

Without commec, policy is ESCALATE, never RELEASE. Justification fields in the specification cause `speccheck` to refuse the plan. Order splitting across vendors is refused at the censor. A novel ART-like shape that commec does not flag still yields ESCALATE. Emitting raw FASTA outside the specification channel is a host integration error; see `AGENTS.md`.

Repository fixtures are benign or purely structural. Licensed sequences-of-concern from proficiency sets are not vendored here.

---

## Operators

**Clinician or session host:** Read the card. For HOLD or ESCALATE, do not order or transform material. For RELEASE, any BSL-1 build remains a deliberate human act.

**Engineer:** Do not add numeric risk scores, model-based judges, proprietary threat lists, or Python on the thirty-mer path. Consume commec output; emit Flag, No Flag, or Undefined. Preserve fail-closed behavior.

**Design agent:** Follow `AGENTS.md` and emit specification JSON only.

---

## Status

v0.1 frame specification and CLI contract. License: MIT. No sequences of concern in this repository. Wet-laboratory execution is out of process.
