#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceEvolution {
    OneToOne,
    OneToZero,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceEvolutionError {
    EmptyBefore,
    MultipleTargetsLost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceEvolutionEvidence {
    pub before_matches: usize,
    pub after_matches: usize,
    pub evolution: ReferenceEvolution,
    pub unique_identity_preserved: bool,
}

impl ReferenceEvolution {
    /// Classify topology/reference mapping cardinality without guessing identity.
    ///
    /// The counts are semantic match counts produced by an upstream descriptor
    /// resolver. This layer deliberately does not inspect coordinates, renderer
    /// order, native handles, or storage order.
    pub fn classify(before: usize, after: usize) -> Result<Self, ReferenceEvolutionError> {
        if before == 0 {
            return Err(ReferenceEvolutionError::EmptyBefore);
        }
        if after == 0 {
            return if before == 1 {
                Ok(Self::OneToZero)
            } else {
                Err(ReferenceEvolutionError::MultipleTargetsLost)
            };
        }
        Ok(match (before == 1, after == 1) {
            (true, true) => Self::OneToOne,
            (true, false) => Self::OneToMany,
            (false, true) => Self::ManyToOne,
            (false, false) => Self::ManyToMany,
        })
    }

    pub fn is_ambiguous(self) -> bool {
        matches!(self, Self::OneToMany | Self::ManyToOne | Self::ManyToMany)
    }

    pub fn preserves_a_unique_target(self) -> bool {
        matches!(self, Self::OneToOne)
    }

    pub fn evidence(before: usize, after: usize) -> Result<ReferenceEvolutionEvidence, ReferenceEvolutionError> {
        let evolution = Self::classify(before, after)?;
        Ok(ReferenceEvolutionEvidence {
            before_matches: before,
            after_matches: after,
            evolution,
            unique_identity_preserved: evolution.preserves_a_unique_target(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_all_declared_evolution_classes() {
        assert_eq!(ReferenceEvolution::classify(1, 1), Ok(ReferenceEvolution::OneToOne));
        assert_eq!(ReferenceEvolution::classify(1, 0), Ok(ReferenceEvolution::OneToZero));
        assert_eq!(ReferenceEvolution::classify(1, 2), Ok(ReferenceEvolution::OneToMany));
        assert_eq!(ReferenceEvolution::classify(2, 1), Ok(ReferenceEvolution::ManyToOne));
        assert_eq!(ReferenceEvolution::classify(2, 3), Ok(ReferenceEvolution::ManyToMany));
    }

    #[test]
    fn zero_before_fails_closed() {
        assert_eq!(ReferenceEvolution::classify(0, 1), Err(ReferenceEvolutionError::EmptyBefore));
        assert_eq!(ReferenceEvolution::classify(0, 0), Err(ReferenceEvolutionError::EmptyBefore));
    }

    #[test]
    fn multiple_targets_lost_fails_closed() {
        assert_eq!(ReferenceEvolution::classify(2, 0), Err(ReferenceEvolutionError::MultipleTargetsLost));
    }

    #[test]
    fn ambiguous_mappings_are_explicit() {
        assert!(!ReferenceEvolution::OneToOne.is_ambiguous());
        assert!(!ReferenceEvolution::OneToZero.is_ambiguous());
        assert!(ReferenceEvolution::OneToMany.is_ambiguous());
        assert!(ReferenceEvolution::ManyToOne.is_ambiguous());
        assert!(ReferenceEvolution::ManyToMany.is_ambiguous());
    }

    #[test]
    fn only_one_to_one_preserves_unique_identity() {
        assert!(ReferenceEvolution::OneToOne.preserves_a_unique_target());
        assert!(!ReferenceEvolution::OneToZero.preserves_a_unique_target());
        assert!(!ReferenceEvolution::OneToMany.preserves_a_unique_target());
        assert!(!ReferenceEvolution::ManyToOne.preserves_a_unique_target());
        assert!(!ReferenceEvolution::ManyToMany.preserves_a_unique_target());
    }

    #[test]
    fn evidence_preserves_source_cardinalities_and_decision() {
        let evidence = ReferenceEvolution::evidence(3, 1).unwrap();
        assert_eq!(evidence.before_matches, 3);
        assert_eq!(evidence.after_matches, 1);
        assert_eq!(evidence.evolution, ReferenceEvolution::ManyToOne);
        assert!(!evidence.unique_identity_preserved);
    }

    #[test]
    fn classification_and_evidence_are_deterministic() {
        for _ in 0..10_000 {
            assert_eq!(ReferenceEvolution::classify(1, 0), Ok(ReferenceEvolution::OneToZero));
            let evidence = ReferenceEvolution::evidence(1, 1).unwrap();
            assert_eq!(evidence.evolution, ReferenceEvolution::OneToOne);
            assert!(evidence.unique_identity_preserved);
        }
    }
}
