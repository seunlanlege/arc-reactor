use ArcRouting::stripTrailingSlash;
use std::sync::Arc;
#[macro_use]
use ArcProto::{ArcService, MiddleWare, ArcHandler};
use ArcCore::{Request, Response};

use hyper::Method;
use std::collections::HashMap;

pub struct RouteGroup {
	pub(crate) parent: &'static str,
	pub(crate) before: Option<Arc<Box<MiddleWare<Request>>>>,
	pub(crate) after: Option<Arc<Box<MiddleWare<Response>>>>,
	pub(crate) routes: HashMap<String, (Method, Box<ArcService>)>,
}

impl RouteGroup {
	pub(crate) fn new(parent: &'static str) -> Self {
		RouteGroup {
			parent,
			routes: HashMap::new(),
			before: None,
			after: None
		}
	}

	pub fn group(mut self, group: RouteGroup) -> Self {
		let RouteGroup { routes, .. } = group;
		let mut parent = self.parent;

		if self.parent.starts_with("/") {
			parent = self.parent.get(1..).unwrap();
		}



		for (path, (method, handler)) in routes.into_iter() {
			let fullPath = format!("/{}{}", parent, path);
			let mut handler = box ArcHandler {
				before: self.before.clone(),
				handler: Arc::new(handler),
				after: self.after.clone()
			};
			self.routes.insert(fullPath, (method, handler));
		}

		self
	}

	pub fn before<T: 'static + MiddleWare<Request>>(mut self, before: T) -> Self {
		self.before = Some(Arc::new(box before));

		self
	}

	pub fn after<T: 'static + MiddleWare<Response>>(mut self, after: T) -> Self {
		self.after = Some(Arc::new(box after));

		self
	}

	pub fn get<S: ArcService + 'static + Send + Sync>(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Get, route, handler)
	}

	pub fn post<S: ArcService + 'static + Send + Sync>(
		self,
		route: &'static str,
		handler: S,
	) -> Self {
		self.route(Method::Post, route, handler)
	}

	pub fn put<S: ArcService + 'static + Send + Sync>(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Put, route, handler)
	}

	pub fn patch<S: ArcService + 'static + Send + Sync>(
		self,
		route: &'static str,
		handler: S,
	) -> Self {
		self.route(Method::Patch, route, handler)
	}

	pub fn delete<S: ArcService + 'static + Send + Sync>(
		self,
		route: &'static str,
		handler: S,
	) -> Self {
		self.route(Method::Delete, route, handler)
	}

	fn route<S: ArcService + 'static + Send + Sync>(
		mut self,
		method: Method,
		mut path: &'static str,
		handler: S,
	) -> Self {
		let mut parent = self.parent;
		let length = path.chars().count();

		if self.parent.starts_with("/") {
			parent = self.parent.get(1..).unwrap();
		}

		if !path.starts_with("/") && length > 1 {
			panic!("Valid route paths must start with '/' ");
		}

		path = stripTrailingSlash(path);

		let fullPath = format!("/{}{}", parent, path);
		self
			.routes
			.insert(fullPath, (method, box handler));

		self
	}
}
