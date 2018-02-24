#![macro_use]
use ArcCore::{Request, Response};
use std::sync::Arc;
use futures::future::{Future, IntoFuture};

type MiddleWareFuture<I> = Box<Future<Item = I, Error = Response>>;

pub trait MiddleWare<T>: Sync + Send {
	fn call(&self, param: T) -> MiddleWareFuture<T>;
}

impl MiddleWare<Request> for Vec<Arc<Box<MiddleWare<Request>>>> {
	fn call(&self, request: Request) -> MiddleWareFuture<Request> {
		self
			.iter()
			.fold(box Ok(request).into_future(), |request, middleware| {
				let clone = middleware.clone();
				box request.and_then(move |req| clone.call(req))
			})
	}
}

impl MiddleWare<Response> for Vec<Arc<Box<MiddleWare<Response>>>> {
	fn call(&self, response: Response) -> MiddleWareFuture<Response> {
		self
			.iter()
			.fold(box Ok(response).into_future(), |response, middleware| {
				let clone = middleware.clone();
				box response.and_then(move |res| clone.call(res))
			})
	}
}

#[macro_export]
macro_rules! mw {
	($($middlewares:expr), +) => {{
	use std::sync::Arc;
		let middleWares: Vec<Arc<Box<MiddleWare<_>>>> = vec![$(Arc::new(box $middlewares)), +];
     box middleWares as Box<MiddleWare<_>>
	}};
}
