 use ArcProto::*;
// use ArcRouting::Router;
// use hyper::server::Service;

use hyper::{Method};
use std::collections::HashMap;

pub struct RouteGroup<S>
	where S: ArcService {
	pub(crate) parent: &'static str,
	pub(crate) routes: HashMap<String, (Method, Box<Box<ArcService>>)>,
}


impl<S> RouteGroup<S>
	where S: ArcService {
	pub fn new(parent: &'static str) -> Self {
		RouteGroup {
			parent,
			routes: HashMap::new(),
		}
	}

	pub fn add(mut self, group: RouteGroup<S>) -> Self {
		let RouteGroup { routes, .. } = group;

		for (path, (method, handler)) in routes.into_iter() {
			self.routes
			  .insert(path, (method, handler));
		}

		self
	}
	
	pub fn get(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Get, route, handler)
	}
	
	pub fn post(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Post, route, handler)
	}
	
	pub fn put(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Put, route, handler)
	}
	
	pub fn patch(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Patch, route, handler)
	}
	
	pub fn delete(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Delete, route, handler)
	}
	
	fn route(mut self, method: Method, path: &'static str, handler: S) -> Self {
		self.routes
		.insert(format!("/{}{}", &self.parent, path), (method, Box::new(handler)));
	
		self
	}
}