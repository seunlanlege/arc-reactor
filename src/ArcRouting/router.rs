#![feature(generic_associated_types)]

use hyper::{Response, Request, Method, self, StatusCode};
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

pub struct ArcRouter {
	routes: Arc<Mutex<HashMap<Method, Recognizer<Box<ArcService>>>>>,
//	middleware: Option<Box<MiddleWare<ArcRequest>>>,
}

impl Service for ArcRouter {
	type Response = Response;
	type Request = Request;
	type Error = hyper::Error;
	type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

	fn call(&self, req: Request) -> Box<Future<Item=Self::Response, Error=Self::Error>> {
		let routes = self.routes.clone();
		let routes = routes.lock();
		let routes = routes.unwrap();
		if let Some(recognizer) = routes.get(req.method()) {
			if let Some(routeMatch) = recognizer.recognize(req.path()).ok() {
				let mut request: ArcRequest = req.into();
				request.paramsMap.insert(routeMatch.params);
				let response = routeMatch.handler
					.call(request, ArcResponse::new())
					.map(|res| res.into());
				return box response;
			} else {
				return box Ok(Response::new().with_status(StatusCode::NotFound)).into_future()
			}
		} else {
			return box Ok(Response::new().with_status(StatusCode::NotFound)).into_future()
		}
//	 	if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {


//			let modifiedRequest: Result<Response, hyper::Error> = match self.middleware {
//				Some(ref middleware) => {
//					let response = await!(middleware.call(request));
//					let response: Result<Response, hyper::Error> = await!(
//						routeMatch.handler.call(request, ArcResponse::new()).map(|res| res.into())
//					);
//					Ok(response);
//				},
//				_ => hyper::Error::Method
//			};
//			}
	}
}

impl ArcRouter {
	pub fn new() -> Self {
		Self { routes: Arc::new(Mutex::new(HashMap::new())) }
	}

//	pub(crate) fn matchRoute<P>(&self, route: P, method: &Method) -> Option<Match<&Box<ArcService>>>
//	where P: AsRef<str> {
//		if let Some(recognizer) = self.routes.get(method) {
//			return recognizer.recognize(route.as_ref()).ok();
//		} else {
//			None
//		}
//	}

//	pub fn middleware(self, middleware: Box<MiddleWare<ArcRequest>>) -> Self {
//		self.middleware = Some(middleware);
//
//		self
//	}

	pub fn routes(self, group: RouteGroup) -> Self {
		let RouteGroup { routes, .. } = group;
		{
			let mut selfRoutes = self.routes.lock().unwrap();

			for (path, (method, handler)) in routes.into_iter() {
				selfRoutes
					.entry(method)
					.or_insert(Recognizer::new())
					.add(path.as_str(), handler)
			}
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

	fn route<S>(self, method: Method, path: &'static str, handler: S) -> Self
		where
			S: ArcService + 'static + Send + Sync
	{
		{
			let mut routes = self.routes.lock().unwrap();

			routes
				.entry(method)
				.or_insert(Recognizer::new())
				.add(path.as_ref(), box handler);
		}

		self
	}
}
