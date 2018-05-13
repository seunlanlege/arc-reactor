use core::{Request, Response};
use futures::Future;
use proto::MiddleWare;
use std::fmt;

pub type FutureResponse = Box<Future<Item = Response, Error = Response>>;

/// This trait is automatically derived by the #[service] proc_macro.
pub trait ArcService: ArcServiceClone + Send + Sync + fmt::Debug {
	fn call(&self, req: Request, res: Response) -> FutureResponse;
}

#[cfg(feature = "stable")]
impl<T> ArcService for T
where
	T: Fn(Request, Response) -> FutureResponse + Send + Sync + Clone + fmt::Debug + 'static,
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
	T: 'static + ArcService + fmt::Debug + Clone,
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

impl fmt::Debug for ArcHandler {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ArcHandler")
            .field("before", &self.before)
            .field("handler", &self.handler)
            .field("after", &self.after)
            .finish()
    }
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
		println!("{:#?}", self);
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
