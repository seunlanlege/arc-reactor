#![feature(proc_macro, box_syntax, generators)]
#![allow(non_camel_case_types, non_snake_case)]

#[macro_use]
extern crate arc_reactor;
use arc_reactor::{ArcReactor, Router};
use arc_reactor::prelude::*;

fn getMainRoutes() -> Router {
	return Router::new()
		.get("/:username", arc!(mw![middleware1, middleware2], RequestHandler, box middleware3))
}

fn main() {
	// start the reactor and try visiting localhost:3000/your-name
	ArcReactor::new()
		.port(3000) // port to listen on
		.routes(getMainRoutes())
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(request: Request, mut res: Response) {
	// you can unwrap here because this handler wouldn't be matched without the param.
	let url = request.params().unwrap();
	let body = format!("Hello {}", url["username"]);
	res.set_body(body);

	Ok(res)
}

#[middleware(Request)] // This is a request middleware,
fn middleware1(req: Request) {
	println!("[middleware1]");
	Ok(req) // if this middleware returns an Err(response), middleware2 and RequestHandler, would never be called
}

#[middleware(Request)]
fn middleware2(req: Request) {
	println!("[middleware2]");
	Ok(req)
}

#[middleware(Response)]
fn middleware3(res: Response) {
	println!("[middleware3]");
	Ok(res)
}
