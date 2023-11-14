use std::net::{Ipv4Addr, SocketAddr};

use ipnet::Ipv4Net;
use tokio::net::{TcpListener, TcpSocket, TcpStream};

use super::constants::LOCAL_HOST;

pub async fn make_listener(listen_addr: SocketAddr, backlog: u32) -> Result<TcpListener, tokio::io::Error> {
  let listener;

  if listen_addr.ip() == LOCAL_HOST {
    listener = TcpListener::bind(listen_addr).await?;
  } else {
    let socket = TcpSocket::new_v4()?;

    socket.set_reuseaddr(true)?;
    socket.bind(listen_addr)?;

    listener = socket.listen(backlog)?;
  }

  Ok(listener)
}

pub async fn make_outbound(bind_addr: SocketAddr, target_addr: SocketAddr) -> Result<TcpStream, tokio::io::Error> {
  let outbound;

  if bind_addr.ip() == LOCAL_HOST {
    outbound = TcpStream::connect(target_addr).await?;
  } else {
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(true)?;
    socket.bind(SocketAddr::new(bind_addr.ip(), 0))?;

    outbound = socket.connect(target_addr).await?;
  }

  Ok(outbound)
}

pub fn make_subnet_vec(subnet: Ipv4Net) -> Vec<Ipv4Addr> {
  let mut addrs = vec![subnet.network()];
  for host in subnet.hosts() {
    addrs.push(host);
  }
  // Skip the broadcast address, can't be used to listen on.
  addrs
}
