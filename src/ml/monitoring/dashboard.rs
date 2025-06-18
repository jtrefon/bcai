use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::Duration;
use uuid::Uuid;

use super::rule::{AggregationType, ComparisonOperator};

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