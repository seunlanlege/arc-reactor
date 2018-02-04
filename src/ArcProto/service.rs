use hyper::{Error};
use ArcCore::{Request, self, Response};
use futures::{Future, IntoFuture};
use ArcProto::{MiddleWare};
use std::sync::Arc;


pub trait ArcService: Send + Sync {
	fn call (&self, req: Request, res: Response) -> Box<Future<Item = Response, Error = Error> + 'static>;
}

pub type FutureResponse = Box<Future<Item = Response, Error = Error>>;

//impl<B> ArcService for (B, Arc<ArcService>, Arc<MiddleWare<Response>>)
//where
//		B: MiddleWare<Request> + Sync + Send,
//{
//	fn call(&self, req: Request, res: Response) -> FutureResponse {
//		let handler = self.1.clone();
//		let after= self.2.clone();
//		box self.0.call(req)
//			.and_then(move |req| handler.call(req, res))
//			.and_then(move |res| after.call(res))
//	}
//}

//impl ArcService for (Box<MiddleWare<Request>>, Box<ArcService>)
//{
//	fn call(&self, req: Request, res: Response) -> Box<Future<Item = Response, Error = Error>> {
//		box	self.0.call(req).and_then(move |req| self.1.call(req, res))
//	}
//}
