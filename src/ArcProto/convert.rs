use hyper::{StatusCode, self};
use ArcProto::{ArcError};
use ArcCore::{Response};

fn convertToStatusCode(number: u16) -> StatusCode {
	match StatusCode::try_from(number) {
		Ok(status) => {
			status
		},
		Err(_) => {
			StatusCode::BadRequest
		},
	}
}

impl From<(u16, &'static str)> for Response {
	fn from(tuple: (u16, &'static str)) -> Response {
		Response::new()
			.with_status(convertToStatusCode(tuple.0))
			.with_body(tuple.1)
	}
}

impl From<(u16, &'static str)> for ArcError {
	fn from(tuple: (u16, &'static str)) -> ArcError {
		let status = convertToStatusCode(tuple.0);
		ArcError(status, tuple.1.to_owned())
	}
}

impl From<Response> for hyper::Response {
	fn from(res: Response) -> hyper::Response {
		res.inner
	}
}