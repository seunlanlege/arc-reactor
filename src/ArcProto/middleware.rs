use hyper::{Request};
use ArcProto::{ArcResult};


pub trait MiddleWare {
		fn call (&self, request: Request) -> ArcResult;
}

impl<T> MiddleWare for Vec<T>
where T: MiddleWare
{
	fn call	(&self, request: Request) -> ArcResult {
		self
			.into_iter()
			.fold(
				Ok(request),
				|request, middleware| {
					request.and_then(|req| middleware.call(req))
				}
			)
			
	}
}

