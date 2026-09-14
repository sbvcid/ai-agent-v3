//! Core semantic update boundary.
//!
//! This module applies already-constructed semantic updates to the authoritative
//! `KnowledgeStore`. It deliberately does not decide what an observation means.

use crate::core::knowledge_store::{
    EvidenceLink, KnowledgeClaim, KnowledgeStore, KnowledgeStoreError, Unknown,
};
use crate::core::observation_store::ObservationStore;
use crate::core::types::Observation;
use thiserror::Error;

/// A provider-independent semantic mutation requested by a Core semantic producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticUpdate {
    /// Atomically add a claim together with its required provenance.
    ClaimWithEvidence {
        claim: KnowledgeClaim,
        evidence: Vec<EvidenceLink>,
    },
    /// Add an evidence link to an existing claim.
    Evidence(EvidenceLink),
    /// Add a first-class unknown.
    Unknown(Unknown),
}

/// Error returned when a semantic update cannot be applied.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SemanticUpdateError {
    #[error("Knowledge store error: {0}")]
    KnowledgeStore(#[from] KnowledgeStoreError),
}

/// Produces an already-constructed semantic update from a recorded Observation.
///
/// The producer is the reasoning-side seam: it decides whether an Observation
/// warrants a semantic mutation, but it does not own or mutate the KnowledgeStore.
/// The first B2-C implementation does not provide a production reasoning engine;
/// the default producer is intentionally a no-op.
pub trait SemanticUpdateProducer: std::fmt::Debug {
    fn produce(&mut self, observation: &Observation) -> Option<SemanticUpdate>;
}

/// Default producer used by AgentLoop when no semantic producer is configured.
#[derive(Debug, Default)]
pub struct NoOpSemanticUpdateProducer;

impl SemanticUpdateProducer for NoOpSemanticUpdateProducer {
    fn produce(&mut self, _observation: &Observation) -> Option<SemanticUpdate> {
        None
    }
}

/// Core-level mutation boundary for derived semantic state.
///
/// Implementations receive an already-constructed semantic update. They do not
/// perform natural-language reasoning, truth evaluation, confidence scoring, or
/// provider-specific interpretation.
pub trait SemanticUpdater: std::fmt::Debug {
    fn apply(
        &mut self,
        update: SemanticUpdate,
        observations: &dyn ObservationStore,
        knowledge: &mut dyn KnowledgeStore,
    ) -> Result<(), SemanticUpdateError>;
}

/// Default in-memory semantic update boundary.
#[derive(Debug, Default)]
pub struct KnowledgeStoreSemanticUpdater;

impl KnowledgeStoreSemanticUpdater {
    pub fn new() -> Self {
        Self
    }
}

impl SemanticUpdater for KnowledgeStoreSemanticUpdater {
    fn apply(
        &mut self,
        update: SemanticUpdate,
        observations: &dyn ObservationStore,
        knowledge: &mut dyn KnowledgeStore,
    ) -> Result<(), SemanticUpdateError> {
        match update {
            SemanticUpdate::ClaimWithEvidence { claim, evidence } => {
                knowledge.record_claim_with_evidence(claim, evidence, observations)?;
            }
            SemanticUpdate::Evidence(link) => {
                knowledge.record_evidence(link, observations)?;
            }
            SemanticUpdate::Unknown(unknown) => {
                knowledge.record_unknown(unknown)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::knowledge_store::{EvidenceRelation, KnowledgeClaimStatus};
    use crate::core::observation_store::InMemoryObservationStore;
    use crate::core::types::{Observation, ObservationKind};

    fn observations() -> InMemoryObservationStore {
        let mut store = InMemoryObservationStore::new();
        store
            .record(Observation::new(
                "obs-1",
                ObservationKind::ActionResult,
                "action completed",
            ))
            .unwrap();
        store
    }

    fn observed_claim() -> KnowledgeClaim {
        KnowledgeClaim {
            id: "claim-1".into(),
            subject: "action-1".into(),
            predicate: "result".into(),
            value: "completed".into(),
            status: KnowledgeClaimStatus::Observed,
            scope: "current task".into(),
            evidence_refs: vec!["obs-1".into()],
        }
    }

    #[test]
    fn no_op_producer_does_not_create_semantic_state() {
        let observation = Observation::new("obs-1", ObservationKind::ActionResult, "completed");
        let mut producer = NoOpSemanticUpdateProducer;
        assert_eq!(producer.produce(&observation), None);
    }

    #[test]
    fn applies_claim_and_evidence_atomically() {
        let observations = observations();
        let mut knowledge = crate::core::knowledge_store::InMemoryKnowledgeStore::new();
        let mut updater = KnowledgeStoreSemanticUpdater::new();

        updater
            .apply(
                SemanticUpdate::ClaimWithEvidence {
                    claim: observed_claim(),
                    evidence: vec![EvidenceLink {
                        observation_id: "obs-1".into(),
                        claim_id: "claim-1".into(),
                        relation: EvidenceRelation::Supports,
                    }],
                },
                &observations,
                &mut knowledge,
            )
            .unwrap();

        assert_eq!(knowledge.claims().len(), 1);
        assert_eq!(knowledge.evidence().len(), 1);
        assert_eq!(knowledge.get_claim("claim-1"), Some(&observed_claim()));
    }

    #[test]
    fn failed_update_does_not_partially_modify_knowledge() {
        let observations = observations();
        let mut knowledge = crate::core::knowledge_store::InMemoryKnowledgeStore::new();
        let before = knowledge.clone();
        let mut updater = KnowledgeStoreSemanticUpdater::new();

        let error = updater
            .apply(
                SemanticUpdate::ClaimWithEvidence {
                    claim: observed_claim(),
                    evidence: vec![EvidenceLink {
                        observation_id: "missing-observation".into(),
                        claim_id: "claim-1".into(),
                        relation: EvidenceRelation::Supports,
                    }],
                },
                &observations,
                &mut knowledge,
            )
            .expect_err("missing observation must fail deterministically");

        assert!(matches!(
            error,
            SemanticUpdateError::KnowledgeStore(KnowledgeStoreError::MissingObservation(ref id))
                if id == "missing-observation"
        ));
        assert_eq!(knowledge, before);
    }

    #[test]
    fn applies_unknown_without_claim_or_evidence() {
        let observations = observations();
        let mut knowledge = crate::core::knowledge_store::InMemoryKnowledgeStore::new();
        let mut updater = KnowledgeStoreSemanticUpdater::new();

        updater
            .apply(
                SemanticUpdate::Unknown(Unknown {
                    id: "unknown-1".into(),
                    subject: "service".into(),
                    scope: "current task".into(),
                    question: "Which port is serving HTTP?".into(),
                }),
                &observations,
                &mut knowledge,
            )
            .unwrap();

        assert_eq!(knowledge.unknowns().len(), 1);
        assert!(knowledge.claims().is_empty());
        assert!(knowledge.evidence().is_empty());
    }
}
