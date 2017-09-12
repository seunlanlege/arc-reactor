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

#[macro_use]
extern crate impl_service;
extern crate num_cpus;
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
use hyper::{Request, Response, Error, StatusCode};
use futures::future::Future;
use futures::prelude::async_block;
use ArcProto::*;

fn main() {
	let routes = ArcRouter::new()
		.get("/", RequestHandler)
		.get("/seun", PostRequestHandler);

	ArcReactor::new()
		.port(3000)
		.routes(routes)
		.initiate()
		.unwrap()

}

#[service]
pub fn RequestHandler(_req: Request) {
	let res = Response::new()
		.with_body("Hello World".as_bytes());

		Result::Ok(res)
}

#[service]
pub fn PostRequestHandler(_req: Request) {
	let res =	Response::new()
		.with_status(StatusCode::Ok)
		.with_body("GET Seun ".as_bytes());

	Result::Ok(res)
}
