use hyper;
use hyper::header::{Header, Headers, Location};
use hyper::{Body, HttpVersion, StatusCode};

#[derive(Debug)]
pub struct Response {
	pub(crate) inner: hyper::Response,
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
	pub fn set_body<T: Into<Body>>(&mut self, body: T) {
		self.inner.set_body(body);
	}

	/// Set the body and move the Response.
	///
	/// Useful for the "builder-style" pattern.
	#[inline]
	pub fn with_body<T: Into<Body>>(mut self, body: T) -> Self {
		self.inner = self.inner.with_body(body);
		self
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

	pub fn redirectTo(url: &'static str) -> hyper::Response {
		let mut headers = Headers::new();
		headers.set(Location::new(url));
		res()
			.with_status(StatusCode::MovedPermanently)
			.with_headers(headers)
			.inner
	}

	pub fn redirect(self, url: &'static str) -> Response {
		let mut headers = Headers::new();
		headers.set(Location::new(url));
		self
			.with_status(StatusCode::MovedPermanently)
			.with_headers(headers)
	}

	pub fn ok(self) -> Self {
		self.with_status(StatusCode::Ok)
	}

	pub fn unauthorized(self) -> Self {
		self.with_status(StatusCode::Unauthorized)
	}

	pub fn forbidden(self) -> Self {
		self.with_status(StatusCode::Forbidden)
	}

	pub fn methodNotAllowed(self) -> Self {
		self.with_status(StatusCode::MethodNotAllowed)
	}

	pub fn notAcceptable(self) -> Self {
		self.with_status(StatusCode::NotAcceptable)
	}

	pub fn requestTimeout(self) -> Self {
		self.with_status(StatusCode::RequestTimeout)
	}

	pub fn internalServerError(self) -> Self {
		self.with_status(StatusCode::InternalServerError)
	}

	pub fn badGateway(self) -> Self {
		self.with_status(StatusCode::BadGateway)
	}

	pub fn serviceUnavailable(self) -> Self {
		self.with_status(StatusCode::ServiceUnavailable)
	}
}

impl Default for Response {
	fn default() -> Response {
		Response {
			inner: hyper::Response::default(),
		}
	}
}

pub fn res() -> Response {
	Response::default()
}
