use hyper::{Response, Request, Error, StatusCode};
use futures::{future, Future};
use ArcProto::{MiddleWare};

pub trait ArcService: Send + Sync {
	fn call (&self, req: Request) -> ResponseFuture;
}

type ResponseFuture = Box<Future<Item = Response, Error = Error>>;

impl<Before, Handler, After> ArcService for (Before, Handler, After)
where 
		Before: MiddleWare  + Sync + Send,
		Handler: ArcService + Sync + Send,
		After: Fn(ResponseFuture) -> ResponseFuture + Sync + Send,
{
	fn call(&self, req: Request) -> ResponseFuture {
		let request = match self.0.call(req) {
			Ok(request) => request,
			Err(e) => {
				return Box::new(
					future::ok(
						Response::new()
							.with_status(StatusCode::BadRequest)
							.with_body(e.0)
					)
				)
			}
		};
		let response = (self.1).call(request);
		(self.2)(response)
	}

}

impl<Before, Handler> ArcService for (Before, Handler)
where 
		Before: MiddleWare + Sync + Send,
		Handler: ArcService + Sync + Send,
{
	fn call(&self, req: Request) -> ResponseFuture {
		let request = match self.0.call(req) {
			Ok(request) => request,
			Err(e) => {
				return Box::new(
					future::ok(
						Response::new()
							.with_status(StatusCode::BadRequest)
							.with_body(e.0)
					)
				)
			}
		};
		(self.1).call(request)
	}

}