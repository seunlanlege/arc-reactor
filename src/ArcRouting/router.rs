use hyper::{self, Method, StatusCode};
use futures::{Future, IntoFuture};
use futures::prelude::{async_block, await};
use hyper::server::Service;
use std::collections::HashMap;
use recognizer::{Match, Router as Recognizer};
use ArcProto::ArcService;
use ArcRouting::RouteGroup;
use ArcCore::{Request, Response};
use std::sync::{Arc};

pub struct Router {
	pub(crate) routes: HashMap<Method, Recognizer<Box<ArcService>>>,
}

impl Router {
	pub fn new() -> Self {
		Self {
			routes: HashMap::new(),
		}
	}

	// 	pub fn middleware(self, middleware: Box<MiddleWare<ArcRequest>>) -> Self {
	// 		self.middleware = Some(middleware);
	//
	// 		self
	// 	}

	pub fn group(mut self, group: RouteGroup) -> Self {
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
		self.routes
			.get(method)
			.and_then(|recognizer| {
				recognizer.recognize(route.as_ref()).ok()
			})
	}
}

pub struct ArcRouter {
	pub(crate) routes: Arc<HashMap<Method, Recognizer<Box<ArcService>>>>,
}

impl Service for ArcRouter {
	type Request = hyper::Request;
	type Response = hyper::Response;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Self::Request) -> Self::Future {
		if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {
			let mut request: Request = req.into();
			request.set(routeMatch.params);

			let responseFuture = routeMatch
				.handler
				.call(request, Response::new());

			let future = async_block! {
				let response = await!(responseFuture);
				match response {
					Ok(res) => Ok(res.into()),
					Err(res) => Ok(res.into())
				}
			};

			return box future;
		} else {
			return box Ok(hyper::Response::new().with_status(StatusCode::NotFound)).into_future();
		}
	}
}
