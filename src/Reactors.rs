use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use std::net::SocketAddr;
use futures::task::{Task, self};
use std::thread;
use futures::future::Future;
use futures::{Async, Poll};
use std::sync::{Arc, Mutex};
use hyper::server::Http;
use super::AsyncServer::AsyncService as AsyncServer;

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

pub fn SpawnReactors() -> Vec<ArcReactor> {
	let mut reactors = Vec::new();

	for _ in 1 ... 5 {
		let reactor = Reactor::new();
		reactors.push(reactor.clone());

		thread::spawn(move || {
			let mut core = Core::new().unwrap();
			let handle = core.handle();
			let http = Http::new();

			core.run(ReactorHandler {
				handler: || {
					let mut reactor = reactor.lock().unwrap();
					for (socket, peerAddr) in reactor.peers.drain(..) {
						http.bind_connection(&handle, socket, peerAddr, AsyncServer);
					}
					reactor.taskHandle = Some(task::current());
				},
			}).unwrap();
		});
	}

	reactors
}
