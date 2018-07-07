#![allow(non_camel_case_types, non_snake_case)]
#![feature(proc_macro, proc_macro_non_items, generators)]
extern crate arc_reactor;
use arc_reactor::{
	core::{ArcReactor, Request},
	prelude::*,
	tokio,
};

fn main() {
	// Start the reactor and try visiting localhost:3000/your-name
	tokio::run(
		ArcReactor::default()
			.port(3000)
			.service(Service)
			.start()
			.expect("Counldn't start server")
	)
}

#[service]
fn Service(req: Request, mut res: Response) {
	res.text("hello world");
	Ok(res)
}
