use crate::dns::DnsResolver;
use bytes::{Buf, Bytes, BytesMut};
use moka::future::{Cache, CacheBuilder};
use socks5_proto::{Address, UdpHeader};
use std::io::Cursor;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;

use super::error::AssociationSocketError;

pub struct AssociationSocketHelper {
  socket: UdpSocket,
  socket_ttl: Duration,
  max_capacity: usize,
  dns_resolver: DnsResolver,
  buffer: BytesMut,
  target_cache: Cache<SocketAddr, SocketAddr>,
  client_addr: SocketAddr,
}

impl AssociationSocketHelper {
  pub async fn new(
    socket: UdpSocket,
    dns_resolver: DnsResolver,
    client_addr: SocketAddr,
    socket_ttl: Duration,
    max_capacity: usize,
    cache_size: u64,
  ) -> Result<Self, AssociationSocketError> {
    let helper = AssociationSocketHelper {
      socket,
      socket_ttl,
      client_addr,
      max_capacity,
      dns_resolver,
      buffer: BytesMut::with_capacity(max_capacity),
      target_cache: CacheBuilder::new(cache_size).build(),
    };
    Ok(helper)
  }

  pub async fn execute(&mut self) -> Result<(), AssociationSocketError> {
    loop {
      match tokio::time::timeout(self.socket_ttl, self.handle_recv_from()).await {
        Ok(res) => res?,
        Err(_) => {
          return Err(AssociationSocketError::SocketTTLError(self.socket_ttl));
        }
      }
    }
  }

  async fn handle_recv_from(&mut self) -> Result<(), AssociationSocketError> {
    match self.recv_from().await {
      // Client -> Proxy -> Server
      // Make sure that header.frag is 0, fragmentation is not supported.
      Ok(((pkt, Some(header), dest), _)) if header.frag == 0 => {
        debug!(
          "received UDP socket message. PKT LEN = {}, HEADER LEN = {}, DEST = {}",
          pkt.len(),
          header.serialized_len(),
          dest,
        );

        if let Err(e) = self.send_to(pkt, &None, dest).await {
          warn!("error while sending to server during UDP ASSOCIATE. Dst = {}, Err = {}", dest, e);
        }
      }
      // Server -> Proxy -> Client
      Ok(((pkt, None, dest), _)) => {
        debug!("sent UDP socket message. PKT LEN = {}, DEST = {}", pkt.len(), dest);

        let header = UdpHeader::new(0, Address::SocketAddress(dest));
        if let Err(e) = self.send_to(pkt, &Some(header), dest).await {
          warn!("error while sending to client during UDP ASSOCIATE. Dst = {}, Err = {}", dest, e);
        }
      }
      // Frag pkt, ignore.
      Ok(((_, Some(header), dest), _)) if header.frag != 0 => {
        warn!(
          "fragmented packet received and ignored during UDP ASSOCIATE. Dst = {}, Frag = {}",
          dest, header.frag
        );
      }
      Ok(_) => {
        debug!("Unknown UDP socket event received, please check further");
      }
      Err(e) => return Err(e),
    };
    Ok(())
  }

  // UdpSocket recv_from implementation to match SOCKS5 UDP ASSOCIATE use case:
  // Client (PKT + S5_UDP_HEADER) -> PROXY (checks and strips S5_UDP_HEADER from PKT) -> Server (PKT)
  // Server (PKT) -> PROXY (adds S5_UDP_HEADER) -> Client (PKT + UDP_HEADER)
  async fn recv_from(&mut self) -> Result<((Bytes, Option<UdpHeader>, SocketAddr), SocketAddr), AssociationSocketError> {
    self.buffer.resize(self.max_capacity, 0); // Initialize buffer for recv_from.

    let (len, src_addr) = self.socket.recv_from(&mut self.buffer).await.map_err(AssociationSocketError::SocketReadError)?;

    if len > self.max_capacity {
      return Err(AssociationSocketError::RecvBufferOverflow(len, self.max_capacity));
    }

    self.buffer.truncate(len);

    // Request comes from the server because the src_addr matches a target_cache entry.
    if let Some(client_addr) = self.target_cache.get(&src_addr) {
      // Check for intruders.
      if client_addr.ip() != self.client_addr.ip() {
        return Err(AssociationSocketError::SrcAddrMismatchSTCError(self.client_addr, client_addr));
      }
      return Ok(((self.buffer.copy_to_bytes(self.buffer.len()), None, client_addr), src_addr));
    }
    // Request comes from the client, target_cache does not match.

    // Check for intruders.
    if src_addr.ip() != self.client_addr.ip() {
      return Err(AssociationSocketError::SrcAddrMismatchCTSError(self.client_addr, src_addr));
    }

    let header = UdpHeader::read_from(&mut Cursor::new(&self.buffer))
      .await
      .map_err(AssociationSocketError::UdpHeaderParseError)?;

    let pkt = self.buffer.split_off(header.serialized_len()).freeze();

    // This is the target server address
    let header_addr = match &header.address {
      Address::SocketAddress(addr) => *addr,
      Address::DomainAddress(domain, port) => {
        let domain = String::from_utf8_lossy(domain);
        self
          .dns_resolver
          .resolve(&domain, *port)
          .await
          .map_err(|e| AssociationSocketError::DnsResolutionError(domain.to_string(), *port, e))?
      }
    };

    // Inserting <TargetServerAddr, ClientAddr>
    self.target_cache.insert(header_addr, src_addr).await;

    Ok(((pkt, Some(header), header_addr), src_addr))
  }

  async fn send_to<P: AsRef<[u8]>>(&mut self, pkt: P, header: &Option<UdpHeader>, addr: SocketAddr) -> Result<usize, AssociationSocketError> {
    self.buffer.clear();

    if let Some(header) = header {
      header.write_to_buf(&mut self.buffer);
    }

    self.buffer.extend_from_slice(pkt.as_ref());

    if self.buffer.len() > self.max_capacity {
      return Err(AssociationSocketError::SendBufferOverflow(self.buffer.len(), self.max_capacity));
    }

    self.socket.send_to(&self.buffer, addr).await.map_err(AssociationSocketError::SocketWriteError)
  }

  pub fn close(self) {
    drop(self.socket);
  }
}
