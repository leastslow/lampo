mod latency_stat;

use std::{
  io::ErrorKind,
  net::{IpAddr, SocketAddr},
  sync::Arc,
  // time::Duration,
};

// use moka::future::Cache;
// use tokio::{io, sync::Mutex, time::timeout};
use trust_dns_resolver::{
  config::{ResolverConfig, ResolverOpts},
  // error::ResolveError,
  AsyncResolver,
  TokioAsyncResolver,
};

use crate::utils::config::DnsCacheConfig;

// use self::latency_stat::IpLatencyTracker;

#[derive(Clone)]
pub struct DnsResolver {
  resolver: Arc<TokioAsyncResolver>,
  // cache: Cache<String, Vec<SocketAddr>>,
  // latency_tracker: Arc<Mutex<IpLatencyTracker>>,
  // resolution_timeout: Duration,
  // resolution_lock: Arc<Mutex<()>>,
}

impl DnsResolver {
  pub fn new(config: DnsCacheConfig) -> Self {
    let mut resolver_opts = ResolverOpts::default();
    resolver_opts.cache_size = config.max_size;
    resolver_opts.timeout = config.resolution_timeout;
    resolver_opts.use_hosts_file = false;
    resolver_opts.authentic_data = true;
    Self {
      resolver: Arc::new(AsyncResolver::tokio(ResolverConfig::cloudflare(), resolver_opts)),
      // cache: Cache::builder().max_capacity(config.max_size).time_to_live(config.time_to_live).build(),
      // resolution_timeout: config.resolution_timeout,
      // latency_tracker: Arc::new(Mutex::new(IpLatencyTracker::new())),
      // resolution_lock: Arc::new(Mutex::new(())),
    }
  }

  pub async fn resolve(&self, host: &str, port: u16) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
    // Check if it's an address first
    if let Ok(addr) = host.parse::<IpAddr>() {
      return Ok(SocketAddr::new(addr, port));
    }
    if let Some(addr) = self.resolver.lookup_ip(host).await?.iter().next() {
      Ok(SocketAddr::new(addr, port))
    } else {
      Err(Box::new(tokio::io::Error::from(ErrorKind::AddrNotAvailable)))
    }
  }

  // pub async fn resolve(&mut self, target: &str) -> io::Result<SocketAddr> {
  //   match self.cache.get(target) {
  //     Some(ips) => {
  //       // Use the latency tracker to select an IP from the cached entries
  //       let tracker = self.latency_tracker.lock().await;
  //       match tracker.select_weighted_ip(&ips) {
  //         Some(ip) => Ok(ip),
  //         None => Err(io::Error::from(ErrorKind::AddrNotAvailable)),
  //       }
  //     }
  //     _ => {
  //       let _lock = self.resolution_lock.lock().await; // Locks here
  //       if let Some(ips) = self.cache.get(target) {
  //         // If another task has already resolved and cached it while we were waiting for the lock
  //         let tracker = self.latency_tracker.lock().await;
  //         match tracker.select_weighted_ip(&ips) {
  //           Some(ip) => Ok(ip),
  //           None => Err(io::Error::from(ErrorKind::AddrNotAvailable)),
  //         }
  //       } else {
  //         let ips = self.resolve_and_cache(target).await?;
  //         let tracker = self.latency_tracker.lock().await;
  //         match tracker.select_weighted_ip(&ips) {
  //           Some(ip) => Ok(ip),
  //           None => Err(io::Error::from(ErrorKind::AddrNotAvailable)),
  //         }
  //       }
  //     }
  //   }
  // }

  // async fn resolve_and_cache(&self, target: &str) -> io::Result<Vec<SocketAddr>> {
  //   info!("Resolving DNS {}", target);

  //   let resolved = timeout(self.resolution_timeout, tokio::net::lookup_host(target))
  //     .await
  //     .map_err(|_| io::Error::new(ErrorKind::TimedOut, "DNS resolution timed out"))?
  //     .map_err(|e| io::Error::new(ErrorKind::Other, e))?
  //     .collect::<Vec<_>>();

  //   if resolved.is_empty() {
  //     warn!("Cannot resolve DNS {}", target);
  //     return Err(io::Error::from(ErrorKind::AddrNotAvailable));
  //   }

  //   self.cache.insert(target.to_string(), resolved.clone()).await;

  //   Ok(resolved)
  // }

  // pub async fn record_latency(&self, ip: SocketAddr, latency: Duration) {
  // let mut tracker = self.latency_tracker.lock().await;
  // tracker.record_latency(ip, latency);
  // }
}
