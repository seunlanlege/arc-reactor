use futures::Stream;
use std::io;
use hyper;
use hyper::server::{Http, Service};
use tokio_core::reactor::Core;
use tokio_core::net::{TcpStream, TcpListener};
use ArcCore::{ReactorHandler};
use std::sync::{Arc, Mutex};
use std::marker::{Send, Sync};
use futures::task::{Task, self};
use std::net::SocketAddr;
use std::thread;

pub type ReactorAlias = Arc<Mutex<Reactor>>;

pub struct Reactor {
	pub peers: Vec<(TcpStream, SocketAddr)>,
	pub taskHandle: Option<Task>,
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
	S: 'static + Clone + Send + Sync + Service<Request=hyper::Request, Response=hyper::Response, Error=hyper::Error>,
{
	port: i16,
	RouteService: Option<S>
}

impl<S> ArcReactor<S>
where
	S: 'static + Clone + Send + Sync + Service<Request=hyper::Request, Response=hyper::Response, Error=hyper::Error>,
{
	pub fn new() -> ArcReactor<S> {
		ArcReactor {
			port: 8080,
			RouteService: None
		}
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
		let reactors = spawn(self.RouteService.expect("This thing needs routes to work!"));
		
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
		
		// TODO: Dispatch to threads with less connected clients?.

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

fn spawn<S>(RouteService: S) -> Vec<ReactorAlias>
	where
		S: 'static + Clone + Send + Sync + Service<Request=hyper::Request, Response=hyper::Response, Error=hyper::Error>,
{
	let mut reactors = Vec::new();
		
		// TODO: replace with call to num_cpus::get() * 2
	for _ in 1..5 {
			let reactor = Reactor::new();
			reactors.push(reactor.clone());
		let routeService = RouteService.clone();

			thread::spawn(move || {
				let mut core = Core::new().unwrap();
				let handle = core.handle();
				let http = Http::new();

				core.run(ReactorHandler {
					handler: || {
						let mut reactor = reactor.lock().unwrap();
						for (socket, peerAddr) in reactor.peers.drain(..) {
							http.bind_connection(&handle, socket, peerAddr, routeService.clone());
						}
						reactor.taskHandle = Some(task::current());
					},
				}).unwrap();
			});
	}

		reactors
	}
