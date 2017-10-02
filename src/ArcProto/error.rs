use hyper::{Response, Request, StatusCode};

pub struct ArcError(pub StatusCode, pub String);

impl Into<ArcError> for &'static str {
	fn into(self) -> ArcError {
		ArcError(Default::default(), self.to_owned())
	}
}

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

impl Into<ArcError> for (u16, &'static str) {
	fn into(self) -> ArcError {
		let status = convertToStatusCode(self.0);
		ArcError(status, self.1.to_owned())
	}
}

impl Default for ArcError {
	fn default() -> ArcError {
		ArcError(StatusCode::BadRequest, "Request Could not be processed!".to_owned())
	}
}

pub enum ArcResult<R, E>
where
		R: Into<Response>,
		E: Into<ArcError>,
{
	Ok(Request),
	Err(E),
	Res(R)
}

impl<R, E> ArcResult<R, E>
	where
		R: Into<Response>,
		E: Into<ArcError>,
{
	#[inline]
	pub fn and_then<F> (self, predicate: F) -> ArcResult<R, E>
	where
			F: FnOnce(Request) -> ArcResult<R, E>
	{
		use ArcResult::*;
		match self {
			Ok(t) => predicate(t),
			Res(r) => Res(r),
			Err(e) => Err(e),
		}
	}
}

impl Into<Response> for ArcError {
	fn into(self) -> Response {
		Response::new()
			.with_status(self.0)
			.with_body(self.1)
	}
}

pub trait Convert<T> {
	fn convert(self) -> T;
}

impl Convert<Response> for (u16, &'static str) {
	fn convert(self) -> Response {
		Response::new()
			.with_status(convertToStatusCode(self.0))
			.with_body(self.1)
	}
}