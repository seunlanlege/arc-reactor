use futures::{Future, Stream};
use futures::prelude::{async_block, await};
use core::{Request, Response};
use proto::MiddleWare;
use impl_service::middleware;
use hyper::header::ContentType;

#[middleware(Request)]
pub fn body_parser(mut req: Request) {
	let mut isJson = false;
	{
		if let Some(ct) = req.headers.get::<ContentType>() {
			isJson = *ct == ContentType::json();
		}
	}

	if isJson {
		// this unwrap is safe. check request.rs: line 50
		let body = req.body.take().unwrap();

		let chunk = match await!(body.concat2()) {
			Ok(chunk) => chunk,
			Err(_) => {
				let error = json!({
					"error": "Could not read request payload"
				});

				return Err((400, error).into());
			}
		};

		// assert that the chunk length is > 0
		// otherwise bad request.
		if chunk.len() == 0 {
			let error = json!({ "error": "Empty request body" });
			return Err((400, error).into());
		}

		req.set(chunk);
	}

	Ok(req)
}
