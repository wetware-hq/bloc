use crate::pattern::PatternHit;
use crate::spec::{Bsl, IntendedFunction};
use crate::verdict::{Policy, Reason, Rubric};

pub struct EngineSignals {
    pub commec_required: bool,
    pub commec_ran: bool,
    pub commec_flag: bool,
    pub pattern: PatternHit,
    pub intended: IntendedFunction,
    pub intended_bsl: Bsl,
    pub not_for_synthesis: bool,
}

impl Default for EngineSignals {
    fn default() -> Self {
        Self {
            commec_required: false,
            commec_ran: false,
            commec_flag: false,
            pattern: PatternHit {
                repeat_array: false,
                rt_like_cds: false,
                programmable_system_shape: false,
            },
            intended: IntendedFunction::Reporter,
            intended_bsl: Bsl::Bsl1,
            not_for_synthesis: false,
        }
    }
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
    if !matches!(sig.intended_bsl, Bsl::Bsl1) {
        return (
            Rubric::Undefined,
            Policy::Escalate,
            vec![Reason {
                code: "bsl_not_bsl1".into(),
                clause: "RELEASE requires declared BSL-1 intent".into(),
                engine: "reduce".into(),
                fragment_id: None,
                detail: None,
            }],
        );
    }
    if sig.not_for_synthesis {
        return (
            Rubric::Undefined,
            Policy::Escalate,
            vec![Reason {
                code: "not_for_synthesis".into(),
                clause: "plan attests not for synthesis or ordering".into(),
                engine: "reduce".into(),
                fragment_id: None,
                detail: None,
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
    use crate::spec::{Bsl, IntendedFunction};

    fn release_ready() -> EngineSignals {
        EngineSignals {
            commec_required: true,
            commec_ran: true,
            commec_flag: false,
            pattern: PatternHit {
                repeat_array: false,
                rt_like_cds: false,
                programmable_system_shape: false,
            },
            intended: IntendedFunction::Reporter,
            intended_bsl: Bsl::Bsl1,
            not_for_synthesis: false,
        }
    }

    #[test]
    fn missing_commec_escalates() {
        let mut sig = release_ready();
        sig.commec_ran = false;
        let (_, p, _) = reduce(&sig);
        assert_eq!(p, Policy::Escalate);
    }

    #[test]
    fn commec_flag_holds() {
        let mut sig = release_ready();
        sig.commec_flag = true;
        let (r, p, _) = reduce(&sig);
        assert_eq!(r, Rubric::Flag);
        assert_eq!(p, Policy::Hold);
    }

    #[test]
    fn commec_clear_releases_reporter() {
        let (_, p, _) = reduce(&release_ready());
        assert_eq!(p, Policy::Release);
    }

    #[test]
    fn not_for_synthesis_escalates() {
        let mut sig = release_ready();
        sig.not_for_synthesis = true;
        let (r, p, reasons) = reduce(&sig);
        assert_eq!(r, Rubric::Undefined);
        assert_eq!(p, Policy::Escalate);
        assert_eq!(reasons[0].code, "not_for_synthesis");
    }

    #[test]
    fn bsl2_escalates() {
        let mut sig = release_ready();
        sig.intended_bsl = Bsl::Bsl2;
        let (r, p, reasons) = reduce(&sig);
        assert_eq!(r, Rubric::Undefined);
        assert_eq!(p, Policy::Escalate);
        assert_eq!(reasons[0].code, "bsl_not_bsl1");
    }

    #[test]
    fn bsl2_commec_flag_still_holds() {
        let mut sig = release_ready();
        sig.intended_bsl = Bsl::Bsl2;
        sig.commec_flag = true;
        let (r, p, _) = reduce(&sig);
        assert_eq!(r, Rubric::Flag);
        assert_eq!(p, Policy::Hold);
    }

    #[test]
    fn novelty_escalates_even_if_commec_cleared() {
        let mut sig = release_ready();
        sig.pattern = PatternHit {
            repeat_array: true,
            rt_like_cds: true,
            programmable_system_shape: true,
        };
        sig.intended = IntendedFunction::ReverseTranscriptase;
        let (r, p, _) = reduce(&sig);
        assert_eq!(r, Rubric::Undefined);
        assert_eq!(p, Policy::Escalate);
    }
}
