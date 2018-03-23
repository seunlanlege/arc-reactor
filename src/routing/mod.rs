mod router;
mod routegroup;
mod util;

pub use self::routegroup::*;
pub use self::router::*;
pub use self::util::*;

#[cfg(test)]
mod tests {
	use impl_service::*;
	use hyper::{Method, StatusCode};
	use futures::Future;
	use futures::prelude::async_block;
	use proto::ArcService;
	use core::{Request, Response};

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
		let router = Router::new().get("/hello", AsyncService);

		let shouldExist = router.matchRoute("/hello", &Method::Get);
		let shouldNotExist = router.matchRoute("/world", &Method::Get);

		assert!(shouldExist.is_some());
		assert!(shouldNotExist.is_none());
	}

	#[test]
	fn it_matches_nested_routes() {
		let routegroup = RouteGroup::new("admin")
			.get("/roles", AsyncService)
			.get("/", AsyncService);
		let router = Router::new().group(routegroup);

		let shouldExist = router.matchRoute("/admin/roles", &Method::Get);
		let shouldExist1 = router.matchRoute("/admin", &Method::Get);
		let shouldExist2 = router.matchRoute("/admin/roles/", &Method::Get);
		let shouldExist3 = router.matchRoute("/admin/", &Method::Get);

		assert!(shouldExist.is_some());
		assert!(shouldExist1.is_some());
		assert!(shouldExist2.is_some());
		assert!(shouldExist3.is_some());
	}

	#[test]
	fn it_matches_routes_with_params() {
		let router = Router::new().get("/hello/:name", AsyncService);

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
		let subrouter = RouteGroup::new("users").get("/profile", AsyncService);

		let routegroup = RouteGroup::new("admin")
			.get("/roles", AsyncService)
			.group(subrouter);

		let router = Router::new().group(routegroup);


		let shouldExist = router.matchRoute("/admin/roles/", &Method::Get);
		let shouldExist1 = router.matchRoute("/admin/users/profile/", &Method::Get);

		assert!(shouldExist.is_some());
		assert!(shouldExist1.is_some());
	}
}
