use std::net::SocketAddr;

use socks5_proto::{Address, Reply};
use tokio::{io::AsyncReadExt, net::UdpSocket};

use crate::proxy::socks5::utils::{association_socket::AssociationSocketHelper, error::Socks5HandlerError};

use super::CommandHandler;

impl<'a> CommandHandler<'a> {
  pub async fn associate(&mut self) -> Result<(), Socks5HandlerError> {
    let client_addr = self.stream.peer_addr().map_err(Socks5HandlerError::PeerAddrError)?;

    if !self.socket_state.try_incr() {
      return Err(Socks5HandlerError::SocketLimitReached(self.bind_addr, client_addr));
    }

    let (socket, socket_addr) = match self.bind_udp_socket().await {
      Ok(r) => r,
      Err(e) => return Err(Socks5HandlerError::BindUdpSocketError(e)),
    };

    let mut socket_helper = AssociationSocketHelper::new(socket, self.dns_resolver.clone(), client_addr, self.udp_config.stale_ttl, 65535, 10000)
      .await
      .map_err(Socks5HandlerError::AssociationError)?;

    self.reply(Reply::Succeeded, Address::SocketAddress(socket_addr)).await?;

    tokio::select! {
      result = socket_helper.execute() => {
        if let Err(e) = result {
          socket_helper.close();
          self.socket_state.decr();
          return Err(Socks5HandlerError::AssociationError(e));
        }
      }
      _ = self.wait_close() => {
        debug!("socket closed by either party, freeing socket and releasing count.");
        socket_helper.close();
        self.socket_state.decr();
      }
    };

    Ok(())
  }

  async fn wait_close(&mut self) -> Result<(), tokio::io::Error> {
    loop {
      match self.stream.read(&mut [0]).await {
        Ok(0) => break Ok(()),
        Ok(_) => {}
        Err(err) => break Err(err.into()),
      }
    }
  }

  async fn bind_udp_socket(&self) -> Result<(UdpSocket, SocketAddr), tokio::io::Error> {
    let socket = UdpSocket::bind(self.bind_addr).await?;
    let socket_addr = socket.local_addr()?;
    Ok((socket, socket_addr))
  }
}
