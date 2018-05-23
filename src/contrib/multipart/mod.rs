mod parser;
use core::Request;
use futures::{future::lazy, prelude::*, sync::oneshot::channel};
use header::{ContentLength, ContentType};
use mime::Mime;
use proto::{MiddleWare, MiddleWareFuture};
use std::{collections::HashMap, path::PathBuf};
use POOL;

/// A Multipart request parser.
/// This is only avaible behind the `unstable` features flag.
/// 
#[derive(Clone)]
pub struct Multipart {
	/// list of mimes you want to accept,
	pub mimes: Option<Vec<Mime>>,
	/// maximum upload size for files.
	pub size_limit: Option<u64>,
	/// directory to put the uploaded files.
	pub dir: PathBuf,
}

impl Multipart {
	pub fn new(dir: PathBuf, mimes: Option<Vec<Mime>>, size_limit: Option<u64>) -> Self {
		Self {
			mimes,
			dir,
			size_limit,
		}
	}
}

pub(crate) struct MultiPartMap(pub(crate) HashMap<String, String>);

impl MiddleWare<Request> for Multipart {
	fn call(&self, mut req: Request) -> MiddleWareFuture<Request> {
		let (dir, mimes) = (self.dir.clone(), self.mimes.clone());
		if let (Some(len), Some(limit)) = {
			(
				req.headers().get::<ContentLength>().clone(),
				self.size_limit,
			)
		} {
			if **len > limit {
				return Box::new(
					Err((400, format!("File upload limit {} Exceeded!", limit)).into())
						.into_future(),
				);
			}
		}

		let future = async_block! {
			let body = { req.body() };

			let boundary = {
				let content = match req.headers_mut().remove::<ContentType>() {
					None => return Ok(req),
					Some(c) => {
						req.headers_mut().set(c.clone());
						c
					},
				};

				match content
					.get_param("boundary")
					.and_then(|val| Some(String::from(val.as_str())))
				{
					Some(b) => b,
					None => return Err((400, "Unspecified Boundary for Multipart").into()),
				}
			};

			// we need to stream the body to a file using tokio::fs
			// but we need to do so in the context of a tokio executor
			// therefore, we spawn the Parser(that will poll the body, parse it and stream
			// to a file) Future on the tokio executor
			// but we need a way to return the output of the parsing back to the Middleware
			// so we use a oneshot channel.
			let (snd, rcv) = channel::<parser::ParseResult>();

			let future = lazy(|| {
				parser::parse(body, boundary, snd, dir, mimes)
					.map_err(|err| println!("[MultiPartParser][Error] {:?}", err))
			});

			POOL.sender().spawn(future).unwrap();
			if let Ok(result) = await!(rcv) {
				match result {
					parser::ParseResult::Ok(map) => {
						req.set(MultiPartMap(map));
					},
					parser::ParseResult::InvalidMime => {
						return Err((400, "Invalid Content-Type").into())
					},
					parser::ParseResult::Io(_) => {
						return Err((500, "internal server error").into())
					}
				};
			}

			Ok(req)
		};

		Box::new(future)
	}
}
