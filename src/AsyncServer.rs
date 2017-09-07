use hyper::{self, StatusCode};
use hyper::server::{Request, Response, Service};
use hyper::{Method};
use futures::prelude::*;
use futures::future;

pub struct AsyncService;

impl Service for AsyncService {
	type Request = Request;
	type Response = Response;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Request) -> Self::Future {
		let response = Response::new();
		match (req.path(), req.method()) {
			("/", &Method::Post) => {
				Box::new(
					async_block! {
						let body = await!(req.body().concat2())?;
						println!("{:?}", String::from_utf8_lossy(&body.to_vec()));
						Ok(response.with_body(body))
					}
				)
			}

			_ => Box::new(future::ok(response.with_status(StatusCode::NotFound)))
		}
	}
}
