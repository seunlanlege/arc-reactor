use hyper;
use futures::future::{self, Future};
use hyper::server::{Request, Response, Service};
use hyper::{StatusCode, Headers, Method};
use hyper::header::ContentType;
use std::thread;
use std::time::Duration;

pub struct AsyncService;

impl Service for AsyncService {
	type Request = Request;
	type Response = Response;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Request) -> Self::Future {
		use self::Method::*;
		let mut headers = Headers::new();
    headers.set(ContentType::html());

		match (req.method(), req.path()) {
			(&Get, "/") => {
				future::ok(Response::new()
					.with_status(StatusCode::Ok)
					.with_headers(headers)
					.with_body("<html><h1>Nathaniel is learning rust lang!</h1></html>")
				).boxed()
			},

			(&Get, "/sleep") => {
				thread::sleep(Duration::from_secs(20));
				future::ok(Response::new()
					.with_status(StatusCode::Ok)
					.with_headers(headers)
					.with_body("<html><h1>Slept for 20 seconds</h1></html>")
				).boxed()
			},
			_ => {
				future::ok(Response::new()
					.with_status(StatusCode::NotFound)
					.with_headers(headers)
					.with_body("<html><h1>Not Found</h1></html>")
				).boxed()
			}
		}
	}
}
