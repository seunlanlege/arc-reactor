#![feature(
	proc_macro,
	conservative_impl_trait,
	generators,
	inclusive_range_syntax,
	conservative_impl_trait,
	catch_expr,
	associated_type_defaults
)]

#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(deprecated)]

#[macro_use]
extern crate impl_service;
extern crate tokio_core;
extern crate route_recognizer as recognizer;
extern crate futures_await as futures;
extern crate hyper;

mod ArcRouting;
mod ArcCore;
mod ArcProto;

use impl_service::service;
use ArcCore::ArcReactor;
use ArcRouting::*;
use hyper::*;
use futures::future::Future;
use ArcProto::*;

fn main() {
	let routes = ArcRouter::new()
		.get("/hello/:name", RequestHandler);
		// .post("/", PostRequestHandler);

	ArcReactor::new()
		.port(3000)
		.routes(routes)
		.initiate();

}

#[service]
pub fn RequestHandler(req: hyper::Request) {
	Box::new(
		futures::future::ok(
			hyper::Response::new()
				.with_status(hyper::StatusCode::Ok)
				.with_body("Hello World".as_bytes())
		)
	)
}

#[service]
pub fn PostRequestHandler(req: hyper::Request) {
	Box::new(
		futures::future::ok(
			hyper::Response::new()
				.with_status(hyper::StatusCode::Ok)
				.with_body("Hello World".as_bytes())
		)
	)
}
