use core::file;
use futures::{future::lazy, prelude::*, sync::oneshot::channel};
use hyper::{
	self,
	header::{ContentLength, ContentType, Header, Headers, Location},
	Body,
	HttpVersion,
	StatusCode,
};
use std::path::Path;
use tokio::{fs::File, io::ErrorKind};
use tokio_core::reactor::Handle;
use POOL;
use mime_guess::guess_mime_type;

#[derive(Debug)]
pub struct Response {
	pub(crate) inner: hyper::Response,
	pub(crate) handle: Option<Handle>,
}

#[derive(Debug)]
pub(crate) enum State {
	Len(u64),
	NotFound,
	__Exhaustive,
}

impl Response {
	pub fn new() -> Self {
		Response::default()
	}

	/// Get the HTTP version of this response.
	#[inline]
	pub fn version(&self) -> HttpVersion {
		self.inner.version()
	}

	/// Get the headers from the response.
	#[inline]
	pub fn headers(&self) -> &Headers {
		self.inner.headers()
	}

	/// Get a mutable reference to the headers.
	#[inline]
	pub fn headers_mut(&mut self) -> &mut Headers {
		self.inner.headers_mut()
	}

	/// Get the status from the server.
	#[inline]
	pub fn status(&self) -> StatusCode {
		self.inner.status()
	}

	/// Set the `StatusCode` for this response.
	///
	/// # Example
	///
	/// ```rust, ignore
	/// use hyper::StatusCode;
	///
	/// pub fn get_profile(req: Request, _res: Response) {
	/// 	// performed task on request
	///
	/// 	// return an Ok response
	/// 	_res.set_status(StatusCode::Unauthorized);
	/// 	}
	/// ```
	///
	#[inline]
	pub fn set_status(&mut self, status: StatusCode) {
		self.inner.set_status(status);
	}

	/// Set the status and move the Response.
	///
	/// Useful for the "builder-style" pattern.
	#[inline]
	pub fn with_status(mut self, status: StatusCode) -> Self {
		self.inner = self.inner.with_status(status);
		self
	}

	/// Set a header and move the Response.
	///
	/// Useful for the "builder-style" pattern.
	#[inline]
	pub fn with_header<H: Header>(mut self, header: H) -> Self {
		self.inner = self.inner.with_header(header);
		self
	}

	/// Set the headers and move the Response.
	///
	/// Useful for the "builder-style" pattern.
	#[inline]
	pub fn with_headers(mut self, headers: Headers) -> Self {
		self.inner = self.inner.with_headers(headers);
		self
	}

	/// Set the body.
	#[inline]
	pub fn text<T: Into<String>>(&mut self, body: T) {
		let body = body.into();
		self.inner
			.headers_mut()
			.set(ContentLength(body.len() as u64));
		self.inner.set_body(body);
		self.inner.headers_mut().set(ContentType::plaintext());
	}

	#[inline]
	pub fn with_text<T: Into<String>>(mut self, body: T) -> Self {
		self.text(body);

		self
	}

	pub fn with_file<P>(mut self, pathbuf: P) -> impl Future<Item = Response, Error = Response>
	where
		P: AsRef<Path> + Send + Clone + 'static,
	{
		let (sender, recv) = Body::pair();
		self.inner.set_body(recv);

		let (snd, rec) = channel::<State>();
		let path_clone = pathbuf.clone();

		// this future is spawned on the tokio ThreadPool executor
		let future = lazy(|| {
			File::open(pathbuf)
				.and_then(file::metadata)
				.then(|result| {
					match result {
						Ok((file, meta)) => {
							snd.send(State::Len(meta.len())).unwrap();
							let stream = file::stream(file);
							let future = stream
								.map(Ok)
								.map_err(|err| println!("whoops filestream error occured {}", err))
								.forward(sender.sink_map_err(|err| {
									println!("whoops filestream error occured {}", err)
								}))
								.then(|_| Ok(()));

							Ok(future)
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
				.and_then(|f| f)
		});

		// attempt to spawn future on tokio threadpool
		POOL.sender().spawn(future).unwrap();

		rec.then(move |len| {
			match len {
				Ok(state) => {
					match state {
						State::Len(len) => {
							let mime_type = guess_mime_type(path_clone);
							self.headers_mut().set(ContentLength(len));
							self.headers_mut().set(ContentType(mime_type));
							return Ok(self);
						}
						State::NotFound => {
							self.set_status(StatusCode::NotFound);
						}
						State::__Exhaustive => self.set_status(StatusCode::InternalServerError),
					}
				}
				_ => {}
			}

			Ok(self)
		})
	}

	/// Set the body and move the Response.
	///
	/// Useful for the "builder-style" pattern.
	#[inline]
	pub fn with_body<T: Into<Body>>(mut self, body: T) -> Self {
		self.inner = self.inner.with_body(body);
		self
	}

	#[inline]
	pub fn set_body<T: Into<Body>>(&mut self, body: T) {
		self.inner.set_body(body);
	}

	/// Read the body.
	#[inline]
	pub fn body_ref(&self) -> Option<&Body> {
		self.inner.body_ref()
	}

	/// Take the `Body` of this response.
	#[inline]
	pub fn body(self) -> Body {
		self.inner.body()
	}

	/// Set a HTTP redirect on the response header
	pub fn redirect(self, url: &'static str) -> Response {
		let mut headers = Headers::new();
		headers.set(Location::new(url));
		self.with_status(StatusCode::MovedPermanently)
			.with_headers(headers)
	}

	/// Set the status code 200 on the response
	pub fn ok(self) -> Self {
		self.with_status(StatusCode::Ok)
	}

	/// Set the status code 401 on the response
	pub fn unauthorized(self) -> Self {
		self.with_status(StatusCode::Unauthorized)
	}

	/// Set the status code 403 on the response
	pub fn forbidden(self) -> Self {
		self.with_status(StatusCode::Forbidden)
	}

	/// Set the status code 405 on the response
	pub fn methodNotAllowed(self) -> Self {
		self.with_status(StatusCode::MethodNotAllowed)
	}

	/// Set the status code 406 on the response
	pub fn notAcceptable(self) -> Self {
		self.with_status(StatusCode::NotAcceptable)
	}

	/// Set the status code 408 on the response
	pub fn requestTimeout(self) -> Self {
		self.with_status(StatusCode::RequestTimeout)
	}

	/// Set the status code 500 on the response
	pub fn internalServerError(self) -> Self {
		self.with_status(StatusCode::InternalServerError)
	}

	/// Set the status code 502 on the response
	pub fn badGateway(self) -> Self {
		self.with_status(StatusCode::BadGateway)
	}

	/// Set the status code 503 on the response
	pub fn serviceUnavailable(self) -> Self {
		self.with_status(StatusCode::ServiceUnavailable)
	}
}

impl Default for Response {
	fn default() -> Response {
		Response {
			inner: hyper::Response::default(),
			handle: None,
		}
	}
}

pub fn res() -> Response {
	Response::default()
}
