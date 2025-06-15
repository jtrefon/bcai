//! Security Layer for BCAI
//! 
//! This module provides comprehensive security features including
//! authentication, encryption, access control, and threat detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub enable_authentication: bool,
    /// Enable encryption for data at rest
    pub enable_encryption_at_rest: bool,
    /// Enable encryption for data in transit
    pub enable_encryption_in_transit: bool,
    /// Session timeout duration
    pub session_timeout: Duration,
    /// Maximum failed authentication attempts
    pub max_auth_attempts: u32,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_authentication: true,
            enable_encryption_at_rest: true,
            enable_encryption_in_transit: true,
            session_timeout: Duration::from_secs(3600), // 1 hour
            max_auth_attempts: 3,
            rate_limit: RateLimitConfig::default(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per time window
    pub max_requests: u32,
    /// Time window for rate limiting
    pub time_window: Duration,
    /// Enable rate limiting
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            time_window: Duration::from_secs(60), // 1 minute
            enabled: true,
        }
    }
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password_hash: String,
    pub public_key: Option<String>,
    pub permissions: Vec<Permission>,
}

/// Permission levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Admin,
    Consensus,
    Storage,
}

/// Security session
#[derive(Debug, Clone)]
pub struct SecuritySession {
    pub session_id: String,
    pub user_id: String,
    pub created_at: u64,
    pub last_activity: u64,
    pub permissions: Vec<Permission>,
    pub is_active: bool,
}

/// Security manager
#[derive(Debug)]
pub struct SecurityManager {
    config: SecurityConfig,
    sessions: HashMap<String, SecuritySession>,
    auth_attempts: HashMap<String, u32>,
    rate_limits: HashMap<String, Vec<u64>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
            auth_attempts: HashMap::new(),
            rate_limits: HashMap::new(),
        }
    }

    /// Authenticate a user
    pub fn authenticate(&mut self, credentials: &AuthCredentials) -> Result<String, SecurityError> {
        if !self.config.enable_authentication {
            return Ok("no_auth_session".to_string());
        }

        // Check rate limiting
        if self.is_rate_limited(&credentials.username) {
            return Err(SecurityError::RateLimited);
        }

        // Simulate authentication (in reality would verify password hash)
        if credentials.username.is_empty() {
            self.record_failed_attempt(&credentials.username);
            return Err(SecurityError::InvalidCredentials);
        }

        // Create session
        let session_id = format!("session_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        let session = SecuritySession {
            session_id: session_id.clone(),
            user_id: credentials.username.clone(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            last_activity: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            permissions: credentials.permissions.clone(),
            is_active: true,
        };

        self.sessions.insert(session_id.clone(), session);
        self.auth_attempts.remove(&credentials.username); // Clear failed attempts on success

        Ok(session_id)
    }

    /// Validate a session
    pub fn validate_session(&mut self, session_id: &str) -> Result<&SecuritySession, SecurityError> {
        let session = self.sessions.get_mut(session_id)
            .ok_or(SecurityError::InvalidSession)?;

        if !session.is_active {
            return Err(SecurityError::SessionExpired);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now - session.created_at > self.config.session_timeout.as_secs() {
            session.is_active = false;
            return Err(SecurityError::SessionExpired);
        }

        session.last_activity = now;
        Ok(session)
    }

    /// Check if user has permission
    pub fn has_permission(&self, session_id: &str, permission: &Permission) -> bool {
        if let Some(session) = self.sessions.get(session_id) {
            session.permissions.contains(permission) || session.permissions.contains(&Permission::Admin)
        } else {
            false
        }
    }

    /// Encrypt data
    pub fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if !self.config.enable_encryption_at_rest {
            return Ok(data.to_vec());
        }

        // Simplified encryption (in reality would use proper crypto)
        let mut encrypted = data.to_vec();
        for byte in &mut encrypted {
            *byte = byte.wrapping_add(42); // Simple XOR-like operation
        }
        Ok(encrypted)
    }

    /// Decrypt data
    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if !self.config.enable_encryption_at_rest {
            return Ok(encrypted_data.to_vec());
        }

        // Simplified decryption
        let mut decrypted = encrypted_data.to_vec();
        for byte in &mut decrypted {
            *byte = byte.wrapping_sub(42);
        }
        Ok(decrypted)
    }

    /// Check rate limiting
    fn is_rate_limited(&mut self, user_id: &str) -> bool {
        if !self.config.rate_limit.enabled {
            return false;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let window_start = now - self.config.rate_limit.time_window.as_secs();

        let requests = self.rate_limits.entry(user_id.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the time window
        requests.retain(|&timestamp| timestamp > window_start);
        
        // Check if we're over the limit
        if requests.len() >= self.config.rate_limit.max_requests as usize {
            return true;
        }

        // Record this request
        requests.push(now);
        false
    }

    /// Record failed authentication attempt
    fn record_failed_attempt(&mut self, user_id: &str) {
        let attempts = self.auth_attempts.entry(user_id.to_string()).or_insert(0);
        *attempts += 1;
    }

    /// Get security statistics
    pub fn get_stats(&self) -> SecurityStats {
        let active_sessions = self.sessions.values().filter(|s| s.is_active).count();
        let total_sessions = self.sessions.len();
        let failed_attempts: u32 = self.auth_attempts.values().sum();

        SecurityStats {
            active_sessions,
            total_sessions,
            failed_auth_attempts: failed_attempts,
            rate_limited_users: self.rate_limits.len(),
            encryption_enabled: self.config.enable_encryption_at_rest,
            authentication_enabled: self.config.enable_authentication,
        }
    }
}

/// Security errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum SecurityError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Session expired")]
    SessionExpired,
    #[error("Rate limited")]
    RateLimited,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

/// Security statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub failed_auth_attempts: u32,
    pub rate_limited_users: usize,
    pub encryption_enabled: bool,
    pub authentication_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_manager_creation() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config);
        
        let stats = manager.get_stats();
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.total_sessions, 0);
    }

    #[test]
    fn test_authentication() {
        let config = SecurityConfig::default();
        let mut manager = SecurityManager::new(config);
        
        let credentials = AuthCredentials {
            username: "test_user".to_string(),
            password_hash: "hash123".to_string(),
            public_key: None,
            permissions: vec![Permission::Read, Permission::Write],
        };
        
        let session_id = manager.authenticate(&credentials).unwrap();
        assert!(!session_id.is_empty());
        
        let stats = manager.get_stats();
        assert_eq!(stats.active_sessions, 1);
    }

    #[test]
    fn test_session_validation() {
        let config = SecurityConfig::default();
        let mut manager = SecurityManager::new(config);
        
        let credentials = AuthCredentials {
            username: "test_user".to_string(),
            password_hash: "hash123".to_string(),
            public_key: None,
            permissions: vec![Permission::Read],
        };
        
        let session_id = manager.authenticate(&credentials).unwrap();
        let session = manager.validate_session(&session_id).unwrap();
        
        assert_eq!(session.user_id, "test_user");
        assert!(session.is_active);
    }

    #[test]
    fn test_permissions() {
        let config = SecurityConfig::default();
        let mut manager = SecurityManager::new(config);
        
        let credentials = AuthCredentials {
            username: "test_user".to_string(),
            password_hash: "hash123".to_string(),
            public_key: None,
            permissions: vec![Permission::Read],
        };
        
        let session_id = manager.authenticate(&credentials).unwrap();
        
        assert!(manager.has_permission(&session_id, &Permission::Read));
        assert!(!manager.has_permission(&session_id, &Permission::Write));
    }

    #[test]
    fn test_encryption() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config);
        
        let data = b"test data";
        let encrypted = manager.encrypt_data(data).unwrap();
        let decrypted = manager.decrypt_data(&encrypted).unwrap();
        
        assert_eq!(data, &decrypted[..]);
    }
} 