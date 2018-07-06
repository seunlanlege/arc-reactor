#![allow(non_camel_case_types, non_snake_case)]
#![feature(proc_macro, proc_macro_non_items, generators)]
extern crate arc_reactor;
use arc_reactor::{
	core::{ArcReactor, Request},
	prelude::*,
	routing::Router,
	contrib::MultiPart,
	tokio,
};

fn getMainRoutes() -> Router {
	// Setup and maps routes to Services.
	return Router::new()
		.get("/", RequestHandler)
		.post("/", RequestHandler);
}

fn main() {
	// Start the reactor and try visiting localhost:3000/your-name
	tokio::run(
		ArcReactor::new()
			.port(3000)
			.routes(getMainRoutes())
			.before(MultiPart { dir: ::std::path::PathBuf::from("/home/seun/Desktop"), mimes: None, size_limit: None })
			.start()
			.unwrap()
			.map_err(|err| println!("err {}", err)),
	)
}

#[service]
fn RequestHandler(req: Request, mut res: Response) {
	res.text("hello world");
	Ok(res)
}
