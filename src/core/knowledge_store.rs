//! Agent Core Knowledge / Evidence semantic foundation.
//!
//! `KnowledgeStore` owns derived semantic state only. `ObservationStore` remains
//! the authoritative owner of observation history; evidence links reference
//! observations by stable identity.

use crate::core::observation_store::ObservationStore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeClaimStatus {
    Observed,
    Inferred,
    Hypothesis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    Supports,
    Contradicts,
    Qualifies,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeClaim {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub value: String,
    pub status: KnowledgeClaimStatus,
    pub scope: String,
    /// Stable observation IDs referenced by this claim's provenance.
    /// EvidenceLink remains the authoritative relationship between a claim and observation.
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceLink {
    pub observation_id: String,
    pub claim_id: String,
    pub relation: EvidenceRelation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unknown {
    pub id: String,
    pub subject: String,
    pub scope: String,
    pub question: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KnowledgeStoreError {
    #[error("Knowledge validation error: {0}")]
    Validation(String),
    #[error("Duplicate knowledge claim ID: {0}")]
    DuplicateClaimId(String),
    #[error("Duplicate evidence link: observation '{observation_id}', claim '{claim_id}', relation '{relation:?}'")]
    DuplicateEvidenceLink {
        observation_id: String,
        claim_id: String,
        relation: EvidenceRelation,
    },
    #[error("Duplicate unknown ID: {0}")]
    DuplicateUnknownId(String),
    #[error("Referenced knowledge claim does not exist: {0}")]
    MissingClaim(String),
    #[error("Referenced observation does not exist: {0}")]
    MissingObservation(String),
}

pub trait KnowledgeStore: std::fmt::Debug {
    /// Records a claim atomically. Observed/Inferred claims must use
    /// `record_claim_with_evidence` so provenance is never transiently invalid.
    fn record_claim(
        &mut self,
        claim: KnowledgeClaim,
        observations: &dyn ObservationStore,
    ) -> Result<(), KnowledgeStoreError>;

    /// Atomically records an Observed/Inferred claim and its evidence links.
    fn record_claim_with_evidence(
        &mut self,
        claim: KnowledgeClaim,
        evidence: Vec<EvidenceLink>,
        observations: &dyn ObservationStore,
    ) -> Result<(), KnowledgeStoreError>;

    fn record_evidence(
        &mut self,
        link: EvidenceLink,
        observations: &dyn ObservationStore,
    ) -> Result<(), KnowledgeStoreError>;

    fn record_unknown(&mut self, unknown: Unknown) -> Result<(), KnowledgeStoreError>;

    fn get_claim(&self, id: &str) -> Option<&KnowledgeClaim>;
    fn claims(&self) -> Vec<&KnowledgeClaim>;
    fn evidence(&self) -> Vec<&EvidenceLink>;
    fn unknowns(&self) -> Vec<&Unknown>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    /// Validate all semantic invariants against the authoritative observation store.
    fn validate(&self, observations: &dyn ObservationStore) -> Result<(), KnowledgeStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct InMemoryKnowledgeStore {
    claims: Vec<KnowledgeClaim>,
    evidence: Vec<EvidenceLink>,
    unknowns: Vec<Unknown>,
}

impl InMemoryKnowledgeStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn validate_claim(claim: &KnowledgeClaim) -> Result<(), KnowledgeStoreError> {
        if claim.id.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("claim id cannot be empty".into()));
        }
        if claim.subject.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("claim subject cannot be empty".into()));
        }
        if claim.predicate.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("claim predicate cannot be empty".into()));
        }
        if claim.value.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("claim value cannot be empty".into()));
        }
        if claim.scope.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("claim scope cannot be empty".into()));
        }
        if claim.evidence_refs.iter().any(|id| id.trim().is_empty()) {
            return Err(KnowledgeStoreError::Validation("claim evidence_refs cannot contain empty IDs".into()));
        }
        Ok(())
    }

    fn validate_unknown(unknown: &Unknown) -> Result<(), KnowledgeStoreError> {
        if unknown.id.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("unknown id cannot be empty".into()));
        }
        if unknown.subject.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("unknown subject cannot be empty".into()));
        }
        if unknown.scope.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("unknown scope cannot be empty".into()));
        }
        if unknown.question.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation("unknown question cannot be empty".into()));
        }
        Ok(())
    }

    fn validate_new_evidence(
        &self,
        links: &[EvidenceLink],
        observations: &dyn ObservationStore,
    ) -> Result<(), KnowledgeStoreError> {
        for link in links {
            if link.observation_id.trim().is_empty() || link.claim_id.trim().is_empty() {
                return Err(KnowledgeStoreError::Validation(
                    "evidence observation_id and claim_id cannot be empty".into(),
                ));
            }
            if observations.get(&link.observation_id).is_none() {
                return Err(KnowledgeStoreError::MissingObservation(link.observation_id.clone()));
            }
            if self.get_claim(&link.claim_id).is_some() {
                // The caller may be updating an existing claim only through a future
                // semantic update API; this increment does not define claim mutation.
            }
            if self.evidence.iter().any(|existing| existing == link)
                || links.iter().filter(|existing| *existing == link).count() > 1
            {
                return Err(KnowledgeStoreError::DuplicateEvidenceLink {
                    observation_id: link.observation_id.clone(),
                    claim_id: link.claim_id.clone(),
                    relation: link.relation,
                });
            }
        }
        Ok(())
    }
}

impl KnowledgeStore for InMemoryKnowledgeStore {
    fn record_claim(
        &mut self,
        claim: KnowledgeClaim,
        observations: &dyn ObservationStore,
    ) -> Result<(), KnowledgeStoreError> {
        Self::validate_claim(&claim)?;
        if self.get_claim(&claim.id).is_some() {
            return Err(KnowledgeStoreError::DuplicateClaimId(claim.id));
        }
        if matches!(claim.status, KnowledgeClaimStatus::Observed | KnowledgeClaimStatus::Inferred) {
            return Err(KnowledgeStoreError::Validation(
                "Observed/Inferred claims require atomic record_claim_with_evidence".into(),
            ));
        }
        for observation_id in &claim.evidence_refs {
            if observations.get(observation_id).is_none() {
                return Err(KnowledgeStoreError::MissingObservation(observation_id.clone()));
            }
        }
        self.claims.push(claim);
        Ok(())
    }

    fn record_claim_with_evidence(
        &mut self,
        claim: KnowledgeClaim,
        evidence: Vec<EvidenceLink>,
        observations: &dyn ObservationStore,
    ) -> Result<(), KnowledgeStoreError> {
        Self::validate_claim(&claim)?;
        if self.get_claim(&claim.id).is_some() {
            return Err(KnowledgeStoreError::DuplicateClaimId(claim.id));
        }
        if !matches!(claim.status, KnowledgeClaimStatus::Observed | KnowledgeClaimStatus::Inferred) {
            return Err(KnowledgeStoreError::Validation(
                "record_claim_with_evidence is only for Observed/Inferred claims".into(),
            ));
        }
        if evidence.is_empty() {
            return Err(KnowledgeStoreError::Validation(
                "Observed/Inferred claims require at least one evidence link".into(),
            ));
        }
        if claim.evidence_refs.is_empty() {
            return Err(KnowledgeStoreError::Validation(
                "Observed/Inferred claims require evidence_refs".into(),
            ));
        }
        for observation_id in &claim.evidence_refs {
            if observations.get(observation_id).is_none() {
                return Err(KnowledgeStoreError::MissingObservation(observation_id.clone()));
            }
        }
        if evidence.iter().any(|link| link.claim_id != claim.id) {
            return Err(KnowledgeStoreError::Validation(
                "all evidence links must reference the recorded claim".into(),
            ));
        }
        self.validate_new_evidence(&evidence, observations)?;
        if evidence
            .iter()
            .all(|link| !claim.evidence_refs.contains(&link.observation_id))
        {
            return Err(KnowledgeStoreError::Validation(
                "claim evidence_refs must overlap its evidence links".into(),
            ));
        }

        let mut candidate = self.clone();
        candidate.claims.push(claim);
        candidate.evidence.extend(evidence);
        candidate.validate(observations)?;
        *self = candidate;
        Ok(())
    }

    fn record_evidence(
        &mut self,
        link: EvidenceLink,
        observations: &dyn ObservationStore,
    ) -> Result<(), KnowledgeStoreError> {
        if link.observation_id.trim().is_empty() || link.claim_id.trim().is_empty() {
            return Err(KnowledgeStoreError::Validation(
                "evidence observation_id and claim_id cannot be empty".into(),
            ));
        }
        if observations.get(&link.observation_id).is_none() {
            return Err(KnowledgeStoreError::MissingObservation(link.observation_id));
        }
        if self.get_claim(&link.claim_id).is_none() {
            return Err(KnowledgeStoreError::MissingClaim(link.claim_id));
        }
        if self.evidence.iter().any(|existing| existing == &link) {
            return Err(KnowledgeStoreError::DuplicateEvidenceLink {
                observation_id: link.observation_id,
                claim_id: link.claim_id,
                relation: link.relation,
            });
        }
        let mut candidate = self.clone();
        candidate.evidence.push(link);
        candidate.validate(observations)?;
        *self = candidate;
        Ok(())
    }

    fn record_unknown(&mut self, unknown: Unknown) -> Result<(), KnowledgeStoreError> {
        Self::validate_unknown(&unknown)?;
        if self.unknowns.iter().any(|existing| existing.id == unknown.id) {
            return Err(KnowledgeStoreError::DuplicateUnknownId(unknown.id));
        }
        self.unknowns.push(unknown);
        Ok(())
    }

    fn get_claim(&self, id: &str) -> Option<&KnowledgeClaim> {
        self.claims.iter().find(|claim| claim.id == id)
    }

    fn claims(&self) -> Vec<&KnowledgeClaim> {
        self.claims.iter().collect()
    }

    fn evidence(&self) -> Vec<&EvidenceLink> {
        self.evidence.iter().collect()
    }

    fn unknowns(&self) -> Vec<&Unknown> {
        self.unknowns.iter().collect()
    }

    fn len(&self) -> usize {
        self.claims.len()
    }

    fn is_empty(&self) -> bool {
        self.claims.is_empty() && self.evidence.is_empty() && self.unknowns.is_empty()
    }

    fn validate(&self, observations: &dyn ObservationStore) -> Result<(), KnowledgeStoreError> {
        for claim in &self.claims {
            Self::validate_claim(claim)?;
            for observation_id in &claim.evidence_refs {
                if observations.get(observation_id).is_none() {
                    return Err(KnowledgeStoreError::MissingObservation(observation_id.clone()));
                }
            }
            if matches!(claim.status, KnowledgeClaimStatus::Observed | KnowledgeClaimStatus::Inferred)
                && !self.evidence.iter().any(|link| link.claim_id == claim.id)
            {
                return Err(KnowledgeStoreError::Validation(format!(
                    "{} claim must have at least one evidence link",
                    match claim.status {
                        KnowledgeClaimStatus::Observed => "Observed",
                        KnowledgeClaimStatus::Inferred => "Inferred",
                        KnowledgeClaimStatus::Hypothesis => "Hypothesis",
                    }
                )));
            }
        }
        for link in &self.evidence {
            if observations.get(&link.observation_id).is_none() {
                return Err(KnowledgeStoreError::MissingObservation(link.observation_id.clone()));
            }
            if self.get_claim(&link.claim_id).is_none() {
                return Err(KnowledgeStoreError::MissingClaim(link.claim_id.clone()));
            }
        }
        for unknown in &self.unknowns {
            Self::validate_unknown(unknown)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::observation_store::InMemoryObservationStore;
    use crate::core::types::{Observation, ObservationKind};

    fn observation_store() -> InMemoryObservationStore {
        let mut store = InMemoryObservationStore::new();
        store
            .record(Observation::new("obs-1", ObservationKind::Environment, "port 8080 is open"))
            .unwrap();
        store
            .record(Observation::new("obs-2", ObservationKind::Environment, "port 8080 is closed"))
            .unwrap();
        store
    }

    fn observed_claim() -> KnowledgeClaim {
        KnowledgeClaim {
            id: "claim-1".into(),
            subject: "service".into(),
            predicate: "port".into(),
            value: "8080".into(),
            status: KnowledgeClaimStatus::Observed,
            scope: "current host".into(),
            evidence_refs: vec!["obs-1".into()],
        }
    }

    #[test]
    fn observed_claim_requires_traceable_evidence() {
        let observations = observation_store();
        let mut store = InMemoryKnowledgeStore::new();
        store
            .record_claim_with_evidence(
                observed_claim(),
                vec![EvidenceLink {
                    observation_id: "obs-1".into(),
                    claim_id: "claim-1".into(),
                    relation: EvidenceRelation::Supports,
                }],
                &observations,
            )
            .unwrap();
        assert!(store.validate(&observations).is_ok());
    }

    #[test]
    fn evidence_link_requires_existing_observation_and_claim() {
        let observations = observation_store();
        let mut store = InMemoryKnowledgeStore::new();
        let err = store
            .record_evidence(
                EvidenceLink {
                    observation_id: "missing".into(),
                    claim_id: "claim-1".into(),
                    relation: EvidenceRelation::Supports,
                },
                &observations,
            )
            .unwrap_err();
        assert!(matches!(err, KnowledgeStoreError::MissingObservation(_)));
    }

    #[test]
    fn duplicate_evidence_link_is_rejected_deterministically() {
        let observations = observation_store();
        let mut store = InMemoryKnowledgeStore::new();
        store
            .record_claim_with_evidence(
                observed_claim(),
                vec![EvidenceLink {
                    observation_id: "obs-1".into(),
                    claim_id: "claim-1".into(),
                    relation: EvidenceRelation::Supports,
                }],
                &observations,
            )
            .unwrap();
        let err = store
            .record_evidence(
                EvidenceLink {
                    observation_id: "obs-1".into(),
                    claim_id: "claim-1".into(),
                    relation: EvidenceRelation::Supports,
                },
                &observations,
            )
            .unwrap_err();
        assert!(matches!(err, KnowledgeStoreError::DuplicateEvidenceLink { .. }));
    }

    #[test]
    fn conflicting_evidence_can_coexist() {
        let observations = observation_store();
        let mut store = InMemoryKnowledgeStore::new();
        store
            .record_claim_with_evidence(
                observed_claim(),
                vec![EvidenceLink {
                    observation_id: "obs-1".into(),
                    claim_id: "claim-1".into(),
                    relation: EvidenceRelation::Supports,
                }],
                &observations,
            )
            .unwrap();
        store
            .record_evidence(
                EvidenceLink {
                    observation_id: "obs-2".into(),
                    claim_id: "claim-1".into(),
                    relation: EvidenceRelation::Contradicts,
                },
                &observations,
            )
            .unwrap();
        assert_eq!(store.evidence().len(), 2);
    }

    #[test]
    fn unknown_is_independent_from_claim_status() {
        let mut store = InMemoryKnowledgeStore::new();
        store
            .record_unknown(Unknown {
                id: "unknown-1".into(),
                subject: "service".into(),
                scope: "current host".into(),
                question: "Which process owns the port?".into(),
            })
            .unwrap();
        assert!(!store.is_empty());
    }
}
