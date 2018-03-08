use hyper::{self, StatusCode};
use core::{res, Request, Response};

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
