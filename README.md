# bloc

**System card.** Local inhibitory frame for agent-designed nucleic-acid sequences. Wetware issue [MCHU-63](https://linear.app/mchu001/issue/MCHU-63/bloc).

bloc blocks a designer sequence that may pose a biosecurity risk before anyone synthesises it. One process exposes two faces. A typed card is the only object a human has to act on.

---

## Clinical abstract

Frontier models now propose enzymes and constructs by reading genomic neighborhoods at swarm scale. The physical chokepoint is still a tube, a vendor cart, or a bench protocol. bloc is the local gate on that chokepoint.

A clinician or session host does not read a model essay. They read one card of seven lines or fewer.

```
BLOC     construct_id=…
verdict: HOLD | RELEASE | ESCALATE
rubric:  Flag | No Flag | Undefined
why:     short clause
do:      physical instruction
next:    named human, or none
receipt: content-addressed identity
```

Three outcomes, in the language already used by sequence-biosecurity standards:

| Rubric | Policy | Tube or vendor | Human |
|---|---|---|---|
| No Flag | RELEASE | allowed at BSL-1 intent | not required |
| Flag | HOLD | forbidden | only to honour a pre-issued exemption |
| Undefined | ESCALATE | forbidden | required |

Undefined is the expected path for a novel neighborhood, for example a repeat array beside an odd polymerase or reverse transcriptase. It is not a software failure. It is the request for a person.

This card is not a diagnosis, a pathogen name, a risk percentage, or permission to handle BSL-3 work. Laboratory work stays human. The model does not pipette. If the local screening engine is missing, the card is ESCALATE. Silence is not clearance.

---

## Intended use

Use bloc to bind a design agent so that a sequence cannot move toward synthesis without a typed verdict. Use it to screen a design specification on a laptop at a Wetware session before anyone talks about ordering oligos. Use the receipt so that the same construct is the same decision later.

Do not use bloc to replace IBBIS, SBRC, SecureDNA, or a commercial synthesis provider’s compliance stack. Do not use it as a customer-identity product, a hosted FASTA upload, or a training set for function models on restricted corpora.

---

## Engineering specification

### System model

A design agent emits a design specification only. The censor inspects that plan. The suppressor inspects the stitched molecule. The reducer writes a verdict. The card and the receipt are the only user-facing objects.

```
design agent
    │  design specification only
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

Rust is the containing frame. Censor and suppressor are two subcommands of one binary. IBBIS commec is borrowed muscle. It does not own policy. A language model does not own policy.

### Inputs and outputs

The input is a design specification as JSON, version `0.1.0`. Unknown keys are forbidden. The required fields are construct identity, designer (agent and human), chassis, intended function, intended BSL, a not-for-synthesis flag, and fragments that each carry a role, an alphabet, and a sequence.

The output is a Verdict JSON object and a card of seven lines or fewer. The first line of action is the policy word. The physical line is either `do not order / assemble / transform` or `cleared for BSL-1 construct build`.

### Censor predicates

The frame fails closed before any database is touched. It refuses BSL-3, `order_split: true`, extra fields, alphabet or sequence mismatch, an empty fragment list, and stitched DNA above the fifty-kilobase v0 cap.

### Suppressor engines

The normaliser rewrites `U` to `T`, uppercases the letters, concatenates fragments in listed order, records fifty-nucleotide windows with a thirty-nucleotide oligo floor, and makes a six-frame translation available to adapters.

The pattern gate is structural only. It fires on evenly spaced repeats of at least six units with period twenty to fifty nucleotides plus an RT-plausible coding sequence, or on a declared function in the set `{reverse_transcriptase, programmable_nuclease_system, unknown}`. It contains no pathogen table.

The commec adapter is a local subprocess. An uncleared biorisk or regulated-taxonomy hit becomes Flag and HOLD. A crash or absence becomes Undefined and ESCALATE. RELEASE is illegal unless commec ran and cleared.

### Policy reducer

```
engine unavailable          → Undefined / ESCALATE
commec uncleared hit        → Flag / HOLD
pattern or RT/programmable
  / unknown function        → Undefined / ESCALATE
else                        → No Flag / RELEASE
```

Later optional engines may raise HOLD or ESCALATE. None of them may be the sole author of RELEASE.

### Receipt

SHA-256 is deterministic on bytes. The ledger key is therefore only stable if those bytes are frozen.

```
spec_sha256  = SHA-256(canonical JSON)
fasta_sha256 = SHA-256(normalised stitched FASTA)
identity     = SHA-256(spec_sha256 || 0x1E || fasta_sha256)
```

Canonical JSON means sorted keys and no insignificant whitespace. The timestamp and the card sit beside that identity. They are not mixed into it, or yesterday’s screen and today’s identical rescreen become two keys. An identical submit is the same row. One codon changed is a new row. Pretty construct identifiers are not keys.

### Fast path after the first screen

An exact identity hit is System 0: reuse the stored verdict. No model is consulted. A typed System One decision model, for example Jev using Choice, Score, or Noul, may route a hash miss on metadata only, choosing `rescreen`, `escalate`, or `drop`. It must not receive raw sequence and must not mint RELEASE. A generative model may write the ESCALATE brief. It may not write the policy field.

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

Exit codes are `0` for RELEASE, `10` for HOLD, `20` for ESCALATE, and `2` for a schema or engine error. Branch on the code, not on the card’s English.

### Failure and misuse

If commec is absent, the card is ESCALATE and never RELEASE. If an agent adds justification keys, speccheck refuses the plan. If an agent splits an order across vendors, the censor hard-fails. If a novel ART-like shape is silent in commec, the card is ESCALATE. If a host gives the model a raw FASTA shell, that is a protocol violation; see AGENTS.md. Default upload of FASTA to a third party is forbidden.

Public fixtures in this repository are benign or purely structural. Sequences of concern from SBRC or NIST proficiency sets are licensed to organisations and are not vendored here.

---

## Operators

A clinician or session host reads the card. If the policy is HOLD or ESCALATE, nothing is ordered or transformed. If the policy is RELEASE, a BSL-1 build remains a human act.

An engineer does not add a risk score, an LLM judge, a new threat database, or Python on the thirty-mer path. Consume commec. Speak Flag, No Flag, or Undefined. Keep the frame fail-closed.

A frontier model reads AGENTS.md and emits a design specification only.

---

## Status

This is the v0.1 frame specification and CLI contract. License is MIT. No sequences of concern are in git. Wet laboratory work is out of process.
