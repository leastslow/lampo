use std::{
  net::{IpAddr, SocketAddr},
  sync::Arc,
};

use crate::{
  database::auth_manager::AuthManager,
  dns::DnsResolver,
  proxy::{http::HttpProxy, socks5::Socks5Proxy},
  utils::config::ProxyConfig,
};
use tokio::sync::{Barrier, Semaphore};

mod http;
mod socks5;

#[derive(Clone)]
pub struct Proxy {
  barrier: Arc<Barrier>,
  semaphore: Arc<Semaphore>,
  listen_addr: IpAddr,
  config: ProxyConfig,
  auth_manager: AuthManager,
  dns_resolver: DnsResolver,
}

impl Proxy {
  pub fn new(
    barrier: Arc<Barrier>,
    semaphore: Arc<Semaphore>,
    config: ProxyConfig,
    listen_addr: IpAddr,
    auth_manager: AuthManager,
    dns_resolver: DnsResolver,
  ) -> Self {
    Self {
      barrier,
      semaphore,
      listen_addr,
      config,
      auth_manager,
      dns_resolver,
    }
  }
  pub async fn listen(&self) {
    let http_proxy = HttpProxy::new(
      SocketAddr::from((self.listen_addr, self.config.ports.http)),
      self.config.backlog,
      self.auth_manager.clone(),
      self.dns_resolver.clone(),
      self.barrier.clone(),
      self.semaphore.clone(),
    );
    let socks5_proxy = Socks5Proxy::new(
      SocketAddr::from((self.listen_addr, self.config.ports.socks)),
      self.config.backlog,
      self.auth_manager.clone(),
      self.dns_resolver.clone(),
      self.config.udp.clone(),
      self.barrier.clone(),
      self.semaphore.clone(),
    );

    info!(
      "Launched instance on {} (HTTP: {}, SOCKS5: {})",
      self.listen_addr, self.config.ports.http, self.config.ports.socks
    );

    tokio::join!(http_proxy.listen(), socks5_proxy.listen(),);
  }
}
