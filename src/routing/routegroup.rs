use routing::stripTrailingSlash;
use std::sync::Arc;
#[macro_use]
use proto::{ArcHandler, ArcService, MiddleWare};
use core::{Request, Response};

use hyper::Method;
use std::collections::HashMap;

/// The routegroup allows for conviniently nesting route handlers
/// and applying group middlewares for protected routes

pub struct RouteGroup {
	pub(crate) parent: String,
	pub(crate) before: Option<Arc<Box<MiddleWare<Request>>>>,
	pub(crate) after: Option<Arc<Box<MiddleWare<Response>>>>,
	pub(crate) routes: HashMap<Method, HashMap<String, ArcHandler>>,
}

impl RouteGroup {
	/// create a new routegroup with the specified parent
	///
	/// ```
	///   let routegroup = RouteGroup::new("api");
	///   routegroup.get("/users", UserService); // this will match "/api/users"
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
	/// on the Parent `RouteGroup` to all the routes on the child `RouteGroup`
	///
	/// ```
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
		let RouteGroup { routes, .. } = group;
		let mut parent = self.parent.clone();

		if parent.starts_with("/") {
			parent.remove(0);
		}

		for (method, map) in routes.into_iter() {
			for (path, handler) in map.into_iter() {
				let fullPath = format!("/{}{}", parent, path);
				let mut handler = ArcHandler {
					before: self.before.clone(),
					handler: Arc::new(box handler),
					after: self.after.clone(),
				};
				self
					.routes
					.entry(method.clone())
					.or_insert(HashMap::new())
					.insert(fullPath, handler);
			}
		}

		self
	}

	/// mount a request middleware on this routegroup
	///
	/// ensure that the request middleware is added before any routes on the route group.
	/// the middleware only applies to the routes that are added after it has been mounted.
	pub fn before<T: 'static + MiddleWare<Request>>(mut self, before: T) -> Self {
		self.before = Some(Arc::new(box before));

		self
	}

	/// mount a response middleware on this routegroup
	///
	/// ensure that the response middleware is added before any routes on the route group.
	/// the middleware only applies to the routes that are added after it has been mounted.
	pub fn after<T: 'static + MiddleWare<Response>>(mut self, after: T) -> Self {
		self.after = Some(Arc::new(box after));

		self
	}

	/// add a route and a ServiceHandler for a get request
	pub fn get<S: ArcService + 'static + Send + Sync>(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Get, route, handler)
	}

	/// add a route and a ServiceHandler for a post request
	pub fn post<S: ArcService + 'static + Send + Sync>(
		self,
		route: &'static str,
		handler: S,
	) -> Self {
		self.route(Method::Post, route, handler)
	}

	/// add a route and a ServiceHandler for a put request
	pub fn put<S: ArcService + 'static + Send + Sync>(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Put, route, handler)
	}

	/// add a route and a ServiceHandler for a patch request
	pub fn patch<S: ArcService + 'static + Send + Sync>(
		self,
		route: &'static str,
		handler: S,
	) -> Self {
		self.route(Method::Patch, route, handler)
	}

	/// add a route and a ServiceHandler for a delete request
	pub fn delete<S: ArcService + 'static + Send + Sync>(
		self,
		route: &'static str,
		handler: S,
	) -> Self {
		self.route(Method::Delete, route, handler)
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
			handler: Arc::new(box routehandler),
			after: self.after.clone(),
		};
		
		self
			.routes
			.entry(method)
			.or_insert(HashMap::new())
			.insert(fullPath, handler);

		self
	}
}
