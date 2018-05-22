//! Parses the request body as json if the content-type header is set.
//! Note that if there are any errors in parsing the json
//! it will return an error response.
use core::Request;
use futures::{
	Stream,
	IntoFuture,
	Future
};
use hyper::{self, header::ContentType};
use proto::{MiddleWare, MiddleWareFuture};
use std::ops::Deref;

pub(crate) struct Json(hyper::Chunk);

impl Deref for Json {
	type Target = hyper::Chunk;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, Debug)]
pub struct BodyParser;

impl MiddleWare<Request> for BodyParser {
	fn call(&self, mut req: Request) -> MiddleWareFuture<Request> {
		let mut isJson = false;
		{
			if let Some(ct) = req.headers.get::<ContentType>() {
				isJson = *ct == ContentType::json();
			}
		}

		if !isJson {
			return Box::new(Ok(req).into_future())
		}

		let body = req.body();

		let future = body.concat2().then(|result| {
			let json = match result {
				Ok(chunk) => {
					// assert that the chunk length is > 0
					// otherwise bad request.
					if chunk.len() == 0 {
						let error = json!({ "error": "Empty request body" });
						return Err((400, error).into());
					}
					Json(chunk)
				}
				Err(_) => {
					let error = json!({
						"error": "Could not read request payload"
					});
					return Err((400, error).into());
				}
			};

			req.set(json);
			Ok(req)
		});
		
		return Box::new(future);
	}
}
