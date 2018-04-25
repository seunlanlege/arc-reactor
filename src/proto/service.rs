#![macro_use]
use core::{Request, Response};
use futures::Future;
use proto::MiddleWare;

/// This trait is automatically derived by the #[service] proc_macro.
pub trait ArcService: ArcServiceClone + Send + Sync {
	fn call(&self, req: Request, res: Response) -> Box<Future<Item = Response, Error = Response>>;
}

pub trait ArcServiceClone {
	fn clone_service(&self) -> Box<ArcService>;
}

impl<T> ArcServiceClone for T
where
	T: 'static + ArcService + Clone,
{
	fn clone_service(&self) -> Box<ArcService> {
		box self.clone()
	}
}

impl Clone for Box<ArcService> {
	fn clone(&self) -> Self {
		self.clone_service()
	}
}

pub type FutureResponse = Box<Future<Item = Response, Error = Response>>;

#[derive(Clone)]
pub struct ArcHandler {
	pub before: Option<Box<MiddleWare<Request>>>,
	pub handler: Box<ArcService>,
	pub after: Option<Box<MiddleWare<Response>>>,
}

impl ArcHandler {
	pub fn new<T: 'static + ArcService>(h: T) -> Self {
		Self {
			before: None,
			handler: box h,
			after: None,
		}
	}

	pub fn before<T: 'static + MiddleWare<Request>>(&mut self, before: T) {
		self.before = Some(box before);
	}

	pub fn after<T: 'static + MiddleWare<Response>>(&mut self, after: T) {
		self.after = Some(box after);
	}
}

impl ArcService for ArcHandler {
	fn call(&self, req: Request, res: Response) -> FutureResponse {
		let extended = unsafe { &*(self as *const ArcHandler) };

		if extended.before.is_some() && extended.after.is_none() {
			let before = match extended.before {
				Some(ref before) => before,
				_ => unreachable!(),
			};
			return box before
				.call(req)
				.and_then(move |req| extended.handler.call(req, res));
		}

		if extended.before.is_none() && extended.after.is_some() {
			let after = match extended.after {
				Some(ref after) => after,
				_ => unreachable!(),
			};
			return box extended
				.handler
				.call(req, res)
				.and_then(move |res| after.call(res));
		}

		if extended.before.is_some() && extended.after.is_some() {
			let before = match extended.before {
				Some(ref before) => before,
				_ => unreachable!(),
			};
			let after = match extended.after {
				Some(ref after) => after,
				_ => unreachable!(),
			};
			return box before
				.call(req)
				.and_then(move |req| extended.handler.call(req, res))
				.and_then(move |res| after.call(res));
		}

		return box extended.handler.call(req, res);
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
		use $crate::ArcHandler;
		ArcHandler::new($handler)
		}};
	($before:expr, $handler:expr) => {{
		use $crate::ArcHandler;
		let mut handler = ArcHandler::new($handler);
		handler.before($before);
		handler
		}};
	($before:expr, $handler:expr, $after:expr) => {{
		use $crate::ArcHandler;
		let mut handler = ArcHandler::new($handler);
		handler.before($before);
		handler.after($after);
		handler
		}};
	(_, $handler:expr, $after:expr) => {{
		use $crate::ArcHandler;
		let mut handler = ArcHandler::new($handler);
		handler.after($before);
		handler
		}};
}
