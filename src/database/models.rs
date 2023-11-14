use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserOrder {
  pub _id: ObjectId,
  pub uuid: String,
  pub user_id: String,
  pub product_name: String,
  pub product_slug: String,
  pub proxy: UserOrderProxy,
  pub counters: UserOrderCounters,
  pub expiration: DateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserOrderCounters {
  pub requests_limit: u64,
  pub requests_usage: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserOrderProxy {
  pub count: u64,
  pub username: Vec<String>,
  pub password: Vec<String>,
  pub whitelist: Vec<String>,
  pub use_credentials: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stock {
  pub subnet: String,
  pub provider: String,
  pub public: bool,
  pub address: String,
  pub used_by: Option<String>,
  pub used_in_order: Option<ObjectId>,
  pub used_until: Option<DateTime>,
}
