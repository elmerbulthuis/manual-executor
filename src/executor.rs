use once_cell::sync::Lazy;
use std::{
  future::Future,
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Wake, Waker},
};

pub static MANUAL_EXECUTOR: Lazy<Arc<ManualExecutor>> = Lazy::new(ManualExecutor::new);

#[no_mangle]
extern "C" fn wake() {
  MANUAL_EXECUTOR.wake()
}

pub struct ManualExecutor {
  tasks: Mutex<Vec<Pin<Box<dyn Future<Output = ()> + Send>>>>,
}

impl ManualExecutor {
  pub fn new() -> Arc<Self> {
    Arc::new(Self {
      tasks: Default::default(),
    })
  }

  pub fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
    let mut tasks = self.tasks.lock();
    let tasks = tasks.as_mut().unwrap();
    tasks.push(Box::pin(task));
  }

  pub fn wake(self: &Arc<Self>) {
    let wake = ManualWake::new(self.clone());
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);

    let mut tasks = self.tasks.lock();
    let tasks = tasks.as_deref_mut().unwrap();

    let mut tasks_new = Vec::new();
    while let Some(mut task) = tasks.pop() {
      match task.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(_) => {}
        std::task::Poll::Pending => tasks_new.push(task),
      };
    }

    *tasks = tasks_new;
  }
}

struct ManualWake {
  executor: Arc<ManualExecutor>,
}

impl ManualWake {
  fn new(executor: Arc<ManualExecutor>) -> Arc<Self> {
    Arc::new(Self { executor })
  }
}

impl Wake for ManualWake {
  fn wake(self: Arc<Self>) {
    self.executor.clone().wake()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{collections::HashSet, sync::Mutex};

  #[test]
  fn test_executor() {
    let set = Arc::new(Mutex::new(HashSet::new()));

    {
      let set = set.clone();
      MANUAL_EXECUTOR.spawn(async move {
        let mut set = set.lock();
        set.as_mut().unwrap().insert("a");
      });
    }
    {
      let set = set.clone();
      MANUAL_EXECUTOR.spawn(async move {
        let mut set = set.lock();
        set.as_mut().unwrap().insert("b");
      });
    }

    MANUAL_EXECUTOR.wake();

    let actual = set.lock().unwrap();
    let expected: HashSet<_> = ["a", "b"].into();

    assert_eq!(*actual, expected);
  }
}
