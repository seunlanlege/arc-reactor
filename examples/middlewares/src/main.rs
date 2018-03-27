#![feature(proc_macro, box_syntax, generators, conservative_impl_trait)]
extern crate arc_reactor;

use arc_reactor::prelude::*;
use arc_reactor::{ArcReactor, Router, futures};

fn get_main_routes() -> Router {
	Router::new() // you can also mount a routegroup on the router.
		.before(VerifyAuth) // mount a request middleware, for all the routes on the router.
		.get("/", IndexService)
}

fn main() {
	ArcReactor::new()
		.routes(get_main_routes())
		.initiate()
		.unwrap()
}

/// Route handlers must return Result<Response, Response>
/// if a Route handler returns Ok(Response)
/// that response is passed to the Response Middlewares in the chain
///
/// Otherwise if it returns Err(Response)
/// the response middlewares are skipped and the response is forwarded directly to the client.
///
#[service]
fn IndexService(_req: Request, res: Response) {
	Ok(res)
}

/// middlewares must return Result<Request, Response>
/// if a request middleware returns Ok(request)
/// the returned request is passed on to the next middleware in the chain (if there is one)
/// or the route handler.
///
/// If a middleware returns Err(response)
/// that response is forwarded directly to the client
///
///
#[middleware(Request)]
fn VerifyAuth(req: Request) {
	if let Ok(is_auth) = await!(get_isauth()) { // await! is exported in the arc_reactor::prelude.
		if is_auth {
			return Ok(req)
		}
	}

	Err((401, "Unauthorized!").into()) // <T: Serialize> From<(i32, T)> is implemented for Response
}


/// some fake future.
fn get_isauth() -> impl Future<Item=bool, Error=()> {
	futures::future::ok(true)
}
