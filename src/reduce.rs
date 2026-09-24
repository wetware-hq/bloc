use crate::pattern::PatternHit;
use crate::spec::IntendedFunction;
use crate::verdict::{Policy, Reason, Rubric};

pub struct EngineSignals {
    pub commec_required: bool,
    pub commec_ran: bool,
    pub commec_flag: bool,
    pub pattern: PatternHit,
    pub intended: IntendedFunction,
}

pub fn reduce(sig: &EngineSignals) -> (Rubric, Policy, Vec<Reason>) {
    if sig.commec_required && !sig.commec_ran {
        return (
            Rubric::Undefined,
            Policy::Escalate,
            vec![Reason {
                code: "engine_unavailable".into(),
                clause: "RELEASE requires commec".into(),
                engine: "commec".into(),
                fragment_id: None,
                detail: Some("commec missing or failed".into()),
            }],
        );
    }
    if sig.commec_flag {
        return (
            Rubric::Flag,
            Policy::Hold,
            vec![Reason {
                code: "commec_flag".into(),
                clause: "biorisk or regulated taxonomy uncleared".into(),
                engine: "commec".into(),
                fragment_id: None,
                detail: None,
            }],
        );
    }
    if sig.pattern.programmable_system_shape || sig.intended.escalates_if_uncleared() {
        return (
            Rubric::Undefined,
            Policy::Escalate,
            vec![Reason {
                code: "novelty_or_function".into(),
                clause: "ART-shaped pattern or RT/programmable/unknown function".into(),
                engine: "pattern".into(),
                fragment_id: None,
                detail: Some(format!(
                    "repeat_array={} rt_like_cds={}",
                    sig.pattern.repeat_array, sig.pattern.rt_like_cds
                )),
            }],
        );
    }
    (
        Rubric::NoFlag,
        Policy::Release,
        vec![Reason {
            code: "cleared".into(),
            clause: "commec clear and no novelty gate".into(),
            engine: "reduce".into(),
            fragment_id: None,
            detail: None,
        }],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PatternHit;
    use crate::spec::IntendedFunction;

    fn silent() -> PatternHit {
        PatternHit {
            repeat_array: false,
            rt_like_cds: false,
            programmable_system_shape: false,
        }
    }

    #[test]
    fn missing_commec_escalates() {
        let (_, p, _) = reduce(&EngineSignals {
            commec_required: true,
            commec_ran: false,
            commec_flag: false,
            pattern: silent(),
            intended: IntendedFunction::Reporter,
        });
        assert_eq!(p, Policy::Escalate);
    }

    #[test]
    fn commec_flag_holds() {
        let (r, p, _) = reduce(&EngineSignals {
            commec_required: true,
            commec_ran: true,
            commec_flag: true,
            pattern: silent(),
            intended: IntendedFunction::Reporter,
        });
        assert_eq!(r, Rubric::Flag);
        assert_eq!(p, Policy::Hold);
    }

    #[test]
    fn commec_clear_releases_reporter() {
        let (_, p, _) = reduce(&EngineSignals {
            commec_required: true,
            commec_ran: true,
            commec_flag: false,
            pattern: silent(),
            intended: IntendedFunction::Reporter,
        });
        assert_eq!(p, Policy::Release);
    }

    #[test]
    fn novelty_escalates_even_if_commec_cleared() {
        let (r, p, _) = reduce(&EngineSignals {
            commec_required: true,
            commec_ran: true,
            commec_flag: false,
            pattern: PatternHit {
                repeat_array: true,
                rt_like_cds: true,
                programmable_system_shape: true,
            },
            intended: IntendedFunction::ReverseTranscriptase,
        });
        assert_eq!(r, Rubric::Undefined);
        assert_eq!(p, Policy::Escalate);
    }
}
