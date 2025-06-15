use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub model_type: ModelType,
    pub framework: MLFramework,
    pub tags: Vec<String>,
    pub metrics: ModelMetrics,
    pub artifacts: Vec<ModelArtifact>,
    pub dependencies: Vec<Dependency>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub status: ModelStatus,
    pub stage: ModelStage,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Classification,
    Regression,
    Clustering,
    NeuralNetwork,
    DeepLearning,
    ReinforcementLearning,
    NaturalLanguageProcessing,
    ComputerVision,
    TimeSeries,
    Ensemble,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MLFramework {
    TensorFlow,
    PyTorch,
    ScikitLearn,
    XGBoost,
    LightGBM,
    Keras,
    ONNX,
    Huggingface,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub accuracy: Option<f64>,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1_score: Option<f64>,
    pub auc_roc: Option<f64>,
    pub mse: Option<f64>,
    pub mae: Option<f64>,
    pub r2_score: Option<f64>,
    pub custom_metrics: HashMap<String, f64>,
    pub training_time_seconds: Option<f64>,
    pub inference_time_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArtifact {
    pub artifact_type: ArtifactType,
    pub path: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub compression: Option<CompressionType>,
    pub encryption: Option<EncryptionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Model,
    Weights,
    Config,
    Tokenizer,
    Preprocessor,
    Schema,
    Documentation,
    Notebook,
    Dataset,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    Gzip,
    Bzip2,
    Lz4,
    Zstd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionType {
    AES256,
    ChaCha20,
    RSA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Python,
    System,
    Model,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    Draft,
    Training,
    Validating,
    Ready,
    Deployed,
    Deprecated,
    Archived,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStage {
    Development,
    Staging,
    Production,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub version: String,
    pub model_id: Uuid,
    pub parent_version: Option<String>,
    pub changes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub is_latest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelExperiment {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub model_id: Uuid,
    pub parameters: HashMap<String, serde_json::Value>,
    pub metrics: ModelMetrics,
    pub artifacts: Vec<ModelArtifact>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: ExperimentStatus,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<Uuid, ModelMetadata>>>,
    versions: Arc<RwLock<HashMap<String, Vec<ModelVersion>>>>, // model_name -> versions
    experiments: Arc<RwLock<HashMap<Uuid, ModelExperiment>>>,
    name_to_id: Arc<RwLock<HashMap<String, Uuid>>>,
    tags_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    storage_backend: Arc<dyn StorageBackend + Send + Sync>,
}

#[async_trait::async_trait]
pub trait StorageBackend {
    async fn store_artifact(&self, artifact: &ModelArtifact, data: &[u8]) -> Result<String, String>;
    async fn retrieve_artifact(&self, path: &str) -> Result<Vec<u8>, String>;
    async fn delete_artifact(&self, path: &str) -> Result<(), String>;
    async fn list_artifacts(&self, prefix: &str) -> Result<Vec<String>, String>;
    async fn get_artifact_metadata(&self, path: &str) -> Result<ArtifactMetadata, String>;
}

#[derive(Debug, Clone)]
pub struct ArtifactMetadata {
    pub size_bytes: u64,
    pub checksum: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

pub struct LocalStorageBackend {
    base_path: String,
}

#[async_trait::async_trait]
impl StorageBackend for LocalStorageBackend {
    async fn store_artifact(&self, artifact: &ModelArtifact, data: &[u8]) -> Result<String, String> {
        // Simulate storing artifact locally
        let full_path = format!("{}/{}", self.base_path, artifact.path);
        // In real implementation, would write to filesystem
        Ok(full_path)
    }

    async fn retrieve_artifact(&self, path: &str) -> Result<Vec<u8>, String> {
        // Simulate retrieving artifact
        Ok(vec![0u8; 1024]) // Mock data
    }

    async fn delete_artifact(&self, path: &str) -> Result<(), String> {
        // Simulate deletion
        Ok(())
    }

    async fn list_artifacts(&self, prefix: &str) -> Result<Vec<String>, String> {
        // Simulate listing
        Ok(vec![format!("{}/model.bin", prefix)])
    }

    async fn get_artifact_metadata(&self, path: &str) -> Result<ArtifactMetadata, String> {
        Ok(ArtifactMetadata {
            size_bytes: 1024,
            checksum: "abc123".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        })
    }
}

impl LocalStorageBackend {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }
}

impl ModelRegistry {
    pub fn new(storage_backend: Arc<dyn StorageBackend + Send + Sync>) -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
            experiments: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            tags_index: Arc::new(RwLock::new(HashMap::new())),
            storage_backend,
        }
    }

    pub async fn register_model(&self, mut model: ModelMetadata) -> Result<Uuid, String> {
        // Validate model metadata
        self.validate_model_metadata(&model)?;
        
        // Check for name conflicts
        {
            let name_to_id = self.name_to_id.read().await;
            if name_to_id.contains_key(&model.name) {
                return Err(format!("Model with name '{}' already exists", model.name));
            }
        }

        let model_id = model.id;
        model.created_at = chrono::Utc::now();
        model.updated_at = chrono::Utc::now();

        // Store model metadata
        {
            let mut models = self.models.write().await;
            models.insert(model_id, model.clone());
        }

        // Update name index
        {
            let mut name_to_id = self.name_to_id.write().await;
            name_to_id.insert(model.name.clone(), model_id);
        }

        // Update tags index
        {
            let mut tags_index = self.tags_index.write().await;
            for tag in &model.tags {
                tags_index.entry(tag.clone()).or_insert_with(Vec::new).push(model_id);
            }
        }

        // Create initial version
        let initial_version = ModelVersion {
            version: model.version.clone(),
            model_id,
            parent_version: None,
            changes: vec!["Initial version".to_string()],
            created_at: chrono::Utc::now(),
            created_by: model.created_by.clone(),
            is_latest: true,
        };

        {
            let mut versions = self.versions.write().await;
            versions.insert(model.name.clone(), vec![initial_version]);
        }

        Ok(model_id)
    }

    pub async fn create_version(&self, model_name: &str, new_version: &str, changes: Vec<String>, created_by: &str) -> Result<String, String> {
        let model_id = {
            let name_to_id = self.name_to_id.read().await;
            *name_to_id.get(model_name).ok_or("Model not found")?
        };

        let mut versions = self.versions.write().await;
        let model_versions = versions.get_mut(model_name).ok_or("Model versions not found")?;

        // Check if version already exists
        if model_versions.iter().any(|v| v.version == new_version) {
            return Err(format!("Version '{}' already exists", new_version));
        }

        // Mark previous latest as not latest
        for version in model_versions.iter_mut() {
            version.is_latest = false;
        }

        // Get parent version (current latest)
        let parent_version = model_versions.iter()
            .find(|v| v.is_latest)
            .map(|v| v.version.clone());

        // Create new version
        let new_model_version = ModelVersion {
            version: new_version.to_string(),
            model_id,
            parent_version,
            changes,
            created_at: chrono::Utc::now(),
            created_by: created_by.to_string(),
            is_latest: true,
        };

        model_versions.push(new_model_version);

        // Update model metadata
        {
            let mut models = self.models.write().await;
            if let Some(model) = models.get_mut(&model_id) {
                model.version = new_version.to_string();
                model.updated_at = chrono::Utc::now();
            }
        }

        Ok(new_version.to_string())
    }

    pub async fn get_model(&self, model_id: Uuid) -> Option<ModelMetadata> {
        let models = self.models.read().await;
        models.get(&model_id).cloned()
    }

    pub async fn get_model_by_name(&self, name: &str) -> Option<ModelMetadata> {
        let name_to_id = self.name_to_id.read().await;
        if let Some(&model_id) = name_to_id.get(name) {
            drop(name_to_id);
            self.get_model(model_id).await
        } else {
            None
        }
    }

    pub async fn get_model_version(&self, model_name: &str, version: &str) -> Option<ModelVersion> {
        let versions = self.versions.read().await;
        if let Some(model_versions) = versions.get(model_name) {
            model_versions.iter().find(|v| v.version == version).cloned()
        } else {
            None
        }
    }

    pub async fn list_models(&self) -> Vec<ModelMetadata> {
        let models = self.models.read().await;
        models.values().cloned().collect()
    }

    pub async fn list_model_versions(&self, model_name: &str) -> Vec<ModelVersion> {
        let versions = self.versions.read().await;
        versions.get(model_name).cloned().unwrap_or_default()
    }

    pub async fn search_models_by_tag(&self, tag: &str) -> Vec<ModelMetadata> {
        let tags_index = self.tags_index.read().await;
        if let Some(model_ids) = tags_index.get(tag) {
            let models = self.models.read().await;
            model_ids.iter()
                .filter_map(|id| models.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn update_model_status(&self, model_id: Uuid, status: ModelStatus) -> Result<(), String> {
        let mut models = self.models.write().await;
        if let Some(model) = models.get_mut(&model_id) {
            model.status = status;
            model.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Model not found".to_string())
        }
    }

    pub async fn update_model_stage(&self, model_id: Uuid, stage: ModelStage) -> Result<(), String> {
        let mut models = self.models.write().await;
        if let Some(model) = models.get_mut(&model_id) {
            model.stage = stage;
            model.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Model not found".to_string())
        }
    }

    pub async fn add_model_artifact(&self, model_id: Uuid, artifact: ModelArtifact) -> Result<(), String> {
        let mut models = self.models.write().await;
        if let Some(model) = models.get_mut(&model_id) {
            model.artifacts.push(artifact);
            model.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Model not found".to_string())
        }
    }

    pub async fn create_experiment(&self, experiment: ModelExperiment) -> Result<Uuid, String> {
        let experiment_id = experiment.id;
        
        // Validate that model exists
        {
            let models = self.models.read().await;
            if !models.contains_key(&experiment.model_id) {
                return Err("Model not found".to_string());
            }
        }

        let mut experiments = self.experiments.write().await;
        experiments.insert(experiment_id, experiment);
        
        Ok(experiment_id)
    }

    pub async fn get_experiment(&self, experiment_id: Uuid) -> Option<ModelExperiment> {
        let experiments = self.experiments.read().await;
        experiments.get(&experiment_id).cloned()
    }

    pub async fn list_experiments(&self, model_id: Option<Uuid>) -> Vec<ModelExperiment> {
        let experiments = self.experiments.read().await;
        if let Some(model_id) = model_id {
            experiments.values()
                .filter(|exp| exp.model_id == model_id)
                .cloned()
                .collect()
        } else {
            experiments.values().cloned().collect()
        }
    }

    pub async fn update_experiment_status(&self, experiment_id: Uuid, status: ExperimentStatus) -> Result<(), String> {
        let mut experiments = self.experiments.write().await;
        if let Some(experiment) = experiments.get_mut(&experiment_id) {
            experiment.status = status;
            if matches!(experiment.status, ExperimentStatus::Completed | ExperimentStatus::Failed | ExperimentStatus::Cancelled) {
                experiment.completed_at = Some(chrono::Utc::now());
            }
            Ok(())
        } else {
            Err("Experiment not found".to_string())
        }
    }

    pub async fn delete_model(&self, model_id: Uuid) -> Result<(), String> {
        let model = {
            let mut models = self.models.write().await;
            models.remove(&model_id).ok_or("Model not found")?
        };

        // Remove from name index
        {
            let mut name_to_id = self.name_to_id.write().await;
            name_to_id.remove(&model.name);
        }

        // Remove from tags index
        {
            let mut tags_index = self.tags_index.write().await;
            for tag in &model.tags {
                if let Some(model_ids) = tags_index.get_mut(tag) {
                    model_ids.retain(|&id| id != model_id);
                    if model_ids.is_empty() {
                        tags_index.remove(tag);
                    }
                }
            }
        }

        // Remove versions
        {
            let mut versions = self.versions.write().await;
            versions.remove(&model.name);
        }

        // Delete artifacts from storage
        for artifact in &model.artifacts {
            let _ = self.storage_backend.delete_artifact(&artifact.path).await;
        }

        Ok(())
    }

    pub async fn get_model_statistics(&self) -> ModelRegistryStats {
        let models = self.models.read().await;
        let experiments = self.experiments.read().await;
        
        let total_models = models.len();
        let total_experiments = experiments.len();
        
        let mut status_counts = HashMap::new();
        let mut stage_counts = HashMap::new();
        let mut framework_counts = HashMap::new();
        let mut total_size_bytes = 0u64;
        
        for model in models.values() {
            *status_counts.entry(format!("{:?}", model.status)).or_insert(0) += 1;
            *stage_counts.entry(format!("{:?}", model.stage)).or_insert(0) += 1;
            *framework_counts.entry(format!("{:?}", model.framework)).or_insert(0) += 1;
            total_size_bytes += model.size_bytes;
        }
        
        ModelRegistryStats {
            total_models,
            total_experiments,
            status_counts,
            stage_counts,
            framework_counts,
            total_size_bytes,
        }
    }

    fn validate_model_metadata(&self, model: &ModelMetadata) -> Result<(), String> {
        if model.name.is_empty() {
            return Err("Model name cannot be empty".to_string());
        }
        
        if model.version.is_empty() {
            return Err("Model version cannot be empty".to_string());
        }
        
        if model.created_by.is_empty() {
            return Err("Model creator cannot be empty".to_string());
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistryStats {
    pub total_models: usize,
    pub total_experiments: usize,
    pub status_counts: HashMap<String, usize>,
    pub stage_counts: HashMap<String, usize>,
    pub framework_counts: HashMap<String, usize>,
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_registration() {
        let storage = Arc::new(LocalStorageBackend::new("/tmp/models".to_string()));
        let registry = ModelRegistry::new(storage);

        let model = ModelMetadata {
            id: Uuid::new_v4(),
            name: "test_model".to_string(),
            version: "1.0.0".to_string(),
            description: "Test model".to_string(),
            model_type: ModelType::Classification,
            framework: MLFramework::ScikitLearn,
            tags: vec!["test".to_string(), "classification".to_string()],
            metrics: ModelMetrics {
                accuracy: Some(0.95),
                precision: Some(0.94),
                recall: Some(0.96),
                f1_score: Some(0.95),
                auc_roc: Some(0.98),
                mse: None,
                mae: None,
                r2_score: None,
                custom_metrics: HashMap::new(),
                training_time_seconds: Some(120.0),
                inference_time_ms: Some(5.0),
            },
            artifacts: vec![],
            dependencies: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: "test_user".to_string(),
            status: ModelStatus::Ready,
            stage: ModelStage::Development,
            size_bytes: 1024,
            checksum: "abc123".to_string(),
        };

        let model_id = registry.register_model(model.clone()).await.unwrap();
        assert_eq!(model_id, model.id);

        let retrieved_model = registry.get_model(model_id).await.unwrap();
        assert_eq!(retrieved_model.name, model.name);
        assert_eq!(retrieved_model.version, model.version);
    }

    #[tokio::test]
    async fn test_model_versioning() {
        let storage = Arc::new(LocalStorageBackend::new("/tmp/models".to_string()));
        let registry = ModelRegistry::new(storage);

        let model = ModelMetadata {
            id: Uuid::new_v4(),
            name: "versioned_model".to_string(),
            version: "1.0.0".to_string(),
            description: "Versioned model".to_string(),
            model_type: ModelType::Regression,
            framework: MLFramework::TensorFlow,
            tags: vec!["regression".to_string()],
            metrics: ModelMetrics {
                accuracy: None,
                precision: None,
                recall: None,
                f1_score: None,
                auc_roc: None,
                mse: Some(0.05),
                mae: Some(0.03),
                r2_score: Some(0.92),
                custom_metrics: HashMap::new(),
                training_time_seconds: Some(300.0),
                inference_time_ms: Some(10.0),
            },
            artifacts: vec![],
            dependencies: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: "test_user".to_string(),
            status: ModelStatus::Ready,
            stage: ModelStage::Development,
            size_bytes: 2048,
            checksum: "def456".to_string(),
        };

        let model_id = registry.register_model(model.clone()).await.unwrap();
        
        // Create new version
        let new_version = registry.create_version(
            &model.name,
            "1.1.0",
            vec!["Improved accuracy".to_string(), "Added new features".to_string()],
            "test_user"
        ).await.unwrap();
        
        assert_eq!(new_version, "1.1.0");
        
        let versions = registry.list_model_versions(&model.name).await;
        assert_eq!(versions.len(), 2);
        
        let latest_version = versions.iter().find(|v| v.is_latest).unwrap();
        assert_eq!(latest_version.version, "1.1.0");
    }

    #[tokio::test]
    async fn test_model_search_by_tag() {
        let storage = Arc::new(LocalStorageBackend::new("/tmp/models".to_string()));
        let registry = ModelRegistry::new(storage);

        let model1 = ModelMetadata {
            id: Uuid::new_v4(),
            name: "model1".to_string(),
            version: "1.0.0".to_string(),
            description: "Model 1".to_string(),
            model_type: ModelType::Classification,
            framework: MLFramework::PyTorch,
            tags: vec!["nlp".to_string(), "classification".to_string()],
            metrics: ModelMetrics {
                accuracy: Some(0.90),
                precision: None,
                recall: None,
                f1_score: None,
                auc_roc: None,
                mse: None,
                mae: None,
                r2_score: None,
                custom_metrics: HashMap::new(),
                training_time_seconds: Some(600.0),
                inference_time_ms: Some(15.0),
            },
            artifacts: vec![],
            dependencies: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: "user1".to_string(),
            status: ModelStatus::Ready,
            stage: ModelStage::Production,
            size_bytes: 4096,
            checksum: "ghi789".to_string(),
        };

        let model2 = ModelMetadata {
            id: Uuid::new_v4(),
            name: "model2".to_string(),
            version: "1.0.0".to_string(),
            description: "Model 2".to_string(),
            model_type: ModelType::ComputerVision,
            framework: MLFramework::TensorFlow,
            tags: vec!["cv".to_string(), "classification".to_string()],
            metrics: ModelMetrics {
                accuracy: Some(0.88),
                precision: None,
                recall: None,
                f1_score: None,
                auc_roc: None,
                mse: None,
                mae: None,
                r2_score: None,
                custom_metrics: HashMap::new(),
                training_time_seconds: Some(1200.0),
                inference_time_ms: Some(20.0),
            },
            artifacts: vec![],
            dependencies: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: "user2".to_string(),
            status: ModelStatus::Ready,
            stage: ModelStage::Staging,
            size_bytes: 8192,
            checksum: "jkl012".to_string(),
        };

        registry.register_model(model1).await.unwrap();
        registry.register_model(model2).await.unwrap();

        let classification_models = registry.search_models_by_tag("classification").await;
        assert_eq!(classification_models.len(), 2);

        let nlp_models = registry.search_models_by_tag("nlp").await;
        assert_eq!(nlp_models.len(), 1);
        assert_eq!(nlp_models[0].name, "model1");
    }

    #[tokio::test]
    async fn test_experiment_management() {
        let storage = Arc::new(LocalStorageBackend::new("/tmp/models".to_string()));
        let registry = ModelRegistry::new(storage);

        let model = ModelMetadata {
            id: Uuid::new_v4(),
            name: "experiment_model".to_string(),
            version: "1.0.0".to_string(),
            description: "Model for experiments".to_string(),
            model_type: ModelType::NeuralNetwork,
            framework: MLFramework::PyTorch,
            tags: vec!["experiment".to_string()],
            metrics: ModelMetrics {
                accuracy: Some(0.85),
                precision: None,
                recall: None,
                f1_score: None,
                auc_roc: None,
                mse: None,
                mae: None,
                r2_score: None,
                custom_metrics: HashMap::new(),
                training_time_seconds: Some(1800.0),
                inference_time_ms: Some(25.0),
            },
            artifacts: vec![],
            dependencies: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: "researcher".to_string(),
            status: ModelStatus::Training,
            stage: ModelStage::Development,
            size_bytes: 16384,
            checksum: "mno345".to_string(),
        };

        let model_id = registry.register_model(model).await.unwrap();

        let experiment = ModelExperiment {
            id: Uuid::new_v4(),
            name: "hyperparameter_tuning".to_string(),
            description: "Tuning learning rate and batch size".to_string(),
            model_id,
            parameters: {
                let mut params = HashMap::new();
                params.insert("learning_rate".to_string(), serde_json::json!(0.001));
                params.insert("batch_size".to_string(), serde_json::json!(32));
                params
            },
            metrics: ModelMetrics {
                accuracy: Some(0.87),
                precision: None,
                recall: None,
                f1_score: None,
                auc_roc: None,
                mse: None,
                mae: None,
                r2_score: None,
                custom_metrics: HashMap::new(),
                training_time_seconds: Some(2400.0),
                inference_time_ms: Some(22.0),
            },
            artifacts: vec![],
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: ExperimentStatus::Running,
            tags: vec!["hyperparameter_tuning".to_string()],
        };

        let experiment_id = registry.create_experiment(experiment.clone()).await.unwrap();
        assert_eq!(experiment_id, experiment.id);

        let retrieved_experiment = registry.get_experiment(experiment_id).await.unwrap();
        assert_eq!(retrieved_experiment.name, experiment.name);
        assert_eq!(retrieved_experiment.model_id, model_id);

        // Update experiment status
        registry.update_experiment_status(experiment_id, ExperimentStatus::Completed).await.unwrap();
        
        let updated_experiment = registry.get_experiment(experiment_id).await.unwrap();
        assert!(matches!(updated_experiment.status, ExperimentStatus::Completed));
        assert!(updated_experiment.completed_at.is_some());
    }
} 