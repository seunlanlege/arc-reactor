use super::rootservice::RootService;
use futures::{Future, Stream};
use hyper::{self, server::Http};
use native_tls::TlsAcceptor;
use num_cpus;
use proto::{ArcHandler, ArcService};
use routing::Router;
use std::io;
use std::net::{self, SocketAddr};
use std::sync::Arc;
use std::thread;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_tls::TlsAcceptorExt;

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
	threads: usize,
	handler: Option<ArcHandler>,
	tls_acceptor: Option<Arc<TlsAcceptor>>,
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
			tls_acceptor: None,
			threads: num_cpus::get(),
		}
	}

	/// Sets the port for the server to listen on and returns the instance.
	pub fn port(mut self, port: i16) -> Self {
		self.port = port;

		self
	}

	/// set the number of threads, for Arc-Reactor
	/// to spawn reactors on.
	///
	/// Default is num_cpus::get()
	pub fn threads(mut self, num: usize) -> Self {
		self.threads = num;

		self
	}

	/// set the TlsAcceptor
	/// check the `examples` folder for more info.
	pub fn tls(mut self, acceptor: TlsAcceptor) -> Self {
		self.tls_acceptor = Some(Arc::new(acceptor));

		self
	}

	/// Mounts the Router on the ArcReactor.
	pub fn routes(mut self, routes: Router) -> Self {
		let routes = box routes as Box<ArcService>;
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

		let routes = Arc::new(self.handler.expect("This thing needs routes to work!"));
		let mut http = Http::new();
		http.sleep_on_errors(true);
		let acceptor = self.tls_acceptor;

		for _ in 0..self.threads {
			let listener = listener.try_clone().expect("Could not clone listener!");
			let routes = routes.clone();
			let http = http.clone();
			let acceptor = acceptor.clone();

			thread::spawn(move || spawn(routes, listener, addr, http, acceptor));
		}

		println!(
			"[arc-reactor]: Starting main event loop!\n[arc-reactor]: Spawned {} threads",
			self.threads + 1
		);

		spawn(routes, listener, addr, http, acceptor);
		Ok(())
	}
}

fn spawn(
	routes: Arc<ArcHandler>,
	listener: net::TcpListener,
	addr: SocketAddr,
	http: Http<hyper::Chunk>,
	acceptor: Option<Arc<TlsAcceptor>>,
) {
	let mut core = Core::new().expect("Could not start event loop");
	let handle = core.handle();
	let listener = TcpListener::from_listener(listener, &addr, &handle)
		.expect("Could not convert TCPListener to async");

	let future = listener.incoming().for_each(|(socket, remote_ip)| {
		let service = routes.clone();
		// user has configured a tls acceptor
		if let Some(ref acceptor) = acceptor {
			let http_clone = http.clone();
			let handle_clone = handle.clone();
			let connection_future = acceptor
				.accept_async(socket)
				.map_err(|err| {
					// could not complete tls handshake
					println!("[arc-reactor] Handshake Error {:?}", err);
					Err(())
				})
				.and_then(move |socket| {
					// handshake successful
					let service = RootService {
						service,
						remote_ip,
						handle: handle_clone,
					};
					let conn_future = http_clone
						.serve_connection(socket, service)
						.then(|_| Ok(()));
					conn_future
				})
				.then(|_: Result<(), Result<(), ()>>| Ok(()));
			handle.spawn(connection_future);
			return Ok(());
		}

		// default to http
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
