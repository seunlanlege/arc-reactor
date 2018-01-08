use hyper::{Error};
use ArcCore::{Request, self, Response};
use futures::{Future, IntoFuture};
use ArcProto::{MiddleWare, result};

pub trait ArcService: Send + Sync {
	fn call (&self, req: Request, res: Response) -> FutureResponse;
}

pub type FutureResponse = Box<Future<Item = Response, Error = Error>>;

impl<B, H, A> ArcService for (B, H, A)
where
		B: MiddleWare + Sync + Send,
		H: ArcService + Sync + Send,
		A: 'static + Fn(Response) -> Response + Sync + Send,
{
	fn call(&self, req: Request, res: Response) -> FutureResponse {
		let request = match self.0.call(req) {
			result::Ok(request) => request,
			result::response(res) => {
				return box Ok(res).into_future()
			}
			result::error(e) => {
				return box Ok(e.into()).into_future()
			}
		};
		let response = (self.1).call(request, res);
		return box response.map(&self.2)
	}
}

impl<B, H> ArcService for (B, H)
where
		B: MiddleWare + Sync + Send,
		H: ArcService + Sync + Send,
{
	fn call(&self, req: Request, res: Response) -> FutureResponse {
		let request = match self.0.call(req) {
			result::Ok(request) => request,
			result::response(res) => {
				return box Ok(res).into_future()
			}
			result::error(e) => {
				return box Ok(e.into()).into_future()
			}
		};
		(self.1).call(request, res)
	}

}
