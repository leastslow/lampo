use std::net::SocketAddr;

use socks5_proto::{Address, Command, Reply, Request, Response};
use tokio::{net::TcpStream, time::timeout};

use crate::{dns::DnsResolver, utils::config::ProxyConfigUdpSocket};

use super::{
  handler::Socks5Handler,
  utils::{error::Socks5HandlerError, socket_state::SocketState},
};

pub mod associate;
pub mod bind;
pub mod connect;

pub struct CommandHandler<'a> {
  stream: &'a mut TcpStream,
  request: Request,
  bind_addr: SocketAddr,
  dns_resolver: DnsResolver,
  socket_state: SocketState,
  udp_config: ProxyConfigUdpSocket,
}

impl<'a> CommandHandler<'a> {
  pub fn new(
    stream: &'a mut TcpStream,
    request: Request,
    bind_addr: SocketAddr,
    dns_resolver: DnsResolver,
    socket_state: SocketState,
    udp_config: ProxyConfigUdpSocket,
  ) -> CommandHandler<'a> {
    CommandHandler {
      stream,
      request,
      bind_addr,
      dns_resolver,
      socket_state,
      udp_config,
    }
  }

  pub async fn execute(&mut self) -> Result<(), Socks5HandlerError> {
    match self.request.command {
      Command::Connect => self.connect().await,
      Command::Associate => self.associate().await,
      Command::Bind => self.bind().await,
    }
  }

  async fn reply(&mut self, reply: Reply, address: Address) -> Result<(), Socks5HandlerError> {
    match timeout(Socks5Handler::MAX_TIMEOUT, Response::new(reply, address).write_to(self.stream)).await {
      Ok(req) => match req {
        Ok(req) => Ok(req),
        Err(e) => Err(Socks5HandlerError::StreamReadError(e.into())),
      },
      Err(_) => Err(Socks5HandlerError::StreamReadTimeout),
    }
  }

  async fn resolve_address(&mut self) -> Result<SocketAddr, Socks5HandlerError> {
    match self.request.address.clone() {
      Address::DomainAddress(domain, port) => {
        let domain = String::from_utf8_lossy(&domain);
        match self.dns_resolver.resolve(&domain, port).await {
          Ok(addr) => Ok(addr),
          Err(e) => Err(Socks5HandlerError::DnsResolutionError(domain.to_string(), port, e)),
        }
      }
      Address::SocketAddress(addr) => Ok(addr),
    }
  }
}
