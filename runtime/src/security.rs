//! Security Module for BCAI
//!
//! This module provides comprehensive security features:
//! - Advanced key management and encryption
//! - Attack detection and prevention
//! - Secure communication protocols
//! - Security monitoring and alerting

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Security-related errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Key management error: {0}")]
    KeyManagementError(String),
    #[error("Attack detected: {0} from {1}")]
    AttackDetected(String, String),
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
}

/// Security levels for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub node_id: String,
    pub public_key: String,
    pub signature: String,
    pub timestamp: u64,
    pub nonce: u64,
}

/// Access permissions for different operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPermissions {
    pub can_validate: bool,
    pub can_submit_jobs: bool,
    pub can_participate_training: bool,
    pub can_vote: bool,
    pub security_clearance: SecurityLevel,
}

/// Attack detection metrics
#[derive(Debug, Clone)]
pub struct AttackMetrics {
    pub failed_authentications: u32,
    pub rate_limit_violations: u32,
    pub suspicious_patterns: u32,
    pub last_attack_time: Option<u64>,
    pub attack_source: Option<String>,
}

/// Security event for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub severity: SecurityLevel,
    pub source: String,
    pub message: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityEventType {
    AuthenticationFailure,
    AuthenticationSuccess,
    AuthorizationDenied,
    RateLimitExceeded,
    SuspiciousActivity,
    AttackDetected,
    SecurityViolation,
    KeyRotation,
    EncryptionFailure,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_minute: u32,
    pub max_auth_attempts_per_hour: u32,
    pub ban_duration_secs: u64,
    pub burst_threshold: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 100,
            max_auth_attempts_per_hour: 10,
            ban_duration_secs: 3600, // 1 hour
            burst_threshold: 20,
        }
    }
}

/// Key management for secure operations
#[derive(Debug, Clone)]
pub struct KeyManager {
    private_keys: HashMap<String, String>, // node_id -> private_key
    public_keys: HashMap<String, String>,  // node_id -> public_key
    key_rotation_schedule: HashMap<String, u64>, // node_id -> next_rotation_time
    compromised_keys: Vec<String>,
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            private_keys: HashMap::new(),
            public_keys: HashMap::new(),
            key_rotation_schedule: HashMap::new(),
            compromised_keys: Vec::new(),
        }
    }

    /// Generate a new key pair for a node
    pub fn generate_keypair(&mut self, node_id: &str) -> Result<(String, String), SecurityError> {
        // Simplified key generation (in production, use proper cryptographic libraries)
        let private_key = self.generate_private_key(node_id)?;
        let public_key = self.derive_public_key(&private_key)?;

        self.private_keys.insert(node_id.to_string(), private_key.clone());
        self.public_keys.insert(node_id.to_string(), public_key.clone());

        // Schedule key rotation in 30 days
        let rotation_time =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 30 * 24 * 3600; // 30 days
        self.key_rotation_schedule.insert(node_id.to_string(), rotation_time);

        Ok((private_key, public_key))
    }

    /// Get public key for a node
    pub fn get_public_key(&self, node_id: &str) -> Option<&String> {
        self.public_keys.get(node_id)
    }

    /// Check if key rotation is needed
    pub fn needs_rotation(&self, node_id: &str) -> bool {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        self.key_rotation_schedule
            .get(node_id)
            .map(|&rotation_time| current_time >= rotation_time)
            .unwrap_or(false)
    }

    /// Rotate keys for a node
    pub fn rotate_keys(&mut self, node_id: &str) -> Result<(String, String), SecurityError> {
        // Mark old key as compromised if it exists
        if let Some(old_key) = self.public_keys.get(node_id) {
            self.compromised_keys.push(old_key.clone());
        }

        // Generate new key pair
        self.generate_keypair(node_id)
    }

    /// Check if a key is compromised
    pub fn is_key_compromised(&self, public_key: &str) -> bool {
        self.compromised_keys.contains(&public_key.to_string())
    }

    /// Sign a message
    pub fn sign_message(&self, node_id: &str, message: &str) -> Result<String, SecurityError> {
        let private_key = self.private_keys.get(node_id).ok_or_else(|| {
            SecurityError::KeyManagementError("Private key not found".to_string())
        })?;

        // Simplified signing (use proper crypto in production)
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        hasher.update(private_key.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Verify a signature
    pub fn verify_signature(
        &self,
        node_id: &str,
        message: &str,
        signature: &str,
    ) -> Result<bool, SecurityError> {
        let public_key = self
            .public_keys
            .get(node_id)
            .ok_or_else(|| SecurityError::KeyManagementError("Public key not found".to_string()))?;

        if self.is_key_compromised(public_key) {
            return Err(SecurityError::KeyManagementError("Key is compromised".to_string()));
        }

        // Simplified verification (use proper crypto in production)
        let _private_key = self.private_keys.get(node_id).unwrap();
        let expected = self.sign_message(node_id, message)?;
        Ok(signature == expected)
    }

    /// Generate private key (simplified)
    fn generate_private_key(&self, node_id: &str) -> Result<String, SecurityError> {
        let mut hasher = Sha256::new();
        hasher.update(node_id.as_bytes());
        hasher
            .update(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_le_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Derive public key from private key (simplified)
    fn derive_public_key(&self, private_key: &str) -> Result<String, SecurityError> {
        let mut hasher = Sha256::new();
        hasher.update(private_key.as_bytes());
        hasher.update(b"public");
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Security manager for comprehensive protection
pub struct SecurityManager {
    key_manager: KeyManager,
    rate_limits: HashMap<String, VecDeque<u64>>, // node_id -> request_timestamps
    attack_metrics: HashMap<String, AttackMetrics>,
    banned_nodes: HashMap<String, u64>, // node_id -> ban_end_time
    access_permissions: HashMap<String, AccessPermissions>,
    security_events: VecDeque<SecurityEvent>,
    config: RateLimitConfig,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            key_manager: KeyManager::new(),
            rate_limits: HashMap::new(),
            attack_metrics: HashMap::new(),
            banned_nodes: HashMap::new(),
            access_permissions: HashMap::new(),
            security_events: VecDeque::new(),
            config,
        }
    }

    /// Register a new node with security credentials
    pub fn register_node(
        &mut self,
        node_id: &str,
        security_level: SecurityLevel,
    ) -> Result<(String, String), SecurityError> {
        // Generate key pair
        let (private_key, public_key) = self.key_manager.generate_keypair(node_id)?;

        // Set default permissions based on security level
        let permissions = AccessPermissions {
            can_validate: security_level >= SecurityLevel::High,
            can_submit_jobs: security_level >= SecurityLevel::Medium,
            can_participate_training: true,
            can_vote: security_level >= SecurityLevel::Medium,
            security_clearance: security_level,
        };

        self.access_permissions.insert(node_id.to_string(), permissions);

        // Initialize attack metrics
        self.attack_metrics.insert(
            node_id.to_string(),
            AttackMetrics {
                failed_authentications: 0,
                rate_limit_violations: 0,
                suspicious_patterns: 0,
                last_attack_time: None,
                attack_source: None,
            },
        );

        self.log_security_event(SecurityEvent {
            event_type: SecurityEventType::AuthenticationSuccess,
            severity: SecurityLevel::Low,
            source: node_id.to_string(),
            message: "Node registered successfully".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::new(),
        });

        Ok((private_key, public_key))
    }

    /// Authenticate a node
    pub fn authenticate(&mut self, credentials: &AuthCredentials) -> Result<(), SecurityError> {
        let node_id = &credentials.node_id;

        // Check if node is banned
        if self.is_node_banned(node_id) {
            return Err(SecurityError::AuthorizationDenied("Node is banned".to_string()));
        }

        // Check rate limits
        self.check_rate_limit(node_id)?;

        // Verify timestamp (prevent replay attacks)
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if current_time > credentials.timestamp + 300 {
            // 5 minute window
            return Err(SecurityError::AuthenticationFailed("Timestamp too old".to_string()));
        }

        // Verify signature
        let message =
            format!("{}:{}:{}", credentials.node_id, credentials.timestamp, credentials.nonce);
        let is_valid =
            self.key_manager.verify_signature(node_id, &message, &credentials.signature)?;

        if !is_valid {
            self.record_failed_authentication(node_id);
            return Err(SecurityError::SignatureVerificationFailed);
        }

        // Authentication successful
        self.log_security_event(SecurityEvent {
            event_type: SecurityEventType::AuthenticationSuccess,
            severity: SecurityLevel::Low,
            source: node_id.to_string(),
            message: "Authentication successful".to_string(),
            timestamp: current_time,
            metadata: HashMap::new(),
        });

        Ok(())
    }

    /// Check if a node has permission for an operation
    pub fn has_permission(
        &mut self,
        node_id: &str,
        operation: &str,
    ) -> Result<bool, SecurityError> {
        let permissions = self
            .access_permissions
            .get(node_id)
            .ok_or_else(|| SecurityError::AuthorizationDenied("Node not registered".to_string()))?;

        let has_permission = match operation {
            "validate" => permissions.can_validate,
            "submit_job" => permissions.can_submit_jobs,
            "participate_training" => permissions.can_participate_training,
            "vote" => permissions.can_vote,
            _ => false,
        };

        if !has_permission {
            self.log_security_event(SecurityEvent {
                event_type: SecurityEventType::AuthorizationDenied,
                severity: SecurityLevel::Medium,
                source: node_id.to_string(),
                message: format!("Permission denied for operation: {}", operation),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                metadata: HashMap::new(),
            });
        }

        Ok(has_permission)
    }

    /// Encrypt sensitive data
    pub fn encrypt_data(&self, data: &str, node_id: &str) -> Result<String, SecurityError> {
        let public_key = self
            .key_manager
            .get_public_key(node_id)
            .ok_or_else(|| SecurityError::EncryptionError("Public key not found".to_string()))?;

        // Simplified encryption (use proper crypto in production)
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(public_key.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Detect and prevent attacks
    pub fn detect_attack(&mut self, node_id: &str, behavior: &str) -> Option<SecurityEvent> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut attack_detected = false;
        let mut attack_type = String::new();

        // Pattern-based attack detection
        if behavior.contains("brute_force") || behavior.contains("dictionary") {
            attack_detected = true;
            attack_type = "Brute Force Attack".to_string();
        } else if behavior.contains("ddos") || behavior.contains("flood") {
            attack_detected = true;
            attack_type = "DDoS Attack".to_string();
        } else if behavior.contains("injection") || behavior.contains("script") {
            attack_detected = true;
            attack_type = "Injection Attack".to_string();
        }

        if attack_detected {
            // Update attack metrics
            if let Some(metrics) = self.attack_metrics.get_mut(node_id) {
                metrics.suspicious_patterns += 1;
                metrics.last_attack_time = Some(current_time);
                metrics.attack_source = Some(behavior.to_string());

                // Ban node if multiple attacks detected
                if metrics.suspicious_patterns >= 3 {
                    self.ban_node(node_id, current_time + self.config.ban_duration_secs);
                }
            }

            let event = SecurityEvent {
                event_type: SecurityEventType::AttackDetected,
                severity: SecurityLevel::Critical,
                source: node_id.to_string(),
                message: format!("Attack detected: {}", attack_type),
                timestamp: current_time,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("attack_type".to_string(), attack_type);
                    meta.insert("behavior".to_string(), behavior.to_string());
                    meta
                },
            };

            self.log_security_event(event.clone());
            return Some(event);
        }

        None
    }

    /// Get security statistics
    pub fn get_security_stats(&self) -> SecurityStats {
        let total_events = self.security_events.len();
        let critical_events =
            self.security_events.iter().filter(|e| e.severity == SecurityLevel::Critical).count();
        let banned_nodes = self.banned_nodes.len();
        let total_attack_attempts: u32 = self
            .attack_metrics
            .values()
            .map(|m| m.failed_authentications + m.suspicious_patterns)
            .sum();

        SecurityStats {
            total_security_events: total_events,
            critical_events,
            banned_nodes_count: banned_nodes,
            total_attack_attempts,
            active_nodes: self.access_permissions.len(),
            keys_rotated_today: 0, // TODO: Track this
        }
    }

    /// Check rate limiting
    fn check_rate_limit(&mut self, node_id: &str) -> Result<(), SecurityError> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let requests = self.rate_limits.entry(node_id.to_string()).or_default();

        // Remove old requests (older than 1 minute)
        while let Some(&front_time) = requests.front() {
            if current_time - front_time > 60 {
                requests.pop_front();
            } else {
                break;
            }
        }

        // Check if rate limit exceeded
        if requests.len() >= self.config.max_requests_per_minute as usize {
            if let Some(metrics) = self.attack_metrics.get_mut(node_id) {
                metrics.rate_limit_violations += 1;
            }

            self.log_security_event(SecurityEvent {
                event_type: SecurityEventType::RateLimitExceeded,
                severity: SecurityLevel::High,
                source: node_id.to_string(),
                message: "Rate limit exceeded".to_string(),
                timestamp: current_time,
                metadata: HashMap::new(),
            });

            return Err(SecurityError::RateLimitExceeded(node_id.to_string()));
        }

        // Add current request
        requests.push_back(current_time);
        Ok(())
    }

    /// Record failed authentication
    fn record_failed_authentication(&mut self, node_id: &str) {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if let Some(metrics) = self.attack_metrics.get_mut(node_id) {
            metrics.failed_authentications += 1;

            // Ban node if too many failed attempts
            if metrics.failed_authentications >= self.config.max_auth_attempts_per_hour {
                self.ban_node(node_id, current_time + self.config.ban_duration_secs);
            }
        }

        self.log_security_event(SecurityEvent {
            event_type: SecurityEventType::AuthenticationFailure,
            severity: SecurityLevel::Medium,
            source: node_id.to_string(),
            message: "Authentication failed".to_string(),
            timestamp: current_time,
            metadata: HashMap::new(),
        });
    }

    /// Ban a node
    fn ban_node(&mut self, node_id: &str, ban_end_time: u64) {
        self.banned_nodes.insert(node_id.to_string(), ban_end_time);

        self.log_security_event(SecurityEvent {
            event_type: SecurityEventType::SecurityViolation,
            severity: SecurityLevel::Critical,
            source: node_id.to_string(),
            message: format!("Node banned until {}", ban_end_time),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::new(),
        });
    }

    /// Check if node is banned
    fn is_node_banned(&mut self, node_id: &str) -> bool {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if let Some(&ban_end_time) = self.banned_nodes.get(node_id) {
            if current_time < ban_end_time {
                return true;
            } else {
                // Ban expired, remove from banned list
                self.banned_nodes.remove(node_id);
            }
        }

        false
    }

    /// Log security event
    fn log_security_event(&mut self, event: SecurityEvent) {
        // Keep only last 1000 events
        if self.security_events.len() >= 1000 {
            self.security_events.pop_front();
        }

        self.security_events.push_back(event);
    }
}

/// Security statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub total_security_events: usize,
    pub critical_events: usize,
    pub banned_nodes_count: usize,
    pub total_attack_attempts: u32,
    pub active_nodes: usize,
    pub keys_rotated_today: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_manager_creation() {
        let mut key_manager = KeyManager::new();
        let result = key_manager.generate_keypair("test_node");

        assert!(result.is_ok());
        let (private_key, public_key) = result.unwrap();
        assert!(!private_key.is_empty());
        assert!(!public_key.is_empty());
    }

    #[test]
    fn security_manager_authentication() {
        let mut security_manager = SecurityManager::new(RateLimitConfig::default());

        // Register node
        let result = security_manager.register_node("test_node", SecurityLevel::Medium);
        assert!(result.is_ok());

        // Create valid credentials
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let message = format!("test_node:{}:123", timestamp);
        let signature = security_manager.key_manager.sign_message("test_node", &message).unwrap();

        let credentials = AuthCredentials {
            node_id: "test_node".to_string(),
            public_key: "test_key".to_string(),
            signature,
            timestamp,
            nonce: 123,
        };

        // Test authentication
        assert!(security_manager.authenticate(&credentials).is_ok());
    }

    #[test]
    fn rate_limiting() {
        let config = RateLimitConfig { max_requests_per_minute: 2, ..Default::default() };
        let mut security_manager = SecurityManager::new(config);

        // Register node
        security_manager.register_node("test_node", SecurityLevel::Low).unwrap();

        // First two requests should succeed
        assert!(security_manager.check_rate_limit("test_node").is_ok());
        assert!(security_manager.check_rate_limit("test_node").is_ok());

        // Third request should fail
        assert!(security_manager.check_rate_limit("test_node").is_err());
    }

    #[test]
    fn attack_detection() {
        let mut security_manager = SecurityManager::new(RateLimitConfig::default());

        let attack_event = security_manager.detect_attack("attacker_node", "brute_force_attempt");
        assert!(attack_event.is_some());

        let event = attack_event.unwrap();
        assert_eq!(event.event_type, SecurityEventType::AttackDetected);
        assert_eq!(event.severity, SecurityLevel::Critical);
    }
}
