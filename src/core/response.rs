use core::file;
use futures::{future::lazy, prelude::*, sync::oneshot::channel};
use http::response::Parts;
use hyper::{
	self,
	header::{HeaderMap, HeaderValue, CONTENT_LENGTH, CONTENT_TYPE, LOCATION},
	Body,
	StatusCode,
	Version,
};
use mime_guess::guess_mime_type;
use std::path::Path;
use tokio::{fs::File, io::ErrorKind};
use POOL;

#[derive(Debug)]
pub struct Response {
	pub(crate) parts: Parts,
	pub(crate) body: Body,
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
	pub fn version(&self) -> Version {
		self.parts.version
	}

	/// Get the headers from the response.
	#[inline]
	pub fn headers(&self) -> &HeaderMap<HeaderValue> {
		&self.parts.headers
	}

	/// Get a mutable reference to the headers.
	#[inline]
	pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
		&mut self.parts.headers
	}

	/// Get the status from the server.
	#[inline]
	pub fn status(&self) -> StatusCode {
		self.parts.status
	}

	pub fn status_mut(&mut self) -> &mut StatusCode {
		&mut self.parts.status
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
	pub fn set_status(&mut self, status: u16) {
		self.parts.status = StatusCode::from_u16(status).unwrap();
	}

	/// Set the status and move the Response.
	///
	/// Useful for the "builder-style" pattern.
	#[inline]
	pub fn with_status(mut self, status: u16) -> Self {
		self.parts.status = StatusCode::from_u16(status).unwrap();
		self
	}

	/// Set a text/plain response.
	#[inline]
	pub fn text<T: Into<String>>(&mut self, body: T) {
		let body = body.into();
		self.headers_mut().insert(
			CONTENT_LENGTH,
			HeaderValue::from_str(&body.len().to_string()).unwrap(),
		);
		self.body = Body::from(body.into());
	}

	/// set a text/plain response
	/// moves the response
	///
	/// useful for builder-style
	#[inline]
	pub fn with_text<T: Into<String>>(mut self, body: T) -> Self {
		self.text(body);

		self
	}

	/// get a reference to a type previously set on the response
	pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
		self.parts.extensions.get::<T>()
	}

	/// Set a type on the response.
	pub fn set<T: Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
		self.parts.extensions.insert::<T>(value)
	}

	/// Removes the type previously set on the response.
	pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
		self.parts.extensions.remove::<T>()
	}

	/// respond with a file
	/// sets the appropriate Content-type and Content-Length headers
	/// unfortunately, this doesn't support byte ranges, yet.
	/// The file is streamed asynchronously from the filesystem to the client
	/// Content-Encoding: chunked.
	pub fn with_file<P>(mut self, pathbuf: P) -> impl Future<Item = Response, Error = Response>
	where
		P: AsRef<Path> + Send + Clone + 'static,
	{
		let (sender, body) = Body::channel();
		self.body = body;

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
								.map_err(|err| println!("whoops filestream error occured {}", err))
								.for_each(|chunk| {
									if sender.poll_ready().is_ok() {
										sender.send_data(chunk);
									}

									Ok(())
								});

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
							self.headers_mut().insert(
								CONTENT_LENGTH,
								HeaderValue::from_str(&len.to_string()).unwrap(),
							);
							self.headers_mut().insert(
								CONTENT_TYPE,
								HeaderValue::from_str(mime_type.as_ref()).unwrap(),
							);
							return Ok(self);
						}
						State::NotFound => {
							self.set_status(404);
						}
						State::__Exhaustive => self.set_status(500),
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
		self.body = body.into();
		self
	}

	#[inline]
	pub fn set_body<T: Into<Body>>(&mut self, body: T) {
		self.body = body.into();
	}

	/// Read the body.
	#[inline]
	pub fn body_ref(&self) -> &Body {
		&self.body
	}

	/// Take the `Body` of this response.
	#[inline]
	pub fn body(self) -> Body {
		self.body
	}

	/// Set a HTTP redirect on the response header
	pub fn redirect(self, url: &'static str) -> Response {
		self.with_status(301)
			.headers_mut()
			.insert(LOCATION, HeaderValue::from_static(url));

		self
	}

	/// Set the status code 401 on the response
	pub fn unauthorized(self) -> Self {
		self.with_status(401)
	}

	/// Set the status code 400 on the response
	pub fn badRequest(self) -> Self {
		self.with_status(400)
	}

	/// Set the status code 403 on the response
	pub fn forbidden(self) -> Self {
		self.with_status(403)
	}

	/// Set the status code 405 on the response
	pub fn methodNotAllowed(self) -> Self {
		self.with_status(405)
	}

	/// Set the status code 406 on the response
	pub fn notAcceptable(self) -> Self {
		self.with_status(406)
	}

	/// Set the status code 408 on the response
	pub fn requestTimeout(self) -> Self {
		self.with_status(408)
	}

	/// Set the status code 500 on the response
	pub fn internalServerError(self) -> Self {
		self.with_status(500)
	}

	/// Set the status code 502 on the response
	pub fn badGateway(self) -> Self {
		self.with_status(502)
	}

	/// Set the status code 503 on the response
	pub fn serviceUnavailable(self) -> Self {
		self.with_status(503)
	}
}

impl Default for Response {
	fn default() -> Response {
		let (parts, body) = hyper::Response::new(Body::empty()).into_parts();
		Response { parts, body }
	}
}

pub fn res() -> Response {
	Response::default()
}
