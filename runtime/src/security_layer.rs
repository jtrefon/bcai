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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub failed_auth_attempts: u32,
    pub rate_limited_users: usize,
    pub encryption_enabled: bool,
    pub authentication_enabled: bool,
}

// NOTE: Removed placeholder implementation structs:
// - SecurityManager
// This file now only defines the data models for the security layer. 