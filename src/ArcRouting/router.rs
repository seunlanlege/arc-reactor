use hyper::{self, Method, StatusCode};
use futures::{Future, IntoFuture};
use futures::prelude::{async_block, await};
use hyper::server::Service;
use std::collections::HashMap;
use recognizer::{Match, Router as Recognizer};
use ArcProto::{ArcService, ArcHandler, MiddleWare};
use ArcRouting::{RouteGroup, stripTrailingSlash};
use ArcCore::{Request, Response};
use std::sync::{Arc};

pub struct Router {
	pub(crate) routes: HashMap<Method, Recognizer<ArcHandler>>,
	pub(crate) before: Option<Arc<Box<MiddleWare<Request>>>>,
	pub(crate) after: Option<Arc<Box<MiddleWare<Response>>>>,
}

impl Router {
	pub fn new() -> Self {
		Self {
			before: None,
			routes: HashMap::new(),
			after: None,
		}
	}

	pub fn group(mut self, group: RouteGroup) -> Self {
		let RouteGroup { routes, .. } = group;
		{
			for (path, (method, routehandler)) in routes.into_iter() {
				let handler = ArcHandler {
					before: self.before.clone(),
					handler: Arc::new(box routehandler),
					after: self.after.clone()
				};
				self
					.routes
					.entry(method)
					.or_insert(Recognizer::new())
					.add(path.as_str(), handler)
			}
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

	fn route<S>(mut self, method: Method, path: &'static str, routehandler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		{
			let handler = ArcHandler {
				before: self.before.clone(),
				handler: Arc::new(box routehandler),
				after: self.after.clone()
			};
			self
				.routes
				.entry(method)
				.or_insert(Recognizer::new())
				.add(path.as_ref(), handler);
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

	pub(crate) fn matchRoute<P>(&self, route: P, method: &Method) -> Option<Match<&ArcHandler>>
	where
		P: AsRef<str>,
	{
		let route = stripTrailingSlash(route.as_ref());
		self.routes
			.get(method)
			.and_then(|recognizer| {
				recognizer.recognize(route).ok()
			})
	}
}

pub struct ArcRouter {
	pub(crate) routes: Arc<HashMap<Method, Recognizer<ArcHandler>>>,
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
