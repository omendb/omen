//! Test authentication bypass for development
//! 
//! WARNING: This module should ONLY be used in test/development environments!

use crate::types::{Permission, SubscriptionTier, TenantContext, TenantUsage};
use tracing::warn;
use uuid::Uuid;

/// Test tenant context for development
pub fn test_tenant_context() -> TenantContext {
    TenantContext {
        tenant_id: Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap(),
        name: "Test Tenant".to_string(),
        tier: SubscriptionTier::Platform,
        usage: TenantUsage {
            vectors_stored: 0,
            queries_this_hour: 0,
            bandwidth_this_month: 0,
            storage_used_bytes: 0,
        },
        permissions: vec![
            Permission::Read,
            Permission::Write,
            Permission::Delete,
            Permission::ManageCollections,
            Permission::Analytics,
            Permission::Admin,
        ],
    }
}

