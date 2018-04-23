#![feature(proc_macro, box_syntax, generators)]
#![allow(non_camel_case_types, non_snake_case)]

#[macro_use]
extern crate arc_reactor;
use arc_reactor::{ArcReactor, Router};
use arc_reactor::prelude::*;

fn getMainRoutes() -> Router {
	/// Setup and maps routes to actions.
	return Router::new()
		.get("/", RequestHandler)
}

fn main() {
	// Start the reactor and try visiting localhost:3000/your-name
	ArcReactor::new()
		.port(3000) // port to listen on
		.routes(getMainRoutes())
		.threads(2)
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(_request: Request, mut res: Response) {
	res.set_body("hello world");

	Ok(res)
}
