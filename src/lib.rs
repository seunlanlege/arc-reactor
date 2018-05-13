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
//! arc-reactor = "0.1"
//! ```
//! ## Guides
//! Check out the examples folder to get a feel for how `arc reactor`, it's
//! well documented. and i'm terrible at explaining things without using code.
//!
//! ## Demo
//!
//! ```rust,
//! #![feature(proc_macro, generators, box_syntax)] // <== need to add this.
//! extern crate arc_reactor;
//! #[macro_use]
//! extern crate serde_json;
//! use arc_reactor::{prelude::*, ArcReactor, Router, StatusCode};
//!
//! fn main() {
//! 	ArcReactor::new()
//! 		.routes(rootRoutes())
//! 		.port(3000)
//! 		.initiate()
//! 		.unwrap()
//! 	}
//!
//! fn rootRoutes() -> Router {
//! 	Router::new().get("/", IndexRoute)
//! 	}
//!
//! #[service]
//! fn IndexRoute(_req: Request, mut res: Response) {
//! 	let isAuth = await!(fakeFuture()).unwrap();
//! 	if isAuth {
//! 		let payload = json!({
//!       "data": "hello world"
//!     });
//!
//! 		return Ok(payload.into()); // convert json to json response.
//! 		}
//!
//! 	res.set_status(StatusCode::Unauthorized);
//! 	Err(res)
//! 	}
//!
//! fn fakeFuture() -> impl Future<Item = bool, Error = ()> {
//! 	futures::future::ok(true)
//! 	}
//! ```
//!

#![feature(proc_macro, generators, fn_must_use, specialization, proc_macro_non_items)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate anymap;
#[macro_use]
pub extern crate futures_await as futures;
pub extern crate hyper;
#[cfg(not(feature = "stable"))]
extern crate impl_service;
pub extern crate native_tls;
extern crate num_cpus;
extern crate percent_encoding;
extern crate serde_qs;
extern crate tokio;
extern crate tokio_tls;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_json;
pub extern crate tokio_core;

#[macro_use]
pub mod proto;
pub mod contrib;
pub mod core;
pub mod routing;

pub mod prelude {
	pub use core::{Request, Response};
	pub use futures::{
		self,
		prelude::{async_block, await},
		Future,
		IntoFuture,
		Stream,
	};
	#[cfg(not(feature = "stable"))]
	pub use impl_service::{middleware, service};
	pub use proto::{ArcHandler, ArcService, FutureResponse, MiddleWare};
}

pub use hyper::{header, StatusCode};
