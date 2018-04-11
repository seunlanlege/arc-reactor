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
	///
	/// # Example
	///
	/// ```rust, ignore
	/// use hyper::StatusCode;
	///
	/// pub fn get_profile(req: Request, _res: Response) {
	/// 	// performed task on request
	///
	///   // return an Ok response
	/// 	_res.set_status(StatusCode::Unauthorized);
	/// }
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

	/// Set a HTTP redirect on the response header
	pub fn redirect(self, url: &'static str) -> Response {
		let mut headers = Headers::new();
		headers.set(Location::new(url));
		self
			.with_status(StatusCode::MovedPermanently)
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
		}
	}
}

pub fn res() -> Response {
	Response::default()
}
