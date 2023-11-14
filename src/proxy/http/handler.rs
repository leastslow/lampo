use std::net::SocketAddr;

use bytes::BytesMut;
use tokio::{
  io::AsyncWriteExt,
  net::TcpStream,
  time::{timeout, Duration},
};

use crate::{database::auth_manager::AuthManager, dns::DnsResolver, utils::socket::make_outbound};

use super::{parser::HttpParser, utils::constants::HttpResponse};

pub struct HttpHandler<'a> {
  stream: &'a mut TcpStream,
  listen_addr: SocketAddr,
  auth_manager: AuthManager,
  dns_resolver: DnsResolver,
}

impl<'a> HttpHandler<'a> {
  const MAX_TIMEOUT: Duration = Duration::from_secs(10);

  pub fn new(stream: &'a mut TcpStream, listen_addr: SocketAddr, auth_manager: AuthManager, dns_resolver: DnsResolver) -> HttpHandler<'a> {
    HttpHandler {
      stream,
      listen_addr,
      auth_manager,
      dns_resolver,
    }
  }

  pub async fn execute(&mut self) {
    let mut req_data = match HttpParser::new(self.stream).read().await {
      Ok(r) => r,
      Err(e) => return warn!("{}", e),
    };

    if !self.handle_authentication(&req_data.authentication).await {
      return self.reply(HttpResponse::ProxyAuthenticationRequired).await;
    }

    // let target_host = format!("{}:{}", req_data.host.0, req_data.host.1);

    let target_addr = match self.dns_resolver.resolve(&req_data.host.0, req_data.host.1).await {
      Ok(h) => h,
      Err(e) => {
        warn!("failed to resolve host {:?}. Err = {}", req_data.host, e);
        return self.reply(HttpResponse::InternalServerError).await;
      }
    };

    // let dns_start_time = Instant::now();

    let bind_addr = SocketAddr::from((self.listen_addr.ip(), 0));

    let mut outbound = match make_outbound(bind_addr, target_addr).await {
      Ok(outbound) => {
        // self.dns_resolver.record_latency(target_addr, dns_start_time.elapsed()).await;
        outbound
      }
      Err(e) => {
        warn!("failed to create outbound TcpStream ({:?}). Err = {}", req_data.host, e);
        return self.reply(HttpResponse::InternalServerError).await;
      }
    };

    if req_data.method == "CONNECT" {
      self.reply(HttpResponse::OkEstablished).await;
    } else {
      let request = req_data.into_request();
      self.write(&mut outbound, &req_data.host, &request).await;
    }

    if let Err(_) = tokio::time::timeout(HttpHandler::MAX_TIMEOUT, tokio::io::copy_bidirectional(self.stream, &mut outbound)).await {
      warn!("timeout on copy_bidirectional ({:?})", &req_data.host);
      return self.reply(HttpResponse::GatewayTimeout).await;
    }
  }

  async fn handle_authentication(&self, auth_data: &Option<(String, String)>) -> bool {
    if let Some(cv) = self.auth_manager.get_or_fetch_and_insert(&self.listen_addr).await {
      if cv.use_credentials {
        // If credentials are required, check them if provided, otherwise return false.
        return auth_data
          .as_ref()
          .map_or(false, |(username, password)| self.auth_manager.check_credentials(cv, username, password));
      } else if let Ok(client_addr) = self.stream.peer_addr() {
        if auth_data.is_none() {
          // If no credentials required and no credentials are in the request data, check the whitelist.
          return self.auth_manager.check_whitelist(cv, client_addr);
        }
      }
      // Fall-through case: credentials are not used and client address is not retrieved.
      false
    } else {
      // Case where 'get_or_fetch_and_insert' yields None.
      false
    }
  }

  async fn reply(&mut self, response: HttpResponse) {
    match tokio::time::timeout(HttpHandler::MAX_TIMEOUT, self.stream.write_all(response.as_bytes())).await {
      Ok(r) => {
        if let Err(e) = r {
          return warn!("stream write error. Err = {:?}", e);
        }
      }
      Err(_) => return warn!("stream write timeout"),
    }
  }

  async fn write(&mut self, outbound: &mut TcpStream, outbound_host: &(String, u16), request: &BytesMut) {
    match timeout(HttpHandler::MAX_TIMEOUT, outbound.write_all(&request)).await {
      Ok(result) => {
        if let Err(e) = result {
          warn!("failed to send http request to target server. Err = {:?}", e);
          return self.reply(HttpResponse::BadGateway).await;
        }
      }
      Err(_) => {
        warn!("timeout while writing to outbound ({:?})", outbound_host);
        return self.reply(HttpResponse::GatewayTimeout).await;
      }
    }
  }
}
