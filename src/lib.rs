//! # Arc-reactor
//! <br><br>
//! An **asynchronous**, **multi-threaded** and **minimal** web framework for
//! Rust.
//!
//! ![Crates.io](https://img.shields.io/crates/d/arc-reactor.svg)
//!
//! ## Features
//! - **Asynchronous**. In arc reactor, route handlers are asynchronous by
//! default.
//!
//! - **Integration With futures-await**. The `#[service]` proc macro not only
//! derives the `ArcService` trait for your route handler, but also marks it as
//! `#[async]` so you can await on futures in  your route handlers with no
//! extra stress.
//!
//! - **Intuitive Middleware System**. arc reactor exposes a middleware system
//! that is easy to reason about.
//!
//! - **Minimalistic**. arc reactor is designed to be a very thin abstraction
//! over tokio and hyper.
//!
//! - **Nightly Rust**. arc reactor uses a lot of cool features, including
//! `proc_macros` which are only available on the nightly channel.
//!
//! ## Installation
//!
//! Add this to your `cargo.toml`
//! ```toml
//! arc - reactor = "0.1"
//! ```
//! ## Guides
//! Check out the examples folder to get a feel for how `arc reactor`, it's
//! well documented. and i'm terrible at explaining things without using code.
//!
//! ## Demo
//!
//! ```rust, no_run
//! #![feature(proc_macro, generators, proc_macro_non_items)] // <== need to add this.
//! extern crate arc_reactor;
//! extern crate futures_await as futures;
//! extern crate tokio;
//! #[macro_use]
//! extern crate serde_json;
//! 
//! use arc_reactor::{prelude::*, core::ArcReactor, routing::Router};
//!
//! fn main() {
//! 	let server = ArcReactor::default()
//! 		.routes(rootRoutes())
//! 		.port(3000)
//! 		.start()
//! 		.expect("couldn't start server");
//!
//! 	tokio::run(server)
//! }
//!
//! fn rootRoutes() -> Router {
//! 	Router::new().get("/", IndexRoute)
//! }
//!
//! #[service]
//! fn IndexRoute(_req: Request, mut res: Response) {
//! 	let isAuth = await!(fakeFuture()).unwrap();
//! 	if isAuth {
//! 		let payload = json!({
//!       		"data": "hello world"
//!     	});
//!
//! 		return Ok(payload.into()); // convert json to json response.
//! 	}
//!
//! 	res.set_status(400);
//! 	Err(res)
//! }
//!
//! fn fakeFuture() -> impl Future<Item = bool, Error = ()> {
//! 	futures::future::ok(true)
//! }
//! ```
//!
#![cfg_attr(
	feature = "unstable",
	feature(proc_macro, generators, specialization, proc_macro_non_items, test)
)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#[macro_use]
extern crate log;
extern crate http;
#[cfg(feature = "unstable")]
#[macro_use]
#[macro_export]
extern crate futures_await as futures;
#[cfg(not(feature = "unstable"))]
#[macro_use]
extern crate futures;
extern crate hyper;
#[cfg(feature = "unstable")]
extern crate impl_service;
extern crate native_tls;
extern crate percent_encoding;
extern crate serde;
extern crate serde_qs;
extern crate tokio;
extern crate tokio_tls;
#[macro_use]
extern crate serde_json;
extern crate bytes;
extern crate hyperx;
extern crate mime;
extern crate mime_guess;
extern crate regex;

#[macro_use]
pub mod proto;
pub mod contrib;
pub mod core;
pub mod routing;

pub use futures::prelude::*;
pub use hyper::{header, StatusCode};

pub mod prelude {
	pub use core::{Request, Response};
	pub use futures::prelude::*;
	#[cfg(feature = "unstable")]
	pub use futures::prelude::{async_block, await};
	#[cfg(feature = "unstable")]
	pub use impl_service::{middleware, service};
	pub use proto::{ArcHandler, ArcService, FutureResponse, MiddleWare, MiddleWareFuture};
}
