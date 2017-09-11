use hyper::{Response, Request, Method, self, StatusCode};
use futures::{Future, future};	  
use hyper::server::Service;
use std::collections::HashMap;
use recognizer::{Match, Router as Recognizer};
use ArcProto::ArcService;
use ArcRouting::{RouteGroup};
use ArcProto::RouteInterface;

pub struct ArcRouter<S: ArcService> {
	routes: HashMap<Method, Recognizer<Box<S>>>,
}


impl<S: ArcService> Service for ArcRouter<S> {
	type Response = Response;
	type Request = Request;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
	
	fn call(&self, req: Request) -> Self::Future {
		if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {
			println!("{:?}", routeMatch.params["name"]);

			return routeMatch.handler.call(req)
		}

		return Box::new(
			future::ok(
				Response::new().with_status(StatusCode::NotFound)
			)
		)
	}
}

impl <S: ArcService>ArcRouter<S> {
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

	pub fn group(parent: &'static str) -> RouteGroup<S> {
		RouteGroup::new(parent)
	}

	pub fn add(mut self, group: RouteGroup<S>) -> Self {
		let RouteGroup { routes, .. } = group;

		for (path, (method, handler)) in routes.into_iter() {
			self.routes
			  .entry(method)
			  .or_insert(Recognizer::new())
			  .add(path.as_str(), handler)
		}

		self
	}
}

impl<S> RouteInterface<S> for ArcRouter<S>
where S: ArcService {

	fn route(mut self, method: Method, path: &'static str, handler: S) -> ArcRouter<S> {
		self.routes
			.entry(method)
			.or_insert(Recognizer::new())
			.add(path.as_ref(), Box::new(handler));
		
		self
	}
	
}