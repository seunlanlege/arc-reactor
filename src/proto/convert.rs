use core::{res, JsonError, QueryParseError, Request, Response};
use hyper::{
	self,
	header::{ContentLength, ContentType},
	StatusCode,
};
use serde::ser::Serialize;
use serde_json::to_vec;

/// Converts a u16 value into equivalent status code of type ```StatusCode```.
///
/// # Errors
/// This will return a bad request if the provided argument is not within the
/// range 100...599..
fn toStatusCode(number: u16) -> StatusCode {
	match StatusCode::try_from(number) {
		Ok(status) => status,
		Err(_) => StatusCode::BadRequest,
	}
}

impl<T: Serialize> From<(u16, T)> for Response {
	fn from(tuple: (u16, T)) -> Response {
		let body = to_vec(&tuple.1).unwrap();
		res()
			.with_header(ContentLength(body.len() as u64))
			.with_header(ContentType::json())
			.with_status(toStatusCode(tuple.0))
			.with_body(body)
	}
}

#[cfg(feature = "unstable")]
impl<T: Serialize> From<T> for Response {
	default fn from(json: T) -> Response {
		let body = to_vec(&json).unwrap();
		res()
			.with_header(ContentLength(body.len() as u64))
			.with_header(ContentType::json())
			.with_status(toStatusCode(200))
			.with_body(body)
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
				let body = to_vec(&json).unwrap();

				res()
					.with_header(ContentLength(body.len() as u64))
					.with_header(ContentType::json())
					.with_body(body)
			}
			JsonError::Err(e) => {
				let json = json!({
					"error": format!("{}", e),
				});
				let body = to_vec(&json).unwrap();

				res()
					.with_header(ContentLength(body.len() as u64))
					.with_header(ContentType::json())
					.with_body(body)
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
				let body = to_vec(&json).unwrap();

				res()
					.with_header(ContentLength(body.len() as u64))
					.with_header(ContentType::json())
					.with_body(body)
			}

			QueryParseError::Err(err) => {
				let json = json!({
					"error": format!("{}", err),
				});
				let body = to_vec(&json).unwrap();

				res()
					.with_header(ContentLength(body.len() as u64))
					.with_header(ContentType::json())
					.with_body(body)
			}
		}
	}
}
