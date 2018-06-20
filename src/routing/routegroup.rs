use core::{Request, Response};
use proto::{ArcHandler, ArcService, MiddleWare};
use routing::stripTrailingSlash;

use hyper::Method;
use std::collections::HashMap;

/// The RouteGroup allows for conviniently nesting route handlers
/// and applying group middlewares for protected routes.
pub struct RouteGroup {
	pub(crate) parent: String,
	pub(crate) before: Option<Box<MiddleWare<Request>>>,
	pub(crate) after: Option<Box<MiddleWare<Response>>>,
	pub(crate) routes: HashMap<Method, HashMap<String, ArcHandler>>,
}

impl RouteGroup {
	/// Create a new routegroup with the specified parent.
	///
	/// ```rust, ignore
	/// let routegroup = RouteGroup::new("api");
	/// routegroup.get("/users", UserService); // this will match "/api/users"
	/// ```
	///
	pub fn new<T: ToString>(parent: T) -> Self {
		RouteGroup {
			parent: parent.to_string(),
			routes: HashMap::new(),
			before: None,
			after: None,
		}
	}

	/// Mount a routegroup on this routegroup.
	/// It will apply the middlewares already mounted
	/// on the Parent `RouteGroup` to all the routes on the child `RouteGroup`.
	///
	/// ```rust, ignore
	///  let routegroup = RouteGroup::new("api");
	///  routegroup.get("/users", UserService); // this will match "/api/users"
	///
	/// let nestedgroup = RouteGroup::new("admin");
	///  // by nesting it on `routegroup`, this will match "/api/admin/delete/user"
	///  let nestedgroup.get("/delete/user", DeleteService);
	///
	///  routegroup.group(nestedgroup);
	/// ```
	pub fn group(mut self, group: RouteGroup) -> Self {
		let RouteGroup {
			routes, ..
		} = group;
		let mut parent = self.parent.clone();

		if parent.starts_with("/") {
			parent.remove(0);
		}

		for (method, map) in routes.into_iter() {
			for (path, handler) in map.into_iter() {
				let fullPath = format!("/{}{}", parent, path);
				let mut handler = ArcHandler {
					before: self.before.clone(),
					handler: Box::new(handler),
					after: self.after.clone(),
				};
				self.routes
					.entry(method.clone())
					.or_insert(HashMap::new())
					.insert(fullPath, handler);
			}
		}

		self
	}

	/// Mount a request middleware on this routegroup.
	///
	/// Ensure that the request middleware is added before any routes on the
	/// route group. The middleware only applies to the routes that are added
	/// after it has been mounted.
	pub fn before<T: 'static + MiddleWare<Request>>(mut self, before: T) -> Self {
		self.before = Some(Box::new(before));

		self
	}

	/// Mount a response middleware on this routegroup.
	///
	/// Ensure that the response middleware is added before any routes on the
	/// route group. The middleware only applies to the routes that are added
	/// after it has been mounted.
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
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
			handler: Box::new(handler),
			after: Some(Box::new(after)),
		};
		self.route(Method::DELETE, route, handler)
	}

	fn route<T: ToString, S: ArcService + 'static + Send + Sync>(
		mut self,
		method: Method,
		path: T,
		routehandler: S,
	) -> Self {
		let mut parent = self.parent.clone();

		if parent.len() == 0 {
			panic!("RouteGroup cannot have an empty parent route!")
		}

		if parent.starts_with("/") && parent.len() == 1 {
			parent = "".to_owned();
		} else if !parent.starts_with("/") {
			parent.insert(0, '/');
		}

		let mut path = path.to_string();

		path = stripTrailingSlash(&path).to_owned();
		if !path.starts_with("/") && path.len() > 1 {
			path.insert(0, '/');
		}
		let fullPath = format!("{}{}", &parent, path);

		let handler = ArcHandler {
			before: self.before.clone(),
			handler: Box::new(routehandler),
			after: self.after.clone(),
		};

		self.routes
			.entry(method)
			.or_insert(HashMap::new())
			.insert(fullPath, handler);

		self
	}
}
