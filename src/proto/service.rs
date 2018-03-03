#![macro_use]
use core::{Request, Response};
use hyper::server::Service;
use hyper;
use futures::prelude::{async_block, await};
use futures::Future;
use proto::MiddleWare;
use std::sync::Arc;

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
	pub fn new(h: Box<ArcService>) -> Self {
		Self {
			before: None,
			handler: Arc::new(h),
			after: None,
		}
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

impl Service for ArcHandler {
	type Request = hyper::Request;
	type Response = hyper::Response;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Self::Request) -> Self::Future {
		let request: Request = req.into();
		let responseFuture = ArcService::call(&*self, request, Response::new());

		let future = async_block! {
			let response = await!(responseFuture);
			match response {
				Ok(res) => Ok(res.into()),
				Err(res) => Ok(res.into())
			}
		};

		return box future;
	}
}

#[macro_export]
macro_rules! arc {
	($handler:expr) => {{
		use std::sync::Arc;
		use $crate::ArcHandler;
		ArcHandler {
			before: None,
			handler: Arc::new(box $handler),
			after: None
		}
	}};
	($before:expr, $handler:expr) => {{
		use std::sync::Arc;
		use $crate::ArcHandler;
		ArcHandler {
			before: Some(Arc::new($before)),
			handler: Arc::new(box $handler),
			after: None
		}
	}};
	($before:expr, $handler:expr, $after:expr) => {{
		use std::sync::Arc;
		use $crate::ArcHandler;
		ArcHandler {
			before: Some(Arc::new($before)),
			handler: Arc::new(box $handler),
			after: Some(Arc::new($after))
		}
	}};
	(_, $handler:expr, $after:expr) => {{
		use std::sync::Arc;
		use $crate::ArcHandler;
		ArcHandler {
			before: None,
			handler: Arc::new(box $handler),
			after: Some(Arc::new($after))
		}
	}};
}
