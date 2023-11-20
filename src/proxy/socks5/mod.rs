mod commands;
mod handler;
mod utils;

use std::mem::drop;
use tokio::sync::{Barrier, Semaphore};

use crate::{
  database::auth_manager::AuthManager,
  dns::DnsResolver,
  proxy::socks5::handler::Socks5Handler,
  utils::{config::ProxyConfigUdpSocket, socket::make_listener},
};
use std::{net::SocketAddr, sync::Arc};

use self::utils::socket_state::SocketState;

// use self::utils::association_socket::UdpHelper;

#[derive(Clone)]
pub struct Socks5Proxy {
  listen_addr: SocketAddr,
  backlog: u32,
  auth_manager: AuthManager,
  dns_resolver: DnsResolver,
  udp_config: ProxyConfigUdpSocket,
  barrier: Arc<Barrier>,
  semaphore: Arc<Semaphore>,
}

impl Socks5Proxy {
  pub fn new(
    addr: SocketAddr,
    backlog: u32,
    auth_manager: AuthManager,
    dns_resolver: DnsResolver,
    udp_config: ProxyConfigUdpSocket,
    barrier: Arc<Barrier>,
    semaphore: Arc<Semaphore>,
  ) -> Self {
    Self {
      listen_addr: addr,
      backlog,
      auth_manager,
      dns_resolver,
      udp_config,
      barrier,
      semaphore,
    }
  }

  pub async fn listen(&self) {
    let _permit = self.semaphore.acquire().await.expect("failed to acquire semaphore permit on preload");

    let listener = match make_listener(self.listen_addr, self.backlog).await {
      Ok(l) => l,
      Err(e) => {
        return error!("Failed to initialize Socks5Proxy listener. Err = {:?}", e);
      }
    };

    drop(_permit);

    let socket_state = SocketState::new(self.udp_config.max_sockets);

    self.barrier.wait().await;

    debug!("Socks5Proxy {} passed barrier, starting listener", self.listen_addr);

    while let Ok((mut stream, _)) = listener.accept().await {
      let listen_addr = self.listen_addr.clone();
      let auth_manager = self.auth_manager.clone();
      let dns_resolver = self.dns_resolver.clone();
      let socket_state = socket_state.clone();
      let udp_config = self.udp_config.clone();

      tokio::spawn(async move {
        Socks5Handler::new(&mut stream, listen_addr, dns_resolver, auth_manager, socket_state, udp_config)
          .execute()
          .await;
      });
    }
  }
}
