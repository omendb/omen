//! Authentication and authorization for OmenDB Server
//! 
//! Handles JWT tokens, API keys, and role-based access control (RBAC)
//! for multi-tenant environments.

use crate::config::AuthConfig;
use crate::types::{Permission, SubscriptionTier, TenantContext, TenantUsage};
use crate::{Error, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (tenant ID)
    pub sub: String,
    /// Tenant name
    pub tenant_name: String,
    /// Subscription tier
    pub tier: SubscriptionTier,
    /// Permissions
    pub permissions: Vec<Permission>,
    /// Issued at timestamp
    pub iat: u64,
    /// Expiration timestamp
    pub exp: u64,
}

/// API key information
#[derive(Debug, Clone)]
pub struct ApiKey {
    /// Key ID
    pub id: Uuid,
    /// Key name/description
    pub name: String,
    /// Tenant ID this key belongs to
    pub tenant_id: Uuid,
    /// Permissions granted to this key
    pub permissions: Vec<Permission>,
    /// Whether the key is active
    pub active: bool,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Authentication manager
pub struct AuthManager {
    /// Configuration
    config: AuthConfig,
    /// JWT encoding key
    encoding_key: EncodingKey,
    /// JWT decoding key
    decoding_key: DecodingKey,
    /// JWT validation settings
    validation: Validation,
    /// API keys storage (in production, this would be a database)
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// Tenant information cache
    tenants: Arc<RwLock<HashMap<Uuid, TenantInfo>>>,
    /// Rate limiting state
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

/// Tenant information
#[derive(Debug, Clone)]
struct TenantInfo {
    /// Tenant ID
    id: Uuid,
    /// Tenant name
    name: String,
    /// Subscription tier
    tier: SubscriptionTier,
    /// Current usage
    usage: TenantUsage,
    /// Whether tenant is active
    active: bool,
}

/// Simple rate limiter implementation
#[derive(Debug)]
struct RateLimiter {
    /// Requests per tenant per time window
    requests: HashMap<Uuid, Vec<chrono::DateTime<chrono::Utc>>>,
    /// Window duration in minutes
    window_minutes: i64,
    /// Maximum requests per window
    max_requests: usize,
}

impl AuthManager {
    /// Create a new authentication manager
    #[instrument(level = "info")]
    pub fn new(config: AuthConfig) -> Result<Self> {
        info!("Initializing authentication manager");

        // Create JWT keys
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());

        // JWT validation settings
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.clear(); // Don't require standard claims

        let rate_limiter = RateLimiter {
            requests: HashMap::new(),
            window_minutes: 60, // 1 hour window
            max_requests: config.rate_limit.requests_per_minute as usize,
        };

        Ok(AuthManager {
            config,
            encoding_key,
            decoding_key,
            validation,
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            tenants: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(rate_limiter)),
        })
    }

    /// Generate a JWT token for a tenant
    #[instrument(level = "debug", skip(self))]
    pub async fn generate_token(&self, tenant_id: Uuid, permissions: Vec<Permission>) -> Result<String> {
        let tenant_info = self.get_tenant_info(tenant_id).await?;
        
        let now = chrono::Utc::now().timestamp() as u64;
        let exp = now + self.config.jwt_expiration.as_secs();

        let claims = Claims {
            sub: tenant_id.to_string(),
            tenant_name: tenant_info.name.clone(),
            tier: tenant_info.tier.clone(),
            permissions,
            iat: now,
            exp,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| Error::auth(format!("Failed to generate token: {}", e)))?;

        debug!("Generated JWT token for tenant {}", tenant_id);
        Ok(token)
    }

    /// Validate a JWT token and extract tenant context
    #[instrument(level = "debug", skip(self, token))]
    pub async fn validate_token(&self, token: &str) -> Result<TenantContext> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| Error::auth(format!("Invalid token: {}", e)))?;

        let claims = token_data.claims;
        
        // Parse tenant ID
        let tenant_id = Uuid::parse_str(&claims.sub)
            .map_err(|e| Error::auth(format!("Invalid tenant ID in token: {}", e)))?;

        // Check if tenant is still active
        let tenant_info = self.get_tenant_info(tenant_id).await?;
        if !tenant_info.active {
            return Err(Error::auth("Tenant account is disabled"));
        }

        // Check rate limits
        self.check_rate_limit(tenant_id).await?;

        Ok(TenantContext {
            tenant_id,
            name: claims.tenant_name,
            tier: claims.tier,
            usage: tenant_info.usage,
            permissions: claims.permissions,
        })
    }

    /// Create an API key for a tenant
    #[instrument(level = "debug", skip(self))]
    pub async fn create_api_key(
        &self,
        tenant_id: Uuid,
        name: String,
        permissions: Vec<Permission>,
    ) -> Result<(String, ApiKey)> {
        if !self.config.enable_api_keys {
            return Err(Error::auth("API keys are disabled"));
        }

        // Generate a secure random key
        let key_bytes = (0..32).map(|_| fastrand::u8(..)).collect::<Vec<_>>();
        use base64::Engine as _;
        let key_string = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key_bytes);

        let api_key = ApiKey {
            id: Uuid::new_v4(),
            name,
            tenant_id,
            permissions,
            active: true,
            created_at: chrono::Utc::now(),
            last_used: None,
        };

        // Store the API key
        {
            let mut api_keys = self.api_keys.write().await;
            api_keys.insert(key_string.clone(), api_key.clone());
        }

        info!("Created API key {} for tenant {}", api_key.id, tenant_id);
        Ok((key_string, api_key))
    }

    /// Validate an API key and extract tenant context
    #[instrument(level = "debug", skip(self, key))]
    pub async fn validate_api_key(&self, key: &str) -> Result<TenantContext> {
        if !self.config.enable_api_keys {
            return Err(Error::auth("API keys are disabled"));
        }

        let api_key = {
            let mut api_keys = self.api_keys.write().await;
            let api_key = api_keys.get_mut(key)
                .ok_or_else(|| Error::auth("Invalid API key"))?;

            if !api_key.active {
                return Err(Error::auth("API key is disabled"));
            }

            // Update last used timestamp
            api_key.last_used = Some(chrono::Utc::now());
            api_key.clone()
        };

        // Get tenant information
        let tenant_info = self.get_tenant_info(api_key.tenant_id).await?;
        if !tenant_info.active {
            return Err(Error::auth("Tenant account is disabled"));
        }

        // Check rate limits
        self.check_rate_limit(api_key.tenant_id).await?;

        Ok(TenantContext {
            tenant_id: api_key.tenant_id,
            name: tenant_info.name,
            tier: tenant_info.tier,
            usage: tenant_info.usage,
            permissions: api_key.permissions,
        })
    }

    /// Revoke an API key
    #[instrument(level = "debug", skip(self))]
    pub async fn revoke_api_key(&self, key: &str) -> Result<()> {
        let mut api_keys = self.api_keys.write().await;
        if let Some(api_key) = api_keys.get_mut(key) {
            api_key.active = false;
            info!("Revoked API key {} for tenant {}", api_key.id, api_key.tenant_id);
            Ok(())
        } else {
            Err(Error::auth("API key not found"))
        }
    }

    /// Register a new tenant
    #[instrument(level = "info", skip(self))]
    pub async fn register_tenant(
        &self,
        name: String,
        tier: SubscriptionTier,
    ) -> Result<TenantContext> {
        let tenant_id = Uuid::new_v4();
        
        let tenant_info = TenantInfo {
            id: tenant_id,
            name: name.clone(),
            tier: tier.clone(),
            usage: TenantUsage {
                vectors_stored: 0,
                queries_this_hour: 0,
                bandwidth_this_month: 0,
                storage_used_bytes: 0,
            },
            active: true,
        };

        // Store tenant information
        {
            let mut tenants = self.tenants.write().await;
            tenants.insert(tenant_id, tenant_info.clone());
        }

        // Default permissions based on tier
        let permissions = match tier {
            SubscriptionTier::Free => vec![Permission::Read, Permission::Write],
            SubscriptionTier::Platform => vec![
                Permission::Read,
                Permission::Write,
                Permission::Delete,
                Permission::ManageCollections,
            ],
            SubscriptionTier::Enterprise => vec![
                Permission::Read,
                Permission::Write,
                Permission::Delete,
                Permission::ManageCollections,
                Permission::Analytics,
                Permission::Admin,
            ],
        };

        info!("Registered new tenant: {} ({:?})", name, tier);

        Ok(TenantContext {
            tenant_id,
            name,
            tier,
            usage: tenant_info.usage,
            permissions,
        })
    }

    /// Get tenant information
    async fn get_tenant_info(&self, tenant_id: Uuid) -> Result<TenantInfo> {
        let tenants = self.tenants.read().await;
        tenants.get(&tenant_id)
            .cloned()
            .ok_or_else(|| Error::TenantNotFound { id: tenant_id.to_string() })
    }

    /// Check rate limits for a tenant
    async fn check_rate_limit(&self, tenant_id: Uuid) -> Result<()> {
        let mut rate_limiter = self.rate_limiter.write().await;
        let now = chrono::Utc::now();
        
        // Clean old requests outside the window
        let window_start = now - chrono::Duration::minutes(rate_limiter.window_minutes);
        let max_requests = rate_limiter.max_requests;
        
        let requests = rate_limiter.requests.entry(tenant_id).or_insert_with(Vec::new);
        requests.retain(|&req_time| req_time > window_start);
        
        // Check if we're over the limit
        if requests.len() >= max_requests {
            warn!("Rate limit exceeded for tenant {}", tenant_id);
            return Err(Error::RateLimitExceeded {
                tenant_id: tenant_id.to_string(),
            });
        }
        
        // Add current request
        requests.push(now);
        
        Ok(())
    }

    /// Update tenant usage statistics
    #[instrument(level = "debug", skip(self))]
    pub async fn update_tenant_usage(&self, tenant_id: Uuid, usage: TenantUsage) -> Result<()> {
        let mut tenants = self.tenants.write().await;
        if let Some(tenant) = tenants.get_mut(&tenant_id) {
            tenant.usage = usage;
            debug!("Updated usage for tenant {}", tenant_id);
            Ok(())
        } else {
            Err(Error::TenantNotFound { id: tenant_id.to_string() })
        }
    }

    /// List API keys for a tenant
    #[instrument(level = "debug", skip(self))]
    pub async fn list_api_keys(&self, tenant_id: Uuid) -> Result<Vec<ApiKey>> {
        let api_keys = self.api_keys.read().await;
        let keys = api_keys
            .values()
            .filter(|key| key.tenant_id == tenant_id)
            .cloned()
            .collect();
        
        Ok(keys)
    }

    /// Health check for the auth manager
    pub async fn health_check(&self) -> Result<()> {
        // Try to generate a test token
        let test_tenant_id = Uuid::new_v4();
        let _token = self.generate_token(test_tenant_id, vec![Permission::Read]).await;
        
        debug!("Auth manager health check passed");
        Ok(())
    }
}

/// Middleware for extracting authentication from HTTP requests
pub async fn extract_auth_from_request(
    auth_manager: &AuthManager,
    headers: &axum::http::HeaderMap,
) -> Result<TenantContext> {
    // Try Authorization header first (JWT)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return auth_manager.validate_token(token).await;
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = headers.get("x-api-key") {
        if let Ok(api_key) = api_key_header.to_str() {
            return auth_manager.validate_api_key(api_key).await;
        }
    }

    Err(Error::auth("No valid authentication provided"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, RateLimitConfig};
    use std::time::Duration;

    fn create_test_config() -> AuthConfig {
        AuthConfig {
            jwt_secret: "test-secret-key".to_string(),
            jwt_expiration: Duration::from_secs(3600),
            enable_api_keys: true,
            rate_limit: RateLimitConfig {
                requests_per_minute: 1000,
                burst_capacity: 100,
            },
        }
    }

    #[tokio::test]
    async fn test_auth_manager_creation() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config);
        assert!(auth_manager.is_ok());
    }

    #[tokio::test]
    async fn test_tenant_registration() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();
        
        let tenant = auth_manager
            .register_tenant("Test Tenant".to_string(), SubscriptionTier::Platform)
            .await;
        
        assert!(tenant.is_ok());
        let tenant = tenant.unwrap();
        assert_eq!(tenant.name, "Test Tenant");
        assert_eq!(tenant.tier, SubscriptionTier::Platform);
    }

    #[tokio::test]
    async fn test_token_generation_and_validation() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();
        
        // Register a tenant first
        let tenant = auth_manager
            .register_tenant("Test Tenant".to_string(), SubscriptionTier::Platform)
            .await
            .unwrap();
        
        // Generate token
        let token = auth_manager
            .generate_token(tenant.tenant_id, vec![Permission::Read, Permission::Write])
            .await
            .unwrap();
        
        // Validate token
        let validated_tenant = auth_manager.validate_token(&token).await.unwrap();
        assert_eq!(validated_tenant.tenant_id, tenant.tenant_id);
        assert_eq!(validated_tenant.name, tenant.name);
    }

    #[tokio::test]
    async fn test_api_key_creation() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();
        
        // Register a tenant first
        let tenant = auth_manager
            .register_tenant("Test Tenant".to_string(), SubscriptionTier::Platform)
            .await
            .unwrap();
        
        // Create API key
        let (key_string, api_key) = auth_manager
            .create_api_key(
                tenant.tenant_id,
                "Test Key".to_string(),
                vec![Permission::Read],
            )
            .await
            .unwrap();
        
        assert!(!key_string.is_empty());
        assert_eq!(api_key.name, "Test Key");
        assert_eq!(api_key.tenant_id, tenant.tenant_id);
        
        // Validate API key
        let validated_tenant = auth_manager.validate_api_key(&key_string).await.unwrap();
        assert_eq!(validated_tenant.tenant_id, tenant.tenant_id);
    }

    #[test]
    fn test_permissions() {
        let permissions = vec![Permission::Read, Permission::Write];
        assert!(permissions.contains(&Permission::Read));
        assert!(!permissions.contains(&Permission::Admin));
    }
}