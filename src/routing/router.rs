use hyper::{Method, StatusCode};
use futures::{Future, IntoFuture};
use std::collections::HashMap;
use recognizer::{Match, Router as Recognizer};
use proto::{ArcHandler, ArcService, MiddleWare};
use routing::{RouteGroup, stripTrailingSlash};
use core::{Request, Response};

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
				for (path, routehandler) in map {
					let handler = ArcHandler {
						before: self.before.clone(),
						handler: box routehandler,
						after: self.after.clone(),
					};

					self
						.routes
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
	/// Ensure that the request middleware is added before any routes on the router.
	/// The middleware only applies to the routes that are added after it has been mounted.
	pub fn before<T: 'static + MiddleWare<Request>>(mut self, before: T) -> Self {
		self.before = Some(box before);

		self
	}

	/// Mount a reesponse middleware on this router
	///
	/// Ensure that the request middleware is added before any routes on the router.
	/// The middleware only applies to the routes that are added after it has been mounted.
	pub fn after<T: 'static + MiddleWare<Response>>(mut self, after: T) -> Self {
		self.after = Some(box after);

		self
	}

	/// Add a route and a ServiceHandler for a GET request.
	pub fn get<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Get, route, handler)
	}

	/// Add a route and a ServiceHandler for a POST request.
	pub fn post<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Post, route, handler)
	}

	/// add a route and a ServiceHandler for a put request
	pub fn put<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Put, route, handler)
	}

	/// Add a route and a ServiceHandler for a PATCH request.
	pub fn patch<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Patch, route, handler)
	}

	/// Add a route and a ServiceHandler for DELETE request.
	pub fn delete<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.route(Method::Delete, route, handler)
	}

	/// Add a 404 handler.
	pub fn notFound<S>(mut self, handler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		self.notFound = Some(box handler);

		self
	}

	fn route<S>(mut self, method: Method, path: &'static str, routehandler: S) -> Self
	where
		S: ArcService + 'static + Send + Sync,
	{
		{
			let handler = ArcHandler {
				before: self.before.clone(),
				handler: box routehandler,
				after: self.after.clone(),
			};
			self
				.routes
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
		self
			.routes
			.get(method)
			.and_then(|recognizer| recognizer.recognize(route).ok())
	}
}

impl ArcService for Router {
	fn call(&self, req: Request, res: Response) -> Box<Future<Item = Response, Error = Response>> {
		if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {
			let mut request: Request = req.into();
			request.set(routeMatch.params);

			return box ArcService::call(&*routeMatch.handler, request, res);
		} else {
			if let Some(ref notFound) = self.notFound {
				return notFound.call(req, res);
			}
			return box Ok(Response::new().with_status(StatusCode::NotFound)).into_future();
		}
	}
}
