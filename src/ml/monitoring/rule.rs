use serde::{Deserialize, Serialize};
use tokio::time::Duration;
use uuid::Uuid;

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