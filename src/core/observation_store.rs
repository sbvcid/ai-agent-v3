//! Observation Store abstraction for Agent Core.
//!
//! Authoritative in-process observation history.

use crate::core::types::{Observation, ValidationError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ObservationStoreError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("Duplicate observation ID: {0}")]
    DuplicateId(String),
}

/// Abstraction for retaining and retrieving observations.
pub trait ObservationStore: std::fmt::Debug {
    fn record(&mut self, observation: Observation) -> Result<(), ObservationStoreError>;
    fn get(&self, id: &str) -> Option<&Observation>;
    fn recent(&self, limit: usize) -> Vec<&Observation>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

/// In-memory implementation of [`ObservationStore`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct InMemoryObservationStore {
    observations: Vec<Observation>,
}

impl InMemoryObservationStore {
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
        }
    }
}

impl ObservationStore for InMemoryObservationStore {
    fn record(&mut self, observation: Observation) -> Result<(), ObservationStoreError> {
        observation.validate()?;
        if self.observations.iter().any(|o| o.id == observation.id) {
            return Err(ObservationStoreError::DuplicateId(observation.id));
        }
        self.observations.push(observation);
        Ok(())
    }

    fn get(&self, id: &str) -> Option<&Observation> {
        self.observations.iter().find(|o| o.id == id)
    }

    fn recent(&self, limit: usize) -> Vec<&Observation> {
        if limit == 0 || self.observations.is_empty() {
            return Vec::new();
        }
        let start = self.observations.len().saturating_sub(limit);
        self.observations[start..].iter().collect()
    }

    fn len(&self) -> usize {
        self.observations.len()
    }

    fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::ObservationKind;

    #[test]
    fn test_record_and_retrieve_single() {
        let mut store = InMemoryObservationStore::new();
        let obs = Observation::new("obs-1", ObservationKind::Environment, "test summary");
        store.record(obs.clone()).expect("should succeed");

        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
        assert_eq!(store.get("obs-1"), Some(&obs));
    }

    #[test]
    fn test_record_multiple_deterministic_order() {
        let mut store = InMemoryObservationStore::new();
        let o1 = Observation::new("obs-1", ObservationKind::Environment, "summary 1");
        let o2 = Observation::new("obs-2", ObservationKind::Filesystem, "summary 2");
        let o3 = Observation::new("obs-3", ObservationKind::Process, "summary 3");

        store.record(o1.clone()).unwrap();
        store.record(o2.clone()).unwrap();
        store.record(o3.clone()).unwrap();

        let recent = store.recent(10);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0], &o1);
        assert_eq!(recent[1], &o2);
        assert_eq!(recent[2], &o3);
    }

    #[test]
    fn test_bounded_recent_retrieval() {
        let mut store = InMemoryObservationStore::new();
        let o1 = Observation::new("obs-1", ObservationKind::Environment, "summary 1");
        let o2 = Observation::new("obs-2", ObservationKind::Filesystem, "summary 2");
        let o3 = Observation::new("obs-3", ObservationKind::Process, "summary 3");

        store.record(o1).unwrap();
        store.record(o2.clone()).unwrap();
        store.record(o3.clone()).unwrap();

        let recent = store.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], &o2);
        assert_eq!(recent[1], &o3);
    }

    #[test]
    fn test_preserve_source_action_id() {
        let mut store = InMemoryObservationStore::new();
        let mut obs = Observation::new("obs-1", ObservationKind::ActionResult, "action output");
        obs.source_action_id = Some("act-99".to_string());

        store.record(obs.clone()).unwrap();
        let retrieved = store.get("obs-1").unwrap();
        assert_eq!(retrieved.source_action_id.as_deref(), Some("act-99"));
    }

    #[test]
    fn test_duplicate_id_behavior() {
        let mut store = InMemoryObservationStore::new();
        let o1 = Observation::new("obs-1", ObservationKind::Environment, "summary 1");
        let o2 = Observation::new("obs-1", ObservationKind::Environment, "summary 2");

        store.record(o1).unwrap();
        let err = store.record(o2).expect_err("should reject duplicate ID");
        assert!(matches!(err, ObservationStoreError::DuplicateId(ref id) if id == "obs-1"));
    }

    #[test]
    fn test_invalid_observation_handling() {
        let mut store = InMemoryObservationStore::new();
        // Empty ID is invalid per Observation::validate()
        let invalid_obs = Observation::new("   ", ObservationKind::Environment, "summary");
        let err = store
            .record(invalid_obs)
            .expect_err("should fail validation");
        assert!(matches!(err, ObservationStoreError::Validation(_)));
    }

    #[test]
    fn test_deterministic_empty_store() {
        let store = InMemoryObservationStore::new();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
        assert_eq!(store.get("nonexistent"), None);
        assert!(store.recent(5).is_empty());
    }
}
