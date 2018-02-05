use hyper::{Error};
use ArcCore::{Request, self, Response};
use futures::{Future, IntoFuture};
use ArcProto::{MiddleWare};
use std::sync::Arc;


pub trait ArcService: Send + Sync {
	fn call (&self, req: Request, res: Response) -> Box<Future<Item = Response, Error = Error>>;
}

pub type FutureResponse = Box<Future<Item = Response, Error = Error>>;

impl ArcService for (Arc<Box<MiddleWare<Request>>>, Arc<Box<ArcService>>, Arc<Box<MiddleWare<Response>>>)
{
	fn call(&self, req: Request, res: Response) -> FutureResponse {
		let f = self.0.clone();
		let s = self.1.clone();
		let t = self.2.clone();
		box f.call(req)
			.and_then(move |req| s.call(req, res))
			.and_then(move |res| t.call(res))
	}
}

impl ArcService for (Arc<Box<MiddleWare<Request>>>, Arc<Box<ArcService>>)
{
	fn call(&self, req: Request, res: Response) -> Box<Future<Item = Response, Error = Error>> {
		let f = self.0.clone();
		let s = self.1.clone();
		box f.call(req).and_then(move |req| s.call(req, res))
	}
}
