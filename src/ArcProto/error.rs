use hyper::{StatusCode};
use ArcCore::{Request, Response};

pub struct ArcError(pub StatusCode, pub String);

impl From<&'static str> for ArcError {
	fn from(string: &str) -> ArcError {
		ArcError(Default::default(), string.to_owned())
	}
}

impl Default for ArcError {
	fn default() -> ArcError {
		ArcError(StatusCode::BadRequest, "Request Could not be processed!".to_owned())
	}
}

pub enum ArcResult<T> {
	Ok(T),
	error(ArcError),
}

impl ArcResult<Request> {
	#[inline]
	pub fn and_then<F> (self, predicate: F) -> ArcResult<Request>
	where
			F: FnOnce(Request) -> ArcResult<Request>
	{
		match self {
			ArcResult::Ok(t) => predicate(t),
			ArcResult::error(e) => ArcResult::error(e),
		}
	}
}

impl ArcResult<Response> {
	#[inline]
	pub fn and_then<F> (self, predicate: F) -> ArcResult<Response>
		where
			F: FnOnce(Response) -> ArcResult<Response>
	{
		match self {
			ArcResult::Ok(t) => predicate(t),
			ArcResult::error(e) => ArcResult::error(e),
		}
	}
}


 impl From<ArcError> for Response {
 	fn from(arcError: ArcError) -> Response {
 		Response::new()
 			.with_status(arcError.0)
 			.with_body(arcError.1)
 	}
 }

