use hyper::{Response, Request, Method, self, StatusCode};
use futures::{Future, IntoFuture};
use hyper::server::Service;
use std::collections::HashMap;
use recognizer::{Match, Router as Recognizer};
use ArcProto::ArcService;
use ArcRouting::{RouteGroup};
use ArcCore::{Request as ArcRequest};

pub struct ArcRouter {
	routes: HashMap<Method, Recognizer<Box<ArcService>>>,
}

impl Service for ArcRouter {
	type Response = Response;
	type Request = Request;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
	
	fn call(&self, req: Request) -> Self::Future {
		if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {
			let response = Response::new();
			let remote = req.remote_addr();
			let (method, uri, version, headers, body) = req.deconstruct();
			let request = ArcRequest::new(method, uri, version, headers, body, remote);
			// TODO: add anymap!
			// request.put(routeMatch.params);
			return routeMatch.handler.call(request, response)
		}

		return box Ok(
			Response::new().with_status(StatusCode::NotFound)
		).into_future()
	}
}

impl ArcRouter {
	pub fn new() -> Self {
		Self { routes: HashMap::new() }
	}

	pub(crate) fn matchRoute<P>(&self, route: P, method: &Method) -> Option<Match<&Box<ArcService>>>
	where P: AsRef<str> {
		if let Some(recognizer) = self.routes.get(method) {
			return recognizer.recognize(route.as_ref()).ok();
		} else {
			None
		}
	}

	pub fn group(parent: &'static str) -> RouteGroup {
		RouteGroup::new(parent)
	}

	pub fn add(mut self, group: RouteGroup) -> Self {
		let RouteGroup { routes, .. } = group;

		for (path, (method, handler)) in routes.into_iter() {
			self.routes
			  .entry(method)
			  .or_insert(Recognizer::new())
			  .add(path.as_str(), handler)
		}

		self
	}
	
	pub fn get<S>(self, route: &'static str, handler: S) -> Self
	where
			S: ArcService + 'static + Send + Sync
	{
		self.route(Method::Get, route, handler)
	}
	
	pub fn post<S>(self, route: &'static str, handler: S) -> Self
	where
			S: ArcService + 'static + Send + Sync
	{
		self.route(Method::Post, route, handler)
	}
	
	pub fn put<S>(self, route: &'static str, handler: S) -> Self
	where
			S: ArcService + 'static + Send + Sync
	{
		self.route(Method::Put, route, handler)
	}
	
	pub fn patch<S>(self, route: &'static str, handler: S) -> Self
	where
			S: ArcService + 'static + Send + Sync
	{
		self.route(Method::Patch, route, handler)
	}
	
	pub fn delete<S>(self, route: &'static str, handler: S) -> Self
	where
			S: ArcService + 'static + Send + Sync
	{
		self.route(Method::Delete, route, handler)
	}
	
	fn route<S>(mut self, method: Method, path: &'static str, handler: S) -> Self
	where
			S: ArcService + 'static + Send + Sync
	{
		self.routes
			.entry(method)
			.or_insert(Recognizer::new())
			.add(path.as_ref(), box handler);
		
		self
	}
}
