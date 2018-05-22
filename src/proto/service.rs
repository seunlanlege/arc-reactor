use core::{Request, Response};
use futures::Future;
use proto::MiddleWare;

pub type FutureResponse = Box<Future<Item = Response, Error = Response>>;

/// This trait is automatically derived by the #[service] proc_macro.
pub trait ArcService: ArcServiceClone + Send + Sync {
	fn call(&self, req: Request, res: Response) -> FutureResponse;
}

#[cfg(not(feature = "unstable"))]
impl<T> ArcService for T
where
	T: Fn(Request, Response) -> FutureResponse + Send + Sync + Clone + 'static,
{
	fn call(&self, req: Request, res: Response) -> FutureResponse {
		(self)(req, res)
	}
}

pub trait ArcServiceClone {
	fn clone_service(&self) -> Box<ArcService>;
}

impl<T> ArcServiceClone for T
where
	T: 'static + ArcService + Clone,
{
	fn clone_service(&self) -> Box<ArcService> {
		Box::new(self.clone())
	}
}

impl Clone for Box<ArcService> {
	fn clone(&self) -> Self {
		self.clone_service()
	}
}

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
			handler: Box::new(h),
			after: None,
		}
	}

	pub fn before<T: 'static + MiddleWare<Request>>(&mut self, before: T) {
		self.before = Some(Box::new(before));
	}

	pub fn after<T: 'static + MiddleWare<Response>>(&mut self, after: T) {
		self.after = Some(Box::new(after));
	}
}

impl ArcService for ArcHandler {
	fn call(&self, req: Request, res: Response) -> FutureResponse {
		let ptr = self as *const ArcHandler;
		let extended = unsafe { &*ptr };

		if extended.before.is_some() && extended.after.is_none() {
			let before = match extended.before {
				Some(ref before) => before,
				_ => unreachable!(),
			};
			let responsefuture = before
				.call(req)
				.and_then(move |req| extended.handler.call(req, res));
			return Box::new(responsefuture.then(move |res| {
				drop(ptr);
				drop(extended);
				res
			}));
		}

		if extended.before.is_none() && extended.after.is_some() {
			let after = match extended.after {
				Some(ref after) => after,
				_ => unreachable!(),
			};
			let responsefuture = extended
				.handler
				.call(req, res)
				.and_then(move |res| after.call(res));
			return Box::new(responsefuture.then(move |res| {
				drop(ptr);
				drop(extended);
				res
			}));
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
			let responsefuture = before
				.call(req)
				.and_then(move |req| extended.handler.call(req, res))
				.and_then(move |res| after.call(res));
			return Box::new(responsefuture.then(move |res| {
				drop(ptr);
				drop(extended);
				res
			}));
		}

		return Box::new(extended.handler.call(req, res).then(move |res| {
			drop(ptr);
			drop(extended);
			res
		}));
	}
}
