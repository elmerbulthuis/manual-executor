use crate::{key::Key, ManualExecutor};
use std::{sync::Arc, task::Wake};

pub struct TaskWake {
  executor: Arc<ManualExecutor>,
  key: Key,
}

impl TaskWake {
  pub fn new(executor: Arc<ManualExecutor>, key: Key) -> Arc<Self> {
    Arc::new(Self { executor, key })
  }
}

impl Wake for TaskWake {
  fn wake(self: Arc<Self>) {
    self.executor.clone().wake(self.key)
  }
}
