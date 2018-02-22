#![feature(generic_associated_types)]

use hyper::{self, Method, Request, Response, StatusCode};
use futures::{Future, IntoFuture};
use hyper::server::Service;
use std::collections::HashMap;
use recognizer::{Match, Router as Recognizer};
use ArcProto::ArcService;
use ArcProto::MiddleWare;
use ArcRouting::RouteGroup;
use ArcCore::{Request as ArcRequest, Response as ArcResponse};
use std::sync::{Arc, Mutex};
use futures::prelude::{async_block, await};
use futures::future;

pub struct Router {
	pub(crate) routes: HashMap<Method, Recognizer<Box<ArcService>>>,
}

impl Router {
	pub fn new() -> Self {
		Self {
			routes: HashMap::new(),
		}
	}

	//	pub fn middleware(self, middleware: Box<MiddleWare<ArcRequest>>) -> Self {
	//		self.middleware = Some(middleware);
	//
	//		self
	//	}

	pub fn routes(mut self, group: RouteGroup) -> Self {
		let RouteGroup { routes, .. } = group;
		{
			for (path, (method, handler)) in routes.into_iter() {
				self
					.routes
					.entry(method)
					.or_insert(Recognizer::new())
					.add(path.as_str(), handler)
			}
		}

		self
	}

	pub fn get<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Get, route, handler)
	}

	pub fn post<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Post, route, handler)
	}

	pub fn put<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Put, route, handler)
	}

	pub fn patch<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Patch, route, handler)
	}

	pub fn delete<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Delete, route, handler)
	}

	fn route<S>(mut self, method: Method, path: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		{
			self
				.routes
				.entry(method)
				.or_insert(Recognizer::new())
				.add(path.as_ref(), box handler);
		}

		self
	}
}

impl ArcRouter {
	pub fn new() -> Self {
		Self {
			routes: Arc::new(HashMap::new()),
		}
	}

	pub(crate) fn matchRoute<P>(&self, route: P, method: &Method) -> Option<Match<&Box<ArcService>>>
	where
		P: AsRef<str>,
	{
		if let Some(recognizer) = self.routes.get(method) {
			return recognizer.recognize(route.as_ref()).ok();
		} else {
			None
		}
	}
}

pub struct ArcRouter {
	pub(crate) routes: Arc<HashMap<Method, Recognizer<Box<ArcService>>>>,
}

impl Service for ArcRouter {
	type Response = Response;
	type Request = Request;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Request) -> Box<Future<Item = Self::Response, Error = Self::Error>> {
		if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {
			let mut request: ArcRequest = req.into();
			request.paramsMap.insert(routeMatch.params);
			let response = routeMatch
				.handler
				.call(request, ArcResponse::new())
				.map(|res| res.into());
			return box response;
		} else {
			return box Ok(Response::new().with_status(StatusCode::NotFound)).into_future();
		}
	}
}
