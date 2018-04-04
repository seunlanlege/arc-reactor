use core::{res, JsonError, QueryParseError, Request, Response};
use hyper::header::ContentType;
use hyper::{self, StatusCode};
use serde::ser::Serialize;
use serde_json::to_string;

/// Converts a u16 value into equivalent status code of type ```StatusCode```.
///
/// # Errors
/// This will return a bad request if the provided argument is not within the range 100...599..
fn toStatusCode(number: u16) -> StatusCode {
	match StatusCode::try_from(number) {
		Ok(status) => status,
		Err(_) => StatusCode::BadRequest,
	}
}

impl<T: Serialize> From<(u16, T)> for Response {
	fn from(tuple: (u16, T)) -> Response {
		res()
			.with_header(ContentType::plaintext())
			.with_status(toStatusCode(tuple.0))
			.with_body(to_string(&tuple.1).unwrap())
	}
}

impl<T: Serialize> From<T> for Response {
	default fn from(json: T) -> Response {
		res()
			.with_header(ContentType::json())
			.with_status(toStatusCode(200))
			.with_body(to_string(&json).unwrap())
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
				res()
					.with_header(ContentType::json())
					.with_body(to_string(&json).unwrap())
			}
			JsonError::Err(e) => {
				let json = json!({
					"error": format!("{}", e),
				});
				res()
					.with_header(ContentType::json())
					.with_body(to_string(&json).unwrap())
			}
		}
	}
}

impl From<QueryParseError> for Response {
	fn from(error: QueryParseError) -> Response {
		match error {
			QueryParseError::None => {
				let json = json!({
					"error": "query data was empty",
				});
				res()
					.with_header(ContentType::json())
					.with_body(to_string(&json).unwrap())
			}

			QueryParseError::ParseError(_e) => {
				let json = json!({
					"error": format!("{}", _e),
				});
				res()
					.with_header(ContentType::json())
					.with_body(to_string(&json).unwrap())
			}
		}
	}
}
