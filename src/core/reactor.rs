use futures::{Sink, Stream};
use std::io;
use hyper::Chunk;
use hyper::server::Http;
use tokio_core::reactor::Core;
use tokio_core::net::{TcpListener, TcpStream};
use routing::{ArcRouter, Router};
use std::sync::{Arc};
use futures::prelude::{async, async_block, await};
use std::thread;
use native_tls::{Pkcs12, TlsAcceptor};
use crossbeam_channel::futures::mpmc::{channel, Sender};
use tokio_tls::{AcceptAsync, TlsAcceptorExt};
use num_cpus;

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
		let sender = spawn(self.routeService.expect("This thing needs routes to work!"))?;
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

		// for TLS support
		let der = include_bytes!("identity.p12");
		let cert = Pkcs12::from_der(der, "mypass").unwrap();
		let acceptor = TlsAcceptor::builder(cert).unwrap().build().unwrap();
		let acceptor = Arc::new(acceptor);
		println!("[arc-reactor]: Running Main Event loop");
		core.run(listener.incoming().for_each(move |(socket, _)| {
			let accept = acceptor.clone();
			let sink = sender.clone();
			let f = async_block! {
				let socket = accept.accept_async(socket);
				let _void = await!(sink.send(socket));
				Ok(())
			};
			handle.spawn(f);
			Ok(())
		}))?;

		Ok(())
	}
}

fn spawn(routeService: ArcRouter) -> io::Result<Sender<AcceptAsync<TcpStream>>> {
	let (sendr, recv) = channel::<AcceptAsync<TcpStream>>();
	let routeService = Arc::new(routeService);

	for _ in 0..num_cpus::get() * 2 {
		let stream = recv.clone();
		let routeService = routeService.clone();

		thread::spawn(move || {
			let mut core = Core::new().expect("Could not start event loop");
			let http = Http::new();
			let http = Arc::new(http);

			core
				.run(stream.for_each(|socket| socketHandler(socket, http.clone(), routeService.clone())))
				.expect("Error running reactor core!");
		});
	}

	Ok(sendr)
}

#[async]
fn socketHandler(
	tls_stream: AcceptAsync<TcpStream>,
	http: Arc<Http<Chunk>>,
	serviceHandler: Arc<ArcRouter>,
) -> Result<(), ()> {
	let stream = await!(tls_stream);

	if !stream.is_ok() {
		return Err(());
	}

	let _opaque = await!(http.serve_connection(stream.unwrap(), serviceHandler.clone()));
	Ok(())
}
