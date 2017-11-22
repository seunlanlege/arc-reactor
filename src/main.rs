#![feature(
	proc_macro,
	box_syntax,
	generators,
)]

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate anymap;
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

use recognizer::Params;
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
		.post("/", (mw![middleware, middleware2], RequestHandler))
		.post("/:username", (mw![middleware, middleware2], RequestHandler))
}

fn main() {
	ArcReactor::new()
		.port(3000)
		.routes(getMainRoutes())
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(request: Request, res: Response) {
	let url = request.map.get::<Params>().unwrap();
	let body = format!("Hello {}", url["username"]);
	let res =	res
		.with_status(StatusCode::Ok)
		.with_body(body);

	Ok(res)
}

#[middleware]
fn middleware(req: Request){
	arc::Ok(req)
}

#[middleware]
fn middleware2(req: Request) {
	arc::Ok(req)
}
