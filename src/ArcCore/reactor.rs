use futures::Stream;
use std::io;
use hyper;
use hyper::Chunk;
use hyper::server::{Http, Service};
use tokio_core::reactor::Core;
use tokio_core::net::{TcpStream, TcpListener};
use ArcCore::{ReactorHandler};
use ArcRouting::{ArcRouter, RouteGroup};
use std::sync::{Arc, Mutex};
use futures::Future;
use futures::task::{Task, self};
use std::net::SocketAddr;
use std::thread;
use num_cpus;

pub type ReactorAlias = Arc<Mutex<Reactor>>;

pub struct Reactor {
	pub(crate) peers: Vec<(TcpStream, SocketAddr)>,
	pub(crate) taskHandle: Option<Task>,
}

impl Reactor {
	pub fn new() -> ReactorAlias {
		Arc::new(
			Mutex::new(
				Reactor {
					peers: Vec::new(),
					taskHandle: None,
				}
			)
		)
	}
}

pub struct ArcReactor<S>
where
		S: 'static + Send + Sync + Service<Request=hyper::Request, Response=hyper::Response, Error=hyper::Error>,
{
	port: i16,
	timeout: i8,
	RouteService: Option<S>
}

impl<S> ArcReactor<S>
where
		S: 'static + Send + Sync + Service<Request=hyper::Request, Response=hyper::Response, Error=hyper::Error>,
{
	pub fn new() -> ArcReactor<S> {
		ArcReactor {
			port: 8080,
			timeout: 10,
			RouteService: None
		}
	}

	pub fn router() -> ArcRouter {
		ArcRouter::new()
	}

	pub fn routeGroup(parent: &'static str) -> RouteGroup {
			RouteGroup::new(parent)
		}

	pub fn port (mut self, port: i16) -> Self {
		self.port = port;

		self
	}

	pub fn routes(mut self, routes: S) -> Self {
		self.RouteService = Some(routes);

		self
	}
	
	pub fn initiate (self) -> io::Result<()> {
		let reactors = spawn(self.RouteService.expect("This thing needs routes to work!"))?;
		
		let mut core = Core::new()?;
		let handle = core.handle();
		
		let addr = format!("0.0.0.0:{}", &self.port).parse().unwrap();
		
		let listener = match TcpListener::bind(&addr, &handle) {
			Ok(listener) => {
				listener
			}
			Err(e) => {
				eprintln!("Whoops! something else is running on port {}, {}", &self.port, e);
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
}

fn spawn<S>(RouteService: S) -> io::Result<Vec<ReactorAlias>>
where
		S: 'static + Send + Sync + Service<Request=hyper::Request, Response=hyper::Response, Error=hyper::Error>,
{
	let mut reactors = Vec::new();
	let routeService = Arc::new(RouteService);
	
	for _ in 0..num_cpus::get() * 2 {
		let reactor = Reactor::new();
		reactors.push(reactor.clone());
		let routeService = routeService.clone();

		thread::spawn(move || {
			let mut core = Core::new().expect("Could not start event loop");
			let handle = core.handle();
			let http: Http<Chunk> = Http::new();

			core.run(ReactorHandler {
				handler: || {
					let mut reactor = reactor.lock().unwrap();
					for (socket, _peerAddr) in reactor.peers.drain(..) {
						let future = http.serve_connection(socket, routeService.clone())
							.map(|_| ())
							.map_err(|_| ());
						handle.spawn(future);
					}
					reactor.taskHandle = Some(task::current());
				},
			}).expect("Could not spawn threads!");
		});
	}

	Ok(reactors)
}
