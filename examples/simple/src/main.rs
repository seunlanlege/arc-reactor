#![allow(non_camel_case_types, non_snake_case)]
#![feature(proc_macro, proc_macro_non_items, generators)]
extern crate arc_reactor;
use arc_reactor::{
	core::{ArcReactor, Request},
	prelude::*,
	routing::Router,
};

fn getMainRoutes() -> Router {
	// Setup and maps routes to Services.
	return Router::new().get("/", RequestHandler).post(
		"/",
		RequestHandler,
	);
}

fn main() {
	// Start the reactor and try visiting localhost:3000/your-name
	ArcReactor::new()
		.port(3000) // port to listen on
		.routes(getMainRoutes())
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(_req: Request, mut res: Response) {
	res.text("hello world");
	Ok(res)
}
