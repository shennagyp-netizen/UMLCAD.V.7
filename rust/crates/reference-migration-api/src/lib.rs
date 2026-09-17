use umlcad_v6_reference_evolution_api::{ReferenceEvolution, ReferenceEvolutionError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceMigrationKind {
    Preserved,
    Invalidated,
    Split,
    Merged,
    Ambiguous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceMigrationEvidence {
    pub before_matches: usize,
    pub after_matches: usize,
    pub evolution: ReferenceEvolution,
    pub migration: ReferenceMigrationKind,
}

impl ReferenceMigrationKind {
    pub fn from_evolution(evolution: ReferenceEvolution) -> Self {
        match evolution {
            ReferenceEvolution::OneToOne => Self::Preserved,
            ReferenceEvolution::OneToZero => Self::Invalidated,
            ReferenceEvolution::OneToMany => Self::Split,
            ReferenceEvolution::ManyToOne => Self::Merged,
            ReferenceEvolution::ManyToMany => Self::Ambiguous,
        }
    }

    pub fn is_unique_preservation(self) -> bool {
        matches!(self, Self::Preserved)
    }

    pub fn is_automatic_identity_safe(self) -> bool {
        matches!(self, Self::Preserved)
    }
}

pub fn classify_reference_migration(
    before: usize,
    after: usize,
) -> Result<ReferenceMigrationEvidence, ReferenceEvolutionError> {
    let evolution = ReferenceEvolution::classify(before, after)?;
    Ok(ReferenceMigrationEvidence {
        before_matches: before,
        after_matches: after,
        evolution,
        migration: ReferenceMigrationKind::from_evolution(evolution),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_cardinality_to_explicit_migration() {
        assert_eq!(
            classify_reference_migration(1, 1).unwrap().migration,
            ReferenceMigrationKind::Preserved
        );
        assert_eq!(
            classify_reference_migration(1, 0).unwrap().migration,
            ReferenceMigrationKind::Invalidated
        );
        assert_eq!(
            classify_reference_migration(1, 2).unwrap().migration,
            ReferenceMigrationKind::Split
        );
        assert_eq!(
            classify_reference_migration(2, 1).unwrap().migration,
            ReferenceMigrationKind::Merged
        );
        assert_eq!(
            classify_reference_migration(2, 3).unwrap().migration,
            ReferenceMigrationKind::Ambiguous
        );
    }

    #[test]
    fn only_preserved_is_safe_identity_continuation() {
        assert!(ReferenceMigrationKind::Preserved.is_unique_preservation());
        assert!(ReferenceMigrationKind::Preserved.is_automatic_identity_safe());
        for kind in [
            ReferenceMigrationKind::Invalidated,
            ReferenceMigrationKind::Split,
            ReferenceMigrationKind::Merged,
            ReferenceMigrationKind::Ambiguous,
        ] {
            assert!(!kind.is_unique_preservation());
            assert!(!kind.is_automatic_identity_safe());
        }
    }

    #[test]
    fn evidence_retains_source_and_target_cardinality() {
        let evidence = classify_reference_migration(4, 1).unwrap();
        assert_eq!(evidence.before_matches, 4);
        assert_eq!(evidence.after_matches, 1);
        assert_eq!(evidence.evolution, ReferenceEvolution::ManyToOne);
        assert_eq!(evidence.migration, ReferenceMigrationKind::Merged);
    }

    #[test]
    fn zero_cardinality_remains_fail_closed() {
        assert_eq!(
            classify_reference_migration(0, 1),
            Err(ReferenceEvolutionError::EmptyBefore)
        );
        assert_eq!(
            classify_reference_migration(2, 0),
            Err(ReferenceEvolutionError::MultipleTargetsLost)
        );
    }

    #[test]
    fn extreme_cardinality_values_do_not_overflow_or_change_class() {
        let successful_cases = [
            (usize::MAX, usize::MAX, ReferenceMigrationKind::Ambiguous),
            (usize::MAX, 1, ReferenceMigrationKind::Merged),
            (1, usize::MAX, ReferenceMigrationKind::Split),
        ];
        for (before, after, expected) in successful_cases {
            assert_eq!(
                classify_reference_migration(before, after).unwrap().migration,
                expected
            );
        }

        assert_eq!(
            classify_reference_migration(usize::MAX, 0),
            Err(ReferenceEvolutionError::MultipleTargetsLost)
        );
    }

    #[test]
    fn migration_mapping_is_exhaustive_and_preserves_authoritative_cardinality() {
        for before in 1..=32 {
            for after in 1..=32 {
                let evidence = classify_reference_migration(before, after).unwrap();
                assert_eq!(evidence.before_matches, before);
                assert_eq!(evidence.after_matches, after);
                assert_eq!(
                    evidence.migration,
                    ReferenceMigrationKind::from_evolution(evidence.evolution)
                );

                let expected = match (before == 1, after == 1) {
                    (true, true) => ReferenceMigrationKind::Preserved,
                    (true, false) => ReferenceMigrationKind::Split,
                    (false, true) => ReferenceMigrationKind::Merged,
                    (false, false) => ReferenceMigrationKind::Ambiguous,
                };
                assert_eq!(evidence.migration, expected);
            }
        }
    }

    #[test]
    fn repeated_classification_is_deterministic() {
        let cases = [
            (1usize, 0usize),
            (1, 1),
            (1, 7),
            (7, 1),
            (7, 11),
            (usize::MAX, usize::MAX),
        ];
        for _ in 0..10_000 {
            for (before, after) in cases {
                assert_eq!(
                    classify_reference_migration(before, after),
                    classify_reference_migration(before, after)
                );
            }
        }
    }

    #[test]
    fn non_preserved_results_never_claim_automatic_identity_safety() {
        for (before, after) in [(1, 0), (1, 2), (2, 1), (2, 3)] {
            let evidence = classify_reference_migration(before, after).unwrap();
            assert!(!evidence.migration.is_unique_preservation());
            assert!(!evidence.migration.is_automatic_identity_safe());
        }
    }
}
