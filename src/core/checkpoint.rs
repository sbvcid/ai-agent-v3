use crate::core::types::{AgentState, ValidationError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("Checkpoint not found: {0}")]
    NotFound(String),
}

/// Checkpoint payload capturing the minimal necessary state to resume an Agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateCheckpoint {
    pub checkpoint_id: String,
    pub timestamp_epoch_ms: u64,
    pub step_index: u64,
    pub state: AgentState,
}

impl StateCheckpoint {
    pub fn new(checkpoint_id: impl Into<String>, step_index: u64, state: AgentState) -> Self {
        Self {
            checkpoint_id: checkpoint_id.into(),
            timestamp_epoch_ms: 0,
            step_index,
            state,
        }
    }

    pub fn with_timestamp(mut self, timestamp_epoch_ms: u64) -> Self {
        self.timestamp_epoch_ms = timestamp_epoch_ms;
        self
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.checkpoint_id.trim().is_empty() {
            return Err(ValidationError::EmptyField("checkpoint_id".to_string()));
        }
        // Prevent path traversal: checkpoint_id is embedded in a filename, so
        // it must contain only [A-Za-z0-9_-]. Enforced via path_safety module.
        crate::runtime::path_safety::validate_checkpoint_id(&self.checkpoint_id)?;
        self.state.validate()?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, CheckpointError> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }

    pub fn from_json(json_str: &str) -> Result<Self, CheckpointError> {
        let checkpoint: Self = serde_json::from_str(json_str)?;
        checkpoint.validate()?;
        Ok(checkpoint)
    }
}

/// Abstraction for storing and retrieving Agent checkpoints.
///
/// Decoupled from Windows Runtime, Tools, and LLMs.
pub trait CheckpointStore {
    fn save_checkpoint(&self, checkpoint: &StateCheckpoint) -> Result<(), CheckpointError>;
    fn load_checkpoint(&self, checkpoint_id: &str) -> Result<StateCheckpoint, CheckpointError>;
    fn load_latest_checkpoint(&self) -> Result<Option<StateCheckpoint>, CheckpointError>;
}

/// Minimal JSON file-based implementation of [`CheckpointStore`].
pub struct JsonFileCheckpointStore {
    base_dir: PathBuf,
}

impl JsonFileCheckpointStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Result<Self, CheckpointError> {
        let dir = base_dir.into();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(Self { base_dir: dir })
    }

    fn checkpoint_path(&self, checkpoint_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json", checkpoint_id))
    }

    fn latest_pointer_path(&self) -> PathBuf {
        self.base_dir.join("latest.ptr")
    }
}

impl CheckpointStore for JsonFileCheckpointStore {
    fn save_checkpoint(&self, checkpoint: &StateCheckpoint) -> Result<(), CheckpointError> {
        checkpoint.validate()?;
        let json_data = checkpoint.to_json()?;
        let path = self.checkpoint_path(&checkpoint.checkpoint_id);
        fs::write(&path, json_data)?;

        let latest_ptr = self.latest_pointer_path();
        fs::write(latest_ptr, &checkpoint.checkpoint_id)?;

        Ok(())
    }

    fn load_checkpoint(&self, checkpoint_id: &str) -> Result<StateCheckpoint, CheckpointError> {
        let path = self.checkpoint_path(checkpoint_id);
        if !path.exists() {
            return Err(CheckpointError::NotFound(checkpoint_id.to_string()));
        }
        let content = fs::read_to_string(&path)?;
        StateCheckpoint::from_json(&content)
    }

    fn load_latest_checkpoint(&self) -> Result<Option<StateCheckpoint>, CheckpointError> {
        let latest_ptr = self.latest_pointer_path();
        if !latest_ptr.exists() {
            return Ok(None);
        }
        let checkpoint_id = fs::read_to_string(latest_ptr)?;
        let checkpoint_id = checkpoint_id.trim();
        if checkpoint_id.is_empty() {
            return Ok(None);
        }
        self.load_checkpoint(checkpoint_id).map(Some)
    }
}
