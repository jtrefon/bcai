use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
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

pub struct MLMonitoringSystem {
    metrics_store: Arc<RwLock<HashMap<Uuid, Vec<MLMetrics>>>>,
    alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    monitoring_rules: Arc<RwLock<HashMap<Uuid, MonitoringRule>>>,
    dashboards: Arc<RwLock<HashMap<Uuid, Dashboard>>>,
    alert_manager: Arc<AlertManager>,
    drift_detector: Arc<DriftDetector>,
    anomaly_detector: Arc<AnomalyDetector>,
    metrics_aggregator: Arc<MetricsAggregator>,
}

pub struct AlertManager {
    active_alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    notification_channels: Vec<NotificationChannel>,
    escalation_rules: Vec<EscalationRule>,
}

#[derive(Debug, Clone)]
pub enum NotificationChannel {
    Email(String),
    Slack(String),
    Webhook(String),
    SMS(String),
}

#[derive(Debug, Clone)]
pub struct EscalationRule {
    pub severity: AlertSeverity,
    pub delay: Duration,
    pub channels: Vec<NotificationChannel>,
}

pub struct DriftDetector {
    reference_data: Arc<RwLock<HashMap<Uuid, ReferenceDataset>>>,
    drift_algorithms: Vec<DriftAlgorithm>,
}

#[derive(Debug, Clone)]
pub struct ReferenceDataset {
    pub model_id: Uuid,
    pub features: Vec<FeatureStatistics>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct FeatureStatistics {
    pub name: String,
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
    pub percentiles: HashMap<String, f64>,
    pub distribution: Distribution,
}

#[derive(Debug, Clone)]
pub enum Distribution {
    Normal { mean: f64, std: f64 },
    Uniform { min: f64, max: f64 },
    Categorical { frequencies: HashMap<String, f64> },
}

#[derive(Debug, Clone)]
pub enum DriftAlgorithm {
    KolmogorovSmirnov,
    ChiSquare,
    PopulationStabilityIndex,
    JensenShannonDivergence,
    Custom(String),
}

pub struct AnomalyDetector {
    models: HashMap<String, AnomalyModel>,
}

#[derive(Debug, Clone)]
pub enum AnomalyModel {
    IsolationForest,
    OneClassSVM,
    LocalOutlierFactor,
    StatisticalThreshold,
    Custom(String),
}

pub struct MetricsAggregator {
    aggregation_rules: Vec<AggregationRule>,
    time_windows: Vec<Duration>,
}

#[derive(Debug, Clone)]
pub struct AggregationRule {
    pub metric_name: String,
    pub aggregation_type: AggregationType,
    pub time_window: Duration,
}

impl MLMonitoringSystem {
    pub fn new() -> Self {
        Self {
            metrics_store: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            monitoring_rules: Arc::new(RwLock::new(HashMap::new())),
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            alert_manager: Arc::new(AlertManager::new()),
            drift_detector: Arc::new(DriftDetector::new()),
            anomaly_detector: Arc::new(AnomalyDetector::new()),
            metrics_aggregator: Arc::new(MetricsAggregator::new()),
        }
    }

    pub async fn record_metrics(&self, metrics: MLMetrics) -> Result<(), String> {
        let model_id = metrics.model_id;
        
        // Store metrics
        {
            let mut store = self.metrics_store.write().await;
            store.entry(model_id).or_insert_with(Vec::new).push(metrics.clone());
        }

        // Evaluate monitoring rules
        self.evaluate_monitoring_rules(&metrics).await?;

        // Detect drift
        self.detect_drift(&metrics).await?;

        // Detect anomalies
        self.detect_anomalies(&metrics).await?;

        Ok(())
    }

    pub async fn create_monitoring_rule(&self, rule: MonitoringRule) -> Result<Uuid, String> {
        let rule_id = rule.id;
        
        // Validate rule
        self.validate_monitoring_rule(&rule)?;
        
        {
            let mut rules = self.monitoring_rules.write().await;
            rules.insert(rule_id, rule);
        }

        Ok(rule_id)
    }

    pub async fn create_dashboard(&self, dashboard: Dashboard) -> Result<Uuid, String> {
        let dashboard_id = dashboard.id;
        
        {
            let mut dashboards = self.dashboards.write().await;
            dashboards.insert(dashboard_id, dashboard);
        }

        Ok(dashboard_id)
    }

    pub async fn get_metrics(&self, model_id: Uuid, time_range: TimeRange) -> Vec<MLMetrics> {
        let store = self.metrics_store.read().await;
        if let Some(metrics) = store.get(&model_id) {
            metrics.iter()
                .filter(|m| m.timestamp >= time_range.start && m.timestamp <= time_range.end)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn get_active_alerts(&self, model_id: Option<Uuid>) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values()
            .filter(|alert| alert.status == AlertStatus::Active)
            .filter(|alert| model_id.map_or(true, |id| alert.model_id == id))
            .cloned()
            .collect()
    }

    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> Result<(), String> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.status = AlertStatus::Acknowledged;
            Ok(())
        } else {
            Err("Alert not found".to_string())
        }
    }

    pub async fn resolve_alert(&self, alert_id: Uuid) -> Result<(), String> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.resolved_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err("Alert not found".to_string())
        }
    }

    async fn evaluate_monitoring_rules(&self, metrics: &MLMetrics) -> Result<(), String> {
        let rules = self.monitoring_rules.read().await;
        
        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }

            // Check if rule applies to this model
            if let Some(rule_model_id) = rule.model_id {
                if rule_model_id != metrics.model_id {
                    continue;
                }
            }

            // Evaluate condition
            if self.evaluate_condition(&rule.condition, metrics).await? {
                self.trigger_alert(rule, metrics).await?;
            }
        }

        Ok(())
    }

    async fn evaluate_condition(&self, condition: &MonitoringCondition, metrics: &MLMetrics) -> Result<bool, String> {
        let metric_value = self.extract_metric_value(&condition.metric_name, metrics)?;
        
        match condition.operator {
            ComparisonOperator::GreaterThan => Ok(metric_value > condition.value),
            ComparisonOperator::LessThan => Ok(metric_value < condition.value),
            ComparisonOperator::Equal => Ok((metric_value - condition.value).abs() < f64::EPSILON),
            ComparisonOperator::NotEqual => Ok((metric_value - condition.value).abs() >= f64::EPSILON),
            ComparisonOperator::GreaterThanOrEqual => Ok(metric_value >= condition.value),
            ComparisonOperator::LessThanOrEqual => Ok(metric_value <= condition.value),
        }
    }

    fn extract_metric_value(&self, metric_name: &str, metrics: &MLMetrics) -> Result<f64, String> {
        match metric_name {
            "latency_ms" => Ok(metrics.performance_metrics.latency_ms),
            "error_rate" => Ok(metrics.performance_metrics.error_rate),
            "throughput_rps" => Ok(metrics.performance_metrics.throughput_rps),
            "cpu_utilization" => Ok(metrics.performance_metrics.cpu_utilization),
            "memory_utilization" => Ok(metrics.performance_metrics.memory_utilization),
            "accuracy" => Ok(metrics.model_quality_metrics.accuracy),
            "drift_score" => Ok(metrics.data_quality_metrics.drift_score),
            _ => Err(format!("Unknown metric: {}", metric_name))
        }
    }

    async fn trigger_alert(&self, rule: &MonitoringRule, metrics: &MLMetrics) -> Result<(), String> {
        let alert = Alert {
            id: Uuid::new_v4(),
            model_id: metrics.model_id,
            alert_type: self.determine_alert_type(&rule.condition.metric_name),
            severity: self.determine_severity(rule, metrics)?,
            title: format!("Alert: {}", rule.name),
            description: format!("Condition '{}' triggered for model {}", 
                               rule.condition.metric_name, metrics.model_id),
            threshold_value: rule.condition.value,
            actual_value: self.extract_metric_value(&rule.condition.metric_name, metrics)?,
            created_at: chrono::Utc::now(),
            resolved_at: None,
            status: AlertStatus::Active,
            metadata: HashMap::new(),
        };

        // Store alert
        {
            let mut alerts = self.alerts.write().await;
            alerts.insert(alert.id, alert.clone());
        }

        // Execute alert actions
        for action in &rule.actions {
            self.execute_alert_action(action, &alert).await?;
        }

        Ok(())
    }

    fn determine_alert_type(&self, metric_name: &str) -> AlertType {
        match metric_name {
            "latency_ms" => AlertType::HighLatency,
            "error_rate" => AlertType::HighErrorRate,
            "throughput_rps" => AlertType::LowThroughput,
            "accuracy" => AlertType::ModelAccuracyDrop,
            "drift_score" => AlertType::DataDrift,
            _ => AlertType::Custom(metric_name.to_string())
        }
    }

    fn determine_severity(&self, rule: &MonitoringRule, metrics: &MLMetrics) -> Result<AlertSeverity, String> {
        let metric_value = self.extract_metric_value(&rule.condition.metric_name, metrics)?;
        
        if metric_value >= rule.threshold.critical {
            Ok(AlertSeverity::Critical)
        } else if metric_value >= rule.threshold.warning {
            Ok(AlertSeverity::High)
        } else {
            Ok(AlertSeverity::Medium)
        }
    }

    async fn execute_alert_action(&self, action: &AlertAction, alert: &Alert) -> Result<(), String> {
        match action {
            AlertAction::SendEmail(email) => {
                // Simulate sending email
                println!("Sending email to {}: {}", email, alert.title);
                Ok(())
            }
            AlertAction::SendSlack(channel) => {
                // Simulate sending Slack message
                println!("Sending Slack message to {}: {}", channel, alert.title);
                Ok(())
            }
            AlertAction::CallWebhook(url) => {
                // Simulate webhook call
                println!("Calling webhook {}: {}", url, alert.title);
                Ok(())
            }
            _ => Ok(())
        }
    }

    async fn detect_drift(&self, metrics: &MLMetrics) -> Result<(), String> {
        // Simulate drift detection
        if metrics.data_quality_metrics.drift_score > 0.5 {
            let alert = Alert {
                id: Uuid::new_v4(),
                model_id: metrics.model_id,
                alert_type: AlertType::DataDrift,
                severity: AlertSeverity::High,
                title: "Data Drift Detected".to_string(),
                description: format!("Data drift detected for model {} with score {}", 
                                   metrics.model_id, metrics.data_quality_metrics.drift_score),
                threshold_value: 0.5,
                actual_value: metrics.data_quality_metrics.drift_score,
                created_at: chrono::Utc::now(),
                resolved_at: None,
                status: AlertStatus::Active,
                metadata: HashMap::new(),
            };

            let mut alerts = self.alerts.write().await;
            alerts.insert(alert.id, alert);
        }

        Ok(())
    }

    async fn detect_anomalies(&self, metrics: &MLMetrics) -> Result<(), String> {
        // Simulate anomaly detection
        if metrics.data_quality_metrics.anomaly_score > 0.8 {
            let alert = Alert {
                id: Uuid::new_v4(),
                model_id: metrics.model_id,
                alert_type: AlertType::DataQualityIssue,
                severity: AlertSeverity::Medium,
                title: "Anomaly Detected".to_string(),
                description: format!("Anomaly detected for model {} with score {}", 
                                   metrics.model_id, metrics.data_quality_metrics.anomaly_score),
                threshold_value: 0.8,
                actual_value: metrics.data_quality_metrics.anomaly_score,
                created_at: chrono::Utc::now(),
                resolved_at: None,
                status: AlertStatus::Active,
                metadata: HashMap::new(),
            };

            let mut alerts = self.alerts.write().await;
            alerts.insert(alert.id, alert);
        }

        Ok(())
    }

    fn validate_monitoring_rule(&self, rule: &MonitoringRule) -> Result<(), String> {
        if rule.name.is_empty() {
            return Err("Rule name cannot be empty".to_string());
        }

        if rule.condition.metric_name.is_empty() {
            return Err("Metric name cannot be empty".to_string());
        }

        if rule.threshold.warning >= rule.threshold.critical {
            return Err("Warning threshold must be less than critical threshold".to_string());
        }

        Ok(())
    }

    pub async fn get_dashboard_data(&self, dashboard_id: Uuid) -> Result<DashboardData, String> {
        let dashboards = self.dashboards.read().await;
        let dashboard = dashboards.get(&dashboard_id).ok_or("Dashboard not found")?;

        let mut widget_data = HashMap::new();
        
        for widget in &dashboard.widgets {
            let data = self.get_widget_data(widget).await?;
            widget_data.insert(widget.id, data);
        }

        Ok(DashboardData {
            dashboard_id,
            widget_data,
            last_updated: chrono::Utc::now(),
        })
    }

    async fn get_widget_data(&self, widget: &Widget) -> Result<WidgetData, String> {
        // Simulate widget data generation
        match widget.widget_type {
            WidgetType::LineChart => {
                Ok(WidgetData::TimeSeries(vec![
                    TimeSeriesPoint { timestamp: chrono::Utc::now(), value: 0.95 },
                    TimeSeriesPoint { timestamp: chrono::Utc::now() - chrono::Duration::minutes(1), value: 0.94 },
                    TimeSeriesPoint { timestamp: chrono::Utc::now() - chrono::Duration::minutes(2), value: 0.96 },
                ]))
            }
            WidgetType::Gauge => {
                Ok(WidgetData::Scalar(0.95))
            }
            WidgetType::Table => {
                Ok(WidgetData::Table(vec![
                    vec!["Model".to_string(), "Accuracy".to_string(), "Latency".to_string()],
                    vec!["model-1".to_string(), "0.95".to_string(), "10ms".to_string()],
                    vec!["model-2".to_string(), "0.92".to_string(), "15ms".to_string()],
                ]))
            }
            _ => Ok(WidgetData::Scalar(0.0))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub dashboard_id: Uuid,
    pub widget_data: HashMap<Uuid, WidgetData>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetData {
    Scalar(f64),
    TimeSeries(Vec<TimeSeriesPoint>),
    Table(Vec<Vec<String>>),
    Distribution(Vec<f64>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            notification_channels: vec![
                NotificationChannel::Email("admin@example.com".to_string()),
                NotificationChannel::Slack("#alerts".to_string()),
            ],
            escalation_rules: vec![
                EscalationRule {
                    severity: AlertSeverity::Critical,
                    delay: Duration::from_secs(0),
                    channels: vec![
                        NotificationChannel::Email("admin@example.com".to_string()),
                        NotificationChannel::Slack("#critical".to_string()),
                    ],
                },
            ],
        }
    }
}

impl DriftDetector {
    pub fn new() -> Self {
        Self {
            reference_data: Arc::new(RwLock::new(HashMap::new())),
            drift_algorithms: vec![
                DriftAlgorithm::KolmogorovSmirnov,
                DriftAlgorithm::PopulationStabilityIndex,
            ],
        }
    }
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            models: HashMap::from([
                ("isolation_forest".to_string(), AnomalyModel::IsolationForest),
                ("statistical".to_string(), AnomalyModel::StatisticalThreshold),
            ]),
        }
    }
}

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {
            aggregation_rules: vec![
                AggregationRule {
                    metric_name: "latency_ms".to_string(),
                    aggregation_type: AggregationType::Average,
                    time_window: Duration::from_secs(5 * 60),
                },
                AggregationRule {
                    metric_name: "error_rate".to_string(),
                    aggregation_type: AggregationType::Average,
                    time_window: Duration::from_secs(60),
                },
            ],
            time_windows: vec![
                Duration::from_secs(60),
                Duration::from_secs(5 * 60),
                Duration::from_secs(15 * 60),
                Duration::from_secs(3600),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_recording() {
        let monitoring_system = MLMonitoringSystem::new();

        let metrics = MLMetrics {
            model_id: Uuid::new_v4(),
            model_version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now(),
            performance_metrics: PerformanceMetrics {
                latency_ms: 15.0,
                throughput_rps: 100.0,
                error_rate: 0.01,
                cpu_utilization: 60.0,
                memory_utilization: 70.0,
                gpu_utilization: 80.0,
                queue_size: 5,
                cache_hit_rate: 0.85,
            },
            data_quality_metrics: DataQualityMetrics {
                completeness: 0.98,
                consistency: 0.95,
                validity: 0.97,
                uniqueness: 0.99,
                timeliness: 0.96,
                accuracy: 0.94,
                drift_score: 0.2,
                anomaly_score: 0.1,
            },
            model_quality_metrics: ModelQualityMetrics {
                accuracy: 0.95,
                precision: 0.94,
                recall: 0.96,
                f1_score: 0.95,
                auc_roc: 0.98,
                prediction_confidence: 0.92,
                model_drift_score: 0.1,
                feature_importance_stability: 0.88,
            },
            system_metrics: SystemMetrics {
                uptime_seconds: 86400,
                request_count: 10000,
                error_count: 100,
                active_connections: 50,
                disk_usage_gb: 25.5,
                network_io_mbps: 100.0,
                model_load_time_ms: 500.0,
            },
            business_metrics: Some(BusinessMetrics {
                revenue_impact: 1000.0,
                cost_savings: 500.0,
                user_satisfaction: 4.5,
                conversion_rate: 0.15,
                custom_kpis: HashMap::new(),
            }),
        };

        monitoring_system.record_metrics(metrics.clone()).await.unwrap();

        let time_range = TimeRange {
            start: chrono::Utc::now() - chrono::Duration::hours(1),
            end: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        let retrieved_metrics = monitoring_system.get_metrics(metrics.model_id, time_range).await;
        assert_eq!(retrieved_metrics.len(), 1);
        assert_eq!(retrieved_metrics[0].model_id, metrics.model_id);
    }

    #[tokio::test]
    async fn test_monitoring_rule_creation() {
        let monitoring_system = MLMonitoringSystem::new();

        let rule = MonitoringRule {
            id: Uuid::new_v4(),
            name: "High Latency Alert".to_string(),
            model_id: Some(Uuid::new_v4()),
            rule_type: RuleType::Threshold,
            condition: MonitoringCondition {
                metric_name: "latency_ms".to_string(),
                operator: ComparisonOperator::GreaterThan,
                value: 100.0,
                aggregation: AggregationType::Average,
            },
            threshold: Threshold {
                warning: 100.0,
                critical: 200.0,
            },
            evaluation_window: Duration::from_secs(5 * 60),
            cooldown_period: Duration::from_secs(10 * 60),
            enabled: true,
            actions: vec![
                AlertAction::SendEmail("admin@example.com".to_string()),
                AlertAction::SendSlack("#alerts".to_string()),
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let rule_id = monitoring_system.create_monitoring_rule(rule.clone()).await.unwrap();
        assert_eq!(rule_id, rule.id);
    }

    #[tokio::test]
    async fn test_alert_triggering() {
        let monitoring_system = MLMonitoringSystem::new();

        // Create a monitoring rule
        let model_id = Uuid::new_v4();
        let rule = MonitoringRule {
            id: Uuid::new_v4(),
            name: "High Error Rate Alert".to_string(),
            model_id: Some(model_id),
            rule_type: RuleType::Threshold,
            condition: MonitoringCondition {
                metric_name: "error_rate".to_string(),
                operator: ComparisonOperator::GreaterThan,
                value: 0.05,
                aggregation: AggregationType::Average,
            },
            threshold: Threshold {
                warning: 0.05,
                critical: 0.10,
            },
            evaluation_window: Duration::from_secs(60),
            cooldown_period: Duration::from_secs(5 * 60),
            enabled: true,
            actions: vec![AlertAction::SendEmail("admin@example.com".to_string())],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        monitoring_system.create_monitoring_rule(rule).await.unwrap();

        // Record metrics that should trigger the alert
        let metrics = MLMetrics {
            model_id,
            model_version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now(),
            performance_metrics: PerformanceMetrics {
                latency_ms: 10.0,
                throughput_rps: 100.0,
                error_rate: 0.08, // This should trigger the alert
                cpu_utilization: 50.0,
                memory_utilization: 60.0,
                gpu_utilization: 70.0,
                queue_size: 2,
                cache_hit_rate: 0.90,
            },
            data_quality_metrics: DataQualityMetrics {
                completeness: 0.99,
                consistency: 0.98,
                validity: 0.99,
                uniqueness: 1.0,
                timeliness: 0.98,
                accuracy: 0.97,
                drift_score: 0.1,
                anomaly_score: 0.05,
            },
            model_quality_metrics: ModelQualityMetrics {
                accuracy: 0.96,
                precision: 0.95,
                recall: 0.97,
                f1_score: 0.96,
                auc_roc: 0.99,
                prediction_confidence: 0.94,
                model_drift_score: 0.05,
                feature_importance_stability: 0.92,
            },
            system_metrics: SystemMetrics {
                uptime_seconds: 3600,
                request_count: 1000,
                error_count: 80,
                active_connections: 25,
                disk_usage_gb: 15.0,
                network_io_mbps: 50.0,
                model_load_time_ms: 200.0,
            },
            business_metrics: None,
        };

        monitoring_system.record_metrics(metrics).await.unwrap();

        // Check that an alert was created
        let active_alerts = monitoring_system.get_active_alerts(Some(model_id)).await;
        assert!(!active_alerts.is_empty());
        assert_eq!(active_alerts[0].model_id, model_id);
        assert!(matches!(active_alerts[0].alert_type, AlertType::HighErrorRate));
    }
} 