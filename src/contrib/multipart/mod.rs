mod parser;
use core::Request;
use futures::{future, prelude::*};
use header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyperx::header::{ContentType, Header};
use mime::Mime;
use proto::{MiddleWare, MiddleWareFuture};
use std::{collections::HashMap, path::PathBuf};
/// A Multipart request parser.
/// This is only avaible behind the `unstable` features flag.
///
#[derive(Clone)]
pub struct MultiPart {
	/// list of mimes you want to accept,
	pub mimes: Option<Vec<Mime>>,
	/// maximum upload size for files.
	pub size_limit: Option<u64>,
	/// directory to put the uploaded files.
	pub dir: PathBuf,
}

impl MultiPart {
	pub fn new(dir: PathBuf, mimes: Option<Vec<Mime>>, size_limit: Option<u64>) -> Self {
		Self {
			mimes,
			dir,
			size_limit,
		}
	}
}

pub(crate) struct MultiPartMap(pub(crate) HashMap<String, String>);

impl MiddleWare<Request> for MultiPart {
	fn call(&self, mut req: Request) -> MiddleWareFuture<Request> {
		let (dir, mimes) = (self.dir.clone(), self.mimes.clone());

		if let (Some(len), Some(limit)) =
			{ (req.headers().get(CONTENT_LENGTH).clone(), self.size_limit) }
		{
			let len = len
				.to_str()
				.map_err(|_| ())
				.and_then(|len| len.parse::<u64>().map_err(|_| ()));
			match len {
				Ok(len) => {
					if len > limit {
						return Box::new(future::err(
							(400, format!("File upload limit {} Exceeded!", limit)).into(),
						));
					}
				}
				Err(_) => {
					return Box::new(future::err((400, "Invalid Content Length").into()));
				}
			};
		}


		let content_type = req.headers().get(CONTENT_TYPE).and_then(|v| {
			let hv = v.clone();
			hv.to_str().ok().and_then(|v| Some(v.to_string()))
		});

		let future = async_block! {
			let body = { req.body() };

			let boundary = {
				let content_type = match content_type {
					None => return Err((400, "Invalid Content Length").into()),
					Some(c) => ContentType::parse_header(&c.into()).ok()
				};


				match content_type
					.and_then(|disp| disp.get_param("boundary").and_then(|val| Some(String::from(val.as_str()))))
				{
					Some(b) => b,
					None => return Err((400, "Unspecified Boundary for Multipart").into()),
				}
			};

			match await!(parser::parse(body, boundary, dir, mimes)) {
				Ok(map) => {
					req.set(MultiPartMap(map));
				},
				Err(err) => {
					println!("[MultiPartParser][Error] {:?}", err);
					use self::parser::ParseError::*;
					match err {
						InvalidMime => return Err((400, "Invalid Content-Type").into()),
						_ => return Err((500, "internal server error").into()),
					};
				}
			};

			Ok(req)
		};

		Box::new(future)
	}
}
