use mongodb::{options::ClientOptions, Client};

pub mod auth_manager;
pub mod event_manager;
pub mod models;

pub async fn initialize_client(uri: String) -> Result<Client, mongodb::error::Error> {
  let client_options = ClientOptions::parse_async(uri).await?;
  Client::with_options(client_options)
}
