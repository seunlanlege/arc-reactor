use core::Request;
use futures::prelude::*;
use proto::{MiddleWare, MiddleWareFuture};

#[derive(Clone, Debug)]
pub struct StaticFileServer {
	pub root: &'static str,
}

impl MiddleWare<Request> for StaticFileServer {
	fn call(&self, req: Request) -> MiddleWareFuture<Request> {
		let root = self.root.clone();
		let future = async_block! {
			{
				let path = req.path();
				match path.get(1...root.len()) {
					Some(p) if p == root => {
						println!("Nice! {}", path);
					}
					_ => {}
				};
			}
			Ok(req)
		};

		Box::new(future)
	}
}
