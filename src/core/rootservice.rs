use super::{Request, Response};
use futures::Future;
use hyper::{
	self,
	error::Error,
	header::{HeaderValue, SERVER},
	service::Service,
	Body,
};
use proto::{ArcHandler, ArcService};
use std::{net::SocketAddr, panic::AssertUnwindSafe};
use tokio_core::reactor::Handle;
// The only reason this exists is so I can pass the
// clientIp to the ArcService.
pub(crate) struct RootService {
	pub(crate) remote_ip: SocketAddr,
	pub(crate) service: ArcHandler,
	pub(crate) handle: Handle,
}

impl Service for RootService {
	type ReqBody = Body;
	type ResBody = Body;
	type Error = Error;
	type Future = Box<Future<Item = hyper::Response<Body>, Error = Self::Error>>;

	fn call(&mut self, req: hyper::Request<Self::ReqBody>) -> Self::Future {
		let mut request: Request = req.into();
		let res = Response::new();
		let responseFuture = AssertUnwindSafe(self.service.call(request, res)).catch_unwind();

		let responseFuture = responseFuture.then(|result| {
			match result {
				Ok(response) => {
					match response {
						Ok(mut res) | Err(mut res) => {
							res.headers_mut()
								.insert(SERVER, HeaderValue::from_static("Arc-Reactor/0.1.5"));
							Ok(res.into())
						}
					}
				}
				Err(_) => {
					let res = hyper::Response::builder()
						.status(500)
						.body(Body::empty())
						.unwrap();
					Ok(res)
				}
			}
		});

		return Box::new(responseFuture);
	}
}
