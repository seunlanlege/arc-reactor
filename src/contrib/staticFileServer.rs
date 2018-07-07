use core::{file, Request, Response};
use futures::{prelude::*, future};
use hyper::{
	header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
	Method,
};
use mime_guess::guess_mime_type;
use percent_encoding::percent_decode;
use proto::{MiddleWare, MiddleWareFuture};
use std::path::PathBuf;
use tokio::{fs::File, io::ErrorKind};

/// A static File Server implemented as a Middleware<Request>
#[derive(Clone, Debug)]
pub struct StaticFileServer {
	/// the root url for all your files. e.g `static`, `assets`
	/// request urls that begin with the supplied value for root would be
	/// matched.
	pub root: &'static str,
	/// Path to folder to serve your static files from.
	pub public: PathBuf,
}

impl StaticFileServer {
	/// Creates a StaticFileServer with the given
	/// root and pathbuf.
	pub fn new(root: &'static str, public: PathBuf) -> Self {
		Self { root, public }
	}
}

impl MiddleWare<Request> for StaticFileServer {
	fn call(&self, req: Request) -> MiddleWareFuture<Request> {
		let path = {
			req.path()
				.get(1..)
				.and_then(|path| {
					Some(
						percent_decode(path.as_ref())
							.decode_utf8_lossy()
							.into_owned(),
					)
				})
				.and_then(|path| {
					if path.contains("../") {
						None
					} else {
						Some(path)
					}
				})
		};

		let prefix = match path {
			Some(ref r) => r.get(..self.root.len()),
			None => None,
		};

		if prefix == Some(self.root) {
			// supported http-methods
			let method = { req.method().clone() };
			if method != Method::GET && method != Method::HEAD {
				return Box::new(future::ok(req));
			}

			let mut pathbuf = self.public.clone();
			if let Some(ref path) = path {
				if let Some(ref path) = path.get(self.root.len() + 1..) {
					pathbuf.push(path);
				}
			}
			if pathbuf.is_dir() {
				pathbuf.push("index.html");
			}

			let path_clone = pathbuf.clone();

			match method {
				Method::GET => {
					// if a MiddleWare<T> returns Err(Response)
					// that reponse is forwarded directly to the client.
					return Box::new(Response::new().with_file(pathbuf).then(|res| {
						match res {
							Ok(res) | Err(res) => Err(res),
						}
					}));
				}
				Method::HEAD => {
					let future = File::open(path_clone)
						.and_then(file::metadata)
						.then(|result| {
							match result {
								Ok((_, meta)) => {
									let mut res = Response::new();
									let mime_type = guess_mime_type(pathbuf);
									res.headers_mut().insert(
										CONTENT_LENGTH,
										HeaderValue::from_str(&meta.len().to_string()).unwrap(),
									);
									res.headers_mut().insert(
										CONTENT_TYPE,
										HeaderValue::from_str(mime_type.as_ref()).unwrap(),
									);
									return Err(res);
								}
								Err(err) => {
									error!("Error opening file: {}", err);
									match err.kind() {
										ErrorKind::NotFound => {
											let mut res = Response::new().with_status(404);
											return Err(res);
										}
										_ => {
											let mut res = Response::new().with_status(500);
											return Err(res);
										}
									}
								}
							}
						});

					return Box::new(future);
				}
				_ => {}
			}
		}

		Box::new(future::ok(req))
	}
}
