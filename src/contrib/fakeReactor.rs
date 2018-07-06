use core::{Request, Response};
use http::request::Builder;
use hyper::{Body, HeaderMap, Method};
use proto::ArcService;
use routing::Router;
use serde::ser::Serialize;
use serde_json::to_vec;
use tokio::runtime::Runtime;

/// Fake reactor allows for testing your application's endpoints.
///
/// Do note that Ip addresses won't be present on the request struct
/// when testing for obvious reasons.
pub struct FakeReactor {
	pub routes: Router,
}

impl FakeReactor {
	/// Post a request to the fake reactor, and either
	/// returns a `Result<Response, Response>`
	/// or panics if the route wasn't found.
	pub fn post<T>(
		&self,
		route: &str,
		body: Option<T>,
		headers: Option<HeaderMap>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		self.build(Method::POST, route, body, headers)
	}

	/// Send a GET request to the `FakeReactor`.
	pub fn get(&self, route: &str, headers: Option<HeaderMap>) -> Result<Response, Response> {
		self.build(Method::GET, route, None::<u8>, headers)
	}

	/// Send a PUT request to the `FakeReactor`.
	pub fn put<T>(
		&self,
		route: &str,
		body: Option<T>,
		headers: Option<HeaderMap>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		self.build(Method::PUT, route, body, headers)
	}

	/// Send a PATCH request to the `FakeReactor`.
	pub fn patch<T>(
		&self,
		route: &str,
		body: Option<T>,
		headers: Option<HeaderMap>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		self.build(Method::PATCH, route, body, headers)
	}

	/// Send a DELETE request to the `FakeReactor`.
	pub fn delete<T>(
		&self,
		route: &str,
		body: Option<T>,
		headers: Option<HeaderMap>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		self.build(Method::DELETE, route, body, headers)
	}

	fn build<T>(
		&self,
		method: Method,
		route: &str,
		body: Option<T>,
		mut headers: Option<HeaderMap>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		let body = match body {
			Some(b) => to_vec(&b).unwrap().into(),
			None => Body::default(),
		};

		if headers.is_none() {
			headers = Some(HeaderMap::new());
		}

		let headers = headers.unwrap();
		let mut reactor = Runtime::new().expect("Could not start event loop");

		let mut request = Builder::new()
			.method(method.clone())
			.uri(route)
			.body(body)
			.unwrap();
		*request.headers_mut() = headers;
		let req: Request = request.into();

		let matched = self
			.routes
			.matchRoute(req.path(), &method)
			.expect(&format!("No Handler registered for Method::{}", method));

		return reactor.block_on(ArcService::call(matched.handler, req, Response::new()));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use core::{Request, Response};
	use futures::{future, Future, Stream};
	use proto::FutureResponse;
	use routing::*;

	fn AsyncService(_req: Request, res: Response) -> FutureResponse {
		let res = res
			.with_status(200)
			.with_body("Hello World".as_bytes());

		Box::new(future::ok(res))
	}

	#[test]
	fn it_matches_the_correct_route_and_returns_the_correct_body() {
		let routes = Router::new().get("/hello", AsyncService);

		let fakereactor = FakeReactor { routes };

		// Assert it returns Ok.
		let result = fakereactor
			.get("/hello?lol=p", None)
			.expect("Should return ok");
		let body = result.body().concat2().wait().expect("Body should be Ok");
		assert_eq!(&body[..], b"Hello World");
	}
}
