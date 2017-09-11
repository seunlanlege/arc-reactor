 use ArcProto::*;
// use ArcRouting::Router;
// use hyper::server::Service;

use hyper::{Method};
use std::collections::HashMap;

pub struct RouteGroup<S: ArcService> {
	pub(crate) parent: &'static str,
	pub(crate) routes: HashMap<String, (Method, Box<S>)>,
}


impl <S: ArcService> RouteGroup<S> {
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
}

impl<S> RouteInterface<S> for RouteGroup<S>
where S: ArcService {
	fn route(mut self, method: Method, path: &'static str, handler: S) -> Self {
		self.routes
			.insert(format!("/{}{}", &self.parent, path), (method, Box::new(handler)));

		self
	}
}