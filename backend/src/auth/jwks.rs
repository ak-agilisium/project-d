//! Cached JSON Web Key Set used to verify Cognito-issued JWT signatures.
//!
//! Cognito publishes its public signing keys at
//! `https://cognito-idp.<region>.amazonaws.com/<pool-id>/.well-known/jwks.json`.
//! Keys rotate rarely, so we cache them and only re-fetch on a `kid` miss.

use std::collections::HashMap;

use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<JwkEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct JwkEntry {
    kid: String,
    n: String,
    e: String,
}

/// Thread-safe, lazily-populated cache of decoding keys keyed by `kid`.
pub struct JwksCache {
    http: reqwest::Client,
    url: String,
    keys: RwLock<HashMap<String, JwkEntry>>,
}

impl JwksCache {
    pub fn new(jwks_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            url: jwks_url,
            keys: RwLock::new(HashMap::new()),
        }
    }

    /// Return the [`DecodingKey`] for the given `kid`, fetching the JWKS from
    /// Cognito if the key is not already cached.
    pub async fn decoding_key(&self, kid: &str) -> AppResult<DecodingKey> {
        if let Some(entry) = self.keys.read().await.get(kid).cloned() {
            return to_decoding_key(&entry);
        }

        // Cache miss — refresh once (handles key rotation) and look again.
        self.refresh().await?;

        let guard = self.keys.read().await;
        let entry = guard
            .get(kid)
            .ok_or_else(|| AppError::Unauthorized("unknown token signing key".into()))?;
        to_decoding_key(entry)
    }

    async fn refresh(&self) -> AppResult<()> {
        let jwks: Jwks = self
            .http
            .get(&self.url)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to fetch JWKS: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("JWKS endpoint returned error: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid JWKS payload: {e}")))?;

        let mut map = self.keys.write().await;
        map.clear();
        for key in jwks.keys {
            map.insert(key.kid.clone(), key);
        }
        tracing::debug!(count = map.len(), "refreshed Cognito JWKS cache");
        Ok(())
    }
}

fn to_decoding_key(entry: &JwkEntry) -> AppResult<DecodingKey> {
    DecodingKey::from_rsa_components(&entry.n, &entry.e)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid RSA JWK: {e}")))
}
