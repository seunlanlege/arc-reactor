mod parser;
use core::Request;
use futures::{future::lazy, prelude::*, sync::oneshot::channel};
use header::{ContentLength, ContentType};
use mime::Mime;
use proto::{MiddleWare, MiddleWareFuture};
use std::{collections::HashMap, path::PathBuf};
use POOL;

/// Multipart request parser.
#[derive(Clone)]
pub struct Multipart {
	/// mimes you want to accept,
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
				match req.headers()
					.get::<ContentType>()
					.and_then(|contentType| contentType.get_param("boundary"))
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
			let (snd, rcv) = channel::<HashMap<String, String>>();

			let future = lazy(|| {
				parser::parse(body, boundary, snd, dir, mimes)
					.map_err(|err| println!("[MultiPartParser][Error] {:?}", err))
			});

			POOL.sender().spawn(future).unwrap();
			if let Ok(map) = await!(rcv) {
				req.set(MultiPartMap(map));
			}

			Ok(req)
		};

		Box::new(future)
	}
}
