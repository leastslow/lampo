use socks5_proto::{Address, Reply};

use crate::{proxy::socks5::utils::error::Socks5HandlerError, utils::socket::make_outbound};

use super::CommandHandler;

impl<'a> CommandHandler<'a> {
  pub async fn connect(&mut self) -> Result<(), Socks5HandlerError> {
    let target_addr = self.resolve_address().await?;

    // let dns_start_time = Instant::now();

    let mut outbound = match make_outbound(self.bind_addr, target_addr).await {
      Ok(outbound) => {
        // self.dns_resolver.record_latency(target_addr, dns_start_time.elapsed()).await;
        self.reply(Reply::Succeeded, Address::unspecified()).await?;
        outbound
      }
      Err(e) => {
        return Err(Socks5HandlerError::OutboundError(e, target_addr));
      }
    };

    if let Err(e) = tokio::io::copy_bidirectional(&mut outbound, self.stream).await {
      return Err(Socks5HandlerError::ClosedConnection(e));
    }

    Ok(())
  }
}
