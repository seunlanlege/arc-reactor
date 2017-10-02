use hyper::{Response, Request, Error, StatusCode};
use futures::{future, Future, IntoFuture};
use ArcProto::{MiddleWare, arc, ArcError};

pub trait ArcService: Send + Sync {
	fn call (&self, req: Request) -> ResponseFuture;
}

type ResponseFuture = Box<Future<Item = Response, Error = Error>>;

impl<B, H, A> ArcService for (B, H, A)
where
		B: MiddleWare<R = Response, E = ArcError>  + Sync + Send,
		H: ArcService + Sync + Send,
		A: Fn(ResponseFuture) -> ResponseFuture + Sync + Send,
{
	fn call(&self, req: Request) -> ResponseFuture {
		let request = match self.0.call(req) {
			arc::Ok(request) => request,
			arc::Res(res) => {
				return box Ok(res.into()).into_future()
			}
			arc::Err(e) => {
				return box Ok(e.into()).into_future()
			}
		};
		let response = (self.1).call(request);
		(self.2)(response)
	}
}

impl<B, H> ArcService for (B, H)
where
		B: MiddleWare<R = Response, E = ArcError> + Sync + Send,
		H: ArcService + Sync + Send,
{
	fn call(&self, req: Request) -> ResponseFuture {
		let request = match self.0.call(req) {
			arc::Ok(request) => request,
			arc::Res(res) => {
				return box Ok(res.into()).into_future()
			}
			arc::Err(e) => {
				return box Ok(e.into()).into_future()
			}
		};
		(self.1).call(request)
	}

}