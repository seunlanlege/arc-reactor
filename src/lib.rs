#![feature(proc_macro, box_syntax, generators, conservative_impl_trait, fn_must_use,
           specialization)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate anymap;
pub extern crate futures_await as futures;
pub extern crate hyper;
extern crate impl_service;
extern crate num_cpus;
extern crate queryst_prime;
extern crate route_recognizer as recognizer;
extern crate serde;
#[macro_use]
extern crate serde_json;
pub extern crate tokio_core;

#[macro_use]
pub mod proto;
pub mod routing;
pub mod core;
pub mod contrib;

pub use proto::{ArcHandler, ArcService, MiddleWare};
pub use core::{res, ArcReactor, JsonError, QueryParseError};
pub use routing::{RouteGroup, Router};

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
