use hyper::Error;
use ArcCore::{Request, Response};
use std::sync::Arc;
use futures::future::{IntoFuture, Future};

type MiddleWareFuture<'a, I> = Box<Future<Item=I, Error=Error> + 'a>;

pub trait MiddleWare<T>: Sync + Send {
	fn call(&'static self, param: T) -> MiddleWareFuture<T>;
}

//impl MiddleWare<Request> for Vec<Box<MiddleWare<Request>>> {
//	fn call(&self, request: Request) -> MiddleWareFuture<Request> {
//		self
//			.iter()
//			.fold(
//				box Ok(request).into_future(),
//				|request, middleware| {
//					box request.and_then(|req| middleware.call(req))
//				}
//			)
//	}
//}

impl MiddleWare<Response> for Vec<Arc<Box<MiddleWare<Response>>>> {
	fn call(&self, response: Response) -> MiddleWareFuture<Response> {
		self
			.iter()
			.fold(
				box Ok(response).into_future(),
				|response, middleware| {
					box response.and_then(move |res| middleware.call(res))
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
