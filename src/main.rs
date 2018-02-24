#![feature(proc_macro, box_syntax, generators, conservative_impl_trait)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate anymap;
extern crate futures_await as futures;
extern crate hyper;
extern crate impl_service;
extern crate native_tls;
extern crate num_cpus;
extern crate route_recognizer as recognizer;
extern crate tokio_core;
extern crate tokio_tls;


#[macro_use]
mod ArcProto;
mod ArcRouting;
mod ArcCore;

use impl_service::{middleware, service};
use hyper::{StatusCode};
use futures::future::Future;
use futures::prelude::async_block;

use ArcCore::*;
use ArcRouting::*;
#[macro_use]
use ArcProto::*;

fn getMainRoutes() -> Router {
	let router: Router = Router::new()
		.before(LogMiddleware)
		.get("/", RequestHandler)
		.get(
			"/:username",
			arc!(mw![middleware1, middleware2], RequestHandler),
		)
		.get("/hello", RequestHandler);

	return router;
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
	let url = request.params().unwrap();
	println!("[RequestHandler]: Params {:?}", url);
	let res = res.with_status(StatusCode::Ok).with_body("Well Hello there");

	Ok(res)
}

#[middleware(Request)]
fn middleware1(req: Request) {
	println!("[middleware1]: middleware 1");
	Ok(req)
}

#[middleware(Request)]
fn middleware2(req: Request) {
	println!("[middleware2]: middleware 2");
	Ok(req)
}

#[middleware(Request)]
fn LogMiddleware(req: Request) {
	println!("[LogMiddleware]: called on {}", req.path());
	let _res = Response::new()
		.with_body("Lol, that didn't work");

	Ok(req)
}
