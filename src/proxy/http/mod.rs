use std::{net::SocketAddr, sync::Arc};

use tokio::sync::Barrier;

use crate::{database::auth_manager::AuthManager, dns::DnsResolver, utils::socket::make_listener};

use self::handler::HttpHandler;

mod handler;
mod parser;
mod utils;

#[derive(Clone)]
pub struct HttpProxy {
  listen_addr: SocketAddr,
  backlog: u32,
  auth_manager: AuthManager,
  dns_resolver: DnsResolver,
  barrier: Arc<Barrier>,
}

impl HttpProxy {
  pub fn new(addr: SocketAddr, backlog: u32, auth_manager: AuthManager, dns_resolver: DnsResolver, barrier: Arc<Barrier>) -> Self {
    Self {
      listen_addr: addr,
      backlog,
      auth_manager,
      dns_resolver,
      barrier,
    }
  }

  pub async fn listen(&self) {
    let listener = match make_listener(self.listen_addr, self.backlog).await {
      Ok(l) => l,
      Err(e) => {
        return error!("Failed to initialize HttpProxy listener. Err = {:?}", e);
      }
    };

    self.barrier.wait().await;

    debug!("HttpProxy {} passed barrier, starting listener", self.listen_addr);

    while let Ok((mut stream, _)) = listener.accept().await {
      let listen_addr = self.listen_addr;
      let auth_manager = self.auth_manager.clone();
      let dns_resolver = self.dns_resolver.clone();

      tokio::spawn(async move {
        HttpHandler::new(&mut stream, listen_addr, auth_manager, dns_resolver).execute().await;
      });
    }
  }
}
