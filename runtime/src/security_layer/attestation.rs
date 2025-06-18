// This module will contain the remote attestation logic. 

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub failed_auth_attempts: u32,
    pub rate_limited_users: usize,
    pub encryption_enabled: bool,
    pub authentication_enabled: bool,
} 