use crate::Key;
use crate::TaskWake;
use core::{
  future::Future,
  ops::DerefMut,
  pin::Pin,
  task::{Context, Waker},
};
use std::{
  collections::BTreeMap,
  sync::{Arc, Mutex},
};

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct ManualExecutor {
  tasks: Mutex<BTreeMap<Key, Task>>,
}

impl ManualExecutor {
  pub fn new() -> Arc<Self> {
    Arc::new(Self {
      tasks: Default::default(),
    })
  }

  pub fn task_count(&self) -> usize {
    let tasks = self.tasks.lock().unwrap();
    tasks.len()
  }

  pub fn spawn_wake(self: &Arc<Self>, task: impl Future<Output = ()> + Send + 'static) -> Key {
    let key = self.spawn(task);
    self.wake(key);
    key
  }

  pub fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) -> Key {
    let key = Key::new();
    let mut tasks = self.tasks.lock();
    let tasks = tasks.as_mut().unwrap();
    assert!(tasks.insert(key, Box::pin(task)).is_none());
    key
  }

  pub fn wake(self: &Arc<Self>, key: Key) {
    let mut tasks = self.tasks.lock().unwrap();
    let tasks = tasks.deref_mut();

    let Some(mut task) = tasks.remove(&key) else {
      return;
    };

    let pending = self.poll_task(key, &mut task);
    if pending {
      assert!(tasks.insert(key, task).is_none());
    }
  }

  pub fn wake_all(self: &Arc<Self>) {
    let mut tasks = self.tasks.lock().unwrap();
    let tasks = tasks.deref_mut();
    let mut tasks_new = BTreeMap::new();

    while let Some((key, mut task)) = tasks.pop_first() {
      let pending = self.poll_task(key, &mut task);
      if pending {
        assert!(tasks_new.insert(key, task).is_none());
      }
    }

    *tasks = tasks_new;
  }

  fn poll_task(self: &Arc<Self>, key: Key, task: &mut Task) -> bool {
    let wake = TaskWake::new(self.clone(), key);
    let waker = Waker::from(wake.clone());
    let mut context = Context::from_waker(&waker);

    let poll_result = task.as_mut().poll(&mut context);
    poll_result.is_pending()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use once_cell::sync::Lazy;
  use std::{collections::HashSet, sync::Mutex};

  #[test]
  fn test_wake_all() {
    pub static MANUAL_EXECUTOR: Lazy<Arc<ManualExecutor>> = Lazy::new(ManualExecutor::new);

    let set = Arc::new(Mutex::new(HashSet::new()));

    {
      let set = set.clone();
      MANUAL_EXECUTOR.spawn(async move {
        let mut set = set.lock();
        set.as_mut().unwrap().insert("a");
      });
    }
    assert_eq!(MANUAL_EXECUTOR.task_count(), 1);

    {
      let set = set.clone();
      MANUAL_EXECUTOR.spawn(async move {
        let mut set = set.lock();
        set.as_mut().unwrap().insert("b");
      });
    }
    assert_eq!(MANUAL_EXECUTOR.task_count(), 2);

    MANUAL_EXECUTOR.wake_all();
    assert_eq!(MANUAL_EXECUTOR.task_count(), 0);

    let actual = set.lock().unwrap();
    let expected: HashSet<_> = ["a", "b"].into();

    assert_eq!(*actual, expected);
  }

  #[test]
  fn test_wake_spawn() {
    pub static MANUAL_EXECUTOR: Lazy<Arc<ManualExecutor>> = Lazy::new(ManualExecutor::new);

    let set = Arc::new(Mutex::new(HashSet::new()));

    {
      let set = set.clone();
      MANUAL_EXECUTOR.spawn_wake(async move {
        let mut set = set.lock();
        set.as_mut().unwrap().insert("a");
      });
    }
    assert_eq!(MANUAL_EXECUTOR.task_count(), 0);

    {
      let set = set.clone();
      MANUAL_EXECUTOR.spawn_wake(async move {
        let mut set = set.lock();
        set.as_mut().unwrap().insert("b");
      });
    }
    assert_eq!(MANUAL_EXECUTOR.task_count(), 0);

    let actual = set.lock().unwrap();
    let expected: HashSet<_> = ["a", "b"].into();

    assert_eq!(*actual, expected);
  }
}
