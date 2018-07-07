use super::{rootservice::RootService, Request, Response};
use contrib::BodyParser;
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
/// extern crate tokio;
/// use arc_reactor::ArcReactor;
///
/// fn main() {
/// 	let server = ArcReactor::default().port(1234).start().expect("couldn't start server");
///		tokio::run(server);
/// }
/// ```
pub struct ArcReactor {
	port: i16,
	arc_handler: ArcHandler,
	tls_acceptor: Option<Arc<TlsAcceptor>>,
}

impl Default for ArcReactor {
	fn default() -> Self {
		ArcReactor {
			port: 8080,
			arc_handler: ArcHandler {
				before: Some(mw![BodyParser]),
				after: None,
				handler: None,
			},
			tls_acceptor: None,
		}
	}
}

impl ArcReactor {
	/// Creates an instance of the server.
	/// with a default port of `8080`
	pub fn new() -> Self {
		ArcReactor::default()
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
		self.arc_handler.handler = Some(routes);

		self
	}

	pub fn service<S: ArcService + 'static>(mut self, service: S) -> Self {
		let service = Box::new(service) as Box<ArcService>;
		self.arc_handler.handler = Some(service);

		self
	}

	pub fn before<M>(mut self, before: M) -> Self
	where
		M: MiddleWare<Request> + 'static,
	{
		self.arc_handler.before = Some(Box::new(before));

		self
	}

	pub fn after<M>(mut self, after: M) -> Self
	where
		M: MiddleWare<Response> + 'static,
	{
		self.arc_handler.after = Some(Box::new(after));

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
			arc_handler,
			tls_acceptor: acceptor,
		} = self;

		let addr = format!("0.0.0.0:{}", port).parse().unwrap();

		info!("Binding to port {}", port);

		let listener = TcpListener::bind(&addr)?;

		let http = Http::new();

		let conn_stream_future = listener.incoming().for_each(move |socket| {
			let service = arc_handler.clone();
			let remote_ip = socket.peer_addr().ok();
			// user has configured a tls acceptor
			if let Some(ref acceptor) = acceptor {
				let http_clone = http.clone();
				let connection_future = acceptor
					.accept_async(socket)
					.map_err(|err| error!("TLS Handshake Error: {}", err))
					.and_then(move |socket| {
						// handshake successful
						http_clone
							.serve_connection(socket, RootService { service, remote_ip })
							.map_err(|err| error!("serve_connection Error: {}", err))
							.and_then(|_| Ok(()))
					});
					
				tokio::spawn(connection_future);
			} else {
				// default to http
				let connection_future = http
					.serve_connection(socket, RootService { service, remote_ip })
					.map_err(|err| error!("serve_connection Error: {}", err))
					.and_then(|_| Ok(()));

				tokio::spawn(connection_future);
			}

			Ok(())
		});

		Ok(conn_stream_future)
	}
}
