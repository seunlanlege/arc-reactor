use core::{Request, Response};
use futures::{Future, IntoFuture};
use hyper::{Method, StatusCode};
use proto::{ArcHandler, ArcService, MiddleWare};
use super::recognizer::{Match, Router as Recognizer};
use routing::{stripTrailingSlash, RouteGroup};
use std::{collections::HashMap, fmt};


/// The main router of you application that is supplied to the ArcReactor.
///
#[derive(Clone)]
pub struct Router {
	pub(crate) routes: HashMap<Method, Recognizer<ArcHandler>>,
	pub(crate) before: Option<Box<MiddleWare<Request>>>,
	pub(crate) after: Option<Box<MiddleWare<Response>>>,
	pub(crate) notFound: Option<Box<ArcService>>,
	pub(crate) wildcards: HashMap<String, ArcHandler>,
}

impl fmt::Debug for Router {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Router")
            .field("before", &self.before)
            .field("handler", &self.routes)
            .field("after", &self.after)
            .finish()
    }
}


impl Router {
	/// Construct a new Router.
	pub fn new() -> Self {
		Self {
			before: None,
			routes: HashMap::new(),
			after: None,
			notFound: None,
			wildcards: HashMap::new(),
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
		let RouteGroup {
			routes, wildcards, ..
		} = group;
		{
			for (method, map) in routes.into_iter() {
				for (path, handler) in map {
					let handler = ArcHandler {
						before: self.before.clone(),
						handler: Box::new(handler),
						after: self.after.clone(),
					};

					self.routes
						.entry(method.clone())
						.or_insert(Recognizer::new())
						.add(path.as_str(), handler)
				}
			}

			for (route, handler) in wildcards.into_iter() {
				let mut handler = ArcHandler {
					before: self.before.clone(),
					handler: Box::new(handler),
					after: self.after.clone(),
				};
				self.wildcards.insert(route, handler);
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
		self.route(Method::Get, route, handler)
	}

	pub fn get2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Box::new(handler),
			after: None,
		};
		self.route(Method::Get, route, handler)
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
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Get, route, handler)
	}

	pub fn get4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Get, route, handler)
	}

	/// Add a route and a Service for a POST request.
	pub fn post<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::Post, route, handler)
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
			handler: Box::new(handler),
			after: None,
		};
		self.route(Method::Post, route, handler)
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
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Post, route, handler)
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
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Get, route, handler)
	}

	/// add a route and a ServiceHandler for a put request
	pub fn put<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::Put, route, handler)
	}

	pub fn put2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Box::new(handler),
			after: None,
		};
		self.route(Method::Put, route, handler)
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
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Put, route, handler)
	}

	pub fn put4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Put, route, handler)
	}

	/// Add a route and a ServiceHandler for a PATCH request.
	pub fn patch<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::Patch, route, handler)
	}

	pub fn patch2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Box::new(handler),
			after: None,
		};
		self.route(Method::Patch, route, handler)
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
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Patch, route, handler)
	}

	pub fn patch4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Patch, route, handler)
	}

	/// Add a route and a ServiceHandler for DELETE request.
	pub fn delete<S>(self, route: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.route(Method::Delete, route, handler)
	}

	pub fn delete2<S, M>(self, route: &'static str, before: M, handler: S) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Request> + 'static,
	{
		let handler = ArcHandler {
			before: Some(Box::new(before)),
			handler: Box::new(handler),
			after: None,
		};
		self.route(Method::Delete, route, handler)
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
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Delete, route, handler)
	}

	pub fn delete4<S, M>(self, route: &'static str, handler: S, after: M) -> Self
	where
		S: ArcService + 'static,
		M: MiddleWare<Response> + 'static,
	{
		let handler = ArcHandler {
			before: None,
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::Delete, route, handler)
	}

	/// Add a 404 handler.
	pub fn notFound<S>(mut self, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		self.notFound = Some(Box::new(handler));

		self
	}

	pub fn any<S, R>(mut self, route: R, handler: S) -> Self
	where
		S: ArcService + 'static,
		R: ToString,
	{
		let route = route.to_string();
		let handler = ArcHandler {
			before: self.before.clone(),
			handler: Box::new(handler),
			after: self.after.clone(),
		};

		self.wildcards.insert(route, handler);
		self
	}

	fn route<S>(mut self, method: Method, path: &'static str, handler: S) -> Self
	where
		S: ArcService + 'static,
	{
		{
			let handler = ArcHandler {
				before: self.before.clone(),
				handler: Box::new(handler),
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

	pub(crate) fn match_wildcard(&self, route: &str) -> Option<&ArcHandler> {
		for (path, handler) in self.wildcards.iter() {
			if route.contains(path) {
				return Some(handler);
			}
		}
		None
	}
}

impl ArcService for Router {
	fn call(&self, req: Request, res: Response) -> Box<Future<Item = Response, Error = Response>> {
		let wildcard = self.match_wildcard(req.path());
		if let Some(routeMatch) = self.matchRoute(req.path(), req.method()) {
			let mut request: Request = req.into();
			request.set(routeMatch.params);
			return ArcService::call(&*routeMatch.handler, request, res);
		} else if wildcard.is_some() {
			return ArcService::call(wildcard.unwrap(), req.into(), res);
		} else {
			if let Some(ref notFound) = self.notFound {
				return notFound.call(req, res);
			}
			let responseFuture = Ok(Response::new().with_status(StatusCode::NotFound)).into_future();

			return Box::new(responseFuture)
		}
	}
}
