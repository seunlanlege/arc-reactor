use hyper::{Request, Response};
use ArcProto::{ArcResult, ArcError, arc};

pub trait MiddleWare: Sync + Send
{
	type R: Into<Response>;
	type E: Into<ArcError>;
	fn call(&self, request: Request) -> ArcResult<Self::R, Self::E>;
}

impl MiddleWare for Vec<Box<MiddleWare<R = Response, E = ArcError>>>
{
	type R = Response;
	type E = ArcError;
	fn call(&self, request: Request) -> ArcResult<Self::R, Self::E> {
		self
			.iter()
			.fold(
				arc::Ok(request),
				|request, middleware| {
					request.and_then(|req| middleware.call(req))
				}
			)
	}
}

#[macro_export]
macro_rules! mw {
    ($($middlewares:expr), +) => {{
			let middleWares: Vec<Box<MiddleWare<R = Response, E = ArcError>>> = vec![$(Box::new($middlewares)), +];
      middleWares
		}};
}

