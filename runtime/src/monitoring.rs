//! Monitoring and Observability for BCAI
//!
//! This module provides comprehensive monitoring capabilities:
//! - Real-time metrics collection and aggregation
//! - Health checks and system status monitoring
//! - Performance tracking and optimization insights
//! - Alerting and notification system

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Monitoring errors
#[derive(Debug, Error)]
pub enum MonitoringError {
    #[error("Metric collection failed: {0}")]
    MetricCollectionFailed(String),
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    #[error("Alert configuration error: {0}")]
    AlertConfigError(String),
    #[error("Performance threshold exceeded: {metric} = {value}")]
    PerformanceThresholdExceeded { metric: String, value: f64 },
}

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

/// System metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub disk_usage_bytes: u64,
    pub network_in_bytes: u64,
    pub network_out_bytes: u64,
    pub active_connections: u32,
    pub thread_count: u32,
}

/// BCAI-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcaiMetrics {
    pub timestamp: u64,
    pub active_nodes: u32,
    pub total_jobs: u32,
    pub completed_jobs: u32,
    pub pending_jobs: u32,
    pub average_job_completion_time_ms: f64,
    pub total_transactions: u32,
    pub transaction_throughput_per_sec: f64,
    pub consensus_time_ms: f64,
    pub block_height: u64,
    pub validator_count: u32,
    pub total_stake: u64,
    pub network_hashrate: f64,
}

/// Performance metrics for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub p2p_latency_ms: f64,
    pub consensus_latency_ms: f64,
    pub training_accuracy: f64,
    pub model_convergence_rate: f64,
    pub federated_rounds: u32,
    pub pouw_solve_time_ms: f64,
    pub blockchain_sync_time_ms: f64,
    pub memory_efficiency_ratio: f64,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: u64,
    pub response_time_ms: f64,
    pub details: HashMap<String, String>,
}

/// Alert definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub metric: String,
    pub threshold: f64,
    pub comparison: AlertComparison,
    pub severity: AlertSeverity,
    pub message: String,
    pub enabled: bool,
    pub cooldown_seconds: u64,
}

/// Alert comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertComparison {
    Greater,
    Less,
    Equal,
    NotEqual,
}

/// Triggered alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredAlert {
    pub alert_id: String,
    pub metric: String,
    pub actual_value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub resolved: bool,
    pub resolution_time: Option<u64>,
}

/// Monitoring configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub alert_threshold: u64,
    pub health_check_interval: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            alert_threshold: 100,
            health_check_interval: 30,
        }
    }
}

/// Main monitoring system
pub struct MonitoringSystem {
    config: MonitoringConfig,
    system_metrics: VecDeque<SystemMetrics>,
    bcai_metrics: VecDeque<BcaiMetrics>,
    performance_metrics: VecDeque<PerformanceMetrics>,
    health_checks: HashMap<String, HealthCheck>,
    alerts: Vec<Alert>,
    triggered_alerts: Vec<TriggeredAlert>,
    last_alert_times: HashMap<String, u64>,
    status: HealthStatus,
}

impl MonitoringSystem {
    /// Create new monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        let mut system = Self {
            config,
            system_metrics: VecDeque::new(),
            bcai_metrics: VecDeque::new(),
            performance_metrics: VecDeque::new(),
            health_checks: HashMap::new(),
            alerts: Vec::new(),
            triggered_alerts: Vec::new(),
            last_alert_times: HashMap::new(),
            status: HealthStatus::Healthy,
        };

        // Setup default alerts
        system.setup_default_alerts();
        system
    }

    /// Collect system metrics
    pub fn collect_system_metrics(&mut self) -> Result<SystemMetrics, MonitoringError> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // In production, these would be collected from actual system monitoring
        let metrics = SystemMetrics {
            timestamp,
            cpu_usage_percent: self.get_cpu_usage(),
            memory_usage_bytes: self.get_memory_usage(),
            disk_usage_bytes: self.get_disk_usage(),
            network_in_bytes: self.get_network_in(),
            network_out_bytes: self.get_network_out(),
            active_connections: self.get_active_connections(),
            thread_count: self.get_thread_count(),
        };

        // Store metrics
        self.system_metrics.push_back(metrics.clone());
        self.cleanup_old_metrics();

        Ok(metrics)
    }

    /// Collect BCAI-specific metrics
    #[allow(clippy::too_many_arguments)]
    pub fn collect_bcai_metrics(
        &mut self,
        active_nodes: u32,
        total_jobs: u32,
        completed_jobs: u32,
        pending_jobs: u32,
        total_transactions: u32,
        block_height: u64,
        validator_count: u32,
        total_stake: u64,
    ) -> Result<BcaiMetrics, MonitoringError> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let metrics = BcaiMetrics {
            timestamp,
            active_nodes,
            total_jobs,
            completed_jobs,
            pending_jobs,
            average_job_completion_time_ms: self.calculate_avg_job_completion_time(),
            total_transactions,
            transaction_throughput_per_sec: self.calculate_transaction_throughput(),
            consensus_time_ms: self.calculate_consensus_time(),
            block_height,
            validator_count,
            total_stake,
            network_hashrate: self.calculate_network_hashrate(),
        };

        self.bcai_metrics.push_back(metrics.clone());
        self.cleanup_old_metrics();

        Ok(metrics)
    }

    /// Collect performance metrics
    pub fn collect_performance_metrics(
        &mut self,
        p2p_latency_ms: f64,
        consensus_latency_ms: f64,
        training_accuracy: f64,
        federated_rounds: u32,
        pouw_solve_time_ms: f64,
    ) -> Result<PerformanceMetrics, MonitoringError> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let metrics = PerformanceMetrics {
            timestamp,
            p2p_latency_ms,
            consensus_latency_ms,
            training_accuracy,
            model_convergence_rate: self.calculate_convergence_rate(),
            federated_rounds,
            pouw_solve_time_ms,
            blockchain_sync_time_ms: self.calculate_sync_time(),
            memory_efficiency_ratio: self.calculate_memory_efficiency(),
        };

        self.performance_metrics.push_back(metrics.clone());
        self.cleanup_old_metrics();

        // Check for performance alerts
        self.check_performance_alerts(&metrics)?;

        Ok(metrics)
    }

    /// Perform health checks
    pub fn perform_health_checks(&mut self) -> Result<Vec<HealthCheck>, MonitoringError> {
        let health_checks = vec![
            self.check_p2p_health()?,
            self.check_blockchain_health()?,
            self.check_consensus_health()?,
            self.check_storage_health()?,
            self.check_network_health()?,
        ];

        // Update stored health checks
        for check in &health_checks {
            self.health_checks.insert(check.component.clone(), check.clone());
        }

        Ok(health_checks)
    }

    /// Get overall system health
    pub fn get_system_health(&self) -> HealthStatus {
        if self.health_checks.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut critical_count = 0;
        let mut warning_count = 0;
        let total_checks = self.health_checks.len();

        for check in self.health_checks.values() {
            match check.status {
                HealthStatus::Critical => critical_count += 1,
                HealthStatus::Warning => warning_count += 1,
                _ => {}
            }
        }

        if critical_count > 0 {
            HealthStatus::Critical
        } else if warning_count > total_checks / 2 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    /// Add custom alert
    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }

    /// Check all alerts
    pub fn check_alerts(&mut self) -> Result<Vec<TriggeredAlert>, MonitoringError> {
        let mut new_alerts = Vec::new();
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        for alert in self.alerts.iter() {
            if !alert.enabled {
                continue;
            }

            // Check cooldown
            if let Some(&last_time) = self.last_alert_times.get(&alert.id) {
                if current_time - last_time < alert.cooldown_seconds {
                    continue;
                }
            }

            // Get current metric value
            if let Some(current_value) = self.get_current_metric_value(&alert.metric) {
                let triggered = match alert.comparison {
                    AlertComparison::Greater => current_value > alert.threshold,
                    AlertComparison::Less => current_value < alert.threshold,
                    AlertComparison::Equal => (current_value - alert.threshold).abs() < 0.001,
                    AlertComparison::NotEqual => (current_value - alert.threshold).abs() >= 0.001,
                };

                if triggered {
                    let triggered_alert = TriggeredAlert {
                        alert_id: alert.id.clone(),
                        metric: alert.metric.clone(),
                        actual_value: current_value,
                        threshold: alert.threshold,
                        severity: alert.severity,
                        message: alert.message.clone(),
                        timestamp: current_time,
                        resolved: false,
                        resolution_time: None,
                    };

                    new_alerts.push(triggered_alert.clone());
                    self.triggered_alerts.push(triggered_alert);
                    self.last_alert_times.insert(alert.id.clone(), current_time);
                }
            }
        }

        Ok(new_alerts)
    }

    /// Get monitoring dashboard data
    pub fn get_dashboard_data(&self) -> MonitoringDashboard {
        let system_health = self.get_system_health();
        let latest_system_metrics = self.system_metrics.back().cloned();
        let latest_bcai_metrics = self.bcai_metrics.back().cloned();
        let latest_performance_metrics = self.performance_metrics.back().cloned();

        let active_alerts: Vec<TriggeredAlert> =
            self.triggered_alerts.iter().filter(|a| !a.resolved).cloned().collect();

        let critical_alerts_count =
            active_alerts.iter().filter(|a| a.severity == AlertSeverity::Critical).count();

        MonitoringDashboard {
            system_health,
            latest_system_metrics,
            latest_bcai_metrics,
            latest_performance_metrics,
            health_checks: self.health_checks.clone(),
            active_alerts,
            critical_alerts_count,
            total_alerts_today: self.triggered_alerts.len(),
            uptime_percentage: self.calculate_uptime_percentage(),
        }
    }

    /// Get historical metrics for charting
    pub fn get_historical_metrics(&self, metric_type: &str, hours: u32) -> Vec<(u64, f64)> {
        let cutoff_time =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - (hours as u64 * 3600);

        match metric_type {
            "cpu_usage" => self
                .system_metrics
                .iter()
                .filter(|m| m.timestamp >= cutoff_time)
                .map(|m| (m.timestamp, m.cpu_usage_percent))
                .collect(),
            "memory_usage" => self
                .system_metrics
                .iter()
                .filter(|m| m.timestamp >= cutoff_time)
                .map(|m| (m.timestamp, m.memory_usage_bytes as f64))
                .collect(),
            "active_nodes" => self
                .bcai_metrics
                .iter()
                .filter(|m| m.timestamp >= cutoff_time)
                .map(|m| (m.timestamp, m.active_nodes as f64))
                .collect(),
            "training_accuracy" => self
                .performance_metrics
                .iter()
                .filter(|m| m.timestamp >= cutoff_time)
                .map(|m| (m.timestamp, m.training_accuracy))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Setup default monitoring alerts
    fn setup_default_alerts(&mut self) {
        let alerts = vec![
            Alert {
                id: "high_cpu_usage".to_string(),
                metric: "cpu_usage_percent".to_string(),
                threshold: 80.0,
                comparison: AlertComparison::Greater,
                severity: AlertSeverity::Warning,
                message: "High CPU usage detected".to_string(),
                enabled: true,
                cooldown_seconds: 300,
            },
            Alert {
                id: "critical_cpu_usage".to_string(),
                metric: "cpu_usage_percent".to_string(),
                threshold: 95.0,
                comparison: AlertComparison::Greater,
                severity: AlertSeverity::Critical,
                message: "Critical CPU usage - immediate attention required".to_string(),
                enabled: true,
                cooldown_seconds: 60,
            },
            Alert {
                id: "low_training_accuracy".to_string(),
                metric: "training_accuracy".to_string(),
                threshold: 0.5,
                comparison: AlertComparison::Less,
                severity: AlertSeverity::Warning,
                message: "Training accuracy below acceptable threshold".to_string(),
                enabled: true,
                cooldown_seconds: 600,
            },
            Alert {
                id: "high_consensus_latency".to_string(),
                metric: "consensus_latency_ms".to_string(),
                threshold: 5000.0,
                comparison: AlertComparison::Greater,
                severity: AlertSeverity::Error,
                message: "Consensus taking too long - network performance degraded".to_string(),
                enabled: true,
                cooldown_seconds: 300,
            },
        ];

        for alert in alerts {
            self.alerts.push(alert);
        }
    }

    /// Individual health check implementations
    fn check_p2p_health(&self) -> Result<HealthCheck, MonitoringError> {
        let start_time = SystemTime::now();

        // Simulate P2P health check
        let is_healthy = true; // In production: check peer connections, latency, etc.
        let latency = 50.0; // ms

        let response_time = start_time.elapsed().unwrap().as_millis() as f64;

        Ok(HealthCheck {
            component: "P2P Network".to_string(),
            status: if is_healthy { HealthStatus::Healthy } else { HealthStatus::Critical },
            message: if is_healthy {
                "P2P network operating normally".to_string()
            } else {
                "P2P network connectivity issues".to_string()
            },
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: response_time,
            details: {
                let mut details = HashMap::new();
                details.insert("latency_ms".to_string(), latency.to_string());
                details.insert("peer_count".to_string(), "5".to_string());
                details
            },
        })
    }

    fn check_blockchain_health(&self) -> Result<HealthCheck, MonitoringError> {
        let start_time = SystemTime::now();

        // Simulate blockchain health check
        let is_synced = true;
        let block_height = 1000;

        let response_time = start_time.elapsed().unwrap().as_millis() as f64;

        Ok(HealthCheck {
            component: "Blockchain".to_string(),
            status: if is_synced { HealthStatus::Healthy } else { HealthStatus::Warning },
            message: if is_synced {
                "Blockchain fully synchronized".to_string()
            } else {
                "Blockchain synchronization lagging".to_string()
            },
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: response_time,
            details: {
                let mut details = HashMap::new();
                details.insert("block_height".to_string(), block_height.to_string());
                details.insert(
                    "sync_status".to_string(),
                    if is_synced { "synced" } else { "syncing" }.to_string(),
                );
                details
            },
        })
    }

    fn check_consensus_health(&self) -> Result<HealthCheck, MonitoringError> {
        let start_time = SystemTime::now();

        // Simulate consensus health check
        let consensus_working = true;
        let validator_count = 10;

        let response_time = start_time.elapsed().unwrap().as_millis() as f64;

        Ok(HealthCheck {
            component: "Consensus".to_string(),
            status: if consensus_working { HealthStatus::Healthy } else { HealthStatus::Critical },
            message: if consensus_working {
                "Consensus mechanism functioning properly".to_string()
            } else {
                "Consensus mechanism failure detected".to_string()
            },
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: response_time,
            details: {
                let mut details = HashMap::new();
                details.insert("validator_count".to_string(), validator_count.to_string());
                details.insert("consensus_rounds".to_string(), "50".to_string());
                details
            },
        })
    }

    fn check_storage_health(&self) -> Result<HealthCheck, MonitoringError> {
        let start_time = SystemTime::now();

        // Simulate storage health check
        let _storage_healthy = true;
        let disk_usage_percent = 65.0;

        let response_time = start_time.elapsed().unwrap().as_millis() as f64;
        let status = if disk_usage_percent > 90.0 {
            HealthStatus::Critical
        } else if disk_usage_percent > 80.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        Ok(HealthCheck {
            component: "Storage".to_string(),
            status,
            message: format!("Disk usage at {:.1}%", disk_usage_percent),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: response_time,
            details: {
                let mut details = HashMap::new();
                details.insert("disk_usage_percent".to_string(), disk_usage_percent.to_string());
                details.insert("available_space_gb".to_string(), "500".to_string());
                details
            },
        })
    }

    fn check_network_health(&self) -> Result<HealthCheck, MonitoringError> {
        let start_time = SystemTime::now();

        // Simulate network health check
        let network_healthy = true;
        let bandwidth_usage_percent = 25.0;

        let response_time = start_time.elapsed().unwrap().as_millis() as f64;

        Ok(HealthCheck {
            component: "Network".to_string(),
            status: if network_healthy { HealthStatus::Healthy } else { HealthStatus::Warning },
            message: format!("Network bandwidth usage at {:.1}%", bandwidth_usage_percent),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: response_time,
            details: {
                let mut details = HashMap::new();
                details.insert(
                    "bandwidth_usage_percent".to_string(),
                    bandwidth_usage_percent.to_string(),
                );
                details.insert("active_connections".to_string(), "25".to_string());
                details
            },
        })
    }

    /// Helper methods for metric calculation (simplified for demo)
    fn get_cpu_usage(&self) -> f64 {
        45.0
    } // Simulated
    fn get_memory_usage(&self) -> u64 {
        2_147_483_648
    } // 2GB
    fn get_disk_usage(&self) -> u64 {
        10_737_418_240
    } // 10GB
    fn get_network_in(&self) -> u64 {
        1024000
    }
    fn get_network_out(&self) -> u64 {
        512000
    }
    fn get_active_connections(&self) -> u32 {
        15
    }
    fn get_thread_count(&self) -> u32 {
        8
    }

    fn calculate_avg_job_completion_time(&self) -> f64 {
        2500.0
    }
    fn calculate_transaction_throughput(&self) -> f64 {
        10.5
    }
    fn calculate_consensus_time(&self) -> f64 {
        3000.0
    }
    fn calculate_network_hashrate(&self) -> f64 {
        1000000.0
    }
    fn calculate_convergence_rate(&self) -> f64 {
        0.95
    }
    fn calculate_sync_time(&self) -> f64 {
        1500.0
    }
    fn calculate_memory_efficiency(&self) -> f64 {
        0.85
    }
    fn calculate_uptime_percentage(&self) -> f64 {
        99.8
    }

    fn get_current_metric_value(&self, metric: &str) -> Option<f64> {
        match metric {
            "cpu_usage_percent" => self.system_metrics.back().map(|m| m.cpu_usage_percent),
            "memory_usage_bytes" => self.system_metrics.back().map(|m| m.memory_usage_bytes as f64),
            "training_accuracy" => self.performance_metrics.back().map(|m| m.training_accuracy),
            "consensus_latency_ms" => {
                self.performance_metrics.back().map(|m| m.consensus_latency_ms)
            }
            _ => None,
        }
    }

    fn check_performance_alerts(
        &mut self,
        metrics: &PerformanceMetrics,
    ) -> Result<(), MonitoringError> {
        // Check if training accuracy is too low
        if metrics.training_accuracy < 0.3 {
            return Err(MonitoringError::PerformanceThresholdExceeded {
                metric: "training_accuracy".to_string(),
                value: metrics.training_accuracy,
            });
        }

        Ok(())
    }

    fn cleanup_old_metrics(&mut self) {
        let cutoff_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
            - (self.config.health_check_interval as u64 * 3600);

        // Clean system metrics
        while let Some(front) = self.system_metrics.front() {
            if front.timestamp < cutoff_time {
                self.system_metrics.pop_front();
            } else {
                break;
            }
        }

        // Clean BCAI metrics
        while let Some(front) = self.bcai_metrics.front() {
            if front.timestamp < cutoff_time {
                self.bcai_metrics.pop_front();
            } else {
                break;
            }
        }

        // Clean performance metrics
        while let Some(front) = self.performance_metrics.front() {
            if front.timestamp < cutoff_time {
                self.performance_metrics.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn send_alert(&mut self, severity: AlertSeverity, message: String) {
        let alert = Alert {
            id: "custom_alert".to_string(),
            metric: "".to_string(),
            threshold: 0.0,
            comparison: AlertComparison::Equal,
            severity,
            message,
            enabled: true,
            cooldown_seconds: 0,
        };
        self.alerts.push(alert);
    }
    
    pub fn set_status(&mut self, status: HealthStatus) {
        self.status = status;
    }
    
    pub fn status(&self) -> &HealthStatus {
        &self.status
    }
    
    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }
}

/// Dashboard data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDashboard {
    pub system_health: HealthStatus,
    pub latest_system_metrics: Option<SystemMetrics>,
    pub latest_bcai_metrics: Option<BcaiMetrics>,
    pub latest_performance_metrics: Option<PerformanceMetrics>,
    pub health_checks: HashMap<String, HealthCheck>,
    pub active_alerts: Vec<TriggeredAlert>,
    pub critical_alerts_count: usize,
    pub total_alerts_today: usize,
    pub uptime_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitoring_system_creation() {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config);

        assert_eq!(monitoring.system_metrics.len(), 0);
        assert_eq!(monitoring.alerts.len(), 4); // Default alerts
    }

    #[test]
    fn system_metrics_collection() {
        let mut monitoring = MonitoringSystem::new(MonitoringConfig::default());

        let result = monitoring.collect_system_metrics();
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.cpu_usage_percent >= 0.0);
        assert!(metrics.memory_usage_bytes > 0);
    }

    #[test]
    fn health_checks() {
        let mut monitoring = MonitoringSystem::new(MonitoringConfig::default());

        let health_checks = monitoring.perform_health_checks();
        assert!(health_checks.is_ok());

        let checks = health_checks.unwrap();
        assert_eq!(checks.len(), 5); // P2P, Blockchain, Consensus, Storage, Network

        for check in checks {
            assert!(!check.component.is_empty());
            assert!(check.response_time_ms >= 0.0);
        }
    }

    #[test]
    fn alert_management() {
        let mut monitoring = MonitoringSystem::new(MonitoringConfig::default());

        // Add custom alert
        let custom_alert = Alert {
            id: "test_alert".to_string(),
            metric: "cpu_usage_percent".to_string(),
            threshold: 50.0,
            comparison: AlertComparison::Greater,
            severity: AlertSeverity::Warning,
            message: "Test alert".to_string(),
            enabled: true,
            cooldown_seconds: 60,
        };

        monitoring.add_alert(custom_alert);
        assert_eq!(monitoring.alerts.len(), 5); // 4 default + 1 custom
    }
}
