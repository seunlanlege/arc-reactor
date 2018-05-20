use core::{file, Request, Response, State};
use futures::{future::lazy, prelude::*, sync::oneshot::channel};
use hyper::{header::{ContentLength}, Method, StatusCode};
use proto::{MiddleWare, MiddleWareFuture};
use std::{fmt, path::PathBuf};
use tokio::{fs::File, io::ErrorKind};
use POOL;

/// A static File Server implemented as a Middleware<Request>
#[derive(Clone)]
pub struct StaticFileServer {
	pub root: &'static str,
	pub public: PathBuf,
}

impl StaticFileServer {
	pub fn new(root: &'static str, public: PathBuf) -> Self {
		Self { root, public }
	}
}

impl MiddleWare<Request> for StaticFileServer {
	fn call(&self, req: Request) -> MiddleWareFuture<Request> {
		let prefix = {
			req.path()
				.get(1..=self.root.len())
				.clone()
				.map(String::from)
		};

		if prefix == Some(self.root.to_string()) {
			// supported http-methods
			let method = { req.method().clone() };
			if method != Method::Get && method != Method::Head {
				return Box::new(Ok(req).into_future());
			}

			let mut pathbuf = self.public.clone();
			pathbuf.push(req.path().get(2 + self.root.len()..).unwrap());

			match method {
				Method::Get => {
					// if a MiddleWare<T> returns Err(Response)
					// that reponse is forwarded directly to the client.
					return Box::new(
						Response::new()
							.with_handle(req.reactor_handle())
							.with_file(pathbuf)
							.then(|res| {
								match res {
									Ok(res) | Err(res) => Err(res),
								}
							}),
					);
				}
				Method::Head => {
					let (snd, rec) = channel::<State>();
					let future = lazy(move || {
						File::open(pathbuf.clone()).and_then(file::metadata).then(|result| {
							match result {
								Ok((_, meta)) => {
									snd.send(State::Len(meta.len())).unwrap();
									Ok(())
								}
								Err(err) => {
									println!("Aha! Error! {}", err);
									match err.kind() {
										ErrorKind::NotFound => snd.send(State::NotFound).unwrap(),
										_ => snd.send(State::__Exhaustive).unwrap(),
									};
									Err(())
								}
							}
						})
					});

					POOL.sender().spawn(future).unwrap();
					let future = rec.then(|result| {
						match result {
							Ok(state) => {
								match state {
									State::Len(len) => {
										let mut res = Response::new();
										res.headers_mut().set(ContentLength(len));
										// res.headers_mut().set(ContentType(guess_mime_type(pathbuf)));
										return Err(res);
									}
									State::NotFound => {
										let mut res = Response::new();
										res.set_status(StatusCode::NotFound);
										return Err(res);
									}
									State::__Exhaustive => {
										let mut res = Response::new();
										res.set_status(StatusCode::InternalServerError);
										return Err(res);
									}
								}
							}
							_ => {
								let mut res = Response::new();
								res.set_status(StatusCode::InternalServerError);
								return Err(res);
							}
						}
					});

					return Box::new(future);
				}
				_ => {}
			}
		}

		Box::new(Ok(req).into_future())
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
