#![allow(non_camel_case_types, non_snake_case)]
#![feature(proc_macro_non_items, generators, use_extern_macros)]
extern crate arc_reactor;
extern crate futures_await as futures;
extern crate tokio;

use arc_reactor::{core::ArcReactor, prelude::*};

fn main() {
	let server = ArcReactor::default()
		.port(3000)
		.service(Service)
		.start()
		.expect("Counldn't start server");
	tokio::run(server)
}

#[service]
fn Service(_req: Request, mut res: Response) {
	res.text("hello world");
	Ok(res)
}
