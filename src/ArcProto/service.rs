use hyper;
use futures::Future;

pub trait ArcService: Clone {
	fn call (&self, req: hyper::Request) -> Box<Future<Item = hyper::Response, Error = hyper::Error>>;
	fn mock(&self) -> String;
}

impl <S: ArcService>ArcService for Box<S> {
	fn call (&self, req: hyper::Request) -> Box<Future<Item = hyper::Response, Error = hyper::Error>>{
		(**self).call(req)
	}
	
	fn mock(&self) -> String {
		"Boxed".to_string()
	}
}