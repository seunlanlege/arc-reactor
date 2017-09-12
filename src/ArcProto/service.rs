use hyper;
use futures::Future;

pub trait ArcService: Send + Sync {
	fn call (&self, req: hyper::Request) -> Box<Future<Item = hyper::Response, Error = hyper::Error>>;
}

