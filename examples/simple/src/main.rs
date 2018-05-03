#![feature(proc_macro, box_syntax, generators)]
#![allow(non_camel_case_types, non_snake_case)]

#[macro_use]
extern crate arc_reactor;
use arc_reactor::prelude::*;
use arc_reactor::{ArcReactor, FutureResponse, Router};

fn getMainRoutes() -> Router {
	// Setup and maps routes to Services.
	return Router::new().get("/", RequestHandler);
}

fn main() {
	// Start the reactor and try visiting localhost:3000/your-name
	ArcReactor::new()
		.port(3000) // port to listen on
		.routes(getMainRoutes())
		.threads(7)
		.initiate()
		.unwrap()
}

// #[service]
fn RequestHandler(
	_request: Request,
	mut res: Response,
) -> FutureResponse {
	res.set_body("hello world");

	box Ok(res).into_future()
}
