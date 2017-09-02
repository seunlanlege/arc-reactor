#![allow(unused_variables)]
#![allow(non_snake_case)]
extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate hyper;

mod AsyncService;

use AsyncService::AsyncService as AsyncServer;

use futures::Stream;
use hyper::server::Http;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use std::io::{Stderr, Write, stderr, self};

fn serve() -> io::Result<()> {
		let mut core = Core::new()?;
		let handle = core.handle();
		let server = Http::new();
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

		core.run(listener.incoming().for_each(|(socket, peerIp)| {
				server.bind_connection(&handle, socket, peerIp, AsyncServer);
				Ok(())
		}))?;

		Ok(())
}

fn main() {
		serve().unwrap();
}
