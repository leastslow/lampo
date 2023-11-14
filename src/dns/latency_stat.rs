// use std::{collections::HashMap, net::SocketAddr, time::Duration};

// use rand::{distributions::WeightedIndex, Rng};
// use tokio::time::Instant;

// #[derive(Default)]
// pub struct IpLatencyTracker {
//   pub stats: HashMap<SocketAddr, LatencyStat>,
// }

// pub struct LatencyStat {
//   total_latency: Duration,
//   count: u64,
//   last_update: Instant,
// }

// impl IpLatencyTracker {
//   pub fn new() -> Self {
//     Self { stats: HashMap::new() }
//   }

//   pub fn record_latency(&mut self, ip: SocketAddr, latency: Duration) {
//     self.stats.entry(ip).or_default().record(latency);
//   }

//   pub fn select_weighted_ip(&self, available_ips: &[SocketAddr]) -> Option<SocketAddr> {
//     let weights: Vec<_> = available_ips.iter().map(|&ip| self.stats.get(&ip).map_or(1, |stat| stat.weight())).collect();

//     let dist = WeightedIndex::new(&weights).ok()?;
//     let selected_index = rand::thread_rng().sample(dist);
//     Some(*available_ips.get(selected_index)?)
//   }
// }

// impl LatencyStat {
//   const LATENCY_DECAY_TIME: Duration = Duration::from_secs(60 * 5);

//   fn record(&mut self, latency: Duration) {
//     let now = Instant::now();

//     if now.duration_since(self.last_update) > LatencyStat::LATENCY_DECAY_TIME {
//       // Reset stats if too old
//       self.total_latency = latency;
//       self.count = 1;
//     } else {
//       self.total_latency += latency;
//       self.count += 1;
//     }

//     self.last_update = now;
//   }

//   fn average(&self) -> Duration {
//     if self.count == 0 {
//       return Duration::new(0, 0);
//     }

//     // Calculate total nanoseconds
//     let total_nanos = self.total_latency.as_secs() * 1_000_000_000 + self.total_latency.subsec_nanos() as u64;

//     // Calculate average
//     let avg_nanos = total_nanos / self.count;
//     Duration::new(avg_nanos / 1_000_000_000, (avg_nanos % 1_000_000_000) as u32)
//   }

//   fn weight(&self) -> u32 {
//     (1_000_000_000u32 / self.average().as_nanos() as u32).max(1)
//   }
// }

// impl Default for LatencyStat {
//   fn default() -> Self {
//     Self {
//       total_latency: Duration::new(0, 0),
//       count: 0,
//       last_update: Instant::now(),
//     }
//   }
// }
