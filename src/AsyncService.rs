use futures::future::{Future, self};

use hyper;
use hyper::server::{Request, Response, Service};
use hyper::{StatusCode, Headers};
use hyper::header::ContentType;

pub struct AsyncService;

impl Service for AsyncService {
		type Request = Request;
		type Response = Response;
		type Error = hyper::Error;
		type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

		fn call(&self, req: Request) -> Self::Future {
				let mut headers = Headers::new();
				headers.set(ContentType::html());

				future::ok(
						Response::new()
						.with_status(StatusCode::Ok)
						.with_headers(headers)
						.with_body("<html><h1>Nathaniel is learning rust lang!</h1></html>")
				).boxed()
		}
}