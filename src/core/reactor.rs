use futures::{Async, Future, Poll, Stream};
use futures::task::{self, Task};
use std::io;
use std::net::SocketAddr;
use hyper::Chunk;
use hyper::server::Http;
use tokio_core::reactor::Core;
use tokio_core::net::{TcpListener, TcpStream};
use routing::Router;
use std::sync::{Arc, Mutex};
use futures::prelude::{async, await};
use std::thread;
use num_cpus;
use proto::{ArcHandler, ArcService};
use super::rootservice::RootService;

// A wrapper around a closure i can run forever on an event loop.
struct ReactorFuture<F>
where
	F: Fn(),
{
	pub handler: F,
}

// The future never completes.
// This is because it takes care of dispatching connected clients on the event
// loop.
impl<F> Future for ReactorFuture<F>
where
	F: Fn(),
{
	type Item = ();
	type Error = ();

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		(self.handler)();

		Ok(Async::NotReady)
	}
}

type ReactorAlias = Arc<Mutex<Reactor>>;

// Shared Mutable object to connected clients
// This struct is only shared by the main thread, and the another worker thread
// at any point in time. As new clients are connected, The main thread will
// lock the reactor and push the clients to `peers`. then the future running in
// the thread is notified that there are new clients that need to be handled.
struct Reactor {
	pub(crate) peers: Vec<(TcpStream, SocketAddr)>,
	pub(crate) taskHandle: Option<Task>,
}

// Creates a thread safe Mutable reactor.
impl Reactor {
	pub fn new() -> ReactorAlias {
		Arc::new(Mutex::new(Reactor {
			peers: Vec::new(),
			taskHandle: None,
		}))
	}
}

/// The main server, the ArcReactor is where you mount your routes, middlewares and initiate the
/// server.
///
/// #Examples
///
/// ```rust,ignore
/// xtern crate arc_reactor;
/// se arc_reactor::ArcReactor;
///
/// n main() {
/// ArcReactor::new().routes(..).port(1234).initiate().unwrap()
/// 	}
/// ```
pub struct ArcReactor {
	port: i16,
	handler: Option<ArcHandler>,
}

impl ArcReactor {
	/// Creates an instance of the server.
	/// with a default port of `8080`
	/// and *No* routes. Note that calling `initiate` on an `ArcReactor` without routes
	/// will cause your program to panic.
	pub fn new() -> ArcReactor {
		ArcReactor {
			port: 8080,
			handler: None,
		}
	}

	/// sets the port for the server to listen on and returns the instance.
	pub fn port(mut self, port: i16) -> Self {
		self.port = port;

		self
	}

//	/// Mount a global MiddleWare on the server, Note that it takes a type
//	/// [`MiddleWare<Request>`](trait.MiddleWare.html#impl-MiddleWare<Request>) this is because, the
//	/// middleware(s) supplied here are run before any other middleware or route handlers.
//	///
//	/// read the [`MiddleWare<T>`](trait.MiddleWare.html) documentation to understand how middlewares
//	/// work.
//	pub fn before(mut self, before: Box<MiddleWare<Request>>) -> Self {
//		if let Some(ref mut archandler) = self.handler {
//			archandler.before = Some(Arc::new(before));
//		}
//
//		self
//	}

//	/// Mount a global MiddleWare on the server, Note that it takes a type
//	/// [`MiddleWare<Response>`](trait.MiddleWare.html#impl-MiddleWare<Response>) this is because,
//	/// the middleware(s) supplied here are run before any other middleware or route handlers.
//	///
//	/// read the [`MiddleWare<T>`](trait.MiddleWare.html) documentation to understand how middlewares
//	/// work.
//	pub fn after(mut self, after: Box<MiddleWare<Response>>) -> Self {
//		if let Some(ref mut archandler) = self.handler {
//			archandler.after = Some(Arc::new(after));
//		}
//
//		self
//	}

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

	/// Binds the listener and blocks the main thread while listening for incoming connections.
	///
	/// # Panics
	///
	/// Calling this function will panic if: no routes are supplied, or it cannot
	/// start the main event loop.

	#[must_use]
	pub fn initiate(self) -> io::Result<()> {
		println!("[arc-reactor]: Spawning threads!");
		let reactors = spawn(self.handler.expect("This thing needs routes to work!"))?;
		println!(
			"[arc-reactor]: Starting main event loop!\n[arc-reactor]: Spawned {} threads",
			reactors.len()
		);
		let mut core = Core::new()?;

		println!("[arc-reactor]: Started Main event loop!");
		let handle = core.handle();

		let addr = format!("0.0.0.0:{}", self.port).parse().unwrap();
		println!("[arc-reactor]: Binding to port {}", self.port);
		let listener = match TcpListener::bind(&addr, &handle) {
			Ok(listener) => listener,
			Err(e) => {
				eprintln!(
					"[arc-reactor]: Whoops! something else is running on port {}, {}",
					&self.port, e
				);
				return Err(e);
			}
		};

		let mut counter = 0;

		println!("[arc-reactor]: Running Main Event loop");
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
}

fn spawn(RouteService: ArcHandler) -> io::Result<Vec<ReactorAlias>> {
	let mut reactors = Vec::new();
	let routeService = Arc::new(RouteService);

	for _ in 0..num_cpus::get() * 2 {
		let reactor = Reactor::new();
		reactors.push(reactor.clone());
		let routeService = routeService.clone();

		thread::spawn(move || {
			let mut core = Core::new().expect("Could not start event loop");
			let handle = core.handle();
			let http = Http::new();

			let handler = || {
				let mut reactor = reactor.lock().unwrap();
				for (socket, remote_ip) in reactor.peers.drain(..) {
					let service = routeService.clone();
					let future = socketHandler(
						socket,
						http.clone(),
						RootService {
							service,
							remote_ip,
							handle: handle.clone(),
						},
					);
					handle.spawn(future);
				}
				reactor.taskHandle = Some(task::current());
			};

			let future = ReactorFuture { handler };

			core.run(future).expect("Error running reactor core!");
		});
	}

	Ok(reactors)
}

#[async]
fn socketHandler(
	stream: TcpStream,
	http: Http<Chunk>,
	serviceHandler: RootService,
) -> Result<(), ()> {
	let _opaque = await!(http.serve_connection(stream, serviceHandler));
	Ok(())
}
