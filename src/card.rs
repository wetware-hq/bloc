use crate::verdict::{Policy, Verdict};

pub fn render(v: &Verdict) -> String {
    let do_line = match v.policy {
        Policy::Release => "do:      cleared for BSL-1 construct build",
        Policy::Hold | Policy::Escalate => "do:      do not order, do not assemble, do not transform",
    };
    let next = match v.policy {
        Policy::Release => "next:    none",
        Policy::Hold => "next:    biosafety review with the design specification and commec JSON",
        Policy::Escalate => "next:    session host / biosafety",
    };
    let why = v
        .reasons
        .first()
        .map(|r| format!("{}: {}", r.code, r.clause))
        .unwrap_or_else(|| "n/a".into());
    format!(
        "BLOC     construct_id={}\nverdict: {}\nrubric:  {}\nwhy:     {}\n{}\n{}\nreceipt: {}  bloc@{}",
        v.construct_id,
        v.policy.word(),
        v.rubric.word(),
        why,
        do_line,
        next,
        v.receipt.identity,
        v.receipt.inhibit_version
    )
}
