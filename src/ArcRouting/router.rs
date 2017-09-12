use hyper::{Response, Request, Method, self, StatusCode};
use futures::{Future, future};
use hyper::server::Service;
use std::collections::HashMap;
use recognizer::{Match, Router as Recognizer};
use ArcProto::ArcService;
//use ArcRouting::{RouteGroup};
//use std::marker::PhantomData;

#[derive(Clone)]
pub struct ArcRouter<S>
	where S: ArcService {
	routes: HashMap<Method, Recognizer<Box<S>>>,
}

impl<S> Service for ArcRouter<S>
	where S: ArcService {
	type Response = Response;
	type Request = Request;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
	
	fn call(&self, req: Request) -> Self::Future {
		if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {

			return routeMatch.handler.call(req)
		}

		return Box::new(
			future::ok(
				Response::new().with_status(StatusCode::NotFound)
			)
		)
	}
}

impl<S> ArcRouter<S>
	where S: ArcService {
	pub fn new() -> Self {
		Self { routes: HashMap::new() }
	}

	pub(crate) fn matchRoute<P>(&self, route: P, method: &Method) -> Option<Match<&Box<S>>>
	where P: AsRef<str> {
		if let Some(recognizer) = self.routes.get(method) {
			return recognizer.recognize(route.as_ref()).ok();
		} else {
			None
		}
	}

//	pub fn group(parent: &'static str) -> RouteGroup<S> {
//		RouteGroup::new(parent)
//	}

//	pub fn add(mut self, group: RouteGroup<S>) -> Self {
//		let RouteGroup { routes, .. } = group;
//
//		for (path, (method, handler)) in routes.into_iter() {
//			self.routes
//			  .entry(method)
//			  .or_insert(Recognizer::new())
//			  .add(path.as_str(), handler)
//		}
//
//		self
//	}
	
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
			.entry(method)
			.or_insert(Recognizer::new())
			.add(path.as_ref(), Box::new(handler));
		
		self
	}
}
