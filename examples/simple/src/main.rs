#![feature(proc_macro, box_syntax, generators, proc_macro_non_items)]
#![allow(non_camel_case_types, non_snake_case)]

extern crate arc_reactor;
use arc_reactor::{
	contrib::{Multipart, StaticFileServer},
	core::{ArcReactor, Request},
	mime,
	prelude::*,
	routing::Router,
};
use std::path::PathBuf;

fn getMainRoutes() -> Router {
	// Setup and maps routes to Services.
	return Router::new().get("/", RequestHandler).post2(
		"/",
		Multipart::new(
			PathBuf::from("./".to_string()),
			Some(vec![mime::IMAGE_JPEG]),
			None,
		),
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
