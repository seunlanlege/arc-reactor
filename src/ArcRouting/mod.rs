mod router;
mod routegroup;

pub use self::routegroup::*;
pub use self::router::*;

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use impl_service::*;
	use hyper::{Method, StatusCode};
	use futures::Future;
	use futures::prelude::{async_block};
	use ArcProto::ArcService;
	use ArcCore::{Response, Request};

	use super::*;

	#[service]
	fn AsyncService(_req: Request, res: Response) {
		let res = res
			.with_status(StatusCode::Ok)
			.with_body("Hello World".as_bytes());
		Result::Ok(res)
	}

	#[test]
	fn it_matches_the_correct_routes() {
		let router = Router::new()
			.get("/hello", AsyncService);
		let router = ArcRouter {
			routes: Arc::new(router.routes)
		};

		let shouldExist = router.matchRoute("/hello", &Method::Get);
		let shouldNotExist = router.matchRoute("/world", &Method::Get);

		assert!(shouldExist.is_some());
		assert!(shouldNotExist.is_none());
	}

	#[test]
	fn it_matches_nested_routes() {
		let routegroup = RouteGroup::new("admin")
			.get("/roles", AsyncService);
		let router = Router::new()
			.group(routegroup);
		let router = ArcRouter {
			routes: Arc::new(router.routes)
		};

		let shouldExist = router.matchRoute("/admin/roles", &Method::Get);
		let shouldNotExist1 = router.matchRoute("/admin", &Method::Get);
		let shouldNotExist2 = router.matchRoute("/admin/role", &Method::Get);
		let shouldNotExist3 = router.matchRoute("/admin/roless", &Method::Get);
		let shouldNotExist4 = router.matchRoute("/hello/world", &Method::Get);

		assert!(shouldExist.is_some());
		assert!(shouldNotExist1.is_none());
		assert!(shouldNotExist2.is_none());
		assert!(shouldNotExist3.is_none());
		assert!(shouldNotExist4.is_none());
	}

	#[test]
	fn it_matches_routes_with_params() {
		let router = Router::new()
			.get("/hello/:name", AsyncService);
		let router = ArcRouter {
			routes: Arc::new(router.routes)
		};

		let shouldExist = router.matchRoute("/hello/seun", &Method::Get);
		let shouldNotExist = router.matchRoute("/world/seun/lanlege", &Method::Get);

		assert!(shouldExist.is_some());
		let routeMatch = shouldExist.unwrap();
		assert_eq!(routeMatch.params["name"], "seun");
		assert_ne!(routeMatch.params["name"], "seunlanlege");
		assert!(shouldNotExist.is_none());
	}

	#[test]
	fn it_matches_deeply_nested_routes() {
		let subrouter = RouteGroup::new("users")
			.get("/profile", AsyncService);

		let routegroup = RouteGroup::new("admin")
			.get("/roles", AsyncService)
			.group(subrouter);

		let router = Router::new()
			.group(routegroup);

		let router = ArcRouter {
			routes: Arc::new(router.routes)
		};

		let shouldExist = router.matchRoute("/admin/roles", &Method::Get);
		let shouldExist1 = router.matchRoute("/admin/users/profile", &Method::Get);


		assert!(shouldExist.is_some());
		assert!(shouldExist1.is_some());
	}
}
