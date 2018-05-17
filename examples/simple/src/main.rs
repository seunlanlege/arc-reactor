#![feature(proc_macro, box_syntax, generators, proc_macro_non_items)]
#![allow(non_camel_case_types, non_snake_case)]
#[macro_use]
extern crate arc_reactor;
extern crate futures_await;
extern crate regex;
use arc_reactor::{
	bytes::Bytes,
	contrib::StaticFileServer,
	core::{ArcReactor, Request},
	futures::{
		self,
		future::lazy,
		prelude::*,
		sync::oneshot::{channel, Sender},
	},
	header::{ContentDisposition, ContentType, DispositionParam, DispositionType},
	hyper::{Body, Error},
	prelude::*,
	proto::{MiddleWare, MiddleWareFuture},
	routing::Router,
	tokio::fs::File,
	POOL,
};
use regex::bytes::Regex;
use std::path::PathBuf;

fn getMainRoutes() -> Router {
	// Setup and maps routes to Services.
	return Router::new()
		.get("/", RequestHandler)
		.post2("/", Multipart, RequestHandler);
}

fn main() {
	// Start the reactor and try visiting localhost:3000/your-name
	let mut public = PathBuf::new();
	public.push("/home/seun/Documents");
	ArcReactor::new()
		.port(3000) // port to listen on
		.routes(getMainRoutes())
		.before(StaticFileServer::new("public", public))		
		.initiate()
		.unwrap()
}

#[service]
fn RequestHandler(_req: Request, mut res: Response) {
	println!("Handler!");
	res.text("hello world");

	Ok(res)
}

#[derive(Clone)]
struct Multipart;

impl MiddleWare<Request> for Multipart {
	fn call(&self, mut req: Request) -> MiddleWareFuture<Request> {
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
			let (snd, rcv) = channel::<bool>();

			let future = lazy(|| {
				multipart::parse(body, boundary, snd)
					.map_err(|err| println!("[MultiPartParser][Error] {:?}", err))
			});

			POOL.sender().spawn(future).unwrap();
			let _ = await!(rcv);
			Ok(req)
		};

		Box::new(future)
	}
}

mod multipart {
	use super::*;
	use arc_reactor::{header::Header, tokio::io::AsyncWrite};
	use std::collections::HashMap;

	pub fn parse(
		body: Body,
		boundary: String,
		_sender: Sender<bool>,
	) -> impl Future<Item = (), Error = Error> {
		let boundary = format!("--{}", boundary);
		let re = format!("{}*", boundary);
		let boundary_regex = Regex::new(&re).unwrap();
		let crlf_regex = Regex::new(r"\r\n").unwrap();
		let mut map = HashMap::new();
		let mut file: Option<File> = None;
		let mut state = DecodeState::Boundary;

		lazy(|| {
			async_block! {
				#[async]
				for chunk in body {
					let chunk = Bytes::from(chunk);
					let matches = boundary_regex.split(&chunk).map(Bytes::from).collect::<Vec<_>>();
					
					// invalid UTF8 bytes
					if matches.len() == 1 && matches[0].len() == chunk.len() {
						file = match await!(FileWrite::new(file, chunk)) {
							Ok(f) => Some(f),
							Err(err) => return Err(err)
						};
						continue
					}

					for matched in matches {
						let split = crlf_regex.split(&matched).map(Bytes::from).collect::<Vec<_>>();

						for parts in split {
							if parts.len() == 0 {
								match state {
									DecodeState::Header => {
										state = DecodeState::Read;
										continue;
									}
									DecodeState::Read => {
										state = DecodeState::Header;
										continue;
									}
									DecodeState::Boundary => continue,
								};
							}

							match state {
								DecodeState::Header | DecodeState::Boundary => {
									if let Ok(disp) =
										ContentDisposition::parse_header(&parts.to_vec().into())
									{
										if let DispositionType::Ext(ref contentType) = disp.disposition {
											if contentType.to_lowercase().contains("content-type") {
												let mime = contentType
													.get(13..)
													.unwrap()
													.to_string();
												// TODO: abort this future if the content-type is invalid.
												map.insert("content-type".into(), mime);
											}
										}

										for param in disp.parameters {
											match param {
												DispositionParam::Ext(_, name) => {
													map.insert("param".into(), name);
												}
												DispositionParam::Filename(_, _, filename) => {
													let filename = String::from_utf8(filename).unwrap();
													map.insert("file".into(), filename);
												}
											}
										}
									}
									state = DecodeState::Header;
								}

								DecodeState::Read => {
									match (map.remove("file"), map.remove("param")) {
										(Some(filename), Some(_)) => {
											let mut path = PathBuf::new();
											path.push("./");
											path.push(filename);
											file = match await!(File::create(path)) {
												Ok(f) => Some(f),
												Err(_) => return Ok(())
											};
										}
										(None, Some(param)) => {
											map
												.insert(param, String::from_utf8_lossy(&parts).into_owned());
											continue;
										}
										_ => {}
									};



									if &parts[..] != b"--" {
										file = match await!(FileWrite::new(file, parts)) {
											Ok(f) => Some(f),
											Err(err) => return Err(err)
										};
									}
								}
							}
						}
					}
				}

				// sender.take().unwrap().send(true).unwrap();
				Ok(())
			}
		})
	}

	#[derive(Debug, PartialEq)]
	enum DecodeState {
		Boundary,
		Header,
		Read,
	}

	struct FileWrite {
		file: Option<File>,
		buf: Bytes,
	}

	impl FileWrite {
		pub fn new(file: Option<File>, buf: Bytes) -> Self {
			Self { file, buf }
		}
	}

	impl Future for FileWrite {
		type Item = File;
		type Error = Error;

		fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
			if let Some(ref mut file) = self.file {
				let _ = try_ready!(file.poll_write(&self.buf));
			}

			Ok(Async::Ready(self.file.take().unwrap()))
		}
	}
}
