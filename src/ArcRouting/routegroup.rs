use ArcProto::*;

use hyper::Method;
use std::collections::HashMap;

pub struct RouteGroup {
	pub(crate) parent: &'static str,
	pub(crate) routes: HashMap<String, (Method, Box<ArcService>)>,
}

impl RouteGroup {
	pub(crate) fn new(parent: &'static str) -> Self {
		RouteGroup {
			parent,
			routes: HashMap::new(),
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
			self.routes.insert(fullPath, (method, handler));
		}

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
		path: &'static str,
		handler: S,
	) -> Self {
		let mut parent = self.parent;

		if self.parent.starts_with("/") {
			parent = self.parent.get(1..).unwrap();
		}

		let fullPath = format!("/{}{}", parent, path);
		self
			.routes
			.insert(fullPath, (method, box handler));

		self
	}
}
