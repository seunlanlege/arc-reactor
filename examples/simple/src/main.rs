#![feature(proc_macro, box_syntax, generators, proc_macro_non_items)]
#![allow(non_camel_case_types, non_snake_case)]
#[macro_use]
extern crate arc_reactor;
use arc_reactor::{
	anymap::AnyMap,
	bytes::{BufMut, BytesMut},
	contrib::StaticFileServer,
	core::ArcReactor,
	futures::{
		future::lazy,
		prelude::*,
		sync::oneshot::{channel, Sender},
	},
	header::{ContentDisposition, ContentType, DispositionParam, DispositionType, Header},
	hyper::{Body, Error},
	prelude::*,
	routing::Router,
	tokio::{
		fs::{file::CreateFuture, File},
		io::AsyncWrite,
		prelude,
	},
	POOL,
};
use std::path::PathBuf;
// use arc_reactor::futures::future::poll_fn;

fn getMainRoutes() -> Router {
	// Setup and maps routes to Services.
	return Router::new()
		.get("/", RequestHandler)
		.post("/", RequestHandler);
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
fn RequestHandler(mut req: Request, mut res: Response) {
	res.text("hello world");
	let body = { req.body() };

	let boundary = {
		req.headers()
			.get::<ContentType>()
			.and_then(|contentType| contentType.get_param("boundary"))
			.and_then(|val| Some(String::from(val.as_str())))
			.unwrap()
	};
	let (snd, rec) = channel::<u64>();
	let future = lazy(|| MultiPart::new(body, boundary, snd).then(|res| Ok(println!("{:?}", res))));

	POOL.sender().spawn(future).unwrap();
	let data = await!(rec);

	println!("data! {:?}", data);

	Ok(res)
}

use std::collections::HashMap;

struct MultiPart {
	body: Body,
	state: DecodeState,
	boundary: String,
	map: HashMap<String, Vec<u8>>,
	files: Option<Vec<File>>,
	buf: BytesMut,
	create: Option<CreateFuture<PathBuf>>,
	sender: Option<Sender<u64>>,
}

impl MultiPart {
	pub fn new(body: Body, boundary: String, sender: Sender<u64>) -> Self {
		Self {
			body,
			boundary,
			state: DecodeState::Boundary,
			files: None,
			create: None,
			buf: BytesMut::with_capacity(8196),
			sender: Some(sender),
			map: HashMap::new(),
		}
	}
}

#[derive(Debug, PartialEq)]
enum DecodeState {
	Boundary,
	End,
	Header(ContentDisposition),
	File,
}
struct FileName(String);
struct ParamName(String);

impl DecodeState {
	fn header(&self) -> ContentDisposition {
		match *self {
			DecodeState::Header(ref contentDispostion) => contentDispostion.clone(),
			_ => unreachable!(),
		}
	}
}

impl Drop for MultiPart {
	fn drop(&mut self) {
		println!("dropeed");
	}
}

impl Future for MultiPart {
	type Item = ();
	type Error = Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let mut map = AnyMap::new();

		loop {
			println!("looped");
			if let Some(mut f_create) = self.create.take() {
				println!("if let Some(mut f_create) = self.create.take() {{");
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
					self.files = Some(vec![file]);
				}
			}

			if let Some(ref mut files) = self.files {
				if self.buf.len() > 0 {
					let len = { files.len() };
					let file = &mut files[len - 1];
					let frozen = self.buf.take().freeze();
					println!("frozen.len() {}", frozen.len());
					let n_bytes = try_ready!(file.poll_write(&frozen));
					println!("wrote {} bytes", n_bytes);
				}
			}

			let chunk = match try_ready!(self.body.poll()) {
				Some(chunk) => chunk,
				None => {
					println!("stream returned none!");
					return Ok(Async::Ready(()));
				}
			};
			let buf = String::from_utf8_lossy(&chunk).to_owned();
			let buffer = buf.split("\r\n").collect::<Vec<&str>>();

			for mut bytes in buffer {
				if bytes.len() <= 1 {
					continue;
				}

				println!("{:?}", bytes);

				if bytes == format!("--{}", self.boundary) {
					self.state = DecodeState::Boundary;
				}

				if bytes == format!("--{}--", self.boundary) {
					self.state = DecodeState::End;
				}

				if let Ok(c_disp) = ContentDisposition::parse_header(&bytes.as_bytes().into()) {
					if let DispositionType::Ext(ref contentType) = c_disp.disposition {
						if contentType.to_lowercase().contains("content-type") {
							println!("content type! {:?}", contentType);
							continue;
						}
					}

					self.state = DecodeState::Header(c_disp);
				}

				match self.state {
					DecodeState::Boundary => {
						continue;
					}
					DecodeState::Header(_) => {
						for param in self.state.header().parameters {
							match param {
								DispositionParam::Ext(_, name) => {
									map.insert(ParamName(name));
								}
								DispositionParam::Filename(_, _, filename) => {
									let filename = String::from_utf8(filename).unwrap();
									map.insert(FileName(filename));
									self.state = DecodeState::File;
								}
							}
						}
						continue;
					}

					DecodeState::File => {
						println!("added {} bytes", bytes.len());
						self.buf.reserve(bytes.len());
						self.buf.put(&bytes.as_bytes());
					}

					DecodeState::End => {}
				}

				match (map.remove::<FileName>(), map.remove::<ParamName>()) {
					(Some(FileName(filename)), Some(ParamName(_))) => {
						let mut path = PathBuf::new();
						path.push("./");
						path.push(filename);
						self.create = Some(File::create(path));
						println!("self.create = Some(File::create(path));");
					}
					(None, Some(ParamName(param))) => {
						println!(
							"\n  {} \n",
							&param,
						);
						self.map.insert(param, bytes.as_bytes().to_vec());
					}
					(Some(filename), None) => {
						map.insert(filename);
					}
					_ => {}
				};
			}
		}
	}
}
