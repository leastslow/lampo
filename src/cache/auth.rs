use std::sync::Arc;

use chrono::{DateTime, Utc};
use moka::future::Cache;

use crate::{database::models::UserOrder, utils::config::AuthCacheConfig};

pub struct AuthCacheValue {
  pub use_credentials: bool,
  pub username: String,
  pub password: String,
  pub whitelist: Vec<String>,
  pub expiration: DateTime<Utc>,
}

#[derive(Clone)]
pub struct AuthCache {
  pub inner: Cache<String, Arc<AuthCacheValue>>,
}

impl AuthCache {
  pub fn new(config: AuthCacheConfig) -> Self {
    Self {
      inner: Cache::builder().max_capacity(config.max_size).time_to_live(config.time_to_live).build(),
    }
  }

  pub fn get(&self, key: &str) -> Option<Arc<AuthCacheValue>> {
    self.inner.get(key)
  }

  pub async fn insert(&self, key: &str, doc: UserOrder, credentials_pos: usize) -> Arc<AuthCacheValue> {
    debug!("insert auth - {}\n{:?}\nPosition: {}", key, doc, credentials_pos);
    let value = Arc::new(AuthCacheValue {
      username: doc.proxy.username[credentials_pos].clone(),
      password: doc.proxy.password[credentials_pos].clone(),
      use_credentials: doc.proxy.use_credentials,
      whitelist: doc.proxy.whitelist,
      expiration: DateTime::from(doc.expiration.to_system_time()),
    });
    self.inner.insert(String::from(key), value.clone()).await;
    return value;
  }

  pub async fn delete(&self, key: &str) {
    self.inner.invalidate(key).await
  }
}
