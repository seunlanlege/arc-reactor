use hyper::{Response, Request, Error};
use futures::Future;

type ResponseFuture = Box<Future<Item = Response, Error = Error>>;

pub trait ArcService: Send + Sync {
	fn call (&self, req: Request) -> ResponseFuture;
}


impl<Before, Handler, After> ArcService for (Before, Handler, After)
where 
		Before: Fn(Request) -> Request + Sync + Send,
		Handler: ArcService + Sync + Send,
		After: Fn(ResponseFuture) -> ResponseFuture + Sync + Send,
{
	fn call(&self, req: Request) -> ResponseFuture {
		let request = (self.0)(req);
		let response = (self.1).call(request);
		(self.2)(response)
	}

}

impl<Before, Handler> ArcService for (Before, Handler)
where 
		Before: Fn(Request) -> Request + Sync + Send,
		Handler: ArcService + Sync + Send,
{
	fn call(&self, req: Request) -> ResponseFuture {
		let request = (self.0)(req);
		(self.1).call(request)
	}

}

// impl<Handler, After> ArcService for (Handler, After)
// where 
// 		Handler: ArcService + Sync + Send,
// 		After: Fn(ResponseFuture) -> ResponseFuture + Sync + Send,
// {
// 	fn call(&self, req: Request) -> ResponseFuture {
// 		let response = (self.1).call(req);
// 		(self.2)(response)
// 	}

// }