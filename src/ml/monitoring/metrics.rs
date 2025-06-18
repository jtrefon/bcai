use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLMetrics {
    pub model_id: Uuid,
    pub model_version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub performance_metrics: PerformanceMetrics,
    pub data_quality_metrics: DataQualityMetrics,
    pub model_quality_metrics: ModelQualityMetrics,
    pub system_metrics: SystemMetrics,
    pub business_metrics: Option<BusinessMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub latency_ms: f64,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub gpu_utilization: f64,
    pub queue_size: u32,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityMetrics {
    pub completeness: f64,
    pub consistency: f64,
    pub validity: f64,
    pub uniqueness: f64,
    pub timeliness: f64,
    pub accuracy: f64,
    pub drift_score: f64,
    pub anomaly_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQualityMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub auc_roc: f64,
    pub prediction_confidence: f64,
    pub model_drift_score: f64,
    pub feature_importance_stability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub request_count: u64,
    pub error_count: u64,
    pub active_connections: u32,
    pub disk_usage_gb: f64,
    pub network_io_mbps: f64,
    pub model_load_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub revenue_impact: f64,
    pub cost_savings: f64,
    pub user_satisfaction: f64,
    pub conversion_rate: f64,
    pub custom_kpis: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub model_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub threshold_value: f64,
    pub actual_value: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: AlertStatus,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    PerformanceDegradation,
    DataDrift,
    ModelDrift,
    HighErrorRate,
    HighLatency,
    LowThroughput,
    ResourceExhaustion,
    DataQualityIssue,
    ModelAccuracyDrop,
    SystemFailure,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringRule {
    pub id: Uuid,
    pub name: String,
    pub model_id: Option<Uuid>, // None means applies to all models
    pub rule_type: RuleType,
    pub condition: MonitoringCondition,
    pub threshold: Threshold,
    pub evaluation_window: Duration,
    pub cooldown_period: Duration,
    pub enabled: bool,
    pub actions: Vec<AlertAction>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Threshold,
    Anomaly,
    Trend,
    Comparison,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringCondition {
    pub metric_name: String,
    pub operator: ComparisonOperator,
    pub value: f64,
    pub aggregation: AggregationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Average,
    Sum,
    Count,
    Min,
    Max,
    Percentile(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    pub warning: f64,
    pub critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    SendEmail(String),
    SendSlack(String),
    CallWebhook(String),
    AutoScale,
    AutoRestart,
    CreateTicket,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub widgets: Vec<Widget>,
    pub filters: Vec<DashboardFilter>,
    pub refresh_interval: Duration,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: Uuid,
    pub widget_type: WidgetType,
    pub title: String,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub config: WidgetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    Gauge,
    Table,
    Heatmap,
    Scatter,
    Histogram,
    Text,
    Alert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub metrics: Vec<String>,
    pub time_range: TimeRange,
    pub aggregation: AggregationType,
    pub filters: HashMap<String, String>,
    pub display_options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardFilter {
    pub name: String,
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
}

// NOTE: All the high-level system implementation structs (MLMonitoringSystem,
// AlertManager, DriftDetector, etc.) and their impl blocks have been removed
// as they were unused placeholder code. This file now only contains the core
// data model definitions for ML monitoring. The implementation can be added
// back incrementally when the features are actually developed. 