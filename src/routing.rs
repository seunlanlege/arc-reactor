use hyper::{Method, Response, StatusCode};
use std::collections::HashMap;
use recognizer::{Match, Router as Recognizer, Params};

trait Router {
	fn route<F>(&mut self, method: Method, path: &'static str, handler: F) -> &mut Self
	            where F: 'static + Fn() -> String;

	fn get<F>(&mut self, route: &'static str, handler: F) -> &mut Self
	          where F: 'static + Fn() -> String {
		self.route(Method::Get, route, handler)
	}

	fn post<F>(&mut self, route: &'static str, handler: F) -> &mut Self
	           where F: 'static + Fn() -> String {
		self.route(Method::Post, route, handler)
	}

	fn put<F>(&mut self, route: &'static str, handler: F) -> &mut Self
	          where F: 'static + Fn() -> String {
		self.route(Method::Put, route, handler)
	}

	fn patch<F>(&mut self, route: &'static str, handler: F) -> &mut Self
	            where F: 'static + Fn() -> String {
		self.route(Method::Patch, route, handler)
	}

	fn delete<F>(&mut self, route: &'static str, handler: F) -> &mut Self
	             where F: 'static + Fn() -> String {
		self.route(Method::Delete, route, handler)
	}
}

struct RouteGroup {
	parent: &'static str,
	routes: HashMap<String, (Method, Box<Fn() -> String>)>,
}

impl Router for RouteGroup {
	fn route<F>(&mut self, method: Method, path: &'static str, handler: F) -> &mut RouteGroup
	            where F: 'static + Fn() -> String {
		self.routes
		    .insert(format!("/{}{}", &self.parent, path), (method, Box::new(handler)));

		self
	}
}

struct ArcRouter {
	routes: HashMap<Method, Recognizer<Box<Fn() -> String>>>,
}

impl Router for ArcRouter {
	fn route<F>(&mut self, method: Method, path: &'static str, handler: F) -> &mut ArcRouter
	            where F: 'static + Fn() -> String {
		self.routes
		    .entry(method)
		    .or_insert(Recognizer::new())
		    .add(path.as_ref(), Box::new(handler));
		self
	}
}

impl ArcRouter {
	pub fn new() -> ArcRouter {
		ArcRouter { routes: HashMap::new() }
	}

	fn matchRoute<P>(&self, route: P, method: Method) -> Option<Match<&Box<Fn() -> String>>>
	                 where P: AsRef<str> {
		if let Some(recognizer) = self.routes.get(&method) {
			return recognizer.recognize(route.as_ref()).ok();
		} else {
			None
		}
	}

	pub fn group(parent: &'static str) -> RouteGroup {
		RouteGroup {
			parent,
			routes: HashMap::new(),
		}
	}

	pub fn add(&mut self, group: RouteGroup) -> &mut ArcRouter {
		let RouteGroup { routes, .. } = group;

		for (path, (method, handler)) in routes.into_iter() {
			self.routes
			    .entry(method)
			    .or_insert(Recognizer::new())
			    .add(path.as_str(), handler)
		}

		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ShouldMatchRoute() {
		let handler = || String::from("Hello World Handler");

		let mut route = ArcRouter::new();
		route.get("/users", handler);

		match route.matchRoute("/users", Method::Get) {
			Some(routeMatch) => {
				assert_eq!((**routeMatch.handler)(), "Hello World Handler".to_owned())
			}
			None => panic!("Whoops!!!"),
		};
	}

	#[test]
	fn ShouldMatchRouteWithParams() {
		let handler = || String::from("Hello World Handler");

		let mut route = ArcRouter::new();
		route.get("/users/:id", handler);

		match route.matchRoute("/users/267543", Method::Get) {
			Some(routeMatch) => {
				assert_eq!((**routeMatch.handler)(), "Hello World Handler".to_owned());
				assert_eq!(routeMatch.params.find("id"), Some("267543"));
			}
			None => panic!("Whoops!!!"),
		};
	}

	#[test]
	fn ShouldMatchRouteFromMultiple() {
		let handler = || String::from("UserHandler");

		let mut route = ArcRouter::new();
		let mut FooRouteGroup = ArcRouter::group("foo");

		FooRouteGroup
		.get("/bar", || String::from("Foo Handler"));
		route
		.get("/world", || String::from("Hello Handler"))
		.post("/hello", || String::from("World Handler"))
		.get("/users/:id", handler)
		.add(FooRouteGroup);

		match route.matchRoute("/users/267543", Method::Get) {
			Some(routeMatch) => {
				assert_eq!((**routeMatch.handler)(), "UserHandler".to_owned());
				assert_eq!(routeMatch.params.find("id"), Some("267543"));
			}
			None => panic!("Whoops!!!"),
		};
	}

	#[test]
	fn ShouldMatchRouteGroup() {
		let handler = || String::from("Hello World Handler");

		let mut route = ArcRouter::new();
		let mut routeGroup = ArcRouter::group("users");

		routeGroup
		.get("/:id", handler)
		.get("/profile", || String::from("profile"));
		route.add(routeGroup);

		match route.matchRoute("/users/455454", Method::Get) {
			Some(routeMatch) => {
				assert_eq!((**routeMatch.handler)(), "Hello World Handler".to_owned())
			}
			None => panic!("Whoops!!!"),
		};
	}
}
