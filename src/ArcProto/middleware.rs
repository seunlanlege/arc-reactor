use ArcCore::{Request};
use ArcProto::{ArcResult, arc};

pub trait MiddleWare: Sync + Send {
	fn call(&self, request: Request) -> ArcResult;
}

impl MiddleWare for Vec<Box<MiddleWare>> {
	fn call(&self, request: Request) -> ArcResult {
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
		let middleWares: Vec<Box<MiddleWare>> = vec![$(Box::new($middlewares)), +];
     middleWares
	}};
}
