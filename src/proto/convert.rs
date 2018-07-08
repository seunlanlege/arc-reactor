use core::{JsonError, QueryParseError, Request, Response};
use hyper::{
	self,
	header::{HeaderValue, CONTENT_TYPE},
	Body,
};
use mime;
use serde::ser::Serialize;
use serde_json::to_vec;

impl<T: Serialize> From<(u16, T)> for Response {
	fn from(tuple: (u16, T)) -> Response {
		let body = to_vec(&tuple.1).unwrap();

		let mut res = Response::new();
		res.headers_mut().insert(
			CONTENT_TYPE,
			HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
		);

		res.with_status(tuple.0).with_body(body)
	}
}

#[cfg(feature = "unstable")]
impl<T: Serialize> From<T> for Response {
	default fn from(json: T) -> Response {
		let body = to_vec(&json).unwrap();
		let mut res = Response::new();
		res.headers_mut().insert(
			CONTENT_TYPE,
			HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
		);
		res.with_body(body)
	}
}

impl From<Response> for hyper::Response<Body> {
	fn from(res: Response) -> hyper::Response<Body> {
		let Response { parts, body } = res;
		hyper::Response::from_parts(parts, body)
	}
}

impl From<hyper::Request<Body>> for Request {
	fn from(req: hyper::Request<Body>) -> Request {
		let (parts, body) = req.into_parts();

		let request = Request::new(parts, body);

		request
	}
}

impl From<JsonError> for Response {
	fn from(error: JsonError) -> Response {
		match error {
			JsonError::None => {
				error!("No json body");
				let json = json!({
					"error": "Json was empty",
				});
				let body = to_vec(&json).unwrap();
				let mut res = Response::new();
				res.headers_mut().insert(
					CONTENT_TYPE,
					HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
				);
				res.badRequest().with_body(body)
			}
			JsonError::Err(e) => {
				error!("serde deserialization error: {}", e);
				let json = json!({
					"error": format!("{}", e),
				});
				let body = to_vec(&json).unwrap();
				let mut res = Response::new();
				res.headers_mut().insert(
					CONTENT_TYPE,
					HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
				);
				res.badRequest().with_body(body)
			}
		}
	}
}

impl From<QueryParseError> for Response {
	fn from(error: QueryParseError) -> Response {
		match error {
			QueryParseError::None => {
				error!("No query string");
				let json = json!({
					"error": "query data was empty",
				});
				let body = to_vec(&json).unwrap();
				let mut res = Response::new();
				res.headers_mut().insert(
					CONTENT_TYPE,
					HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
				);
				res.badRequest().with_body(body)
			}

			QueryParseError::Err(err) => {
				error!("Error deserializing query: {}", err);
				let json = json!({
					"error": format!("{}", err),
				});
				let body = to_vec(&json).unwrap();
				let mut res = Response::new();
				res.headers_mut().insert(
					CONTENT_TYPE,
					HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
				);
				res.badRequest().with_body(body)
			}
		}
	}
}
