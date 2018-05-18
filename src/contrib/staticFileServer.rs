use core::{Request, Response};
use futures::prelude::*;
use hyper::Method;
use proto::{MiddleWare, MiddleWareFuture};
use std::{fmt, path::PathBuf};

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
		// supported http-methods
		let method = { req.method().clone() };
		if method != Method::Get && method != Method::Head {
			return Box::new(Ok(req).into_future());
		}

		match method {
			Method::Get => {
				let prefix = req.path().get(1..=self.root.len());

				if prefix == Some(self.root) {
					let mut pathbuf = self.public.clone();

					pathbuf.push(req.path().get(2 + self.root.len()..).unwrap());

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
			}
			Method::Head => {}
			_ => {}
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
