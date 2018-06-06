#![feature(proc_macro, generators, proc_macro_non_items)]
extern crate arc_reactor;

use arc_reactor::{
	contrib::BodyParser,
	core::{ArcReactor, Request},
	prelude::*,
	routing::{RouteGroup, Router},
};

/// Setup and mounts routes to actions
fn get_main_routes() -> Router {
	// create a routegroup.
	let group = RouteGroup::new("api")
		.before(BodyParser) // mount the bodyparser request middleware on this routegroup.
		.get("/route", IndexService); // you can also nest a routegroup on a routegroup, node-style.

	Router::new() // you can also mount a routegroup on the router.
		.before(VerifyAuth) // mount a request middleware, for all the routes on the router. including the nested routes.
		.get("/", IndexService)
		.group(group)
}

fn main() {
	ArcReactor::new()
		.routes(get_main_routes())
		.initiate()
		.unwrap()
}

/// Route handlers must return Result<Response, Response>
/// if a Route handler returns Ok(Response)
/// that response is passed to the Response Middlewares in the chain;
///
/// Otherwise if it returns Err(Response)
/// the response middlewares are skipped and the response is forwarded directly
/// to the client.
///
#[service]
fn IndexService(req: Request, res: Response) {
	println!("Request Path => {}", req.path());
	Ok(res)
}

/// Middlewares must return Result<Request, Response>;
/// If a request middleware returns Ok(request)
/// the returned request is passed on to the next middleware in the chain (if
/// there is one) or the route handler.
///
/// If a middleware returns Err(response),
/// that response is forwarded directly to the client
///
#[middleware(Request)]
fn VerifyAuth(req: Request) {
	if let Ok(is_auth) = await!(get_isauth()) {
		// await! is exported in the arc_reactor::prelude.
		if is_auth {
			return Ok(req);
		}
	}

	Err((401, "Unauthorized!").into()) // <T: Serialize> From<(i16, T)> is implemented for Response
}

/// Some fake future.
fn get_isauth() -> impl Future<Item = bool, Error = ()> {
	futures::future::ok(true)
}
