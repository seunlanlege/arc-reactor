#![feature(proc_macro, box_syntax, generators, conservative_impl_trait)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate anymap;
pub extern crate futures_await as futures;
extern crate hyper;
extern crate impl_service;
extern crate num_cpus;
extern crate route_recognizer as recognizer;
extern crate tokio_core;

#[macro_use]
mod proto;
mod routing;
mod core;

pub use proto::*;
pub use core::*;
pub use routing::*;

pub mod prelude {
	pub use futures::prelude::{async_block, await};
	pub use impl_service::{middleware, service};
	pub use futures::future::Future;
	pub use futures;
	pub use proto::{ArcService, ArcHandler};
}

pub use hyper::StatusCode;
