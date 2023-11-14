use config::{Config, ConfigError};
use serde::Deserialize;
use std::env;
use std::net::Ipv4Addr;
use std::time::Duration;

pub fn parse_args() -> Option<String> {
  let args: Vec<String> = env::args().collect();

  let mut opts = getopts::Options::new();
  opts.optopt("c", "config", "config file source (basename only, ex: /etc/lampo/config)", "CONFIG");

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => m,
    Err(f) => {
      panic!("{}", f.to_string())
    }
  };
  matches.opt_str("c")
}

pub fn load_config(path: String) -> Result<GlobalConfig, ConfigError> {
  let config = Config::builder().add_source(config::File::with_name(&path)).build()?;
  config.try_deserialize::<GlobalConfig>()
}

#[derive(Clone, Deserialize)]
pub struct GlobalConfig {
  pub proxy: ProxyConfig,
  pub cache: CacheConfigContainer,
  pub mongodb: MongoDBConfig,
  pub log4rs: Log4rsConfig,
}

#[derive(Clone, Deserialize)]
pub struct CacheConfigContainer {
  pub dns: DnsCacheConfig,
  pub auth: AuthCacheConfig,
}

#[derive(Clone, Deserialize)]
pub struct MongoDBConfig {
  pub uri: String,
  pub database: String,
}

#[derive(Clone, Deserialize)]
pub struct ProxyConfig {
  pub ports: ProxyConfigPorts,
  pub preload: ProxyConfigPreload,
  pub backlog: u32,
  pub udp: ProxyConfigUdpSocket,
}

#[derive(Clone, Deserialize)]
pub struct ProxyConfigUdpSocket {
  #[serde(with = "humantime_serde")]
  pub stale_ttl: Duration,
  pub max_sockets: usize,
}

#[derive(Clone, Deserialize)]
pub struct ProxyConfigPorts {
  pub http: u16,
  pub socks: u16,
}

#[derive(Clone, Deserialize)]
pub struct ProxyConfigPreload {
  pub tasks: usize,
  pub subnets: Option<Vec<ipnet::Ipv4Net>>,
  pub addrs: Option<Vec<Ipv4Addr>>,
}

#[derive(Clone, Deserialize)]
pub struct AuthCacheConfig {
  pub max_size: u64,
  #[serde(with = "humantime_serde")]
  pub time_to_live: Duration,
}

#[derive(Clone, Deserialize)]
pub struct DnsCacheConfig {
  pub max_size: usize,
  #[serde(with = "humantime_serde")]
  pub resolution_timeout: Duration,
}

#[derive(Clone, Deserialize)]
pub struct Log4rsConfig {
  pub location: String,
}
