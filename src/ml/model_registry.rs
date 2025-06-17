use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// --- Core Data Structures for the Model Registry ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub versions: Vec<ModelVersion>,
    pub tags: Vec<String>,
    pub owner: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub version: String,
    pub model_format: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub artifacts: Vec<ModelArtifact>,
    pub training_metadata: Option<TrainingMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArtifact {
    pub id: Uuid,
    pub name: String,
    pub artifact_type: ArtifactType,
    pub location: String, // e.g., a path in a distributed file system
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    ModelWeights,
    Tokenizer,
    Configuration,
    Code,
    Dataset,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetadata {
    pub training_framework: String,
    pub framework_version: String,
    pub dataset_id: Uuid,
    pub hyperparameters: HashMap<String, serde_json::Value>,
    pub metrics: HashMap<String, f64>,
}

// NOTE: Removed the placeholder `ModelRegistry` struct and its implementation.
// This file now only defines the data models for the model registry.