use std::{
  net::{IpAddr, Ipv4Addr},
  sync::Arc,
};

use cache::auth::AuthCache;
use database::{auth_manager::AuthManager, event_manager::DBEventManager, initialize_client};
use dns::DnsResolver;
use proxy::Proxy;
use tokio::sync::{Barrier, Semaphore};
use utils::{
  config::{load_config, parse_args, ProxyConfig},
  socket::make_subnet_vec,
};

#[macro_use]
extern crate log;

mod cache;
mod database;
mod dns;
mod proxy;
mod utils;

#[tokio::main]
async fn main() {
  let config_path = parse_args().expect("Missing required option c (config)");
  let config = load_config(config_path).expect("Error parsing config.toml file");
  log4rs::init_file(config.log4rs.location, Default::default()).expect("Failed to initialize log4rs");

  let client = initialize_client(config.mongodb.uri).await.expect("Failed to initialize MongoDB Client");

  let auth_cache = AuthCache::new(config.cache.auth);
  let auth_manager = AuthManager::new(&client, &config.mongodb.database, auth_cache.clone()).await;
  let event_manager = DBEventManager::new(&client, &config.mongodb.database, auth_cache.clone()).await;

  let dns_resolver = DnsResolver::new(config.cache.dns);

  tokio::join!(event_manager.monitor(), handle_preload(config.proxy, auth_manager, dns_resolver),);
}

async fn handle_preload(config: ProxyConfig, auth_manager: AuthManager, dns_resolver: DnsResolver) {
  let preload = config.clone().preload;

  let semaphore = Arc::new(Semaphore::new(preload.tasks)); // Limit sockets binding concurrency to 100
  let addrs = preload.addrs.unwrap_or(Vec::new());
  let subnets_addrs: Vec<Ipv4Addr> = preload.subnets.unwrap_or(Vec::new()).iter().flat_map(|&sub| make_subnet_vec(sub)).collect();

  let barrier = Arc::new(Barrier::new(addrs.len() + subnets_addrs.len()));

  debug!(
    "Preload primitives loaded. Barrier Size = {}, Semaphore Permits = {}",
    addrs.len() + subnets_addrs.len(),
    preload.tasks
  );

  for &addr in addrs.iter() {
    let semaphore = semaphore.clone();
    let barrier = barrier.clone();
    let auth_manager = auth_manager.clone();
    let dns_resolver = dns_resolver.clone();
    let config = config.clone();
    tokio::spawn(async move {
      let _permit = semaphore.acquire().await.expect("failed to acquire semaphore permit on preload");
      Proxy::new(barrier, config, IpAddr::V4(addr), auth_manager, dns_resolver).listen().await;
    });
  }

  for &addr in subnets_addrs.iter() {
    let semaphore = semaphore.clone();
    let barrier = barrier.clone();
    let auth_manager = auth_manager.clone();
    let dns_resolver = dns_resolver.clone();
    let config = config.clone();
    tokio::spawn(async move {
      let _permit = semaphore.acquire().await.expect("failed to acquire semaphore permit on preload");
      Proxy::new(barrier, config, IpAddr::V4(addr), auth_manager, dns_resolver).listen().await;
    });
  }
}
