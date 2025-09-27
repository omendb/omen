//! Core traits for learned indexes in OmenDB

use super::{Key, Position, Value};
use thiserror::Error;

/// Errors that can occur with learned indexes
#[derive(Debug, Error)]
pub enum LearnedIndexError {
    #[error("Key not found: {0}")]
    KeyNotFound(Key),

    #[error("Index not trained")]
    NotTrained,

    #[error("Training failed: {0}")]
    TrainingFailed(String),

    #[error("Prediction error too high: {0}")]
    PredictionError(usize),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, LearnedIndexError>;

/// Core trait that all learned indexes must implement
pub trait LearnedIndex: Send + Sync {
    /// Train the model on sorted data
    fn train(&mut self, data: &[(Key, Position)]) -> Result<()>;

    /// Predict the position of a key (O(1) operation)
    fn predict(&self, key: Key) -> Position;

    /// Search for exact key using prediction + refinement
    fn search(&self, key: Key) -> Result<Position>;

    /// Insert a new key-value pair
    fn insert(&mut self, key: Key, position: Position) -> Result<()>;

    /// Range query between start and end keys
    fn range(&self, start: Key, end: Key) -> Result<Vec<Position>>;

    /// Get current error bound
    fn error_bound(&self) -> usize;

    /// Check if retraining is needed
    fn needs_retrain(&self) -> bool;

    /// Get statistics about performance
    fn stats(&self) -> String;
}