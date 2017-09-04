// #![allow(unused_variables)]
#![allow(non_snake_case)]
#![feature(inclusive_range_syntax)]
#![allow(dead_code)]
// #![allow(unused_imports)]
#![feature(conservative_impl_trait)]
extern crate tokio_core;
// extern crate tokio_io;
extern crate futures;
extern crate hyper;

mod AsyncService;
mod SetTimeout;
mod Reactors;

use Reactors::*;
// use SetTimeout::SetTimeout as setTimeout;
use AsyncService::AsyncService as AsyncServer;
// use futures::future::Future;
use futures::task;
use futures::Stream;
use hyper::server::Http;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use std::io::{self, Stderr, Write, stderr};
use std::thread;
// use std::time::Duration;
// use std::net::{SocketAddr};
//use std::sync::{Arc, Mutex};

fn SpawnReactors() -> Vec<ArcReactor> {
  let mut reactors = Vec::new();

  for _ in 1...5 {
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

fn serve() -> io::Result<()> {
	let reactors = SpawnReactors();
	println!("Server Running on {} threads", reactors.len());

	let mut core = Core::new()?;
	let handle = core.handle();

	let addr = "0.0.0.0:8080".parse().unwrap();
	let listener = match TcpListener::bind(&addr, &handle) {
		Ok(listener) => {
			println!("Server Bound to port 8080");
			listener
		}
		Err(e) => {
			Stderr::write(&mut stderr(), b"Failed to bind to port 8080")?;
			return Err(e);
		}
	};

	let mut counter = 0;
	core.run(listener.incoming().for_each(move |(socket, peerIp)| {
		let mut reactor = reactors[counter].lock().unwrap();
		reactor.peers.push((socket, peerIp));

		if let Some(ref task) = reactor.taskHandle {
			task.notify();
		}

		counter += 1;
		if counter == reactors.len() {
			counter = 0
		}
		Ok(())
	}))?;

	Ok(())
}


fn main() {
    serve().unwrap();
}
