use std::{net::SocketAddr, time::Duration};

use socks5_proto::{
  handshake::password::{Request as PasswordRequest, Response as PasswordResponse},
  handshake::{Method as HandshakeMethod, Request as HandshakeRequest, Response as HandshakeResponse},
  Request,
};

use tokio::{net::TcpStream, time::timeout};

use crate::{database::auth_manager::AuthManager, dns::DnsResolver, utils::config::ProxyConfigUdpSocket};

use super::{
  commands::CommandHandler,
  utils::{error::Socks5HandlerError, socket_state::SocketState},
};

pub struct Socks5Handler<'a> {
  stream: &'a mut TcpStream,
  listen_addr: SocketAddr,
  auth_manager: AuthManager,
  dns_resolver: DnsResolver,
  socket_state: SocketState,
  udp_config: ProxyConfigUdpSocket,
}

impl<'a> Socks5Handler<'a> {
  pub const MAX_TIMEOUT: Duration = Duration::from_secs(10);

  pub fn new(
    stream: &'a mut TcpStream,
    listen_addr: SocketAddr,
    dns_resolver: DnsResolver,
    auth_manager: AuthManager,
    socket_state: SocketState,
    udp_config: ProxyConfigUdpSocket,
  ) -> Socks5Handler<'a> {
    Socks5Handler {
      stream,
      listen_addr,
      dns_resolver,
      auth_manager,
      socket_state,
      udp_config,
    }
  }

  pub async fn execute(&mut self) {
    if let Err(e) = self.check_methods().await {
      return warn!("{}", e);
    }

    if let Err(e) = self.handle_authentication().await {
      match e {
        Socks5HandlerError::AuthenticationError => return debug!("{}", e),
        _ => return warn!("{}", e),
      }
    }

    let request = match self.request_read().await {
      Ok(r) => r,
      Err(e) => return warn!("{}", e),
    };

    let bind_addr = SocketAddr::new(self.listen_addr.ip(), 0);

    if let Err(e) = CommandHandler::new(
      &mut self.stream,
      request,
      bind_addr,
      self.dns_resolver.clone(),
      self.socket_state.clone(),
      self.udp_config.clone(),
    )
    .execute()
    .await
    {
      return warn!("{}", e);
    }
  }

  async fn check_methods(&mut self) -> Result<(), Socks5HandlerError> {
    let req = self.handshake_read().await?;

    if !req.methods.contains(&HandshakeMethod::NONE) && !req.methods.contains(&HandshakeMethod::PASSWORD) {
      return self.handshake_reply(HandshakeMethod::UNACCEPTABLE).await;
    }

    Ok(())
  }

  async fn handle_authentication(&mut self) -> Result<(), Socks5HandlerError> {
    let cache_value = match self.auth_manager.get_or_fetch_and_insert(&self.listen_addr).await {
      Some(cv) => cv,
      None => {
        self.handshake_reply(HandshakeMethod::UNACCEPTABLE).await?;
        return Err(Socks5HandlerError::AuthenticationError);
      }
    };

    if cache_value.use_credentials {
      self.handshake_reply(HandshakeMethod::PASSWORD).await?;
      let req = self.handshake_password_read().await?;
      let username = std::str::from_utf8(&req.username);
      let password = std::str::from_utf8(&req.password);

      if username.is_err() || password.is_err() {
        self.handshake_password_reply(false).await?;
        return Err(Socks5HandlerError::AuthenticationError);
      }

      if !self.auth_manager.check_credentials(cache_value, username.unwrap(), password.unwrap()) {
        self.handshake_password_reply(false).await?;
        return Err(Socks5HandlerError::AuthenticationError);
      }

      self.handshake_password_reply(true).await?;
      return Ok(());
    }

    let client_addr = self.stream.peer_addr().map_err(Socks5HandlerError::PeerAddrError)?;

    if self.auth_manager.check_whitelist(cache_value, client_addr) {
      self.handshake_reply(HandshakeMethod::NONE).await?;
    } else {
      self.handshake_reply(HandshakeMethod::UNACCEPTABLE).await?;
      return Err(Socks5HandlerError::AuthenticationError);
    }

    Ok(())
  }

  async fn handshake_reply(&mut self, method: HandshakeMethod) -> Result<(), Socks5HandlerError> {
    match timeout(Socks5Handler::MAX_TIMEOUT, HandshakeResponse::new(method).write_to(self.stream)).await {
      Ok(result) => {
        if let Err(e) = result {
          Err(Socks5HandlerError::StreamWriteError(e))
        } else {
          Ok(())
        }
      }
      Err(_) => Err(Socks5HandlerError::StreamWriteTimeout),
    }
  }

  async fn handshake_read(&mut self) -> Result<HandshakeRequest, Socks5HandlerError> {
    match timeout(Socks5Handler::MAX_TIMEOUT, HandshakeRequest::read_from(self.stream)).await {
      Ok(req) => match req {
        Ok(req) => Ok(req),
        Err(e) => Err(Socks5HandlerError::StreamReadError(e.into())),
      },
      Err(_) => Err(Socks5HandlerError::StreamReadTimeout),
    }
  }

  async fn handshake_password_read(&mut self) -> Result<PasswordRequest, Socks5HandlerError> {
    match timeout(Socks5Handler::MAX_TIMEOUT, PasswordRequest::read_from(self.stream)).await {
      Ok(req) => match req {
        Ok(req) => Ok(req),
        Err(e) => Err(Socks5HandlerError::StreamReadError(e.into())),
      },
      Err(_) => Err(Socks5HandlerError::StreamReadTimeout),
    }
  }

  async fn handshake_password_reply(&mut self, authorized: bool) -> Result<(), Socks5HandlerError> {
    match timeout(Socks5Handler::MAX_TIMEOUT, PasswordResponse::new(authorized).write_to(self.stream)).await {
      Ok(result) => {
        if let Err(e) = result {
          Err(Socks5HandlerError::StreamWriteError(e))
        } else {
          Ok(())
        }
      }
      Err(_) => Err(Socks5HandlerError::StreamWriteTimeout),
    }
  }

  async fn request_read(&mut self) -> Result<Request, Socks5HandlerError> {
    match timeout(Socks5Handler::MAX_TIMEOUT, Request::read_from(self.stream)).await {
      Ok(req) => match req {
        Ok(req) => Ok(req),
        Err(e) => Err(Socks5HandlerError::StreamReadError(e.into())),
      },
      Err(_) => Err(Socks5HandlerError::StreamReadTimeout),
    }
  }
}
