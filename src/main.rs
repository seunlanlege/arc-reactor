#![feature(
	proc_macro,
	box_syntax,
	type_ascription,
	conservative_impl_trait,
	generators,
	inclusive_range_syntax,
)]

#![allow(non_camel_case_types)]
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
#[macro_use]
mod ArcProto;

use impl_service::{service, middleware};
use ArcCore::ArcReactor;
use ArcRouting::*;
use hyper::{Response, Error, StatusCode};
use ArcCore::{Request};
use futures::future::Future;
use futures::prelude::{async_block};
use ArcProto::*;

fn getMainRoutes() -> ArcRouter {
	ArcRouter::new()
		.get("/", GetRequest)
		.post("/", (mw![middleware, middleware2], RequestHandler))
}

fn main() {
	ArcReactor::new()
		.port(3000)
		.routes(getMainRoutes())
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(_request: Request, res: Response) {
	println!("Post Request!");
	let res =	res
		.with_status(StatusCode::Ok)
		.with_body("Hello World");

	Ok(res)
}

#[middleware]
fn middleware(req: Request){
	println!("middleware 1: {:?}", &req);
	if req.path() != "/" {
		return arc::Err("Failed to get the data!".into())
	}

	return arc::Ok(req)
}

#[middleware]
fn middleware2(req: Request) {
	println!("middleware 2: {}, {}", req.path(), req.method());
	if req.path() != "/" {
		return arc::Err((401, "failed to acquire type info!").into())
	}
	
	if false {
		return arc::Res((401, "failed to acquire type info!").into())
	}

	return arc::Ok(req)
}

#[service]
fn GetRequest(_req: Request, res: Response) {
	return Ok(
		res
		.with_body("hello world".as_bytes())
	)
}
