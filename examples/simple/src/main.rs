#![feature(proc_macro, box_syntax, generators, proc_macro_non_items)]
#![allow(non_camel_case_types, non_snake_case)]
#[macro_use]
extern crate arc_reactor;
extern crate regex;
use arc_reactor::{
	anymap::AnyMap,
	bytes::BytesMut,
	contrib::StaticFileServer,
	core::{ArcReactor, Request},
	futures::{
		future::lazy,
		prelude::*,
		sync::oneshot::{channel, Sender},
	},
	header::{ContentDisposition, ContentType, DispositionParam, DispositionType, Header},
	hyper::{Body, Error},
	prelude::*,
	proto::{MiddleWare, MiddleWareFuture},
	routing::Router,
	tokio::{
		fs::{file::CreateFuture, File},
		io::AsyncWrite,
	},
	POOL,
};
use regex::bytes::Regex;
use std::path::PathBuf;
// use arc_reactor::futures::future::poll_fn;

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
fn RequestHandler(req: Request, mut res: Response) {
	println!("Handler!");
	res.text("hello world");

	Ok(res)
}

#[derive(Clone)]
struct Multipart;

impl MiddleWare<Request> for Multipart {
	fn call(&self, mut req: Request) -> MiddleWareFuture<Request> {
		let body = { req.body() };

		let boundary = {
			req.headers()
				.get::<ContentType>()
				.and_then(|contentType| contentType.get_param("boundary"))
				.and_then(|val| Some(String::from(val.as_str())))
				.unwrap()
		};

		// we need to stream the body to a file using tokio::fs
		// but we need to do so in the context of a tokio executor
		// therefore, we spawn the Parser(that will poll the body, parse it and stream
		// to a file) Future on the tokio executor
		// but we need a way to return the output of the parsing back to the Middleware
		// so we use a oneshot channel.
		let (snd, rcv) = channel::<bool>();

		let future = lazy(|| {
			MultiPartParser::new(body, boundary, snd)
				.map_err(|err| println!("[MultiPartParser][Error] {:?}", err))
		});

		POOL.sender().spawn(future).unwrap();

		return Box::new(rcv.then(|_| Ok(req)));
	}
}

use std::collections::HashMap;

struct MultiPartParser {
	body: Body,
	state: DecodeState,
	map: HashMap<String, String>,
	files: Option<Vec<File>>,
	wr_buf: BytesMut,
	create: Option<CreateFuture<PathBuf>>,
	sender: Option<Sender<bool>>,
	boundary_regex: Regex,
	crlf_regex: Regex,
	done: bool
}

impl MultiPartParser {
	pub fn new(body: Body, boundary: String, sender: Sender<bool>) -> Self {
		let boundary = format!("--{}", boundary);
		let re = format!("{}*", boundary);
		let boundary_regex = Regex::new(&re).unwrap();
		let crlf_regex = Regex::new(r"\r\n").unwrap();
		Self {
			body,
			state: DecodeState::Boundary,
			files: None,
			create: None,
			wr_buf: BytesMut::new(),
			map: HashMap::new(),
			sender: Some(sender),
			boundary_regex,
			crlf_regex,
			done: false
		}
	}
}

#[derive(Debug, PartialEq)]
enum DecodeState {
	Boundary,
	Header,
	Read,
}

impl Drop for MultiPartParser {
	fn drop(&mut self) {
		println!("dropeed");
	}
}

// polls a stream of [u8] and parses them
// as multipart/form-data
impl Future for MultiPartParser {
	type Item = ();
	type Error = Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		loop {
			// is there a pending file to be created?
			if let Some(mut f_create) = self.create.take() {
				println!("if let Some(mut f_create) = self.create.take()");
				let file = match f_create.poll() {
					Ok(Async::Ready(f)) => f,
					Ok(Async::NotReady) => {
						self.create = Some(f_create);
						return Ok(Async::NotReady);
					}
					Err(e) => {
						println!("error {:?}", e);
						return Err(Error::Header);
					}
				};

				if let Some(ref mut files) = self.files {
					println!("pushed file!");
					files.push(file)
				} else {
					println!("created file!");
					self.files = Some(vec![file]);
				}
			}

			// we've created a new file,
			// let's write whats in our buffer to it.self
			if let Some(ref mut files) = self.files {
				if self.wr_buf.len() > 0 && !self.done {
					let len = { files.len() };
					let file = &mut files[len - 1];
					let buf = self.wr_buf.take();
					let n_bytes = try_ready!(file.poll_write(&buf));
					println!("wrote {}bytes", n_bytes);
				}
			}

			let chunk = match try_ready!(self.body.poll()) {
				Some(c) => c,
				None => {
					// stream has ended
					// send the params back to the middleware.
					self.sender.take().unwrap().send(true).unwrap();
					return Ok(Async::Ready(()));
				}
			};

			let matches = self.boundary_regex.split(&chunk).collect::<Vec<_>>();
			if matches.len() == 1 && matches[0].len() == chunk.len() {
				// non UTF-8 bytes.
				self.wr_buf.extend_from_slice(&chunk);
				self.done = false;
				continue;
			}

			for matched in matches {
				let split = self.crlf_regex.split(matched).collect::<Vec<_>>();

				for parts in split {
					if parts.len() == 0 {
						match self.state {
							DecodeState::Header => {
								self.state = DecodeState::Read;
								continue;
							}
							DecodeState::Read => {
								self.state = DecodeState::Header;
								self.done = true;
								continue;
							}
							DecodeState::Boundary => continue,
						};
					}

					match self.state {
						DecodeState::Header | DecodeState::Boundary => {
							if let Ok(disp) =
								ContentDisposition::parse_header(&parts.to_vec().into())
							{
								println!("{:?}", disp);
								if let DispositionType::Ext(ref contentType) = disp.disposition {
									if contentType.to_lowercase().contains("content-type") {
										let mime = contentType
											.get(13..)
											.unwrap()
											.to_string();
										// TODO: abort this future if the content-type is invalid.
										self.map.insert("content-type".into(), mime);
									}
								}

								for param in disp.parameters {
									match param {
										DispositionParam::Ext(_, name) => {
											self.map.insert("param".into(), name);
										}
										DispositionParam::Filename(_, _, filename) => {
											let filename = String::from_utf8(filename).unwrap();
											self.map.insert("file".into(), filename);
										}
									}
								}
							}
							self.state = DecodeState::Header;
						}

						DecodeState::Read => {
							match (self.map.remove("file"), self.map.remove("param")) {
								(Some(filename), Some(_)) => {
									let mut path = PathBuf::new();
									path.push("./");
									path.push(filename);
									self.create = Some(File::create(path));
									println!("self.create = Some(File::create(path));");
								}
								(None, Some(param)) => {
									self.map
										.insert(param, String::from_utf8_lossy(parts).into_owned());
									continue;
								}
								_ => {}
							};
							
							if &parts[..] != b"--" {
								self.done = false;
								self.wr_buf.extend_from_slice(parts);
							}
						}
					}
				}
			}
		}
	}
}
