mod router;
//mod routegroup;

//pub use self::routegroup::*;
pub use self::router::*;

#[cfg(test)]
mod tests {
	use impl_service::*;
	use hyper::*;
	use futures::Future;
	use ArcProto::ArcService;
	use futures::future;
	
	use super::*;
	
	#[service]
	fn AsyncService()  {
		Box::new(
			future::ok(
				Response::new()
					.with_status(StatusCode::Ok)
					.with_body("Hello World".as_bytes())
			)
		)
	}

	#[test]
	fn ShouldMatchRoute() {
		let mut route = ArcRouter::new();
		route.get("/users", AsyncService);

		match route.matchRoute("/users", &Method::Get) {
			Some(routeMatch) => {
				assert_eq!((**routeMatch.handler).mock(), "AsyncService".to_owned())
			}
			None => panic!("Whoops!!!"),
		};
	}

	// #[test]
	// fn ShouldMatchRouteWithParams() {
	// 	let handler = || String::from("Hello World Handler");

	// 	let mut route = ArcRouter::new();
	// 	route.get("/users/:id", handler);

	// 	match route.matchRoute("/users/267543", Method::Get) {
	// 		Some(routeMatch) => {
	// 			assert_eq!((**routeMatch.handler)(), "Hello World Handler".to_owned());
	// 			assert_eq!(routeMatch.params.find("id"), Some("267543"));
	// 		}
	// 		None => panic!("Whoops!!!"),
	// 	};
	// }

	// #[test]
	// fn ShouldMatchRouteFromMultiple() {
	// 	let handler = || String::from("UserHandler");

	// 	let mut route = ArcRouter::new();
	// 	let mut FooRouteGroup = ArcRouter::group("foo");

	// 	FooRouteGroup
	// 		.get("/bar", || String::from("Foo Handler"));
	// 	route
	// 		.get("/world", || String::from("Hello Handler"))
	// 		.post("/hello", || String::from("World Handler"))
	// 		.get("/users/:id", handler)
	// 		.add(FooRouteGroup);

	// 	match route.matchRoute("/users/267543", Method::Get) {
	// 		Some(routeMatch) => {
	// 			assert_eq!((**routeMatch.handler)(), "UserHandler".to_owned());
	// 			assert_eq!(routeMatch.params.find("id"), Some("267543"));
	// 		}
	// 		None => panic!("Whoops!!!"),
	// 	};
	// }

	// #[test]
	// fn ShouldMatchRouteGroup() {
	// 	let handler = || String::from("Hello World Handler");

	// 	let mut route = ArcRouter::new();
	// 	let mut routeGroup = ArcRouter::group("users");

	// 	routeGroup
	// 		.get("/:id", handler)
	// 		.get("/profile", || String::from("profile"));
	// 	route.add(routeGroup);

	// 	match route.matchRoute("/users/455454", Method::Get) {
	// 		Some(routeMatch) => {
	// 			assert_eq!((**routeMatch.handler)(), "Hello World Handler".to_owned())
	// 		}
	// 		None => panic!("Whoops!!!"),
	// 	};
	// }
}
