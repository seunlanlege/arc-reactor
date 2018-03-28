//! # arc-reactor
//!
//! > Minimalistic, Asynchronous, Multithreaded Web framework
//!
//!
//! - **Asynchronous**. In arc reactor, route handlers are asynchronous by default.

//! - **Integration With futures-await**. The `#[service]` proc macro not only derives the `ArcService` trait for your route handler, but also marks it as //! `#[async]` so you can await on futures in  your route handlers with no extra stress.
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
//! add this to your `cargo.toml`
//! ```toml
//! arc-reactor = "0.1"
//! ```
//! ## Guides
//! check out the examples folder to get a feel for how `arc reactor`, it's well documented. and i'm terrible at
//! explaining things without using code.
//!
//!
//!
#![feature(proc_macro, box_syntax, generators, conservative_impl_trait, fn_must_use,
           specialization)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate anymap;
pub extern crate futures_await as futures;
pub extern crate hyper;
extern crate impl_service;
extern crate num_cpus;
extern crate route_recognizer as recognizer;
extern crate regex;
extern crate url;

#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_json;
pub extern crate tokio_core;



#[macro_use]
pub(crate) mod proto;
pub(crate) mod routing;
pub(crate) mod core;
pub(crate) mod contrib;

pub use proto::{ArcHandler, ArcService, MiddleWare};
pub use core::{ArcReactor, JsonError, QueryParseError};
pub use routing::{RouteGroup, Router};
pub use contrib::*;

pub mod prelude {
	pub use futures::prelude::{async_block, await};
	pub use impl_service::{middleware, service};
	pub use core::{Request, Response};
	pub use futures::{Future, Stream};
	pub use futures;
	pub use proto::{ArcHandler, ArcService, MiddleWare};
}

pub use hyper::StatusCode;
pub use hyper::header;
