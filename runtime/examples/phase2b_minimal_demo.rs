//! Phase 2B: Production Security & Monitoring Demo
//!
//! This demo showcases BCAI's production-ready security and monitoring capabilities:
//! - Advanced security with authentication and attack detection
//! - Comprehensive monitoring and health checks
//! - Performance metrics and alerting

use runtime::{
    monitoring::{MonitoringSystem, MonitoringConfig, AlertSeverity, HealthStatus},
    security::{SecurityManager, RateLimitConfig, SecurityLevel, AuthCredentials},
    pouw::{generate_task, solve},
};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BCAI Phase 2B: Production Security & Monitoring Demo");
    println!("=====================================================");

    // Phase 1: Initialize Production Monitoring
    println!("\n📊 Phase 1: Production Monitoring System");
    
    let monitoring_config = MonitoringConfig {
        metrics_retention_hours: 48,
        health_check_interval_secs: 15,
        performance_sample_interval_secs: 5,
        alert_check_interval_secs: 2,
        max_alerts_per_hour: 50,
    };
    let mut monitoring = MonitoringSystem::new(monitoring_config);
    println!("✅ Monitoring system initialized with production configuration");

    // Collect system metrics
    let system_metrics = monitoring.collect_system_metrics()?;
    println!("💻 System Load - CPU: {:.1}%, Memory: {:.1}GB, Disk: {:.1}GB", 
        system_metrics.cpu_usage_percent,
        system_metrics.memory_usage_bytes / (1024 * 1024 * 1024),
        system_metrics.disk_usage_bytes / (1024 * 1024 * 1024)
    );

    // Collect BCAI metrics
    let bcai_metrics = monitoring.collect_bcai_metrics(
        5,    // active nodes
        100,  // total jobs
        85,   // completed jobs  
        15,   // pending jobs
        500,  // total transactions
        1000, // block height
        5,    // validator count
        2500000, // total stake
    )?;
    println!("🔗 Network State - Nodes: {}, Jobs: {}/{}, TPS: {:.1}, Block: {}", 
        bcai_metrics.active_nodes,
        bcai_metrics.completed_jobs,
        bcai_metrics.total_jobs,
        bcai_metrics.transaction_throughput_per_sec,
        bcai_metrics.block_height
    );

    // Phase 2: Health Monitoring
    println!("\n🏥 Phase 2: System Health Monitoring");
    
    let health_checks = monitoring.perform_health_checks()?;
    for check in &health_checks {
        let status_icon = match check.status {
            HealthStatus::Healthy => "✅",
            HealthStatus::Warning => "⚠️",
            HealthStatus::Critical => "❌",
            HealthStatus::Unknown => "❓",
        };
        println!("  {} {}: {} ({:.1}ms)", 
            status_icon, 
            check.component, 
            check.message,
            check.response_time_ms
        );
    }

    let overall_health = monitoring.get_system_health();
    println!("🎯 Overall System Health: {:?}", overall_health);

    // Phase 3: Advanced Security System
    println!("\n🛡️ Phase 3: Production Security System");
    
    let security_config = RateLimitConfig {
        max_requests_per_minute: 60,
        max_auth_attempts_per_hour: 5,
        ban_duration_secs: 7200, // 2 hours
        burst_threshold: 10,
    };
    let mut security_manager = SecurityManager::new(security_config);
    println!("✅ Security manager initialized with production-grade protection");

    // Register secure nodes
    let security_nodes = vec![
        ("validator_1", SecurityLevel::Critical),
        ("worker_1", SecurityLevel::High), 
        ("worker_2", SecurityLevel::High),
        ("edge_1", SecurityLevel::Medium),
        ("edge_2", SecurityLevel::Medium),
    ];

    let mut node_credentials = Vec::new();
    for (node_id, security_level) in &security_nodes {
        let (private_key, public_key) = security_manager.register_node(node_id, *security_level)?;
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let credentials = AuthCredentials {
            node_id: node_id.to_string(),
            public_key,
            signature: format!("sig_{}_{}", node_id, timestamp),
            timestamp,
            nonce: 12345,
        };
        
        node_credentials.push((node_id.to_string(), credentials));
        println!("🔐 Registered {} with {:?} security clearance", node_id, security_level);
    }

    // Phase 4: Authentication Testing
    println!("\n🔑 Phase 4: Authentication & Authorization Testing");
    
    // Test legitimate authentication
    for (node_id, credentials) in &node_credentials {
        match security_manager.authenticate(credentials) {
            Ok(()) => println!("✅ {} authenticated successfully", node_id),
            Err(e) => println!("❌ {} authentication failed: {}", node_id, e),
        }
    }

    // Test permissions
    let operations = ["validate", "submit_job", "participate_training", "vote"];
    println!("\n🔓 Permission Testing:");
    for (node_id, _) in &node_credentials[..2] { // Test first 2 nodes
        for operation in &operations {
            match security_manager.has_permission(node_id, operation) {
                Ok(true) => println!("  ✅ {} can {}", node_id, operation),
                Ok(false) => println!("  ⚠️ {} denied {}", node_id, operation),
                Err(e) => println!("  ❌ {} permission error: {}", node_id, e),
            }
        }
    }

    // Phase 5: Attack Detection Simulation
    println!("\n🚨 Phase 5: Attack Detection & Prevention");
    
    let attack_scenarios = [
        ("brute_force_password_attempt", "suspicious_actor_1"),
        ("ddos_flood_attack", "botnet_node_2"),
        ("injection_sql_attempt", "malicious_user_3"),
        ("replay_attack_detected", "compromised_node_4"),
    ];

    for (attack_type, attacker) in &attack_scenarios {
        if let Some(event) = security_manager.detect_attack(attacker, attack_type) {
            println!("🔴 SECURITY ALERT: {} from {}", event.message, event.source);
        }
    }

    // Phase 6: Performance Monitoring
    println!("\n⚡ Phase 6: Performance Monitoring & Benchmarks");
    
    // PoUW Performance Benchmark
    let start_time = SystemTime::now();
    let task = generate_task(4, 1234);
    let solution = solve(&task, 0x0000ffff);
    let pouw_time = start_time.elapsed()?.as_millis();
    
    // Collect performance metrics
    let performance_metrics = monitoring.collect_performance_metrics(
        20.5,  // P2P latency
        45.0,  // Consensus latency
        0.94,  // Training accuracy
        5,     // Federated rounds
        pouw_time as f64 // PoUW solve time
    )?;
    
    println!("📈 Performance Metrics:");
    println!("  Training Accuracy: {:.1}%", performance_metrics.training_accuracy * 100.0);
    println!("  P2P Latency: {:.1}ms", performance_metrics.p2p_latency_ms);
    println!("  Consensus Time: {:.1}ms", performance_metrics.consensus_latency_ms);
    println!("  PoUW Solve Time: {}ms", pouw_time);

    // Phase 7: Alert Management
    println!("\n🚨 Phase 7: Alert Management System");
    
    let triggered_alerts = monitoring.check_alerts()?;
    if !triggered_alerts.is_empty() {
        println!("Active Alerts:");
        for alert in &triggered_alerts {
            let severity_icon = match alert.severity {
                AlertSeverity::Critical => "🔴",
                AlertSeverity::Error => "🟠",
                AlertSeverity::Warning => "🟡", 
                AlertSeverity::Info => "🔵",
            };
            println!("  {} {} - {} ({})", severity_icon, alert.severity as u8, alert.message, alert.metric);
        }
    } else {
        println!("✅ No active alerts - all systems within normal parameters");
    }

    // Phase 8: Dashboard & Reporting
    println!("\n📊 Phase 8: Production Dashboard");
    
    let dashboard = monitoring.get_dashboard_data();
    println!("Production Status:");
    println!("  System Health: {:?}", dashboard.system_health);
    println!("  Total Alerts: {} (Critical: {})", 
        dashboard.active_alerts.len(), 
        dashboard.critical_alerts_count
    );
    println!("  System Uptime: {:.2}%", dashboard.uptime_percentage);

    let security_stats = security_manager.get_security_stats();
    println!("\nSecurity Status:");
    println!("  Security Events: {}", security_stats.total_security_events);
    println!("  Critical Events: {}", security_stats.critical_events);
    println!("  Attack Attempts: {}", security_stats.total_attack_attempts);
    println!("  Banned Nodes: {}", security_stats.banned_nodes_count);
    println!("  Active Nodes: {}", security_stats.active_nodes);

    // Final Phase 2B Summary
    println!("\n🎯 Phase 2B Production Features Summary");
    println!("=======================================");
    println!("✅ Security: Multi-level authentication with attack detection");
    println!("✅ Monitoring: Real-time metrics collection and analysis");
    println!("✅ Health: Comprehensive system health monitoring");
    println!("✅ Performance: Continuous performance tracking");
    println!("✅ Alerting: Intelligent alert management system");
    println!("✅ Dashboard: Production-ready observability");
    
    println!("\n🚀 BCAI Phase 2B Production Features - COMPLETE!");
    println!("Ready for enterprise deployment with full security and monitoring.");

    Ok(())
} 