use socks5_proto::{Address, Reply};

use crate::proxy::socks5::utils::error::Socks5HandlerError;

use super::CommandHandler;

impl<'a> CommandHandler<'a> {
  pub async fn bind(&mut self) -> Result<(), Socks5HandlerError> {
    self.reply(Reply::CommandNotSupported, Address::unspecified()).await
  }
}
