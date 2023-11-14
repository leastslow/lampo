use std::sync::Arc;

use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct SocketState {
  semaphore: Arc<Semaphore>,
}

impl SocketState {
  pub fn new(limit: usize) -> Self {
    Self {
      semaphore: Arc::new(Semaphore::new(limit)),
    }
  }

  pub fn try_incr(&self) -> bool {
    match self.semaphore.try_acquire() {
      Ok(permit) => {
        permit.forget();
        debug!("incremented socket state. Count = {}", self.semaphore.available_permits());
        true
      }
      Err(_) => false,
    }
  }

  pub fn decr(&self) {
    self.semaphore.add_permits(1);
    debug!("decremented socket state. Count = {}", self.semaphore.available_permits());
  }
}
