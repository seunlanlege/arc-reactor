use core::{Request, Response};
use futures::{future::lazy, prelude::*};
use hyper::{Body, Chunk, Method};
use proto::{MiddleWare, MiddleWareFuture};
use std::{
	cell::UnsafeCell,
	fmt,
	marker::Sync,
	path::PathBuf,
	sync::{Arc, Once, ONCE_INIT},
};
use tokio::{
	executor::thread_pool::{Sender, ThreadPool},
	fs::File,
	io::{AsyncRead, Error},
};

static INIT: Once = ONCE_INIT;

lazy_static! {
	pub static ref POOL: ThreadPool = { ThreadPool::new() };
}

/// A static File Server implemented as a Middleware<Request>
#[derive(Clone)]
pub struct StaticFileServer {
	pub root: &'static str,
	pub public: PathBuf,
	sender: Arc<Alias>,
}

struct Alias(UnsafeCell<Option<Sender>>);

impl StaticFileServer {
	pub fn new(root: &'static str, public: PathBuf) -> Self {
		Self {
			root,
			public,
			sender: Arc::new(Alias(UnsafeCell::new(None))),
		}
	}
}

impl MiddleWare<Request> for StaticFileServer {
	fn call(&self, req: Request) -> MiddleWareFuture<Request> {
		// supported http-methods		
		if *req.method() != Method::Get && *req.method() != Method::Head {
			return Box::new(Ok(req).into_future())
		}

		let f_send = self.sender.0.get();

		// set the sender to the threadpool once.
		INIT.call_once(|| {
			unsafe {
				*f_send = Some(POOL.sender().clone());
			};
		});
		
		{
			let prefix = req.path().get(1..=self.root.len());

			if prefix == Some(self.root) {
				let mut pathbuf = self.public.clone();

				pathbuf.push(req.path().get(2 + self.root.len()..).unwrap());
				let (sender, recv) = Body::pair();

				// this future is spawned on the tokio ThreadPool executor
				let future = lazy(|| {
					File::open(pathbuf)
						.and_then(|file| {
							let stream = FileStream::new(file);
							let future = stream
								.map(Ok)
								.map_err(|err| println!("whoops filestream error occured {}", err))
								.forward(sender.sink_map_err(|err| {
									println!("whoops filestream error occured {}", err)
								}))
								.then(|_| Ok(()));

							future
						})
						.map_err(|err| println!("Aha! Error! {}", err))
				});

				// attempt to spawn future on tokio threadpool
				unsafe {
					match *f_send {
						Some(ref f_send) => f_send.spawn(future).unwrap(),
						_ => {}
					};
				};

				// if a MiddleWare<T> returns Err(Response)
				// that reponse is forwarded directly to the client.
				return Box::new(Err(Response::new().with_body(recv)).into_future());
			}
		}

		Box::new(Ok(req).into_future())
	}
}

/// wraps a tokio::fs::File as a futures::Stream
/// will produce an error if this stream isn't polled in the context of a tokio
/// executor
pub struct FileStream {
	file: File,
	buf: [u8; 4096],
}

impl FileStream {
	pub fn new(file: File) -> Self {
		Self {
			file,
			buf: [0; 4096],
		}
	}
}

impl Stream for FileStream {
	type Item = Chunk;
	type Error = Error;

	fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		let n_bytes = try_ready!(self.file.poll_read(&mut self.buf));
		if n_bytes > 0 {
			Ok(Async::Ready(Some(Chunk::from(self.buf.to_vec()))))
		} else {
			Ok(Async::Ready(None))
		}
	}
}

impl fmt::Debug for StaticFileServer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("StaticFileServer")
			.field("root", &self.root)
			.field("public", &self.public)
			.finish()
	}
}

unsafe impl Sync for Alias {}
