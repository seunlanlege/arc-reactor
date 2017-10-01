#![feature(
	proc_macro,
	conservative_impl_trait,
	generators,
	inclusive_range_syntax,
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

use impl_service::{service, middleware};
use ArcCore::ArcReactor;
use ArcRouting::*;
use hyper::{Request, Response, Error, StatusCode};
use futures::future::Future;
use futures::prelude::{async_block};
use ArcProto::*;

fn getMainRoutes() -> ArcRouter {
	// let mut middlewares = Vec::new();
	// middlewares.push(middleware1);
	// middlewares.push(middleware2);

	ArcRouter::new()
		.post("/", (middleware, RequestHandler))
}

fn main() {
	ArcReactor::new()
		.port(3000)
		.routes(getMainRoutes())
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(_request: Request) {
	println!("Post Request!");
	let res =	Response::new()
		.with_status(StatusCode::Ok)
		.with_body("Hello World");

	Ok(res)
}

#[middleware]
fn middleware(req: Request) -> ArcResult {
	println!("middleware 1: {}", req.path());
	if req.path() != "/" {
		return Err("Failed to get the data!".into())
	}

	return Ok(req)
}

