//! Simplified Time-Bound Permissions Demo
//!
//! This demo focuses on the time-bound permissions system without requiring
//! the full distributed storage infrastructure. It demonstrates:
//! - Temporary access grants with expiration dates
//! - Different access types (Trial, Subscription, Emergency)
//! - Usage limits and automatic cleanup
//! - Permission checking and validation

use std::time::Duration;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use runtime::decentralized_filesystem::{
    FilePermissions, TemporaryAccessType, TemporaryAccess, DfsFile, DfsChunk,
    EncryptionMetadata, FileVisibility
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕰️  Time-Bound Permissions System Demo (Simplified)");
    println!("===================================================");
    
    // Demo scenarios
    demo_temporary_access_types().await?;
    demo_expiration_checking().await?;
    demo_usage_limits().await?;
    demo_permission_hierarchies().await?;
    
    println!("\n🎉 Time-bound permissions demo completed successfully!");
    Ok(())
}

/// Demo 1: Different types of temporary access
async fn demo_temporary_access_types() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 1: Temporary Access Types");
    println!("=================================");
    
    // Create a sample file with time-bound permissions
    let mut temp_grants = Vec::new();
    
    // Trial access - limited time and usage
    let trial_access = TemporaryAccess {
        user_id: "trial_user".to_string(),
        encrypted_key: "trial_key_123".to_string(),
        granted_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::minutes(30),
        access_type: TemporaryAccessType::Trial,
        granted_by: "researcher".to_string(),
        usage_count: 0,
        max_usage: Some(3),
    };
    temp_grants.push(trial_access);
    
    // Subscription access - longer duration, higher usage
    let subscription_access = TemporaryAccess {
        user_id: "subscriber".to_string(),
        encrypted_key: "sub_key_456".to_string(),
        granted_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::days(30),
        access_type: TemporaryAccessType::Subscription,
        granted_by: "researcher".to_string(),
        usage_count: 15,
        max_usage: Some(100),
    };
    temp_grants.push(subscription_access);
    
    // Emergency access - unlimited usage, short duration
    let emergency_access = TemporaryAccess {
        user_id: "emergency_responder".to_string(),
        encrypted_key: "emergency_key_911".to_string(),
        granted_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(4),
        access_type: TemporaryAccessType::Emergency,
        granted_by: "admin".to_string(),
        usage_count: 8,
        max_usage: None, // Unlimited for emergencies
    };
    temp_grants.push(emergency_access);
    
    // Read-only collaboration access
    let readonly_access = TemporaryAccess {
        user_id: "collaborator".to_string(),
        encrypted_key: "collab_key_789".to_string(),
        granted_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(48),
        access_type: TemporaryAccessType::ReadOnly,
        granted_by: "researcher".to_string(),
        usage_count: 2,
        max_usage: Some(10),
    };
    temp_grants.push(readonly_access);
    
    // Create time-bound permissions
    let time_bound_permissions = FilePermissions::TimeBound {
        base_permissions: Box::new(FilePermissions::OwnerOnly {
            owner: "researcher".to_string(),
            encrypted_key: "owner_key_base".to_string(),
        }),
        access_grants: temp_grants.clone(),
        default_expiry: Some(Utc::now() + chrono::Duration::hours(24)),
    };
    
    // Display all access grants
    println!("📊 Current temporary access grants:");
    for (i, grant) in temp_grants.iter().enumerate() {
        let access_type_str = match grant.access_type {
            TemporaryAccessType::Trial => "🆓 Trial",
            TemporaryAccessType::Subscription => "💳 Subscription",
            TemporaryAccessType::Emergency => "🚨 Emergency",
            TemporaryAccessType::ReadOnly => "👁️  Read-Only",
            TemporaryAccessType::ReadWrite => "✏️  Read-Write",
        };
        
        let usage_str = match grant.max_usage {
            Some(max) => format!("{}/{}", grant.usage_count, max),
            None => format!("{}/∞", grant.usage_count),
        };
        
        let time_remaining = grant.expires_at.signed_duration_since(Utc::now());
        let time_str = if time_remaining.num_hours() > 0 {
            format!("{}h {}m", time_remaining.num_hours(), time_remaining.num_minutes() % 60)
        } else {
            format!("{}m", time_remaining.num_minutes())
        };
        
        println!("  {}. {} {} - {} usage, expires in {}",
            i + 1,
            access_type_str,
            grant.user_id,
            usage_str,
            time_str
        );
    }
    
    Ok(())
}

/// Demo 2: Expiration checking and validation
async fn demo_expiration_checking() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 2: Expiration Checking");
    println!("==============================");
    
    // Create access grants with different expiration times
    let now = Utc::now();
    
    let expired_access = TemporaryAccess {
        user_id: "expired_user".to_string(),
        encrypted_key: "expired_key".to_string(),
        granted_at: now - chrono::Duration::hours(2),
        expires_at: now - chrono::Duration::minutes(30), // Expired 30 minutes ago
        access_type: TemporaryAccessType::Trial,
        granted_by: "researcher".to_string(),
        usage_count: 1,
        max_usage: Some(5),
    };
    
    let valid_access = TemporaryAccess {
        user_id: "valid_user".to_string(),
        encrypted_key: "valid_key".to_string(),
        granted_at: now - chrono::Duration::minutes(30),
        expires_at: now + chrono::Duration::hours(2), // Valid for 2 more hours
        access_type: TemporaryAccessType::ReadOnly,
        granted_by: "researcher".to_string(),
        usage_count: 0,
        max_usage: Some(10),
    };
    
    let usage_exceeded_access = TemporaryAccess {
        user_id: "heavy_user".to_string(),
        encrypted_key: "heavy_key".to_string(),
        granted_at: now - chrono::Duration::hours(1),
        expires_at: now + chrono::Duration::hours(1), // Still valid time-wise
        access_type: TemporaryAccessType::Trial,
        granted_by: "researcher".to_string(),
        usage_count: 5,
        max_usage: Some(5), // But usage limit reached
    };
    
    let grants = vec![expired_access, valid_access, usage_exceeded_access];
    
    println!("🔍 Checking access validity:");
    for grant in &grants {
        let is_time_valid = now <= grant.expires_at;
        let is_usage_valid = match grant.max_usage {
            Some(max) => grant.usage_count < max,
            None => true,
        };
        let is_valid = is_time_valid && is_usage_valid;
        
        let status = if is_valid {
            "✅ VALID"
        } else if !is_time_valid {
            "⏰ EXPIRED"
        } else {
            "🚫 USAGE_EXCEEDED"
        };
        
        let message = if is_valid {
            "Access granted".to_string()
        } else if !is_time_valid {
            format!("Expired {} ago", 
                format_duration(now.signed_duration_since(grant.expires_at)))
        } else {
            format!("Usage limit reached ({}/{})", 
                grant.usage_count, grant.max_usage.unwrap_or(0))
        };
        
        println!("  {} {}: {}", status, grant.user_id, message);
    }
    
    // Simulate cleanup
    let valid_grants: Vec<_> = grants.into_iter()
        .filter(|grant| {
            let is_time_valid = now <= grant.expires_at;
            let is_usage_valid = match grant.max_usage {
                Some(max) => grant.usage_count < max,
                None => true,
            };
            is_time_valid && is_usage_valid
        })
        .collect();
    
    println!("\n🧹 After cleanup: {} valid grants remaining", valid_grants.len());
    
    Ok(())
}

/// Demo 3: Usage limits and tracking
async fn demo_usage_limits() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 3: Usage Limits & Tracking");
    println!("==================================");
    
    // Create a trial access with limited usage
    let mut trial_grant = TemporaryAccess {
        user_id: "trial_user".to_string(),
        encrypted_key: "trial_key".to_string(),
        granted_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        access_type: TemporaryAccessType::Trial,
        granted_by: "researcher".to_string(),
        usage_count: 0,
        max_usage: Some(3),
    };
    
    println!("🎯 Trial user attempting to access file multiple times:");
    println!("   Max usage allowed: {}", trial_grant.max_usage.unwrap());
    
    // Simulate multiple access attempts
    for attempt in 1..=5 {
        let can_access = match trial_grant.max_usage {
            Some(max) => trial_grant.usage_count < max,
            None => true,
        };
        
        if can_access {
            trial_grant.usage_count += 1;
            println!("  ✅ Attempt #{}: Access granted (usage: {}/{})", 
                attempt, trial_grant.usage_count, trial_grant.max_usage.unwrap());
        } else {
            println!("  ❌ Attempt #{}: Access denied - usage limit exceeded ({}/{})", 
                attempt, trial_grant.usage_count, trial_grant.max_usage.unwrap());
        }
    }
    
    // Compare with unlimited emergency access
    let mut emergency_grant = TemporaryAccess {
        user_id: "emergency_responder".to_string(),
        encrypted_key: "emergency_key".to_string(),
        granted_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(4),
        access_type: TemporaryAccessType::Emergency,
        granted_by: "admin".to_string(),
        usage_count: 0,
        max_usage: None, // Unlimited
    };
    
    println!("\n🚨 Emergency responder accessing critical data:");
    for attempt in 1..=8 {
        emergency_grant.usage_count += 1;
        println!("  ⚡ Emergency access #{}: Granted (usage: {}/∞)", 
            attempt, emergency_grant.usage_count);
    }
    
    Ok(())
}

/// Demo 4: Permission hierarchies and fallback
async fn demo_permission_hierarchies() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 4: Permission Hierarchies");
    println!("=================================");
    
    // Create nested time-bound permissions
    let inner_permissions = FilePermissions::Group {
        group_id: "research_team".to_string(),
        encrypted_key: "group_key_123".to_string(),
        members: vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()],
    };
    
    let temp_grants = vec![
        TemporaryAccess {
            user_id: "external_reviewer".to_string(),
            encrypted_key: "reviewer_key".to_string(),
            granted_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7),
            access_type: TemporaryAccessType::ReadOnly,
            granted_by: "alice".to_string(),
            usage_count: 0,
            max_usage: Some(20),
        },
        TemporaryAccess {
            user_id: "consultant".to_string(),
            encrypted_key: "consultant_key".to_string(),
            granted_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(3),
            access_type: TemporaryAccessType::ReadWrite,
            granted_by: "bob".to_string(),
            usage_count: 5,
            max_usage: Some(50),
        },
    ];
    
    let time_bound_permissions = FilePermissions::TimeBound {
        base_permissions: Box::new(inner_permissions),
        access_grants: temp_grants,
        default_expiry: Some(Utc::now() + chrono::Duration::days(30)),
    };
    
    // Test access for different users
    let test_users = vec![
        ("alice", "Group member (base permissions)"),
        ("external_reviewer", "Temporary read-only access"),
        ("consultant", "Temporary read-write access"),
        ("unauthorized_user", "No access"),
    ];
    
    println!("🔐 Testing access for different users:");
    
    for (user, description) in test_users {
        let has_temp_access = if let FilePermissions::TimeBound { access_grants, .. } = &time_bound_permissions {
            access_grants.iter().any(|grant| {
                grant.user_id == user && 
                Utc::now() <= grant.expires_at &&
                match grant.max_usage {
                    Some(max) => grant.usage_count < max,
                    None => true,
                }
            })
        } else {
            false
        };
        
        let has_base_access = if let FilePermissions::TimeBound { base_permissions, .. } = &time_bound_permissions {
            match base_permissions.as_ref() {
                FilePermissions::Group { members, .. } => members.contains(&user.to_string()),
                FilePermissions::OwnerOnly { owner, .. } => owner == user,
                FilePermissions::Public => true,
                _ => false,
            }
        } else {
            false
        };
        
        let access_result = if has_temp_access {
            "✅ GRANTED (temporary)"
        } else if has_base_access {
            "✅ GRANTED (base)"
        } else {
            "❌ DENIED"
        };
        
        println!("  {} {}: {}", access_result, user, description);
    }
    
    Ok(())
}

/// Helper function to format duration in a human-readable way
fn format_duration(duration: chrono::Duration) -> String {
    let total_minutes = duration.num_minutes();
    if total_minutes < 60 {
        format!("{}m", total_minutes)
    } else {
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        if hours < 24 {
            format!("{}h {}m", hours, minutes)
        } else {
            let days = hours / 24;
            let remaining_hours = hours % 24;
            format!("{}d {}h", days, remaining_hours)
        }
    }
} 