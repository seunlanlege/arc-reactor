use routing::Router;
use core::{Request, Response};
use proto::ArcService;
use hyper::{Body, Headers, HttpVersion, Method, Uri};
use serde::ser::Serialize;
use anymap::AnyMap;
use std::str::FromStr;
use serde_json::to_vec;
use tokio_core::reactor::Core;

/// Fake reactor allows for testing your application's endpoints
///
/// Do note that Ip addresses won't be present on the request struct when testing for obvious reasons
pub struct FakeReactor {
	pub routes: Router,
}

impl FakeReactor {
	/// post a request to the fake reactor,
	/// returns a `Result<Response, Response>`
	/// or panics if the route wasn't found
	pub fn post<T>(
		&self,
		route: &str,
		body: Option<T>,
		headers: Option<Headers>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		self.build(Method::Post, route, body, headers)
	}

	/// send a get request to the `FakeReactor`
	pub fn get(&self, route: &str, headers: Option<Headers>) -> Result<Response, Response> {
		self.build(Method::Get, route, None::<u8>, headers)
	}

	/// send a put request to the `FakeReactor`
	pub fn put<T>(
		&self,
		route: &str,
		body: Option<T>,
		headers: Option<Headers>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		self.build(Method::Put, route, body, headers)
	}

	/// send a patch request to the `FakeReactor`
	pub fn patch<T>(
		&self,
		route: &str,
		body: Option<T>,
		headers: Option<Headers>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		self.build(Method::Patch, route, body, headers)
	}

	/// send a delete request to the `FakeReactor`
	pub fn delete<T>(
		&self,
		route: &str,
		body: Option<T>,
		headers: Option<Headers>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		self.build(Method::Delete, route, body, headers)
	}

	 fn build<T>(
		&self,
		method: Method,
		route: &str,
		body: Option<T>,
		mut headers: Option<Headers>,
	) -> Result<Response, Response>
	where
		T: Serialize,
	{
		let body = match body {
			Some(b) => Some(to_vec(&b).unwrap().into()),
			None => Some(Body::default()),
		};

		if headers.is_none() {
			headers = Some(Headers::default());
		}

		let headers = headers.unwrap();
		let mut reactor = Core::new().expect("Could not start event loop");

		let req = Request {
			method: method.clone(),
			uri: Uri::from_str(route).unwrap(),
			version: HttpVersion::Http11,
			headers,
			body,
			remote: None,
			anyMap: AnyMap::new(),
			handle: Some(reactor.handle()),
		};

		let matched = self
			.routes
			.matchRoute(req.path(), &method)
			.expect(&format!("No Handler registered for Method::{}", method));

		return reactor.run(ArcService::call(matched.handler, req, Response::new()));
	}
}

#[cfg(test)]
mod tests {
	use impl_service::*;
	use hyper::StatusCode;
	use futures::{Future, Stream};
	use futures::prelude::async_block;
	use proto::ArcService;
	use core::{Request, Response};
	use routing::*;
	use super::*;

	#[service]
	fn AsyncService(_req: Request, res: Response) {
		let res = res
			.with_status(StatusCode::Ok)
			.with_body("Hello World".as_bytes());
		Result::Ok(res)
	}

	#[test]
	fn it_matches_the_correct_route_and_returns_the_correct_body() {
		let routes = Router::new().get("/hello", AsyncService);

		let fakereactor = FakeReactor { routes };

		// assert it returns Ok
		let result = fakereactor
			.get("/hello?lol=p", None)
			.expect("Should return ok");
		let body = result.body().concat2().wait().expect("Body should be Ok");
		assert_eq!(&body[..], b"Hello World");
	}
}
