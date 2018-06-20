use super::{rootservice::RootService, Request, Response};
use futures::{
	sync::mpsc::{unbounded, UnboundedReceiver},
	Future,
	Stream,
};
use hyper::server::conn::Http;
use native_tls::TlsAcceptor;
use num_cpus;
use proto::{ArcHandler, ArcService, MiddleWare};
use routing::Router;
use std::{io, net::SocketAddr, sync::Arc, thread};
use tokio_core::{
	net::{TcpListener, TcpStream},
	reactor::Core,
};
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
		let routes = Box::new(routes) as Box<ArcService>;
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

	pub fn before<M>(mut self, before: M) -> Self
	where
		M: MiddleWare<Request> + 'static,
	{
		if let Some(ref mut archandler) = self.handler {
			archandler.before = Some(Box::new(before));
		}

		self
	}

	pub fn after<M>(mut self, after: M) -> Self
	where
		M: MiddleWare<Response> + 'static,
	{
		if let Some(ref mut archandler) = self.handler {
			archandler.after = Some(Box::new(after));
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

	pub fn initiate(self) -> io::Result<()> {
		let ArcReactor {
			port,
			handler,
			tls_acceptor: acceptor,
			threads,
		} = self;

		let addr = format!("0.0.0.0:{}", port).parse().unwrap();
		let mut core = Core::new().expect("Could not start event loop");
		let handle = core.handle();

		println!("[arc-reactor]: Binding to port {}", port);
		let listener = match TcpListener::bind(&addr, &handle) {
			Ok(listener) => listener,
			Err(e) => {
				eprintln!(
					"[arc-reactor]: Whoops! something else is running on port {}, {}",
					&port, e
				);
				return Err(e);
			}
		};

		println!("[arc-reactor]: Spawning threads!");
		let handler = handler.expect("This thing needs routes to work!");

		let mut receivers = Vec::new();

		for i in 0..threads {
			let (tx, rx) = unbounded::<(TcpStream, SocketAddr)>();
			receivers.push(tx);
			let acceptor = acceptor.clone();
			let handler = handler.clone();

			let reactor = thread::Builder::new().name(format!("Reactor {}", i));

			reactor.spawn(move || spawn(handler, rx, acceptor)).unwrap();
		}

		println!(
			"[arc-reactor]: Starting main event loop!\n[arc-reactor]: Spawned {} threads",
			threads
		);

		let mut count = 0;

		let future = listener.incoming().for_each(|stream| {
			let _ = receivers[count].unbounded_send(stream);
			count += 1;

			if count == threads {
				count = 0
			}

			Ok(())
		});

		core.run(future).expect("Error running reactor core!");

		Ok(())
	}
}

fn spawn(
	routes: ArcHandler,
	listener: UnboundedReceiver<(TcpStream, SocketAddr)>,
	acceptor: Option<Arc<TlsAcceptor>>,
) {
	let mut core = Core::new().expect("Could not start event loop");
	let handle = core.handle();
	let mut http = Http::new();
	// http.sleep_on_errors(true);

	let future = listener.for_each(move |(socket, remote_ip)| {
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
		let connection_future =
			http.serve_connection(
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
