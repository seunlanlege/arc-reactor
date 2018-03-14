use hyper::{self, StatusCode};
use serde_json::to_string;
use core::{res, Request, Response, JsonError};

fn toStatusCode(number: u16) -> StatusCode {
	match StatusCode::try_from(number) {
		Ok(status) => status,
		Err(_) => StatusCode::BadRequest,
	}
}

impl From<(u16, &'static str)> for Response {
	fn from(tuple: (u16, &'static str)) -> Response {
		res().with_status(toStatusCode(tuple.0)).with_body(tuple.1)
	}
}

impl From<Response> for hyper::Response {
	fn from(res: Response) -> hyper::Response {
		res.inner
	}
}

impl From<hyper::Request> for Request {
	fn from(req: hyper::Request) -> Request {
		let (method, uri, version, headers, body) = req.deconstruct();

		let request = Request::new(method, uri, version, headers, body);

		request
	}
}

impl From<JsonError> for Response {
	fn from(error: JsonError) -> Response {
		match error {
			JsonError::None => {
				let json = json!({
					"error": "Json was empty",
				});
				res().with_body(to_string(&json).unwrap())
			},
			JsonError::Err(e) => {
				let json = json!({
					"error": format!("{}", e),
				});
				res().with_body(to_string(&json).unwrap())
			}
		}
	}
}
