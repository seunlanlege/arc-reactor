use hyper::Method;
use ArcProto::ArcService;

pub trait RouteInterface<S: ArcService> : Sized {
	fn route(self, method: Method, path: &'static str, handler: S) -> Self;

	fn get(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Get, route, handler)
	}

	fn post(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Post, route, handler)
	}

	fn put(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Put, route, handler)
	}

	fn patch(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Patch, route, handler)
	}

	fn delete(self, route: &'static str, handler: S) -> Self {
		self.route(Method::Delete, route, handler)
	}
}