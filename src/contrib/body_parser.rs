use core::{Request, Response};
use futures::prelude::{async_block, await};
use futures::{Future, Stream};
use hyper::header::ContentType;
use impl_service::middleware;
use proto::MiddleWare;

/// ! Parses the request body as json if the content-type header is set.
/// ! Note that if there are any errors in parsing the json
/// ! it will return an error response.
#[middleware(Request)]
pub fn bodyParser(mut req: Request) {
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
