use hyper::{Response, Error};
use ArcCore::{Request, self};
use futures::{Future, IntoFuture};
use ArcProto::{MiddleWare, arc};

pub trait ArcService: Send + Sync {
	fn call (&self, req: Request, res: Response) -> ResponseFuture;
}

type ResponseFuture = Box<Future<Item = Response, Error = Error>>;

impl<B, H, A> ArcService for (B, H, A)
where
		B: MiddleWare + Sync + Send,
		H: ArcService + Sync + Send,
		A: Fn(ResponseFuture) -> ResponseFuture + Sync + Send,
{
	fn call(&self, req: Request, res: Response) -> ResponseFuture {
		let request = match self.0.call(req) {
			arc::Ok(request) => request,
			arc::Res(res) => {
				return box Ok(res.into()).into_future()
			}
			arc::Err(e) => {
				let res: ArcCore::Response = e.into();
				return box Ok(Response::from(res)).into_future()
			}
		};
		let response = (self.1).call(request, res);
		(self.2)(response)
	}
}

impl<B, H> ArcService for (B, H)
where
		B: MiddleWare + Sync + Send,
		H: ArcService + Sync + Send,
{
	fn call(&self, req: Request, res: Response) -> ResponseFuture {
		let request = match self.0.call(req) {
			arc::Ok(request) => request,
			arc::Res(res) => {
				return box Ok(res.into()).into_future()
			}
			arc::Err(e) => {
				let res: ArcCore::Response = e.into();
				return box Ok(Response::from(res)).into_future()
			}
		};
		(self.1).call(request, res)
	}

}