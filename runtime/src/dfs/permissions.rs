//! Handles all permission and access control logic for the DFS.

use crate::dfs::types::{
    DfsError, DfsFile, EncryptionMetadata, FilePermissions, GroupPermissions, PermissionGroup,
    TemporaryAccess, TemporaryAccessType,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use std::time::Duration;

/// Checks if a requester has access to a file based on its permissions.
pub async fn check_file_access(file: &DfsFile, requester: &str) -> Result<(), DfsError> {
    if !file.encryption_metadata.is_encrypted {
        return Ok(()); // Public file
    }

    match &file.encryption_metadata.permissions {
        FilePermissions::Public => Ok(()),
        FilePermissions::OwnerOnly { owner, .. } => {
            if owner == requester {
                Ok(())
            } else {
                Err(DfsError::AccessDenied("File is owner-only".to_string()))
            }
        }
        FilePermissions::Group { members, .. } => {
            if members.contains(&requester.to_string()) {
                Ok(())
            } else {
                Err(DfsError::AccessDenied(
                    "User is not a member of the required group".to_string(),
                ))
            }
        }
        FilePermissions::Custom { access_list } => {
            if access_list.contains_key(requester) {
                Ok(())
            } else {
                Err(DfsError::AccessDenied(
                    "User not on the custom access list".to_string(),
                ))
            }
        }
        FilePermissions::TimeBound {
            base_permissions,
            access_grants,
            ..
        } => {
            // Check for a valid temporary grant first
            if let Some(grant) = access_grants
                .iter()
                .find(|g| g.user_id == requester && is_temporary_access_valid(g))
            {
                return Ok(());
            }
            // Fall back to base permissions if no valid grant is found
            check_base_permissions(base_permissions, requester).await
        }
    }
}

/// Retrieves the correct decryption key for a user.
pub async fn get_decryption_key(
    metadata: &EncryptionMetadata,
    requester: &str,
) -> Result<String, DfsError> {
    match &metadata.permissions {
        FilePermissions::OwnerOnly { encrypted_key, .. } => Ok(encrypted_key.clone()),
        FilePermissions::Group { encrypted_key, .. } => Ok(encrypted_key.clone()),
        FilePermissions::Custom { access_list } => access_list
            .get(requester)
            .cloned()
            .ok_or_else(|| DfsError::KeyError("No key found for user".to_string())),
        FilePermissions::TimeBound { access_grants, .. } => {
            if let Some(grant) = access_grants
                .iter()
                .find(|g| g.user_id == requester && is_temporary_access_valid(g))
            {
                Ok(grant.encrypted_key.clone())
            } else {
                Err(DfsError::KeyError(
                    "No valid temporary key grant found".to_string(),
                ))
            }
        }
        _ => Err(DfsError::KeyError(
            "Could not determine key for this permission type".to_string(),
        )),
    }
}

/// Grants temporary access to a file.
pub async fn grant_temporary_access(
    file: &mut DfsFile,
    user_id: String,
    access_type: TemporaryAccessType,
    duration: Duration,
    requester: String,
    max_usage: Option<u64>,
) -> Result<(), DfsError> {
    if !can_grant_access(&file.encryption_metadata.permissions, &requester) {
        return Err(DfsError::AccessDenied(
            "Requester does not have permission to grant access".to_string(),
        ));
    }

    let base_permissions = match &mut file.encryption_metadata.permissions {
        FilePermissions::TimeBound {
            base_permissions, ..
        } => base_permissions.as_mut(),
        _ => &mut file.encryption_metadata.permissions,
    };

    let base_key = get_base_decryption_key(base_permissions, &requester).await?;

    let new_grant = TemporaryAccess {
        user_id,
        encrypted_key: base_key, // In a real system, this might be a new key
        granted_at: Utc::now(),
        expires_at: Utc::now() + duration,
        access_type,
        granted_by: requester,
        usage_count: 0,
        max_usage,
    };

    match &mut file.encryption_metadata.permissions {
        FilePermissions::TimeBound { access_grants, .. } => {
            access_grants.push(new_grant);
        }
        _ => {
            // If not already TimeBound, convert it.
            let original_perms = std::mem::replace(
                &mut file.encryption_metadata.permissions,
                FilePermissions::Public, // Placeholder
            );
            file.encryption_metadata.permissions = FilePermissions::TimeBound {
                base_permissions: Box::new(original_perms),
                access_grants: vec![new_grant],
                default_expiry: None,
            };
        }
    }

    Ok(())
}

/// Creates a new permission group.
pub fn create_permission_group(
    group_id: String,
    name: String,
    description: String,
    owner: String,
    initial_members: Vec<String>,
    permissions: GroupPermissions,
) -> Result<PermissionGroup, DfsError> {
    let mut members = initial_members;
    if !members.contains(&owner) {
        members.push(owner.clone());
    }

    Ok(PermissionGroup {
        group_id,
        name,
        description,
        owner,
        members,
        group_key: generate_group_key()?,
        created_at: Utc::now(),
        last_modified: Utc::now(),
        permissions,
    })
}

// --- Helper Functions ---

fn is_temporary_access_valid(grant: &TemporaryAccess) -> bool {
    Utc::now() < grant.expires_at && grant.max_usage.map_or(true, |max| grant.usage_count < max)
}

fn can_grant_access(permissions: &FilePermissions, requester: &str) -> bool {
    match permissions {
        FilePermissions::OwnerOnly { owner, .. } => owner == requester,
        FilePermissions::Group { group_id, .. } => {
            // In a real system, you'd check if the user is an admin of the group
            false
        }
        _ => false, // Simplified
    }
}

async fn check_base_permissions(
    permissions: &FilePermissions,
    requester: &str,
) -> Result<(), DfsError> {
    // This is a simplified version of the main check_file_access logic
    match permissions {
        FilePermissions::OwnerOnly { owner, .. } if owner == requester => Ok(()),
        _ => Err(DfsError::AccessDenied(
            "User does not have base permissions for this file".to_string(),
        )),
    }
}

async fn get_base_decryption_key(
    permissions: &FilePermissions,
    _requester: &str,
) -> Result<String, DfsError> {
    match permissions {
        FilePermissions::OwnerOnly { encrypted_key, .. } => Ok(encrypted_key.clone()),
        _ => Err(DfsError::KeyError("Cannot get base key for this permission type".to_string())),
    }
}

fn generate_group_key() -> Result<String, DfsError> {
    // This should be a robust key generation method.
    // Reusing symmetric key generation for simplicity.
    let key = aes_gcm::Aes256Gcm::generate_key(&mut aes_gcm::aead::OsRng);
    Ok(general_purpose::STANDARD.encode(key))
} 