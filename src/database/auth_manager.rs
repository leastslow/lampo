use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};
use std::{net::SocketAddr, sync::Arc};
use tokio_stream::StreamExt;

use crate::cache::auth::{AuthCache, AuthCacheValue};

use super::models::{Stock, UserOrder};

#[derive(Clone)]
pub struct AuthManager {
  cache: AuthCache,
  orders: mongodb::Collection<UserOrder>,
  stock: mongodb::Collection<Stock>,
}

impl AuthManager {
  pub async fn new(client: &mongodb::Client, database: &str, cache: AuthCache) -> Self {
    let db = client.database(database);
    Self {
      cache,
      orders: db.collection("orders"),
      stock: db.collection("stock"),
    }
  }

  async fn get_related_proxies(&self, order_id: ObjectId) -> Vec<String> {
    let mut addrs: Vec<String> = Vec::new();
    let filter = doc! {
      "used_in_order": order_id,
    };
    if let Ok(mut docs) = self.stock.find(filter, None).await {
      while let Some(result) = docs.next().await {
        match result {
          Ok(entry) => addrs.push(entry.address),
          _ => (),
        }
      }
    }
    return addrs;
  }

  pub async fn get_or_fetch_and_insert(&self, proxy_addr: &SocketAddr) -> Option<Arc<AuthCacheValue>> {
    debug!("get_or_fetch_and_insert {} from cache", proxy_addr);
    if let Some(cv) = self.cache.get(proxy_addr.ip().to_string().as_str()) {
      return Some(cv);
    }

    let stock = match self
      .stock
      .find_one(
        doc! {
          "address": proxy_addr.ip().to_string(),
        },
        None,
      )
      .await
    {
      Ok(s) => s,
      Err(_) => {
        return None;
      }
    };

    if let Some(stock) = stock {
      let order = match self
        .orders
        .find_one(
          doc! {
            "_id": stock.used_in_order,
          },
          None,
        )
        .await
      {
        Ok(o) => o,
        Err(_) => return None,
      };
      if let Some(order) = order {
        let multi_credentials = order.proxy.username.len() != 1 && order.proxy.password.len() != 1;
        let proxy_list = self.get_related_proxies(order._id).await;
        let mut cv: Option<Arc<AuthCacheValue>> = None;
        for (i, proxy) in proxy_list.iter().enumerate() {
          let mut pos = 0;
          if multi_credentials {
            pos = i;
          }
          // Save auth cache result to return it at the end of the function.
          if &proxy_addr.ip().to_string() == proxy {
            cv = Some(self.cache.insert(proxy, order.clone(), pos).await)
          } else {
            self.cache.insert(proxy, order.clone(), pos).await;
          }
        }
        return cv;
      }
    }

    return None;
  }

  pub fn check_credentials(&self, cache_value: Arc<AuthCacheValue>, username: &str, password: &str) -> bool {
    return cache_value.use_credentials && &cache_value.username == username && &cache_value.password == password && cache_value.expiration > Utc::now();
  }

  pub fn check_whitelist(&self, cache_value: Arc<AuthCacheValue>, client_addr: SocketAddr) -> bool {
    return !cache_value.use_credentials && cache_value.whitelist.contains(&client_addr.ip().to_string()) && cache_value.expiration > Utc::now();
  }
}
