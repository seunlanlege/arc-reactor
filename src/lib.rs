//! # Arc-reactor
//! <br><br>
//! An **asynchronous**, **multi-threaded** and **minimal** web framework for Rust.
//!
//! ![Crates.io](https://img.shields.io/crates/d/arc-reactor.svg)
//!
//! ## Features
//! - **Asynchronous**. In arc reactor, route handlers are asynchronous by default.
//!
//! - **Integration With futures-await**. The `#[service]` proc macro not only derives the `ArcService` trait for your route handler, but also marks it as `#[async]` so you can await on futures in  your route handlers with no extra stress.
//!
//! - **Intuitive Middleware System**. arc reactor exposes a middleware system that is easy to reason about.
//!
//! - **Minimalistic**. arc reactor is designed to be a very thin abstraction over tokio and hyper.
//!
//! - **Nightly Rust**. arc reactor uses a lot of cool features, including `proc_macros` which are only available on
//! the nightly channel.
//!
//! ## Installation
//!
//! Add this to your `cargo.toml`
//! ```toml
//! arc-reactor = "0.1"
//! ```
//! ## Guides
//! Check out the examples folder to get a feel for how `arc reactor`, it's well documented. and i'm terrible at
//! explaining things without using code.
//!
//! ## Demo
//!
//! ```rust
//! #![feature(conservative_impl_trait, proc_macro, generators, box_syntax)] // <== need to add this.
//! extern crate arc_reactor;
//! #[macro_use]
//! extern crate serde_json;
//! use arc_reactor::prelude::*;
//! use arc_reactor::{Router, ArcReactor, StatusCode};
//!
//! fn main() {
//! 	ArcReactor::new()
//! 		.routes(rootRoutes())
//! 		.port(3000)
//! 		.initiate()
//! 		.unwrap()
//! }
//!
//! fn rootRoutes() -> Router {
//! 	Router::new()
//! 		.get("/", IndexRoute)
//! }
//!
//!
//! #[service]
//! fn IndexRoute(_req: Rrequest, res: Response) {
//! 	let isAuth = await!(fakeFuture());
//! 	if isAuth {
//! 		let payload = json!({
//!       "data": "hello world"
//!     });
//!
//! 		return Ok(payload.into()) // convert json to json response.
//! 	}
//!
//! 	res.with_status(StatusCode::UnAuthorized)
//! }
//!
//! fn fakeFuture() -> impl Future<Item=bool, Error=()> {
//! 	futures::future::ok(true)
//! }
//!
//! ```
//!

#![feature(proc_macro, box_syntax, generators, fn_must_use, specialization)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate anymap;
pub extern crate futures_await as futures;
pub extern crate hyper;
extern crate impl_service;
extern crate num_cpus;
extern crate route_recognizer as recognizer;
extern crate serde_qs;
extern crate percent_encoding;

extern crate serde;
#[macro_use]
extern crate serde_json;
pub extern crate tokio_core;

#[macro_use]
pub(crate) mod proto;
pub(crate) mod contrib;
pub(crate) mod core;
pub(crate) mod routing;

pub use contrib::*;
pub use core::{ArcReactor, JsonError, QueryParseError};
pub use proto::{ArcHandler, ArcService, MiddleWare};
pub use routing::{RouteGroup, Router};

pub mod prelude {
	pub use core::{Request, Response};
	pub use futures;
	pub use futures::prelude::{async_block, await};
	pub use futures::{Future, Stream};
	pub use impl_service::{middleware, service};
	pub use proto::{ArcHandler, ArcService, MiddleWare};
}

pub use hyper::StatusCode;
pub use hyper::header;
