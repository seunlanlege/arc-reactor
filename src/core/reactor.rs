use super::rootservice::RootService;
use futures::{Future, Stream};
use hyper::{self, server::Http};
use num_cpus;
use proto::{ArcHandler, ArcService};
use routing::Router;
use std::io;
use std::net::{self, SocketAddr};
use std::sync::Arc;
use std::thread;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

/// The main server, the ArcReactor is where you mount your routes, middlewares
/// and initiate the server.
///
/// #Examples
///
/// ```rust,ignore
/// extern crate arc_reactor;
/// use arc_reactor::ArcReactor;
///
/// fn main() {
/// 	ArcReactor::new().routes(..).port(1234).initiate().unwrap()
/// 	}
/// ```
pub struct ArcReactor {
	port: i16,
	handler: Option<ArcHandler>,
}

impl ArcReactor {
	/// Creates an instance of the server.
	/// with a default port of `8080`
	/// and *No* routes. Note that calling `initiate` on an `ArcReactor` without
	/// routes will cause your program to panic.
	pub fn new() -> ArcReactor {
		ArcReactor {
			port: 8080,
			handler: None,
		}
	}

	/// Sets the port for the server to listen on and returns the instance.
	pub fn port(mut self, port: i16) -> Self {
		self.port = port;

		self
	}

	/// Mounts the Router on the ArcReactor.
	pub fn routes(mut self, routes: Router) -> Self {
		let routes = Arc::new(box routes as Box<ArcService>);
		if let Some(ref mut archandler) = self.handler {
			archandler.handler = routes;
		} else {
			self.handler = Some(ArcHandler {
				before: None,
				after: None,
				handler: routes,
			});
		}

		self
	}

	/// Binds the listener and blocks the main thread while listening for
	/// incoming connections.
	///
	/// # Panics
	///
	/// Calling this function will panic if: no routes are supplied, or it
	/// cannot start the main event loop.

	#[must_use]
	pub fn initiate(self) -> io::Result<()> {
		let addr = format!("0.0.0.0:{}", self.port).parse().unwrap();

		println!("[arc-reactor]: Binding to port {}", self.port);
		let listener = match net::TcpListener::bind(&addr) {
			Ok(listener) => listener,
			Err(e) => {
				eprintln!(
					"[arc-reactor]: Whoops! something else is running on port {}, {}",
					&self.port, e
				);
				return Err(e);
			}
		};

		println!("[arc-reactor]: Spawning threads!");
		let threads = num_cpus::get() * 2;
		let routes = Arc::new(self.handler.expect("This thing needs routes to work!"));

		for _ in 0..threads {
			let listener = listener.try_clone().expect("Could not clone listener!");
			let routes = routes.clone();

			thread::spawn(move || spawn(routes.clone(), listener, addr));
		}
		
		println!(
			"[arc-reactor]: Starting main event loop!\n[arc-reactor]: Spawned {} threads",
			threads
		);

		spawn(routes, listener, addr);
		Ok(())
	}
}

fn spawn(routes: Arc<ArcHandler>, listener: net::TcpListener, addr: SocketAddr) {
	let mut core = Core::new().expect("Could not start event loop");
	let handle = core.handle();
	let http: Http<hyper::Chunk> = Http::new();
	let listener = TcpListener::from_listener(listener, &addr, &handle)
		.expect("Could not convert TCPListener to async");

	let future = listener.incoming().for_each(|(socket, remote_ip)| {
		let service = routes.clone();
		let connection_future = http.serve_connection(
			socket,
			RootService {
				service,
				remote_ip,
				handle: handle.clone(),
			},
		).then(|_| Ok(()));
		handle.spawn(connection_future);
		Ok(())
	});

	core.run(future).expect("Error running reactor core!");
}
