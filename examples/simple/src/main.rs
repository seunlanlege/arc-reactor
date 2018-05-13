#![feature(proc_macro, box_syntax, generators, proc_macro_non_items)]
#![allow(non_camel_case_types, non_snake_case, )]

extern crate arc_reactor;
use arc_reactor::prelude::*;
use arc_reactor::{core::ArcReactor, proto::FutureResponse, routing::Router, contrib::StaticFileServer};

fn getMainRoutes() -> Router {
	// Setup and maps routes to Services.
	return Router::new().get("/", RequestHandler);
}

fn main() {
	// Start the reactor and try visiting localhost:3000/your-name
	ArcReactor::new()
		.port(3000) // port to listen on
		.routes(getMainRoutes())
		.before(StaticFileServer { root: "public" })		
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(_request: Request, mut res: Response) -> FutureResponse {
	res.set_body("hello world");

	Ok(res)
}
