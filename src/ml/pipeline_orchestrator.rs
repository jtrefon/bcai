use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTask {
    pub id: Uuid,
    pub name: String,
    pub task_type: TaskType,
    pub dependencies: Vec<Uuid>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub resources: ResourceRequirements,
    pub retry_policy: RetryPolicy,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    DataPreprocessing,
    FeatureEngineering,
    ModelTraining,
    ModelValidation,
    ModelDeployment,
    DataAugmentation,
    HyperparameterTuning,
    ModelEnsemble,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_count: u32,
    pub storage_gb: u32,
    pub network_bandwidth_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_strategy: BackoffStrategy,
    pub retry_on_failure_types: Vec<FailureType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Linear(Duration),
    Exponential { base: Duration, max: Duration },
    Fixed(Duration),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    NetworkError,
    ResourceExhaustion,
    TimeoutError,
    ValidationError,
    SystemError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tasks: Vec<PipelineTask>,
    pub schedule: Option<Schedule>,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    Once,
    Interval(Duration),
    Cron(String),
    EventDriven(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecution {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub status: ExecutionStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub task_executions: HashMap<Uuid, TaskExecution>,
    pub metrics: ExecutionMetrics,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub task_id: Uuid,
    pub status: ExecutionStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub retry_count: u32,
    pub resource_usage: ResourceUsage,
    pub output: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f64,
    pub memory_usage_gb: f64,
    pub gpu_usage_percent: f64,
    pub network_io_mbps: f64,
    pub disk_io_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_duration: Duration,
    pub task_durations: HashMap<Uuid, Duration>,
    pub resource_efficiency: f64,
    pub throughput: f64,
    pub error_rate: f64,
}

pub struct PipelineOrchestrator {
    pipelines: Arc<RwLock<HashMap<Uuid, Pipeline>>>,
    executions: Arc<RwLock<HashMap<Uuid, PipelineExecution>>>,
    task_queue: Arc<RwLock<VecDeque<(Uuid, Uuid)>>>, // (execution_id, task_id)
    resource_manager: Arc<ResourceManager>,
    scheduler: Arc<PipelineScheduler>,
    event_bus: mpsc::UnboundedSender<PipelineEvent>,
    max_concurrent_executions: usize,
    max_concurrent_tasks: usize,
}

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    PipelineCreated(Uuid),
    PipelineUpdated(Uuid),
    PipelineDeleted(Uuid),
    ExecutionStarted(Uuid),
    ExecutionCompleted(Uuid),
    ExecutionFailed(Uuid, String),
    TaskStarted(Uuid, Uuid),
    TaskCompleted(Uuid, Uuid),
    TaskFailed(Uuid, Uuid, String),
    ResourceAllocated(Uuid, ResourceRequirements),
    ResourceReleased(Uuid),
}

pub struct ResourceManager {
    available_resources: Arc<RwLock<ResourceRequirements>>,
    allocated_resources: Arc<RwLock<HashMap<Uuid, ResourceRequirements>>>,
    resource_pools: Arc<RwLock<HashMap<String, ResourcePool>>>,
}

#[derive(Debug, Clone)]
pub struct ResourcePool {
    pub name: String,
    pub total_resources: ResourceRequirements,
    pub available_resources: ResourceRequirements,
    pub allocated_tasks: Vec<Uuid>,
}

pub struct PipelineScheduler {
    scheduled_pipelines: Arc<RwLock<HashMap<Uuid, ScheduledPipeline>>>,
    scheduler_tx: mpsc::UnboundedSender<SchedulerCommand>,
}

#[derive(Debug)]
pub struct ScheduledPipeline {
    pub pipeline_id: Uuid,
    pub schedule: Schedule,
    pub next_execution: chrono::DateTime<chrono::Utc>,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub enum SchedulerCommand {
    AddSchedule(Uuid, Schedule),
    RemoveSchedule(Uuid),
    UpdateSchedule(Uuid, Schedule),
    TriggerExecution(Uuid),
}

impl PipelineOrchestrator {
    pub fn new(
        max_concurrent_executions: usize,
        max_concurrent_tasks: usize,
        total_resources: ResourceRequirements,
    ) -> Self {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let resource_manager = Arc::new(ResourceManager::new(total_resources));
        let scheduler = Arc::new(PipelineScheduler::new());

        Self {
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(VecDeque::new())),
            resource_manager,
            scheduler,
            event_bus: event_tx,
            max_concurrent_executions,
            max_concurrent_tasks,
        }
    }

    pub async fn create_pipeline(&self, pipeline: Pipeline) -> Result<Uuid, String> {
        let pipeline_id = pipeline.id;
        
        // Validate pipeline structure
        self.validate_pipeline(&pipeline).await?;
        
        // Store pipeline
        let mut pipelines = self.pipelines.write().await;
        pipelines.insert(pipeline_id, pipeline);
        
        // Notify event bus
        let _ = self.event_bus.send(PipelineEvent::PipelineCreated(pipeline_id));
        
        Ok(pipeline_id)
    }

    pub async fn execute_pipeline(&self, pipeline_id: Uuid) -> Result<Uuid, String> {
        let pipeline = {
            let pipelines = self.pipelines.read().await;
            pipelines.get(&pipeline_id)
                .ok_or("Pipeline not found")?
                .clone()
        };

        let execution_id = Uuid::new_v4();
        let execution = PipelineExecution {
            id: execution_id,
            pipeline_id,
            status: ExecutionStatus::Pending,
            started_at: chrono::Utc::now(),
            completed_at: None,
            task_executions: HashMap::new(),
            metrics: ExecutionMetrics {
                total_duration: Duration::from_secs(0),
                task_durations: HashMap::new(),
                resource_efficiency: 0.0,
                throughput: 0.0,
                error_rate: 0.0,
            },
            error_message: None,
        };

        // Store execution
        let mut executions = self.executions.write().await;
        executions.insert(execution_id, execution);
        drop(executions);

        // Start execution
        self.start_execution(execution_id, pipeline).await?;
        
        Ok(execution_id)
    }

    async fn start_execution(&self, execution_id: Uuid, pipeline: Pipeline) -> Result<(), String> {
        // Update execution status
        {
            let mut executions = self.executions.write().await;
            if let Some(execution) = executions.get_mut(&execution_id) {
                execution.status = ExecutionStatus::Running;
            }
        }

        // Build dependency graph and execution order
        let execution_order = self.build_execution_order(&pipeline.tasks)?;
        
        // Execute tasks in dependency order
        for task_batch in execution_order {
            let mut task_handles = Vec::new();
            
            for task in task_batch {
                let task_handle = self.execute_task(execution_id, task).await?;
                task_handles.push(task_handle);
            }
            
            // Wait for all tasks in batch to complete
            for handle in task_handles {
                handle.await.map_err(|e| format!("Task execution failed: {}", e))?;
            }
        }

        // Mark execution as completed
        {
            let mut executions = self.executions.write().await;
            if let Some(execution) = executions.get_mut(&execution_id) {
                execution.status = ExecutionStatus::Completed;
                execution.completed_at = Some(chrono::Utc::now());
            }
        }

        let _ = self.event_bus.send(PipelineEvent::ExecutionCompleted(execution_id));
        Ok(())
    }

    async fn execute_task(&self, execution_id: Uuid, task: PipelineTask) -> Result<tokio::task::JoinHandle<()>, String> {
        // Allocate resources
        self.resource_manager.allocate_resources(task.id, &task.resources).await?;
        
        let task_id = task.id;
        let resource_manager = Arc::clone(&self.resource_manager);
        let executions = Arc::clone(&self.executions);
        let event_bus = self.event_bus.clone();

        let handle = tokio::spawn(async move {
            let start_time = Instant::now();
            
            // Update task execution status
            {
                let mut executions = executions.write().await;
                if let Some(execution) = executions.get_mut(&execution_id) {
                    execution.task_executions.insert(task_id, TaskExecution {
                        task_id,
                        status: ExecutionStatus::Running,
                        started_at: Some(chrono::Utc::now()),
                        completed_at: None,
                        retry_count: 0,
                        resource_usage: ResourceUsage {
                            cpu_usage_percent: 0.0,
                            memory_usage_gb: 0.0,
                            gpu_usage_percent: 0.0,
                            network_io_mbps: 0.0,
                            disk_io_mbps: 0.0,
                        },
                        output: None,
                        error_message: None,
                    });
                }
            }

            let _ = event_bus.send(PipelineEvent::TaskStarted(execution_id, task_id));

            // Execute task logic based on task type
            let result = Self::execute_task_logic(&task).await;
            
            let duration = start_time.elapsed();

            // Update task execution with results
            {
                let mut executions = executions.write().await;
                if let Some(execution) = executions.get_mut(&execution_id) {
                    if let Some(task_execution) = execution.task_executions.get_mut(&task_id) {
                        match result {
                            Ok(output) => {
                                task_execution.status = ExecutionStatus::Completed;
                                task_execution.output = Some(output);
                                let _ = event_bus.send(PipelineEvent::TaskCompleted(execution_id, task_id));
                            }
                            Err(error) => {
                                task_execution.status = ExecutionStatus::Failed;
                                task_execution.error_message = Some(error.clone());
                                let _ = event_bus.send(PipelineEvent::TaskFailed(execution_id, task_id, error));
                            }
                        }
                        task_execution.completed_at = Some(chrono::Utc::now());
                    }
                    execution.metrics.task_durations.insert(task_id, duration);
                }
            }

            // Release resources
            let _ = resource_manager.release_resources(task_id).await;
        });

        Ok(handle)
    }

    async fn execute_task_logic(task: &PipelineTask) -> Result<serde_json::Value, String> {
        // Simulate task execution based on task type
        match &task.task_type {
            TaskType::DataPreprocessing => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(serde_json::json!({"status": "data_preprocessed", "records_processed": 1000}))
            }
            TaskType::FeatureEngineering => {
                tokio::time::sleep(Duration::from_millis(150)).await;
                Ok(serde_json::json!({"status": "features_engineered", "features_count": 50}))
            }
            TaskType::ModelTraining => {
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok(serde_json::json!({"status": "model_trained", "accuracy": 0.95, "loss": 0.05}))
            }
            TaskType::ModelValidation => {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(serde_json::json!({"status": "model_validated", "validation_score": 0.92}))
            }
            TaskType::ModelDeployment => {
                tokio::time::sleep(Duration::from_millis(300)).await;
                Ok(serde_json::json!({"status": "model_deployed", "endpoint": "https://api.example.com/model"}))
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(serde_json::json!({"status": "task_completed"}))
            }
        }
    }

    fn build_execution_order(&self, tasks: &[PipelineTask]) -> Result<Vec<Vec<PipelineTask>>, String> {
        let mut task_map: HashMap<Uuid, PipelineTask> = tasks.iter()
            .map(|task| (task.id, task.clone()))
            .collect();
        
        let mut execution_order = Vec::new();
        let mut completed_tasks = std::collections::HashSet::new();
        
        while !task_map.is_empty() {
            let mut ready_tasks = Vec::new();
            
            // Find tasks with all dependencies completed
            for (_task_id, task) in &task_map {
                if task.dependencies.iter().all(|dep| completed_tasks.contains(dep)) {
                    ready_tasks.push(task.clone());
                }
            }
            
            if ready_tasks.is_empty() {
                return Err("Circular dependency detected in pipeline tasks".to_string());
            }
            
            // Remove ready tasks from the map and add to completed
            for task in &ready_tasks {
                task_map.remove(&task.id);
                completed_tasks.insert(task.id);
            }
            
            execution_order.push(ready_tasks);
        }
        
        Ok(execution_order)
    }

    async fn validate_pipeline(&self, pipeline: &Pipeline) -> Result<(), String> {
        if pipeline.tasks.is_empty() {
            return Err("Pipeline must contain at least one task".to_string());
        }
        
        // Check for duplicate task IDs
        let mut task_ids = std::collections::HashSet::new();
        for task in &pipeline.tasks {
            if !task_ids.insert(task.id) {
                return Err(format!("Duplicate task ID found: {}", task.id));
            }
        }
        
        // Validate dependencies
        for task in &pipeline.tasks {
            for dep_id in &task.dependencies {
                if !task_ids.contains(dep_id) {
                    return Err(format!("Task {} depends on non-existent task {}", task.id, dep_id));
                }
            }
        }
        
        Ok(())
    }

    pub async fn get_pipeline(&self, pipeline_id: Uuid) -> Option<Pipeline> {
        let pipelines = self.pipelines.read().await;
        pipelines.get(&pipeline_id).cloned()
    }

    pub async fn get_execution(&self, execution_id: Uuid) -> Option<PipelineExecution> {
        let executions = self.executions.read().await;
        executions.get(&execution_id).cloned()
    }

    pub async fn list_pipelines(&self) -> Vec<Pipeline> {
        let pipelines = self.pipelines.read().await;
        pipelines.values().cloned().collect()
    }

    pub async fn list_executions(&self) -> Vec<PipelineExecution> {
        let executions = self.executions.read().await;
        executions.values().cloned().collect()
    }

    pub async fn cancel_execution(&self, execution_id: Uuid) -> Result<(), String> {
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = ExecutionStatus::Cancelled;
            Ok(())
        } else {
            Err("Execution not found".to_string())
        }
    }
}

impl ResourceManager {
    pub fn new(total_resources: ResourceRequirements) -> Self {
        Self {
            available_resources: Arc::new(RwLock::new(total_resources.clone())),
            allocated_resources: Arc::new(RwLock::new(HashMap::new())),
            resource_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn allocate_resources(&self, task_id: Uuid, requirements: &ResourceRequirements) -> Result<(), String> {
        let mut available = self.available_resources.write().await;
        
        // Check if resources are available
        if available.cpu_cores < requirements.cpu_cores ||
           available.memory_gb < requirements.memory_gb ||
           available.gpu_count < requirements.gpu_count ||
           available.storage_gb < requirements.storage_gb ||
           available.network_bandwidth_mbps < requirements.network_bandwidth_mbps {
            return Err("Insufficient resources available".to_string());
        }
        
        // Allocate resources
        available.cpu_cores -= requirements.cpu_cores;
        available.memory_gb -= requirements.memory_gb;
        available.gpu_count -= requirements.gpu_count;
        available.storage_gb -= requirements.storage_gb;
        available.network_bandwidth_mbps -= requirements.network_bandwidth_mbps;
        
        // Track allocation
        let mut allocated = self.allocated_resources.write().await;
        allocated.insert(task_id, requirements.clone());
        
        Ok(())
    }

    pub async fn release_resources(&self, task_id: Uuid) -> Result<(), String> {
        let mut allocated = self.allocated_resources.write().await;
        
        if let Some(requirements) = allocated.remove(&task_id) {
            let mut available = self.available_resources.write().await;
            
            // Release resources back to available pool
            available.cpu_cores += requirements.cpu_cores;
            available.memory_gb += requirements.memory_gb;
            available.gpu_count += requirements.gpu_count;
            available.storage_gb += requirements.storage_gb;
            available.network_bandwidth_mbps += requirements.network_bandwidth_mbps;
            
            Ok(())
        } else {
            Err("Task resources not found".to_string())
        }
    }

    pub async fn get_resource_utilization(&self) -> f64 {
        let available = self.available_resources.read().await;
        let allocated = self.allocated_resources.read().await;
        
        if allocated.is_empty() {
            return 0.0;
        }
        
        // Calculate utilization as percentage of total resources used
        let total_allocated: u32 = allocated.values().map(|r| r.cpu_cores + r.memory_gb + r.gpu_count).sum();
        let total_capacity = available.cpu_cores + available.memory_gb + available.gpu_count;
        
        (total_allocated as f64) / (total_capacity as f64 + total_allocated as f64)
    }
}

impl PipelineScheduler {
    pub fn new() -> Self {
        let (scheduler_tx, _scheduler_rx) = mpsc::unbounded_channel();
        Self {
            scheduled_pipelines: Arc::new(RwLock::new(HashMap::new())),
            scheduler_tx,
        }
    }

    pub async fn schedule_pipeline(&self, pipeline_id: Uuid, schedule: Schedule) -> Result<(), String> {
        let next_execution = self.calculate_next_execution(&schedule)?;
        
        let scheduled_pipeline = ScheduledPipeline {
            pipeline_id,
            schedule,
            next_execution,
            last_execution: None,
        };

        let mut scheduled = self.scheduled_pipelines.write().await;
        scheduled.insert(pipeline_id, scheduled_pipeline);
        
        Ok(())
    }

    fn calculate_next_execution(&self, schedule: &Schedule) -> Result<chrono::DateTime<chrono::Utc>, String> {
        match schedule {
            Schedule::Once => Ok(chrono::Utc::now()),
            Schedule::Interval(duration) => {
                Ok(chrono::Utc::now() + chrono::Duration::from_std(*duration)
                    .map_err(|e| format!("Invalid duration: {}", e))?)
            }
            Schedule::Cron(_cron_expr) => {
                // Simplified cron implementation - in production, use a proper cron library
                Ok(chrono::Utc::now() + chrono::Duration::hours(1))
            }
            Schedule::EventDriven(_events) => {
                Ok(chrono::Utc::now())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let orchestrator = PipelineOrchestrator::new(
            10,
            50,
            ResourceRequirements {
                cpu_cores: 16,
                memory_gb: 64,
                gpu_count: 4,
                storage_gb: 1000,
                network_bandwidth_mbps: 1000,
            },
        );

        let task = PipelineTask {
            id: Uuid::new_v4(),
            name: "test_task".to_string(),
            task_type: TaskType::DataPreprocessing,
            dependencies: vec![],
            parameters: HashMap::new(),
            resources: ResourceRequirements {
                cpu_cores: 2,
                memory_gb: 4,
                gpu_count: 0,
                storage_gb: 10,
                network_bandwidth_mbps: 100,
            },
            retry_policy: RetryPolicy {
                max_retries: 3,
                backoff_strategy: BackoffStrategy::Exponential {
                    base: Duration::from_secs(1),
                    max: Duration::from_secs(60),
                },
                retry_on_failure_types: vec![FailureType::NetworkError],
            },
            timeout: Duration::from_secs(300),
        };

        let pipeline = Pipeline {
            id: Uuid::new_v4(),
            name: "test_pipeline".to_string(),
            description: "Test pipeline".to_string(),
            tasks: vec![task],
            schedule: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let pipeline_id = orchestrator.create_pipeline(pipeline).await.unwrap();
        assert!(orchestrator.get_pipeline(pipeline_id).await.is_some());
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        let orchestrator = PipelineOrchestrator::new(
            10,
            50,
            ResourceRequirements {
                cpu_cores: 16,
                memory_gb: 64,
                gpu_count: 4,
                storage_gb: 1000,
                network_bandwidth_mbps: 1000,
            },
        );

        let task1 = PipelineTask {
            id: Uuid::new_v4(),
            name: "preprocessing".to_string(),
            task_type: TaskType::DataPreprocessing,
            dependencies: vec![],
            parameters: HashMap::new(),
            resources: ResourceRequirements {
                cpu_cores: 2,
                memory_gb: 4,
                gpu_count: 0,
                storage_gb: 10,
                network_bandwidth_mbps: 100,
            },
            retry_policy: RetryPolicy {
                max_retries: 3,
                backoff_strategy: BackoffStrategy::Fixed(Duration::from_secs(1)),
                retry_on_failure_types: vec![],
            },
            timeout: Duration::from_secs(300),
        };

        let task2 = PipelineTask {
            id: Uuid::new_v4(),
            name: "training".to_string(),
            task_type: TaskType::ModelTraining,
            dependencies: vec![task1.id],
            parameters: HashMap::new(),
            resources: ResourceRequirements {
                cpu_cores: 4,
                memory_gb: 8,
                gpu_count: 1,
                storage_gb: 20,
                network_bandwidth_mbps: 200,
            },
            retry_policy: RetryPolicy {
                max_retries: 2,
                backoff_strategy: BackoffStrategy::Exponential {
                    base: Duration::from_secs(2),
                    max: Duration::from_secs(120),
                },
                retry_on_failure_types: vec![FailureType::SystemError],
            },
            timeout: Duration::from_secs(1800),
        };

        let pipeline = Pipeline {
            id: Uuid::new_v4(),
            name: "ml_training_pipeline".to_string(),
            description: "ML training pipeline with dependencies".to_string(),
            tasks: vec![task1, task2],
            schedule: Some(Schedule::Interval(Duration::from_secs(24 * 3600))), // 24 hours
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let pipeline_id = orchestrator.create_pipeline(pipeline).await.unwrap();
        let execution_id = orchestrator.execute_pipeline(pipeline_id).await.unwrap();
        
        // Wait a bit for execution to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let execution = orchestrator.get_execution(execution_id).await.unwrap();
        assert!(matches!(execution.status, ExecutionStatus::Running | ExecutionStatus::Completed));
    }

    #[tokio::test]
    async fn test_resource_management() {
        let resource_manager = ResourceManager::new(ResourceRequirements {
            cpu_cores: 8,
            memory_gb: 16,
            gpu_count: 2,
            storage_gb: 100,
            network_bandwidth_mbps: 1000,
        });

        let task_id = Uuid::new_v4();
        let requirements = ResourceRequirements {
            cpu_cores: 4,
            memory_gb: 8,
            gpu_count: 1,
            storage_gb: 50,
            network_bandwidth_mbps: 500,
        };

        // Test allocation
        assert!(resource_manager.allocate_resources(task_id, &requirements).await.is_ok());
        
        // Test utilization
        let utilization = resource_manager.get_resource_utilization().await;
        assert!(utilization > 0.0);
        
        // Test release
        assert!(resource_manager.release_resources(task_id).await.is_ok());
        
        // Test utilization after release
        let utilization_after = resource_manager.get_resource_utilization().await;
        assert!(utilization_after < utilization);
    }
} 