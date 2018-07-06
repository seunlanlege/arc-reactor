use super::{rootservice::RootService, Request, Response};
use futures::{Future, Stream};
use hyper::server::conn::Http;
use native_tls::TlsAcceptor;
use proto::{ArcHandler, ArcService, MiddleWare};
use routing::Router;
use std::{io, sync::Arc};
use tokio::{self, net::TcpListener};
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
		}
	}

	/// Sets the port for the server to listen on and returns the instance.
	pub fn port(mut self, port: i16) -> Self {
		self.port = port;

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

	pub fn service<S: ArcService + 'static>(mut self, service: S) -> Self {
		let service = Box::new(service) as Box<ArcService>;
		if let Some(ref mut archandler) = self.handler {
			archandler.handler = service;
		} else {
			self.handler = Some(ArcHandler {
				before: None,
				after: None,
				handler: service,
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

	/// Binds the listener and returns a future representing the server
	/// this future should be spawned on the tokio runtime.
	///
	/// # Panics
	///
	/// Calling this function will panic if: no routes are supplied, or it
	/// cannot start the main event loop.

	pub fn start(self) -> Result<impl Future<Item = (), Error = io::Error> + Send, io::Error> {
		let ArcReactor {
			port,
			handler,
			tls_acceptor: acceptor,
		} = self;

		let addr = format!("0.0.0.0:{}", port).parse().unwrap();

		println!("[arc-reactor]: Binding to port {}", port);

		let listener = TcpListener::bind(&addr)?;

		println!("[arc-reactor]: Spawning threads!");
		let handler = handler.expect("This thing needs routes to work!");

		let http = Http::new();

		let future = listener
			.incoming()
			.for_each(move |socket| {
				let service = handler.clone();
				let remote_ip = socket.peer_addr().ok();
				// user has configured a tls acceptor
				if let Some(ref acceptor) = acceptor {
					let http_clone = http.clone();
					let connection_future = acceptor
						.accept_async(socket)
						.map_err(|err| {
							// could not complete tls handshake
							println!("[arc-reactor] Handshake Error {:?}", err);
							Err(())
						})
						.and_then(move |socket| {
							// handshake successful
							http_clone
								.serve_connection(socket, RootService { service, remote_ip })
								.then(|_| Ok(()))
						})
						.then(|_: Result<(), Result<(), ()>>| Ok(()));
					tokio::spawn(connection_future);
				} else {
					// default to http
					let connection_future = http
						.serve_connection(socket, RootService { service, remote_ip })
						.then(|_| Ok(()));

					tokio::spawn(connection_future);
				}

				Ok(())
			});

		Ok(future)
	}
}
