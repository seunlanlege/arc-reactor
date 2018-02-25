use futures::Stream;
use std::io;
use hyper::Chunk;
use hyper::server::Http;
use tokio_core::reactor::Core;
use tokio_core::net::{TcpListener, TcpStream};
use ArcCore::ReactorHandler;
use ArcRouting::{ArcRouter, Router};
use std::sync::{Arc, Mutex};
use futures::prelude::{async, await};
use futures::task::Task;
use std::net::SocketAddr;
use std::thread;
use native_tls::{Pkcs12, TlsAcceptor};
use crossbeam_channel::futures::mpsc::{channel, Receiver, Sender};
use tokio_tls::{AcceptAsync, TlsAcceptorExt};
use num_cpus;

pub type ReactorAlias = Arc<Mutex<Reactor>>;

pub struct Reactor {
	pub(crate) peers: Vec<(AcceptAsync<TcpStream>, SocketAddr)>,
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
	timeout: i8,
	RouteService: Option<ArcRouter>,
}

impl ArcReactor {
	pub fn new() -> ArcReactor {
		ArcReactor {
			port: 8080,
			timeout: 10,
			RouteService: None,
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
		self.RouteService = Some(routes);

		self
	}

	pub fn initiate(self) -> io::Result<()> {
		println!("[arc-reactor]: Spawning threads!");
		let mut sender = spawn(self.RouteService.expect("This thing needs routes to work!"))?;
		println!("[arc-reactor]: Starting main event loop!");
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

		// for TLS support
		let der = include_bytes!("identity.p12");
		let cert = Pkcs12::from_der(der, "mypass").unwrap();
		let acceptor = TlsAcceptor::builder(cert).unwrap().build().unwrap();
		println!("[arc-reactor]: Running Main Event loop");
		core.run(listener.incoming().for_each(move |(socket, peerIp)| {
			let socket = acceptor.accept_async(socket);
			let _void = match sender.try_send((socket, peerIp)) {
				Ok(_) => println!("Sent!"),
				Err(e) => println!("Lol, couldn't send {:?}", e),
			};
			Ok(())
		}))?;

		Ok(())
	}
}

fn spawn(RouteService: ArcRouter) -> io::Result<Sender<(AcceptAsync<TcpStream>, SocketAddr)>> {
	let (sendr, recv) = channel::<(AcceptAsync<TcpStream>, SocketAddr)>(4);

	let mut reactors = Vec::new();
	let routeService = Arc::new(RouteService);

	for _ in 0..num_cpus::get() * 2 {
		let stream = recv.clone();
		let reactor = Reactor::new();
		reactors.push(reactor.clone());
		let routeService = routeService.clone();

		thread::spawn(move || {
			let mut core = Core::new().expect("Could not start event loop");
			let handle = core.handle();
			let http: Http<Chunk> = Http::new();
			let http = Arc::new(http);

			core
				.run(ReactorHandler {
					handler: || {
						let future = futureFactory(stream.clone(), http.clone(), routeService.clone());
						handle.spawn(future);
					},
				})
				.expect("Error running reactor core!");
		});
	}

	Ok(sendr)
}

#[async]
fn futureFactory(
	streamFuture: Receiver<(AcceptAsync<TcpStream>, SocketAddr)>,
	http: Arc<Http<Chunk>>,
	serviceHandler: Arc<ArcRouter>,
) -> Result<(), ()> {
	#[async]
	for (socket, _) in streamFuture {
		println!("reading!");
		let stream = await!(socket);
		if !stream.is_ok() {
			continue;
		}
		let _opaque = await!(http.serve_connection(stream.unwrap(), serviceHandler.clone()));
	}
	Ok(())
}
