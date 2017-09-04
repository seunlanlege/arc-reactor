use futures::{task, Future, Poll, Async};
use std::time::SystemTime;

pub struct SetTimeout<T: Fn()> {
  pub timeOut: u64,
  pub handler: T,
  pub systemTime: SystemTime,
  pub count: u64,
}

impl<T: Fn()> SetTimeout<T> {
  pub fn new(timeOut: u64, handler: T) -> SetTimeout<T> {
    SetTimeout {
      timeOut,
      handler,
      systemTime: SystemTime::now(),
      count: 0,
    }
  }
}

impl<T: Fn()> Future for SetTimeout<T> {
  type Item = ();
  type Error = ();

  fn poll(&mut self) -> Poll<(), ()> {
    let handle = task::current();
    self.count += 1;

    if self.systemTime.elapsed().unwrap().as_secs() >= self.timeOut {
      (self.handler)();
      return Ok(Async::Ready(()));
    }

    handle.notify();
    Ok(Async::NotReady)
  }
}
