#![feature(
proc_macro,
box_syntax,
generators,
)]

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]

pub extern crate anymap;
pub extern crate impl_service;
pub extern crate num_cpus;
pub extern crate tokio_core;
pub extern crate route_recognizer as recognizer;
pub extern crate futures_await as futures;
pub extern crate hyper;

mod ArcRouting;
mod ArcCore;
#[macro_use]
mod ArcProto;

pub use impl_service::{service, middleware};
pub use hyper::{Error, StatusCode};
pub use futures::future::Future;
pub use futures::prelude::{async_block};
pub use ArcCore::*;
pub use ArcRouting::*;
pub use ArcProto::*;
