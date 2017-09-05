#![allow(non_snake_case)]
#![feature(inclusive_range_syntax)]
#![allow(dead_code)]
#![feature(conservative_impl_trait)]
extern crate tokio_core;
extern crate futures;
extern crate hyper;

mod AsyncServer;
mod Reactors;

use Reactors::*;
use tokio_core::reactor::Core;
use futures::task;
use futures::Stream;
use tokio_core::net::TcpListener;
use std::io::{self, Stderr, Write, stderr};
use std::thread;

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

	let mut counter: i8 = 0;
	// TODO: Dispatch to threads with less connected clients.
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
