use super::recognizer::{Match, Router as Recognizer};
use core::{Request, Response};
use futures::IntoFuture;
use hyper::Method;
use proto::{ArcHandler, ArcService, FutureResponse, MiddleWare};
use routing::{stripTrailingSlash, RouteGroup};
use std::collections::HashMap;

/// The main router of you application that is supplied to the ArcReactor.
///
#[derive(Clone)]
pub struct Router {
	pub(crate) routes: HashMap<Method, Recognizer<ArcHandler>>,
	pub(crate) before: Option<Box<MiddleWare<Request>>>,
	pub(crate) after: Option<Box<MiddleWare<Response>>>,
	pub(crate) notFound: Option<Box<ArcService>>,
}

impl Router {
	/// Construct a new Router.
	pub fn new() -> Self {
		Self {
			before: None,
			routes: HashMap::new(),
			after: None,
			notFound: None,
		}
	}

	/// Mount a routegroup on this router.
	/// It will apply the middlewares already mounted
	/// on the `Router` to all the routes on the `RouteGroup`
	///
	/// ```rust, ignore
	///  let router = Router::new();
	///  router.get("/users", UserService); // this will match "/users"
	///
	/// let nestedgroup = RouteGroup::new("admin");
	///  // by nesting it on `router`, this will match "/admin/delete/user"
	///  let nestedgroup.get("/delete/user", DeleteService);
	///
	///  router.group(nestedgroup);
	/// ```
	pub fn group(mut self, group: RouteGroup) -> Self {
		let RouteGroup { routes, .. } = group;
		{
			for (method, map) in routes.into_iter() {
				for (path, handler) in map {
					let handler = ArcHandler {
						before: self.before.clone(),
						handler: Some(Box::new(handler)),
						after: self.after.clone(),
					};

					self.routes
						.entry(method.clone())
						.or_insert(Recognizer::new())
						.add(path.as_str(), handler)
				}
			}
		}

		self
	}

	/// Mount a request middleware on this router.
	///
	/// Ensure that the request middleware is added before any routes on the
	/// router. The middleware only applies to the routes that are added after
	/// it has been mounted.
	pub fn before<T: 'static + MiddleWare<Request>>(mut self, before: T) -> Self {
		self.before = Some(Box::new(before));

		self
	}

	/// Mount a reesponse middleware on this router
	///
	/// Ensure that the request middleware is added before any routes on the
	/// router. The middleware only applies to the routes that are added after
	/// it has been mounted.
	pub fn after<T: 'static + MiddleWare<Response>>(mut self, after: T) -> Self {
		self.after = Some(Box::new(after));

		self
	}

	/// Add a route and a ServiceHandler for a GET request.
	pub fn get<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::GET, route, handler)
	}

	pub fn get2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: None,
		};
		self.route(Method::GET, route, handler)
	}

	pub fn get3<S, ReqMW, ResMW>(
		self,
		route: &'static str,
		before: ReqMW,
		handler: S,
		after: ResMW,
	) -> Self
	where
		S: ArcService + 'static,
		ReqMW: MiddleWare<Request> + 'static,
		ResMW: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::GET, route, handler)
	}

	pub fn get4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::GET, route, handler)
	}

	/// Add a route and a Service for a POST request.
	pub fn post<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::POST, route, handler)
	}

	/// mount a Service as well as a MiddleWare<Request>
	/// for a POST request
	pub fn post2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: None,
		};
		self.route(Method::POST, route, handler)
	}

	/// mount a Service as well as a MiddleWare<Request> and
	/// MiddleWare<Response> for a POST request
	pub fn post3<S, ReqMW, ResMW>(
		self,
		route: &'static str,
		before: ReqMW,
		handler: S,
		after: ResMW,
	) -> Self
	where
		S: ArcService + 'static,
		ReqMW: MiddleWare<Request> + 'static,
		ResMW: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::POST, route, handler)
	}

	/// mount a Service as well as a MiddleWare<Response>
	/// for a POST request
	pub fn post4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::GET, route, handler)
	}

	/// add a route and a ServiceHandler for a put request
	pub fn put<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::PUT, route, handler)
	}

	pub fn put2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: None,
		};
		self.route(Method::PUT, route, handler)
	}

	pub fn put3<S, ReqMW, ResMW>(
		self,
		route: &'static str,
		before: ReqMW,
		handler: S,
		after: ResMW,
	) -> Self
	where
		S: ArcService + 'static,
		ReqMW: MiddleWare<Request> + 'static,
		ResMW: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::PUT, route, handler)
	}

	pub fn put4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::PUT, route, handler)
	}

	/// Add a route and a ServiceHandler for a PATCH request.
	pub fn patch<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::PATCH, route, handler)
	}

	pub fn patch2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: None,
		};
		self.route(Method::PATCH, route, handler)
	}

	pub fn patch3<S, ReqMW, ResMW>(
		self,
		route: &'static str,
		before: ReqMW,
		handler: S,
		after: ResMW,
	) -> Self
	where
		S: ArcService + 'static,
		ReqMW: MiddleWare<Request> + 'static,
		ResMW: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::PATCH, route, handler)
	}

	pub fn patch4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::PATCH, route, handler)
	}

	/// Add a route and a ServiceHandler for DELETE request.
	pub fn delete<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::DELETE, route, handler)
	}

	pub fn delete2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: None,
		};
		self.route(Method::DELETE, route, handler)
	}

	pub fn delete3<S, ReqMW, ResMW>(
		self,
		route: &'static str,
		before: ReqMW,
		handler: S,
		after: ResMW,
	) -> Self
	where
		S: ArcService + 'static,
		ReqMW: MiddleWare<Request> + 'static,
		ResMW: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::DELETE, route, handler)
	}

	pub fn delete4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Some(Box::new(handler)),
			after: Some(Box::new(after)),
		};
		self.route(Method::DELETE, route, handler)
	}

	/// Add a 404 handler.
	pub fn notFound<S>(mut self, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.notFound = Some(Box::new(handler));

		self
	}

	fn route<S>(mut self, method: Method, path: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		{
			let handler = ArcHandler {
				before: self.before.clone(),
				handler: Some(Box::new(handler)),
				after: self.after.clone(),
			};
			self.routes
				.entry(method)
				.or_insert(Recognizer::new())
				.add(path.as_ref(), handler);
		}

		self
	}

	pub(crate) fn matchRoute<P>(&self, route: P, method: &Method) -> Option<Match<&ArcHandler>>
	where
		P: AsRef<str>,
	{
		let route = stripTrailingSlash(route.as_ref());
		self.routes
			.get(method)
			.and_then(|recognizer| recognizer.recognize(route).ok())
	}
}

impl ArcService for Router {
	fn call(&self, req: Request, res: Response) -> FutureResponse {
		if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {
			let mut request: Request = req.into();
			request.set(routeMatch.params);
			return ArcService::call(&*routeMatch.handler, request, res);
		} else {
			if let Some(ref notFound) = self.notFound {
				info!("No service registered for route {} and method {}", req.path(), req.method());
				return notFound.call(req, res);
			}
			let responseFuture = Ok(Response::new().with_status(404)).into_future();

			return Box::new(responseFuture);
		}
	}
}
