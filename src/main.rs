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
use futures::prelude::{async_block, await};
use ArcProto::*;

fn getMainRoutes() -> ArcRouter {
	ArcRouter::new()
		.post("/", PostRequestHandler)
}

fn main() {
	ArcReactor::new()
		.port(3000)
		.routes(getMainRoutes())
		.initiate()
		.unwrap()
}

#[service]
pub fn PostRequestHandler(request: Request) {
//	let body = await!(request.body().concat2());
	
	let res =	Response::new()
		.with_status(StatusCode::Ok)
		.with_body("GET Seun".as_bytes());

	Result::Ok(res)
}
