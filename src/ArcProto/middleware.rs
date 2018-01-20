use ArcCore::{Request, Response};
use ArcProto::{ArcResult, result};

pub trait MiddleWare<T>: Sync + Send {
	fn call(&self, param: T) -> ArcResult<T>;
}

impl MiddleWare<Request> for Vec<Box<MiddleWare<Request>>> {
	fn call(&self, request: Request) -> ArcResult<Request> {
		self
			.iter()
			.fold(
				result::Ok(request),
				|request, middleware| {
					request.and_then(|req| middleware.call(req))
				}
			)
	}
}

impl MiddleWare<Response> for Vec<Box<MiddleWare<Response>>> {
	fn call(&self, response: Response) -> ArcResult<Response> {
		self
			.iter()
			.fold(
				result::Ok(response),
				|response, middleware| {
					response.and_then(|res| middleware.call(res))
				}
			)
	}
}

#[macro_export]
macro_rules! mw {
	($($middlewares:expr), +) => {{
		let middleWares: Vec<Box<MiddleWare>> = vec![$(Box::new($middlewares)), +];
     middleWares
	}};
}
