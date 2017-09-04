use tokio_core::net::TcpStream;
use std::net::SocketAddr;
use futures::task::Task;
use futures::future::Future;
use futures::{Async, Poll};
use std::sync::{Arc, Mutex};

pub type ArcReactor = Arc<Mutex<Reactor>>;

pub struct Reactor {
  pub peers: Vec<(TcpStream, SocketAddr)>,
  pub taskHandle: Option<Task>,
}

impl Reactor {
  pub fn new() -> ArcReactor {
    Arc::new(
			Mutex::new(
				Reactor {
          peers: Vec::new(),
          taskHandle: None,
        }
			)
		)
  }
}

pub struct ReactorHandler<F: Fn()> {
  pub handler: F,
}

impl<F: Fn()> Future for ReactorHandler<F> {
  type Item = ();
  type Error = ();
  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
      (self.handler)();

      Ok(Async::NotReady)
  }
}
