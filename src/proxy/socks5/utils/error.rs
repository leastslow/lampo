use std::{io::Error as IoError, net::SocketAddr, time::Duration};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum Socks5HandlerError {
  #[error("socks5-proto error. Err = {0}")]
  Socks5Proto(socks5_proto::Error),
  #[error("failed to extract client address. Err = {0}")]
  PeerAddrError(IoError),
  #[error("failed to create outbound TcpStream ({1}). Err = {0}")]
  OutboundError(IoError, SocketAddr),
  #[error("error while reading from stream. Err = {0}")]
  StreamReadError(IoError),
  #[error("error while writing from stream. Err = {0}")]
  StreamWriteError(IoError),
  #[error("error or invalid credentials while authenticating")]
  AuthenticationError,
  #[error("connection closed by the client/upstream. Err = {0}")]
  ClosedConnection(IoError),
  #[error("stream read timeout")]
  StreamReadTimeout,
  #[error("stream write timeout")]
  StreamWriteTimeout,
  #[error("could not resolve DNS for hostname {0}:{1}. Err = {2}")]
  DnsResolutionError(String, u16, Box<dyn std::error::Error + Send + Sync>),
  #[error("failed to bind UdpSocket. Err = {0}")]
  BindUdpSocketError(IoError),
  #[error("failed to handle UDP ASSOCIATE between client and server. Err = {0}")]
  AssociationError(AssociationSocketError),
  #[error("maximum UDP sockets limit reached (Proxy: {0}, Client: {1})")]
  SocketLimitReached(SocketAddr, SocketAddr),
  #[error("unknown socks5 handler error")]
  Unknown,
}

#[derive(Error, Debug)]
pub enum AssociationSocketError {
  #[error("packet received exceeds socket buffer maximum capacity ({0}/{1})")]
  RecvBufferOverflow(usize, usize),
  #[error("attempted to send a packet which exceeds socket buffer maximum capacity ({0}/{1})")]
  SendBufferOverflow(usize, usize),
  #[error("failed to resolve packet target DNS record ({0}:{1}). Err = {2}")]
  DnsResolutionError(String, u16, Box<dyn std::error::Error + Send + Sync>),
  #[error("failed to parse UDP header. Err = {0}")]
  UdpHeaderParseError(socks5_proto::Error),
  #[error("failed to read data from socket. Err = {0}")]
  SocketReadError(IoError),
  #[error("failed to write data to socket. Err = {0}")]
  SocketWriteError(IoError),
  #[error("stale udp socket reached time-to-live limit ({0:?})")]
  SocketTTLError(Duration),
  #[error("Client -> Server, TCP client addr did not match UDP socket client addr. (TCP: {0}, UDP: {1})")]
  SrcAddrMismatchCTSError(SocketAddr, SocketAddr),
  #[error("Server -> Client, TCP client addr did not match UDP socket client addr. (TCP: {0}, UDP: {1})")]
  SrcAddrMismatchSTCError(SocketAddr, SocketAddr),
  // #[error("failed to get bind address from underlying UdpSocket. Err = {0}")]
  // BindAddrNotAvailable(IoError),
}

// #[derive(Error, Debug)]
// pub enum UdpHelperError {
//   #[error("AssociationSocketError. Err = {0}")]
//   AssociationSocketError(AssociationSocketError),
//   #[error("failed to create UDP socket. Addr = {1}, Err = {0}")]
//   UdpSocketError(IoError, SocketAddr),
// }
