#![macro_use]
use core::{Request, Response};
use futures::Future;
use proto::MiddleWare;
use std::sync::Arc;

/// This trait is automatically derived by the #[service] proc_macro.
pub trait ArcService: Send + Sync {
	fn call(&self, req: Request, res: Response) -> Box<Future<Item = Response, Error = Response>>;
}

pub type FutureResponse = Box<Future<Item = Response, Error = Response>>;

pub struct ArcHandler {
	pub before: Option<Arc<Box<MiddleWare<Request>>>>,
	pub handler: Arc<Box<ArcService>>,
	pub after: Option<Arc<Box<MiddleWare<Response>>>>,
}

impl ArcHandler {
	pub fn new<T: 'static + ArcService>(h: T) -> Self {
		Self {
			before: None,
			handler: Arc::new(box h),
			after: None,
		}
	}

	pub fn before<T: 'static + MiddleWare<Request>>(&mut self, before: T) {
		self.before = Some(Arc::new(box before));
	}

	pub fn after<T: 'static + MiddleWare<Response>>(&mut self, after: T) {
		self.after = Some(Arc::new(box after));
	}
}

impl ArcService for ArcHandler {
	fn call(&self, req: Request, res: Response) -> FutureResponse {
		if self.before.is_some() && self.after.is_none() {
			let before = self.before.clone().unwrap();
			let handler = self.handler.clone();
			return box before.call(req).and_then(move |req| handler.call(req, res));
		}

		if self.before.is_none() && self.after.is_some() {
			let after = self.after.clone().unwrap();
			let handler = self.handler.clone();
			return box handler.call(req, res).and_then(move |res| after.call(res));
		}

		if self.before.is_some() && self.after.is_some() {
			let before = self.before.clone().unwrap();
			let handler = self.handler.clone();
			let after = self.after.clone().unwrap();
			return box before
				.call(req)
				.and_then(move |req| handler.call(req, res))
				.and_then(move |res| after.call(res));
		}

		return box self.handler.call(req, res);
	}
}

/// This macro exists for composing a route handler with middlewares in order
/// to mount them on a router.
///
/// ```rust,ignore
/// fn rootRoutes() -> Router {
///   let RequestMiddlewares = mw![middleware1, middleware2];
///   let ResponseMiddlewares = mw![middleware3, middleware4];
///   Router::new()
///     .get("/", arc!(RequestMiddlewares, RouteHandler, ResponseMiddlewares)) // set both middlewares and Routehandler
///     .get("/test", arc!(RequestMiddleware, RouteHandler)) // set only the request middleware and route handler
///     .get("/test2", arc!(_, RouteHandler, ResponseMiddlewares)) // set only the response middleware and routehandler
/// }
/// ```
#[macro_export]
macro_rules! arc {
	($handler:expr) => {{
		use std::sync::Arc;
		use $crate::ArcHandler;
		ArcHandler::new(box $handler)
		}};
	($before:expr, $handler:expr) => {{
		use std::sync::Arc;
		use $crate::ArcHandler;
		let handler = ArcHandler::new(box $handler);
		handler.before($before);
		handler
		}};
	($before:expr, $handler:expr, $after:expr) => {{
		use std::sync::Arc;
		use $crate::ArcHandler;
		let handler = ArcHandler::new(box $handler);
		handler.before($before);
		handler.after($after);
		handler
		}};
	(_, $handler:expr, $after:expr) => {{
		use std::sync::Arc;
		use $crate::ArcHandler;
		handler.after($before);
		handler
		}};
}
