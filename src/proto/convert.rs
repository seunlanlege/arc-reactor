use hyper::{self, StatusCode};
use proto::ArcError;
use core::{Request, Response};

fn convertToStatusCode(number: u16) -> StatusCode {
	match StatusCode::try_from(number) {
		Ok(status) => status,
		Err(_) => StatusCode::BadRequest,
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

impl From<hyper::Request> for Request {
	fn from(req: hyper::Request) -> Request {
		let remote = req.remote_addr();
		let (method, uri, version, headers, body) = req.deconstruct();

		let request = Request::new(method, uri, version, headers, body, remote);

		request
	}
}
