#![feature(proc_macro, box_syntax, generators, conservative_impl_trait)]
#[macro_use]
extern crate serde_derive;
extern crate arc_reactor;
extern crate dotenv;

mod routes;
mod controllers;
mod middlewares;
mod services;
mod models;

use arc_reactor::prelude::*;
use arc_reactor::ArcReactor;
use routes as AppRouter;

fn main() {
	ArcReactor::new()
		.port(get_port())
		.routes(AppRouter::get_route())
		.initiate()
		.unwrap()

	// match my_app {
	// 	Ok(_) => {
	// 		println!("Application started successfully.");
	// 	}
	// 	Err(_) => {
	// 		println!("Your application failed to start.");
	// 	}
	// }
}

/// Extract application port from .env file.
fn get_port() -> i16 {
	dotenv::dotenv().ok();

	let default = 8080;

	match dotenv::var("PORT") {
		Ok(port) => port.parse::<i16>().unwrap_or(default),
		Err(_) => default,
	}
}
