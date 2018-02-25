#![feature(proc_macro, box_syntax, generators, conservative_impl_trait)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate anymap;
pub extern crate futures_await as futures;
extern crate hyper;
extern crate impl_service;
extern crate native_tls;
extern crate num_cpus;
extern crate route_recognizer as recognizer;
extern crate tokio_core;
extern crate tokio_tls;
extern crate crossbeam_channel;

#[macro_use]
mod ArcProto;
mod ArcRouting;
mod ArcCore;

pub use ArcProto::*;
pub use ArcCore::*;
pub use ArcRouting::*;

pub mod prelude {
	pub use futures::prelude::async_block;
	pub use impl_service::{middleware, service};
	pub use futures::future::Future;
}

pub use hyper::StatusCode;
