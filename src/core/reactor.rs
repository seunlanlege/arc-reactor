use futures::{Async, Future, Poll, Stream};
use futures::task::{self, Task};
use std::io;
use std::net::SocketAddr;
use hyper::Chunk;
use hyper::server::Http;
use tokio_core::reactor::Core;
use tokio_core::net::{TcpListener, TcpStream};
use routing::{ArcRouter, Router};
use std::sync::{Arc, Mutex};
use futures::prelude::{async, await};
use std::thread;
use num_cpus;

pub struct ReactorFuture<F>
where
	F: Fn(),
{
	pub handler: F,
}

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

pub type ReactorAlias = Arc<Mutex<Reactor>>;

pub struct Reactor {
	pub(crate) peers: Vec<(TcpStream, SocketAddr)>,
	pub(crate) taskHandle: Option<Task>,
}

impl Reactor {
	pub fn new() -> ReactorAlias {
		Arc::new(Mutex::new(Reactor {
			peers: Vec::new(),
			taskHandle: None,
		}))
	}
}

pub struct ArcReactor {
	port: i16,
	routeService: Option<ArcRouter>,
}

impl ArcReactor {
	pub fn new() -> ArcReactor {
		ArcReactor {
			port: 8080,
			routeService: None,
		}
	}

	pub fn port(mut self, port: i16) -> Self {
		self.port = port;

		self
	}

	pub fn routes(mut self, routes: Router) -> Self {
		let routes = ArcRouter {
			routes: Arc::new(routes.routes),
		};
		self.routeService = Some(routes);

		self
	}

	pub fn initiate(self) -> io::Result<()> {
		println!("[arc-reactor]: Spawning threads!");
		let reactors = spawn(self.routeService.expect("This thing needs routes to work!"))?;
		println!(
			"[arc-reactor]: Starting main event loop!\n[arc-reactor]: Spawned {} threads",
			num_cpus::get() * 2
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

fn spawn(RouteService: ArcRouter) -> io::Result<Vec<ReactorAlias>> {
	let mut reactors = Vec::new();
	let routeService = Arc::new(RouteService);

	for _ in 0..1 {
		let reactor = Reactor::new();
		reactors.push(reactor.clone());
		let routeService = routeService.clone();

		thread::spawn(move || {
			let mut core = Core::new().expect("Could not start event loop");
			let handle = core.handle();
			let http = Http::new();
			let http = Arc::new(http);

			let handler = || {
				let mut reactor = reactor.lock().unwrap();
				for (socket, _peerAddr) in reactor.peers.drain(..) {
					let future = socketHandler(socket, http.clone(), routeService.clone());
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
	http: Arc<Http<Chunk>>,
	serviceHandler: Arc<ArcRouter>,
) -> Result<(), ()> {
	println!("handling connection");
	let _opaque = await!(http.serve_connection(stream, serviceHandler.clone()));
	Ok(())
}
