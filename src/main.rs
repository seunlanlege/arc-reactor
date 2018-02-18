#![feature(
proc_macro,
box_syntax,
generators,
)]

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(conservative_impl_trait)]

extern crate anymap;
extern crate native_tls;
extern crate tokio_tls;
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
use hyper::{Error, StatusCode};
use futures::future::Future;
use futures::prelude::{async_block};
use futures::IntoFuture;
use std::sync::Arc;

use ArcCore::*;
use ArcRouting::*;
use ArcProto::*;

fn getMainRoutes() -> Router {
	let router: Router = Router::new()
		.get("/:username", arc!(mw![middleware1, middleware2], RequestHandler));

	return router
}

fn main() {
	ArcReactor::new()
		.port(8443)
		.routes(getMainRoutes())
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(request: Request, res: Response) {
	let url = request.params().unwrap();
	let body = format!("Hello {}", url["username"]);
	let res =	res
		.with_status(StatusCode::Ok)
		.with_body(body);

	Ok(res)
}


#[middleware(Request)]
fn middleware1(req: Request){
	println!("params {:?}", req.params());
	Ok(req)
}

#[middleware(Request)]
fn middleware2(req: Request) {
	Ok(req)
}
