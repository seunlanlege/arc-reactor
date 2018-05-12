//! Parses the request body as json if the content-type header is set.
//! Note that if there are any errors in parsing the json
//! it will return an error response.
use core::Request;
use futures::prelude::{async_block, await};
use futures::Stream;
use hyper;
use hyper::header::ContentType;
use proto::MiddleWareFuture;
use std::ops::Deref;

pub(crate) struct Json(hyper::Chunk);

impl Deref for Json {
	type Target = hyper::Chunk;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub fn bodyParser(mut req: Request) -> MiddleWareFuture<Request> {
	let future = async_block! {
		let mut isJson = false;
		{
			if let Some(ct) = req.headers.get::<ContentType>() {
				isJson = *ct == ContentType::json();
			}
		}

		if isJson {
			// this unwrap is safe. check request.rs: line 50
			let body = req.body.take().unwrap();

			let json = match await!(body.concat2()) {
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
		}

		Ok(req)
	};

	Box::new(future)
}
