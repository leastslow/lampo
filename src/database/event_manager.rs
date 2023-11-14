use mongodb::{
  bson::doc,
  change_stream::event::OperationType,
  options::{ChangeStreamOptions, FullDocumentBeforeChangeType, FullDocumentType},
};
use tokio_stream::StreamExt;

use crate::cache::auth::AuthCache;

use super::models::{Stock, UserOrder};

pub struct DBEventManager {
  cache: AuthCache,
  orders: mongodb::Collection<UserOrder>,
  stock: mongodb::Collection<Stock>,
}

impl DBEventManager {
  pub async fn new(client: &mongodb::Client, database: &str, cache: AuthCache) -> Self {
    let db = client.database(database);
    Self {
      cache,
      orders: db.collection("orders"),
      stock: db.collection("stock"),
    }
  }

  pub async fn monitor(&self) {
    info!("Started MongoDB EventManager");
    let cs_opts = ChangeStreamOptions::builder()
      .full_document(Some(FullDocumentType::UpdateLookup))
      .full_document_before_change(Some(FullDocumentBeforeChangeType::WhenAvailable))
      .build();

    let mut change_stream = self.orders.watch(None, cs_opts).await.unwrap();
    while let Some(event) = change_stream.next().await.transpose().unwrap() {
      match event.operation_type {
        OperationType::Update => self.handle_update(event.full_document).await,
        OperationType::Insert => self.handle_insert(event.full_document).await,
        OperationType::Delete => self.handle_delete(event.full_document_before_change).await,
        _ => (),
      }
    }
  }

  async fn handle_update(&self, doc: Option<UserOrder>) {
    if let Some(doc) = doc {
      debug!("handle_update called on document {}", doc._id);
      if doc.product_slug == "isp" {
        let addrs = self.get_order_proxies(&doc).await;
        // Check if order has multi-credentials.
        let multi_credentials = doc.proxy.username.len() != 1 && doc.proxy.password.len() != 1;
        for (i, addr) in addrs.iter().enumerate() {
          let mut pos = 0;
          if multi_credentials {
            pos = i;
          }
          self.cache.insert(addr.as_str(), doc.clone(), pos).await;
        }
      }
    } else {
      debug!("handle_update called on None");
    }
  }

  async fn handle_insert(&self, doc: Option<UserOrder>) {
    if let Some(doc) = doc {
      debug!("handle_insert called on document {}", doc._id);
      if doc.product_slug == "isp" {
        let addrs = self.get_order_proxies(&doc).await;
        // Check if order has multi-credentials.
        let multi_credentials = doc.proxy.username.len() != 1 && doc.proxy.password.len() != 1;
        for (i, addr) in addrs.iter().enumerate() {
          let mut pos = 0;
          if multi_credentials {
            pos = i;
          }
          self.cache.insert(addr.as_str(), doc.clone(), pos).await;
        }
      }
    } else {
      debug!("handle_insert called on None");
    }
  }

  async fn handle_delete(&self, doc: Option<UserOrder>) {
    if let Some(doc) = doc {
      debug!("handle_delete called on document {}", doc._id);
      if doc.product_slug == "isp" {
        let addrs = self.get_order_proxies(&doc).await;
        for addr in addrs.iter() {
          self.cache.delete(addr.as_str()).await;
        }
      }
    } else {
      debug!("handle_delete called on None");
    }
  }

  async fn get_order_proxies(&self, order: &UserOrder) -> Vec<String> {
    let mut addrs: Vec<String> = Vec::new();
    let filter = doc! {
      "used_in_order": order._id,
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
}
